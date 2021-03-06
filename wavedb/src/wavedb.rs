use crate::errors::Waverr;
use crate::hier_map::HierMap;
use crate::vcd_parser::WaveParser;
use crate::storage::in_memory::InMemWave;
use crate::storage::bucket::Bucket;
use crate::{DEFAULT_SLIZE_SIZE};
use bincode;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use std::path::*;
use std::sync::Arc;
use toml;
use vcd::Command;

#[derive(Serialize, Deserialize, Debug, Default)]
struct WDBConfig {
    db_name: String,
    time_range: (u32, u32),
}

///DB for holding buckets
///
///slize_size determines the bounding of size of the timestamp range in each bucket
#[derive(Debug)]
pub struct WaveDB {
    db: Db,
    //TODO: think about what should be wanted from a cfg file
    config: WDBConfig,
    pub hier_map: Arc<HierMap>,
}

impl WaveDB {
    fn new(db_name: String, db_path: Option<&Path>) -> WaveDB {
        WaveDB {
            db: sled::open(db_path.unwrap_or(db_name.as_ref())).unwrap(),
            hier_map: Arc::default(),
            config: WDBConfig {
                db_name: db_name.clone(),
                ..WDBConfig::default()
            },
        }
    }



    fn get_id(&self, sig: &str) -> Result<u32, Waverr> {
        self.hier_map.path_to_id(sig)
    }

    fn get_time_slices(&self) -> std::iter::StepBy<std::ops::Range<u32>> {
        ((self.config.time_range.0 / DEFAULT_SLIZE_SIZE) * DEFAULT_SLIZE_SIZE
            ..(self.config.time_range.1 / DEFAULT_SLIZE_SIZE + 2)
                * DEFAULT_SLIZE_SIZE)
            .step_by(DEFAULT_SLIZE_SIZE as usize)
    }

    fn set_time_range(&mut self, range: (u32, u32)) {
        self.config.time_range = range;
    }

    fn load_config(&mut self) -> Result<(), Waverr> {
        if let Ok(Some(rawbytes)) = self.db.get("config") {
            let config: WDBConfig = toml::from_slice(rawbytes.as_ref())?;
            self.config = config;
            Ok(())
        } else {
            //TODO: maybe make specific error for this?
            Err(Waverr::WDBCfgErr("Config not found in WaveDB".into()))
        }
    }

    fn dump_config(&self) -> Result<(), Waverr> {
        self.db
            .insert("config", toml::to_string(&self.config)?.as_str())?;
        Ok(())
    }

    fn save_idmap(&self) -> Result<(), Waverr> {
        self.db
            .insert("id_map", bincode::serialize(self.hier_map.as_ref())?)?;
        Ok(())
    }

    fn load_idmap(&mut self) -> Result<(), Waverr> {
        if let Ok(Some(rawbytes)) = self.db.get("id_map") {
            self.hier_map = Arc::new(bincode::deserialize(rawbytes.as_ref())?);
            Ok(())
        } else {
            Err(Waverr::WDBCfgErr("HierMap not found in WaveDB"))
        }
    }

    pub fn get_bounds(&self) -> (u32,u32) {
        self.config.time_range.clone()
    }

    pub fn was_recovered(&self) -> bool {
        self.db.was_recovered()
    }

    pub fn open_wdb(wdb_path: &Path) -> Result<WaveDB, Waverr> {
        let mut wdb = WaveDB::new("TempName".into(), Some(wdb_path));
        wdb.load_config()?;
        wdb.load_idmap()?;
        Ok(wdb)
    }

    pub fn get_hier_map(&self) -> Arc<HierMap> {
        self.hier_map.clone()
    }

    //TODO: parallelize this
    pub fn from_vcd(
        vcd_file_path: PathBuf,
        wdb_path: &Path,
    ) -> Result<WaveDB, Waverr> {
        let mut parser = WaveParser::new(vcd_file_path.clone())?;
        let wdb_name = {
            if let Some(vcd_file) = vcd_file_path.file_stem() {
                vcd_file.to_str().unwrap().to_string()
            } else {
                vcd_file_path.to_str().unwrap().to_string()
            }
        };
        let mut wdb = WaveDB::new(wdb_name, Some(wdb_path));
        if wdb.was_recovered() {
            wdb.load_config()?;
            wdb.load_idmap()?;
            return Ok(wdb);
        }
        let mut first_time = None;
        let mut global_time: u32 = 0;
        let mut current_range = (global_time, global_time + DEFAULT_SLIZE_SIZE);
        let mut bucket_mapper: HashMap<vcd::IdCode, Bucket> = HashMap::new();
        wdb.hier_map = Arc::new(parser.create_hiermap()?);
        for item in parser {
            match item {
                Ok(Command::Timestamp(time)) => {
                    let time = time as u32;
                    if time % DEFAULT_SLIZE_SIZE
                        < global_time % DEFAULT_SLIZE_SIZE
                    {
                        for (_, bucket) in bucket_mapper.iter() {
                            wdb.insert_bucket(bucket)?;
                        }
                        bucket_mapper.clear();
                        let rounded_time = time - (time % DEFAULT_SLIZE_SIZE);
                        current_range =
                            (rounded_time, rounded_time + DEFAULT_SLIZE_SIZE)
                    }
                    if first_time.is_none() {
                        first_time = Some(time);
                    }
                    global_time = time;
                }
                //TODO: collapse these arms if possible? good way to share this code?
                Ok(Command::ChangeVector(code, vvalue)) => {
                    if !bucket_mapper.contains_key(&code) {
                        bucket_mapper.insert(
                            code,
                            Bucket::new(
                                code.0 as u32,
                                vvalue.len(),
                                current_range,
                            ),
                        );
                    }
                    let bucket = bucket_mapper.get_mut(&code).unwrap();
                    bucket.add_new_signal(global_time, vvalue);
                }
                Ok(Command::ChangeScalar(code, value)) => {
                    if !bucket_mapper.contains_key(&code) {
                        bucket_mapper.insert(
                            code,
                            Bucket::new(code.0 as u32, 1, current_range),
                        );
                    }
                    let bucket = bucket_mapper.get_mut(&code).unwrap();
                    bucket.add_new_signal(global_time, vec![value]);
                }
                Ok(_) => {}
                Err(_) => {
                    return Err(Waverr::VCDErr("Malformed vcd"));
                }
            }
        }
        for (_, bucket) in bucket_mapper.iter() {
            wdb.insert_bucket(bucket)?;
        }

        wdb.set_time_range((first_time.expect("No timestamp present in VCD!"), global_time));
        wdb.dump_config()?;
        wdb.save_idmap()?;
        wdb.db.flush()?;
        Ok(wdb)
    }

    fn insert_bucket(&self, bucket: &Bucket) -> Result<(), Waverr> {
        let tree: sled::Tree = self.db.open_tree(bucket.get_db_idx())?;
        let serialized = serde_json::to_string(&bucket)?;

        if let Ok(Some(_)) =
            tree.insert(bucket.signal_id().to_be_bytes(), serialized.as_str())
        {
            // is problematic; implies that this value was previously set and we are
            // overwriting it. We should write only once per bucket
            return Err(Waverr::BucketErr {
                id: bucket.signal_id(),
                ts_range: bucket.get_db_idx(),
                context: "We are overwriting a value for this bucket!",
            });
        }
        Ok(())
    }

    fn retrieve_bucket(
        &self,
        id: u32,
        ts_start: u32,
    ) -> Result<Bucket, Waverr> {
        let tree = self.db.open_tree(WaveDB::ts2key(ts_start))?;
        if let Some(bucket) = tree.get(id.to_be_bytes())? {
            let bucket: Bucket = serde_json::from_slice(bucket.as_ref())?;
            return Ok(bucket);
        }
        Err(Waverr::BucketErr {
            id,
            ts_range: WaveDB::ts2key(ts_start),
            context: "failed to retrieve bucket",
        })
    }

    pub fn get_imw_id(
        &self,
        sig_name: String,
        sig_id: u32,
    ) -> Result<Arc<InMemWave>, Arc<Waverr>> {
        let buckets: Vec<Result<Bucket, Waverr>> = self
            .get_time_slices()
            .map(|start_slice| self.retrieve_bucket(sig_id, start_slice))
            .collect();

        InMemWave::new(sig_name, buckets)
            .map_err(|err| Arc::new(err))
            .map(|imw| Arc::new(imw))
    }

    pub fn get_imw(&self, sig: String) -> Result<Arc<InMemWave>, Arc<Waverr>> {
        let id = self.get_id(sig.as_str())?;
        self.get_imw_id(sig, id)
    }

    #[inline]
    fn ts2key(start: u32) -> String {
        let rounded_ts = start - (start % DEFAULT_SLIZE_SIZE);
        format!("{}-{}", rounded_ts, rounded_ts + DEFAULT_SLIZE_SIZE)
    }
}

#[cfg(test)]
#[allow(dead_code, unused_macros, unused_imports, unused_variables)]
mod tests {
    use crate::wavedb::*;
    use std::path::*;
    use crate::signals::SigType;
    use crate::*;


    #[test]
    fn bucket_serde() {
        let in_bucket = Bucket::default();
        let serialized = serde_json::to_string(&in_bucket).unwrap();

        let out_bucket: Bucket =
            serde_json::from_slice(serialized.as_ref()).unwrap();
    }

    #[test]
    #[allow(unused_must_use)]
    fn insert_sanity() {
        std::fs::remove_dir_all("TestDB");
        let tdb = WaveDB::new("TestDB".into(), None);
        let in_bucket = Bucket::default();
        match tdb.insert_bucket(&in_bucket) {
            Ok(()) => (),
            Err(err) => panic!("Inserting bucket sanity errored with {}", err),
        }

        let bucket = tdb.retrieve_bucket(0, 0);
        match bucket {
            Ok(payload) => (),
            Err(err) => {
                panic!("Retrieving buccket fails with {}", err);
            }
        }
    }

    #[test]
    #[allow(unused_must_use)]
    fn wdb_from_wikivcd() {
        let mut path_to_wikivcd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_to_wikivcd.push("test_vcds/wikipedia.vcd");
        //bad but hey... is what it is
        std::fs::remove_dir_all("/tmp/rng");
        let wdb =
            WaveDB::from_vcd(path_to_wikivcd.clone(), Path::new("/tmp/rng"));
        let actualdb = match wdb {
            Ok(wdb) => wdb,
            Err(errors::Waverr::VCDErr(vcdmess)) => {
                panic!("{} is the vcd error message", vcdmess)
            }
            Err(Waverr::GenericErr(message)) => {
                panic!("Unhandled error case: {} ", message)
            }
            Err(_) => panic!("Unhandled error case"),
        };
        let var = actualdb.get_imw("logic.data".into()).unwrap();
        assert_eq!(var.sig_type, SigType::Vector(8));
        drop(actualdb);

        // we need to test what happens when we're loading wdb from disk
        let wdb2 = WaveDB::from_vcd(path_to_wikivcd, Path::new("/tmp/rng"));
        let actualdb = match wdb2 {
            Ok(wdb2) => wdb2,
            Err(errors::Waverr::VCDErr(vcdmess)) => {
                panic!("{} is the vcd error message", vcdmess)
            }
            Err(Waverr::GenericErr(message)) => {
                panic!("Unhandled error case: {} ", message)
            }
            Err(_) => panic!("Unhandled error case"),
        };
        let var = actualdb.get_imw("logic.en".into()).unwrap();
        assert_eq!(var.as_ref().sig_type, SigType::Bit);
    }

    #[test]
    #[allow(unused_must_use)]
    fn wdb_from_vgavcd() {
        let mut path_to_wikivcd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_to_wikivcd.push("test_vcds/vga.vcd");
        //bad but hey... is what it is
        std::fs::remove_dir_all("/tmp/vcddb");
        let wdb = WaveDB::from_vcd(path_to_wikivcd, Path::new("/tmp/vcddb"))
            .expect("could not create wavedb");

        let var = wdb.get_imw("TOP.vga_g_DAC".into()).unwrap();
        assert_eq!(var.as_ref().sig_type, SigType::Vector(10));

        std::fs::remove_dir_all("/tmp/vcddb");
    }

}
