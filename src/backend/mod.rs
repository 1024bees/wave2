use bincode;
use bit_vec::BitVec;
use cached::proc_macro::cached;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use std::io;
use std::path::Path;
use toml;
use vcd::{Command, ReferenceIndex, Value, Var};
mod errors;
mod vcd_parser;
use errors::Waverr;
use vcd_parser::{IDMap, WaveParser};

const DEFAULT_SLIZE_SIZE: u32 = 10000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SigType {
    Bit,
    Vector(u32),
}

impl SigType {
    fn from_width(width: u32) -> SigType {
        match width {
            1 => SigType::Bit,
            bw => SigType::Vector(bw),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Wave {
    name: String,
    pub signal_content: Vec<(u32, u32)>,
    pub sig_type: SigType,
}

//TODO: move to backend, make inmemory wave
impl Default for Wave {
    fn default() -> Self {
        Wave {
            name: String::from("PlaceholderWave"),
            signal_content: vec![(0, 1), (10, 0), (20, 1), (30, 0), (50, 1), (500, 0)],
            sig_type: SigType::Bit,
        }
    }
}

//TODO: move from Wave -> InMemoryWave... should there be a transform there even?
impl Wave {
    pub fn default_vec() -> Self {
        Wave { sig_type: SigType::Vector(4), ..Wave::default() }
    }
}

pub struct InMemWave {
    name: String,
    signal_content: Vec<(u32, ParsedVec)>,
    sig_type: SigType,
}

impl InMemWave {
    fn new(name_str: &str, buckets: Vec<Result<Bucket, Waverr>>) -> Result<InMemWave, Waverr> {
        let mut signal_content = Vec::new();
        for bucket in buckets {
            match bucket {
                Ok(mut bucket) => signal_content.append(&mut bucket.sig_dumps),
                Err(Waverr::MissingID(_)) => (),
                Err(bucket_err) => return Err(bucket_err.clone()),
            }
        }

        let st = SigType::from_width(signal_content.first().unwrap().1.len());
        Ok(InMemWave { name: name_str.into(), signal_content: signal_content, sig_type: st })
    }
}

#[derive(Serialize, Deserialize, Default)]
struct WDBConfig {
    db_name: String,
    populated: bool,
    time_range: (u32, u32),
}

///DB for holding buckets
///
///slize_size determines the bounding of size of the timestamp range in each bucket
struct WaveDB {
    db: Db,
    //TODO: think about what should be wanted from a cfg file
    config: WDBConfig,
    id_map: IDMap,
}

impl WaveDB {
    fn new(db_name: String, db_path: Option<&str>) -> WaveDB {
        WaveDB {
            db: sled::open(db_path.unwrap_or(db_name.as_ref())).unwrap(),
            id_map: IDMap::default(),
            config: WDBConfig { db_name: db_name.clone(), ..WDBConfig::default() },
        }
    }

    fn get_id(&self, sig: &str) -> Result<u32, Waverr> {
        self.id_map.signal_to_id(sig)
    }

    fn get_time_slices(&self) -> std::iter::StepBy<std::ops::Range<u32>> {
        ((self.config.time_range.0 / DEFAULT_SLIZE_SIZE) * DEFAULT_SLIZE_SIZE
            ..(self.config.time_range.1 / DEFAULT_SLIZE_SIZE + 2) * DEFAULT_SLIZE_SIZE)
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
            Err(Waverr::SledError("Error loading config".into()))
        }
    }

    fn dump_config(&self) -> Result<(), Waverr> {
        self.db.insert("config", toml::to_string(&self.config)?.as_str())?;
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
            Err(Waverr::SledError("Error loading config from DB"))
        }
    }

    pub fn open_wdb(wdb_path: &str) -> Result<WaveDB, Waverr> {
        let mut wdb = WaveDB::new("TempName".into(), Some(wdb_path));
        wdb.load_config()?;
        wdb.load_idmap()?;
        Ok(wdb)
    }

    //TODO: parallelize this
    pub fn from_vcd(vcd_file_path: String, wdb_path: &str) -> Result<WaveDB, Waverr> {
        if Path::new(wdb_path).exists() {}

        let parser = WaveParser::new(vcd_file_path.clone())?;
        let wdb_name = {
            if let Some(vcd_file) = Path::new(&vcd_file_path).file_stem() {
                vcd_file.to_str().unwrap_or(vcd_file_path.as_ref()).to_string()
            } else {
                vcd_file_path
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
                    if time % DEFAULT_SLIZE_SIZE < global_time % DEFAULT_SLIZE_SIZE {
                        for (_, bucket) in bucket_mapper.iter() {
                            wdb.insert_bucket(bucket)?;
                        }
                        bucket_mapper.clear();
                        let rounded_time = time - (time % DEFAULT_SLIZE_SIZE);
                        current_range = (rounded_time, rounded_time + DEFAULT_SLIZE_SIZE)
                    }
                    global_time = time;
                }
                //TODO: collapse these arms if possible? good way to share this code?
                Ok(Command::ChangeVector(code, vvalue)) => {
                    if !bucket_mapper.contains_key(&code) {
                        bucket_mapper.insert(code, Bucket::new(code.0 as u32, current_range));
                    }
                    let bucket = bucket_mapper.get_mut(&code).unwrap();
                    bucket.add_new_signal(global_time, vvalue);
                }
                Ok(Command::ChangeScalar(code, value)) => {
                    if !bucket_mapper.contains_key(&code) {
                        bucket_mapper.insert(code, Bucket::new(code.0 as u32, current_range));
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
        if let Ok(Some(value)) = tree.insert(bucket.id.to_be_bytes(), serialized) {
            return Err(errors::Waverr::GenericErr("Value exists in tree already".into()));
        }
        Ok(())
    }

    fn retrieve_bucket(&self, id: u32, ts_start: u32) -> Result<Bucket, Waverr> {
        let tree = self.db.open_tree(WaveDB::ts2key(ts_start))?;
        if let Some(bucket) = tree.get(id.to_be_bytes())? {
            let bucket: Bucket = bincode::deserialize(bucket.as_ref())?;
            return Ok(bucket);
        }
        Err(Waverr::MissingID("No bucket with that ID exists for this ts range!"))
    }

    pub fn get_imw(&self, sig: &str) -> Result<InMemWave, Waverr> {
        let id = self.get_id(sig)?;
        let Buckets: Vec<Result<Bucket, Waverr>> = self
            .get_time_slices()
            .map(|start_slice| self.retrieve_bucket(id, start_slice))
            .collect();

        InMemWave::new(sig, Buckets)
    }

    #[inline]
    fn ts2key(start: u32) -> String {
        let rounded_ts = start - (start % DEFAULT_SLIZE_SIZE);
        format!("{}-{}", rounded_ts, rounded_ts + DEFAULT_SLIZE_SIZE)
    }
}

///Chunk of a signal that is stored in wave2 db.
///
///If Indices is Some() then this bucket is slice of some larger signal
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Bucket {
    timestamp_range: (u32, u32),
    id: u32,
    sig_dumps: Vec<(u32, ParsedVec)>,
}

impl Default for Bucket {
    fn default() -> Self {
        Bucket { timestamp_range: (0, 10000), id: 0, sig_dumps: Vec::new() }
    }
}

/// Most simulators are 4 state, where any signal can be 0,1,z or x
/// We expect signals to be driven, so we optimize for that case
///
///
/// to represent the four states, we have two parallel bit vectors
/// ParsedVec.0[n] -> the 0th "state" bit for the nth signal bit
/// ParsedVec.1[n] -> the 1st "state" bit for the nth signal bit
///
/// If ParsedVec.1 == Option::None, the 1st bit is zero
///
/// We have the following mapping
/// 00 -> 0
/// 01 -> 1
/// 10 -> Z
/// 11 -> X
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct ParsedVec(BitVec, Option<BitVec>);
impl ParsedVec {
    fn len(&self) -> u32 {
        self.0.len() as u32
    }
}

impl From<Vec<Value>> for ParsedVec {
    fn from(vec_val: Vec<Value>) -> ParsedVec {
        let mut parsed_vec = ParsedVec(BitVec::from_elem(vec_val.len(), false), Option::None);
        let ref mut option_vec = parsed_vec.1;
        for (bidx, bit) in vec_val.iter().enumerate() {
            match bit {
                Value::V1 => parsed_vec.0.set(bidx, true),
                Value::X => {
                    parsed_vec.0.set(bidx, true);
                    if *option_vec == Option::None {
                        *option_vec = Some(BitVec::from_elem(vec_val.len(), false));
                    }
                    option_vec.as_mut().unwrap().set(bidx, true);
                }
                Value::Z => {
                    if *option_vec == Option::None {
                        *option_vec = Some(BitVec::from_elem(vec_val.len(), false));
                    }
                    option_vec.as_mut().unwrap().set(bidx, true);
                }
                Value::V0 => (),
            }
        }
        parsed_vec
    }
}

impl Bucket {
    fn get_db_idx(&self) -> String {
        format!("{}-{}", self.timestamp_range.0, self.timestamp_range.1)
    }

    fn new(id_: u32, stamps: (u32, u32)) -> Bucket {
        Bucket { timestamp_range: stamps, id: id_, sig_dumps: Vec::new() }
    }

    fn add_new_signal(&mut self, timestamp: u32, val_vec: Vec<Value>) {
        self.sig_dumps.push((timestamp, ParsedVec::from(val_vec)));
    }

    // fn add_dump(&mut self, timestamp: u32,  {
}

#[cfg(test)]
mod tests {
    use crate::backend::*;
    use bit_vec::BitVec;
    use std::path::*;
    #[test]
    fn hello_test() {
        let mut bv = BitVec::from_elem(9, true);
        assert_eq!(true, true)
    }

    #[test]
    fn wdb_from_wikivcd() {
        let mut path_to_wikivcd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_to_wikivcd.push("test_vcds/wikipedia.vcd");
        let pathstr = path_to_wikivcd.into_os_string().into_string().unwrap();
        println!("{}", pathstr);
        //a little naughty but hey... is what it is
        std::fs::remove_dir_all("/tmp/rng");
        let wdb = WaveDB::from_vcd(pathstr, "/tmp/rng");
        let actualdb = match wdb {
            Ok(wdb) => wdb,
            Err(errors::Waverr::VCDErr(vcdmess)) => panic!("{} is the vcd error message", vcdmess),
            Err(Waverr::GenericErr(message)) => panic!("Unhandled error case: {} ", message),
            Err(_) => panic!("Unhandled error case"),
        };
        let var = actualdb.get_imw("logic.data").unwrap();
        assert_eq!(var.sig_type, SigType::Vector(8));
    }

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
