use bit_vec::BitVec;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::io;
use vcd::{ReferenceIndex, Value, Var, Command};
use bincode;

use std::path::Path;


use std::collections::HashMap;
pub mod errors;
mod vcd_parser;
use vcd_parser::WaveParser;

const DEFAULT_SLIZE_SIZE : u64= 10000;

#[derive(Serialize, Deserialize)]
enum SigIndices {
    BitSelect(u64),
    Range(u64, u64),
}


#[derive(Debug, Clone, Copy)]
pub enum SigType {
    Bit,
    Vector(u32),
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
    signal_content: Vec<(u64, ParsedVec)>,
    sig_type: SigType,
}




///DB for holding buckets
///
///slize_size determines the bounding of size of the timestamp range in each bucket
struct WaveDB {
    db_name: String,
    db: Db
}

impl WaveDB {
    fn new(db_name: String) -> WaveDB {
        WaveDB {
            db_name: db_name.clone(),
            db: sled::open(db_name.to_string()).unwrap() 
        }
    }


    //TODO: parallelize this
    fn from_file(vcd_file_path : String) -> Result<WaveDB, errors::Waverr> { 
        let parser = WaveParser::new(vcd_file_path.clone()).unwrap();
        let wdb_name = {
            if let Some(vcd_file) = Path::new(&vcd_file_path).file_stem() {
                vcd_file.to_str()
                    .unwrap_or(vcd_file_path.as_ref())
                    .to_string()
            } else{
                vcd_file_path
            }
        };
        let mut wdb = WaveDB::new(wdb_name);
        let mut global_time = 0;
        let mut current_range = (global_time, global_time + DEFAULT_SLIZE_SIZE);
        let mut BucketMappers : HashMap<vcd::IdCode,Bucket> = HashMap::new();

        for item in parser {
            match item {
                Ok(Command::Timestamp(time)) => {
                    if time % DEFAULT_SLIZE_SIZE < global_time % DEFAULT_SLIZE_SIZE {
                        for (_ , bucket) in BucketMappers.iter() {
                            wdb.insert_bucket(bucket);
                        }
                    }
                    global_time = time;
                },
                Ok(Command::ChangeVector(code, vvalue)) => {
                    if !BucketMappers.contains_key(&code) {
                        BucketMappers.insert(code, Bucket::new(code.0,current_range));
                    }
                    let bucket = BucketMappers.get_mut(&code).unwrap();
                    bucket.add_new_signal(global_time, vvalue);
                },
                Ok(Command::ChangeScalar(code, value)) => {
                    if !BucketMappers.contains_key(&code) {
                        BucketMappers.insert(code, Bucket::new(code.0,current_range));
                    }
                    let bucket = BucketMappers.get_mut(&code).unwrap();
                    bucket.add_new_signal(global_time, vec![value]);
                },
                Ok(_) => {},
                Err(_) => {
                    return Err(errors::Waverr::GenericErr("NOTHING".into()));
                }
            }
        }

        Err(errors::Waverr::GenericErr("NOTHING".into()))

    }


    fn insert_bucket(&self, bucket : &Bucket ) -> Result<(), errors::Waverr>  {
        let tree : sled::Tree= self.db.open_tree(bucket.get_db_idx())?;
        let serialized = bincode::serialize(&bucket)?;
        if let Ok(Some(value)) = tree.insert(bucket.id.to_be_bytes(), serialized) {
            return Err(errors::Waverr::GenericErr("Value exists in tree already".into()))
        }
        Ok(())
    }

    fn retrieve_bucket(&self, id: u64, ts_start: u64) -> Result<Bucket, errors::Waverr> {
        let tree = self.db.open_tree(WaveDB::ts2key(ts_start))?;
        if let Some(bucket) = tree.get(id.to_be_bytes())?  {
            let bucket : Bucket = bincode::deserialize(bucket.as_ref())?;
            return Ok(bucket);
        }
        Err("No bucket with that ID exists!".into())
        
    }


    fn get_tree_names(&self) -> Vec<sled::IVec>  {
        self.db.tree_names()
    }

    #[inline]
    fn ts2key(start : u64) -> String {
        let rounded_ts = start - (start % DEFAULT_SLIZE_SIZE);
        format!("{}-{}",rounded_ts, rounded_ts + DEFAULT_SLIZE_SIZE)
    }

}

///Chunk of a signal that is stored in wave2 db.
///
///If Indices is Some() then this bucket is slice of some larger signal
#[derive(Serialize, Deserialize,PartialEq,Debug)]
struct Bucket {
    timestamp_range: (u64, u64),
    id: u64,
    sig_dumps: Vec<(u64, ParsedVec)>,
}


impl Default for Bucket {
    fn default() -> Self {
        Bucket {
            timestamp_range : (0,10000),
            id : 0,
            sig_dumps : Vec::new()
        }
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
#[derive(Serialize, Deserialize,PartialEq,Debug)]
struct ParsedVec(BitVec, Option<BitVec>);

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
        format!("{}-{}",self.timestamp_range.0,self.timestamp_range.1)
    }

    fn new(id_: u64, stamps: (u64, u64)) -> Bucket {
        Bucket { timestamp_range: stamps, id: id_, sig_dumps: Vec::new() }
    }

    fn add_new_signal(&mut self, timestamp: u64, val_vec: Vec<Value>) {
        self.sig_dumps.push((timestamp, ParsedVec::from(val_vec)));
    }

    // fn add_dump(&mut self, timestamp: u64,  {
}

#[cfg(test)]
mod tests {
    use bit_vec::BitVec;
    use crate::backend::*;
    #[test]
    fn hello_test() {
        let mut bv = BitVec::from_elem(9, true);
        assert_eq!(true, true)
    }

    #[test]
    fn insert_sanity() {
        let mut tdb = WaveDB::new("TestDB".into());
        tdb.insert_bucket(&Bucket::default());
        let bucket = tdb.retrieve_bucket(0,0);
        match bucket {
            Ok(payload) => {
                assert_eq!(payload, Bucket::default());
            }
            Err(_) =>
            {
                panic!("Bucket not found-- fail");
            }

        }

    }

}
