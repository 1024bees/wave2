use bincode;
use crate::errors::Waverr;
use crate::vcd_parser::{IDMap, WaveParser};
use crate::{Bucket, InMemWave, DEFAULT_SLIZE_SIZE};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use std::path::*;
use toml;
use vcd::Command;

#[derive(Serialize, Deserialize, Debug, Default)]
struct WDBConfig {
    db_name: String,
    populated: bool,
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
    id_map: IDMap,
}

impl WaveDB {
    fn new(db_name: String, db_path: Option<&Path>) -> WaveDB {
        WaveDB {
            db: sled::open(db_path.unwrap_or(db_name.as_ref())).unwrap(),
            id_map: IDMap::default(),
            config: WDBConfig {
                db_name: db_name.clone(),
                ..WDBConfig::default()
            },
        }
    }

    fn get_id(&self, sig: &str) -> Result<u32, Waverr> {
        self.id_map.signal_to_id(sig)
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
            Err(Waverr::WDBCfgErr("Error loading config".into()))
        }
    }

    fn dump_config(&self) -> Result<(), Waverr> {
        self.db
            .insert("config", toml::to_string(&self.config)?.as_str())?;
        Ok(())
    }

    fn save_idmap(&self) -> Result<(), Waverr> {
        self.db.insert("id_map", bincode::serialize(&self.id_map)?);
        Ok(())
    }

    fn load_idmap(&mut self) -> Result<(), Waverr> {
        if let Ok(Some(rawbytes)) = self.db.get("id_map") {
            self.id_map = bincode::deserialize(rawbytes.as_ref())?;
            Ok(())
        } else {
            Err(Waverr::WDBCfgErr("Error loading config from DB"))
        }
    }

    pub fn open_wdb(wdb_path: &Path) -> Result<WaveDB, Waverr> {
        let mut wdb = WaveDB::new("TempName".into(), Some(wdb_path));
        wdb.load_config()?;
        wdb.load_idmap()?;
        Ok(wdb)
    }

    //TODO: parallelize this
    //TODO: move filepath from String to &str!!!
    pub fn from_vcd(
        vcd_file_path: PathBuf,
        wdb_path: &Path,
    ) -> Result<WaveDB, Waverr> {

        let parser = WaveParser::new(vcd_file_path.clone())?;
        let wdb_name = {
            if let Some(vcd_file) = vcd_file_path.file_stem() {
                vcd_file
                    .to_str()
                    .unwrap()
                    .to_string()
            } else {
                vcd_file_path
                    .to_str()
                    .unwrap()
                    .to_string()

            }
        };
        let mut wdb = WaveDB::new(wdb_name, Some(wdb_path));
        let mut global_time: u32 = 0;
        let mut current_range = (global_time, global_time + DEFAULT_SLIZE_SIZE);
        let mut bucket_mapper: HashMap<vcd::IdCode, Bucket> = HashMap::new();
        wdb.id_map = parser.create_idmap();
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
                    global_time = time;
                }
                //TODO: collapse these arms if possible? good way to share this code?
                Ok(Command::ChangeVector(code, vvalue)) => {
                    if !bucket_mapper.contains_key(&code) {
                        bucket_mapper.insert(
                            code,
                            Bucket::new(code.0 as u32, current_range),
                        );
                    }
                    let bucket = bucket_mapper.get_mut(&code).unwrap();
                    bucket.add_new_signal(global_time, vvalue);
                }
                Ok(Command::ChangeScalar(code, value)) => {
                    if !bucket_mapper.contains_key(&code) {
                        bucket_mapper.insert(
                            code,
                            Bucket::new(code.0 as u32, current_range),
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

        wdb.set_time_range((0, global_time));

        Ok(wdb)
    }

    fn insert_bucket(&self, bucket: &Bucket) -> Result<(), Waverr> {
        let tree: sled::Tree = self.db.open_tree(bucket.get_db_idx())?;
        let serialized = bincode::serialize(&bucket)?;
        if let Ok(Some(_)) = tree.insert(bucket.id.to_be_bytes(), serialized) {
            return Err(Waverr::BucketErr {
                id : bucket.id,
                ts : bucket.timestamp_range.0
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
            let bucket: Bucket = bincode::deserialize(bucket.as_ref())?;
            return Ok(bucket);
        }
        Err(Waverr::BucketErr{ id,ts:ts_start})
    }

    pub fn get_imw(&self, sig: &str) -> Result<InMemWave, Waverr> {
        let id = self.get_id(sig)?;
        let buckets: Vec<Result<Bucket, Waverr>> = self
            .get_time_slices()
            .map(|start_slice| self.retrieve_bucket(id, start_slice))
            .collect();

        InMemWave::new(sig, buckets)
    }

    #[inline]
    fn ts2key(start: u32) -> String {
        let rounded_ts = start - (start % DEFAULT_SLIZE_SIZE);
        format!("{}-{}", rounded_ts, rounded_ts + DEFAULT_SLIZE_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use crate::wavedb::*;
    use bit_vec::BitVec;
    use std::path::*;
    #[test]
    fn insert_sanity() {
        let mut tdb = WaveDB::new("TestDB".into(), None);
        tdb.insert_bucket(&Bucket::default());
        let bucket = tdb.retrieve_bucket(0, 0);
        match bucket {
            Ok(payload) => {
                assert_eq!(payload, Bucket::default());
            }
            Err(_) => {
                panic!("Bucket not found-- fail");
            }
        }
    }
}
