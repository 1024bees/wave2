use crate::errors::Waverr;
use crate::hier_map::{HierMap, SignalItem};
use crate::puddle::builder::PuddleBuilder;
use crate::puddle::{Puddle, SignalId};
use crate::storage::in_memory::InMemWave;
use crate::vcd_parser::WaveParser;
use crate::MAX_PUDDLE_WIDTH;
use log::info;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use std::path::*;
use std::sync::Arc;
use vcd::Command;
#[derive(Serialize, Deserialize, Debug, Default)]
struct WdbConfig {
    db_name: String,
    time_range: (u32, u32),
}

///DB for holding buckets
///
///slize_size determines the bounding of size of the timestamp range in each bucket
#[derive(Debug)]
pub struct WaveDb {
    db: Db,
    //TODO: think about what should be wanted from a cfg file
    config: WdbConfig,
    puddle_cache: HashMap<SignalId, Arc<Puddle>>,
    pub hier_map: Arc<HierMap>,
}

impl WaveDb {
    fn new(db_name: String, db_path: Option<&Path>) -> WaveDb {
        WaveDb {
            db: sled::open(db_path.unwrap_or_else(|| db_name.as_ref())).unwrap(),
            hier_map: Arc::default(),
            puddle_cache: HashMap::default(),
            config: WdbConfig {
                db_name,
                ..WdbConfig::default()
            },
        }
    }

    fn get_sigitem(&self, sig: &str) -> Result<SignalItem, Waverr> {
        self.hier_map
            .path_to_signalref(sig)
            .map(|signal| signal.clone())
    }

    fn get_time_slices(&self) -> std::iter::StepBy<std::ops::Range<u32>> {
        info!("END TIME IS {}", self.config.time_range.0);
        ((self.config.time_range.0 / MAX_PUDDLE_WIDTH) * MAX_PUDDLE_WIDTH
            ..(self.config.time_range.1 / MAX_PUDDLE_WIDTH + 1) * MAX_PUDDLE_WIDTH)
            .step_by(MAX_PUDDLE_WIDTH as usize)
    }

    fn set_time_range(&mut self, range: (u32, u32)) {
        self.config.time_range = range;
    }

    fn load_config(&mut self) -> Result<(), Waverr> {
        if let Ok(Some(rawbytes)) = self.db.get("config") {
            let config: WdbConfig = toml::from_slice(rawbytes.as_ref())?;
            self.config = config;
            Ok(())
        } else {
            //TODO: maybe make specific error for this?
            Err(Waverr::WdbCfgErr("Config not found in WaveDB"))
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
            Err(Waverr::WdbCfgErr("HierMap not found in WaveDB"))
        }
    }

    pub fn get_bounds(&self) -> (u32, u32) {
        self.config.time_range
    }

    pub fn was_recovered(&self) -> bool {
        self.db.was_recovered()
    }

    pub fn open_wdb(wdb_path: &Path) -> Result<WaveDb, Waverr> {
        let mut wdb = WaveDb::new("TempName".into(), Some(wdb_path));
        wdb.load_config()?;
        wdb.load_idmap()?;
        Ok(wdb)
    }

    pub fn get_hier_map(&self) -> Arc<HierMap> {
        self.hier_map.clone()
    }

    //TODO: parallelize this
    pub fn from_vcd(vcd_file_path: PathBuf, wdb_path: &Path) -> Result<WaveDb, Waverr> {
        let mut parser = WaveParser::new(vcd_file_path.clone())?;
        let wdb_name = {
            if let Some(vcd_file) = vcd_file_path.file_stem() {
                vcd_file.to_str().unwrap().to_string()
            } else {
                vcd_file_path.to_str().unwrap().to_string()
            }
        };
        let mut wdb = WaveDb::new(wdb_name, Some(wdb_path));
        if wdb.was_recovered() {
            wdb.load_config()?;
            wdb.load_idmap()?;
            return Ok(wdb);
        }
        let mut first_time = None;
        let mut global_time: u32 = 0;
        let mut current_range = (global_time, global_time + MAX_PUDDLE_WIDTH);
        let mut inflight_puddles: HashMap<SignalId, PuddleBuilder> = HashMap::new();
        wdb.hier_map = Arc::new(parser.create_hiermap()?);
        for item in parser {
            match item {
                Ok(Command::Timestamp(time)) => {
                    let time = time as u32;
                    if time % MAX_PUDDLE_WIDTH < global_time % MAX_PUDDLE_WIDTH {
                        for (_, puddle) in inflight_puddles.into_iter() {
                            wdb.insert_puddle(puddle.into())?;
                        }
                        inflight_puddles = HashMap::new();
                        let rounded_time = time - (time % MAX_PUDDLE_WIDTH);
                        current_range = (rounded_time, rounded_time + MAX_PUDDLE_WIDTH)
                    }
                    if first_time.is_none() {
                        first_time = Some(time);
                    }
                    global_time = time;
                }
                //TODO: collapse these arms if possible? good way to share this code?
                Ok(command) => {
                    match command {
                        //TODO: add a get id function to the vcd lib that returns an option
                        Command::ChangeScalar(id, ..)
                        | Command::ChangeVector(id, ..)
                        | Command::ChangeReal(id, ..)
                        | Command::ChangeString(id, ..) => {
                            let base_id = id.0 as u32 - id.0 as u32 % Puddle::signals_per_puddle();
                            let puddle_builder = inflight_puddles
                                .entry(base_id)
                                .or_insert_with(|| PuddleBuilder::new(current_range.0));
                            puddle_builder.add_signal(command, global_time)?;
                        }
                        Command::Begin(_) => {}
                        Command::End(_) => {}
                        _ => return Err(Waverr::VcdCommandErr(command)),
                    }
                }
                Err(_) => {
                    return Err(Waverr::VcdErr("Malformed vcd"));
                }
            }
        }
        for (_, puddle) in inflight_puddles.into_iter() {
            wdb.insert_puddle(puddle.into())?;
        }

        wdb.set_time_range((
            first_time.expect("No timestamp present in VCD!"),
            global_time,
        ));
        wdb.dump_config()?;
        wdb.save_idmap()?;
        wdb.db.flush()?;
        Ok(wdb)
    }

    fn insert_puddle(&self, puddle: Puddle) -> Result<(), Waverr> {
        let tree: sled::Tree = self.db.open_tree(puddle.get_btree_idx().to_le_bytes())?;
        let serialized = serde_json::to_string(&puddle)?;

        info!(
            "Inserted a puddle at index {:?} with key {:?}",
            puddle.get_btree_idx(),
            puddle.get_base_sigid()
        );
        if let Ok(Some(_)) = tree.insert(puddle.get_base_sigid().to_le_bytes(), serialized.as_str())
        {
            // is problematic; implies that this value was previously set and we are
            // overwriting it. We should write only once per bucket
            return Err(Waverr::PuddleErr {
                time: puddle.get_btree_idx(),
                base_sigid: puddle.get_base_sigid(),
                context: "This puddle already exists! We should never double insert",
            });
        }
        Ok(())
    }

    fn retrieve_puddle(&self, id: u32, ts_start: u32) -> Result<Arc<Puddle>, Waverr> {
        let tree = self.db.open_tree(ts_start.to_le_bytes())?;
        let base_id = id - id % Puddle::signals_per_puddle();
        if let Some(puddle) = tree.get(base_id.to_le_bytes())? {
            let puddle: Puddle = serde_json::from_slice(puddle.as_ref())?;
            return Ok(Arc::new(puddle));
        }
        Err(Waverr::PuddleErr {
            time: ts_start,
            base_sigid: base_id,
            context: "failed to retrieve puddle",
        })
    }

    pub fn get_imw_sigitem(&self, sigitem: SignalItem) -> Result<Arc<InMemWave>, Arc<Waverr>> {
        let SignalItem {
            name: sig_name,
            id: sig_id,
            width,
        } = sigitem;

        let puddles = self
            .get_time_slices()
            .map(|start_slice| self.retrieve_puddle(sig_id, start_slice).unwrap())
            .collect();

        InMemWave::new(sig_name, sig_id, width, puddles)
            .map_err(Arc::new)
            .map(Arc::new)
    }

    pub fn get_imw(&self, sig: String) -> Result<Arc<InMemWave>, Arc<Waverr>> {
        let sigitem = self.get_sigitem(sig.as_str())?;
        self.get_imw_sigitem(sigitem)
    }
}

#[cfg(test)]
#[allow(dead_code, unused_macros, unused_imports, unused_variables)]
mod tests {
    use crate::signals::SigType;
    use crate::wavedb::*;
    use crate::*;
    use log::info;
    use std::path::*;

    fn init_test_logger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init();
    }

    //#[test]
    //#[allow(unused_must_use)]
    //fn insert_sanity() {
    //    std::fs::remove_dir_all("TestDB");
    //    let tdb = WaveDB::new("TestDB".into(), None);
    //    let in_bucket = Bucket::default();
    //    match tdb.insert_bucket(&in_bucket) {
    //        Ok(()) => (),
    //        Err(err) => panic!("Inserting bucket sanity errored with {}", err),
    //    }

    //    let bucket = tdb.retrieve_bucket(0, 0);
    //    match bucket {
    //        Ok(payload) => (),
    //        Err(err) => {
    //            panic!("Retrieving buccket fails with {}", err);
    //        }
    //    }
    //}

    #[test]
    #[allow(unused_must_use)]
    fn wdb_from_wikivcd() {
        init_test_logger();
        info!("GREETINGS");
        let mut path_to_wikivcd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_to_wikivcd.push("test_vcds/wikipedia.vcd");
        //bad but hey... is what it is
        std::fs::remove_dir_all("/tmp/rng");
        let wdb = WaveDb::from_vcd(path_to_wikivcd.clone(), Path::new("/tmp/rng"));
        let actualdb = match wdb {
            Ok(wdb) => wdb,
            Err(errors::Waverr::VcdErr(vcdmess)) => {
                panic!("{} is the vcd error message", vcdmess)
            }
            Err(Waverr::GenericErr(message)) => {
                panic!("Unhandled error case: {} ", message)
            }
            Err(err) => panic!("Unhandled error case: {:?}", err),
        };
        let var = actualdb.get_imw("logic.data".into()).unwrap();
        drop(actualdb);

        // we need to test what happens when we're loading wdb from disk
        let wdb2 = WaveDb::from_vcd(path_to_wikivcd, Path::new("/tmp/rng"));
        let actualdb = match wdb2 {
            Ok(wdb2) => wdb2,
            Err(errors::Waverr::VcdErr(vcdmess)) => {
                panic!("{} is the vcd error message", vcdmess)
            }
            Err(Waverr::GenericErr(message)) => {
                panic!("Unhandled error case: {} ", message)
            }
            Err(err) => panic!("Unhandled error case: {:?}", err),
        };
        let var = actualdb.get_imw("logic.en".into()).unwrap();
    }

    #[test]
    #[allow(unused_must_use)]
    fn wdb_from_vgavcd() {
        init_test_logger();

        let mut path_to_wikivcd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_to_wikivcd.push("test_vcds/vga.vcd");
        //bad but hey... is what it is
        std::fs::remove_dir_all("/tmp/vcddb");
        let wdb = WaveDb::from_vcd(path_to_wikivcd, Path::new("/tmp/vcddb"))
            .expect("could not create wavedb");

        let var = wdb
            .get_imw("TOP.clock".into())
            .expect("signal doesn't exist and it definitely should!!");

        let val: (u32, &[u8]) = var.all_data().next().unwrap();
        info!("len is val.1: {}", val.0);
        //assert!(val.1.len() == 8);

        std::fs::remove_dir_all("/tmp/vcddb");
    }
}
