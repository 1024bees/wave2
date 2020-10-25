use bit_vec::BitVec;
use serde::{Deserialize, Serialize};
use vcd::Value;
pub mod api;
mod errors;
pub mod hier_map;
mod vcd_parser;
pub mod wavedb;
use errors::Waverr;
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
            signal_content: vec![
                (0, 1),
                (10, 0),
                (20, 1),
                (30, 0),
                (50, 1),
                (500, 0),
            ],
            sig_type: SigType::Bit,
        }
    }
}

//TODO: move from Wave -> InMemoryWave... should there be a transform there even?
impl Wave {
    pub fn default_vec() -> Self {
        Wave {
            sig_type: SigType::Vector(4),
            ..Wave::default()
        }
    }
}

#[derive(Debug)]
pub struct InMemWave {
    name: String,
    signal_content: Vec<(u32, ParsedVec)>,
    pub sig_type: SigType,
}

impl Default for InMemWave {
    fn default() -> Self {
        InMemWave {
            name: String::from("PlaceholderWave"),
            signal_content: vec![
                (0, ParsedVec::from(0)),
                (10, ParsedVec::from(1)),
                (20, ParsedVec::from(0)),
                (30, ParsedVec::from(1)),
                (50, ParsedVec::from(0)),
                (500, ParsedVec::from(1)),
            ],
            sig_type: SigType::Bit,
        }
    }
}

impl std::fmt::Display for InMemWave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.as_str())
    }
}

///In memory DS for wave content; created from a list of Buckets
impl InMemWave {
    pub fn default_vec() -> Self {
        InMemWave {
            sig_type: SigType::Vector(4),
            ..InMemWave::default()
        }
    }
    pub fn changes(&self) -> std::slice::Iter<'_, (u32, ParsedVec)> {
        self.signal_content.iter()
    }

    fn new(
        name_str: &str,
        buckets: Vec<Result<Bucket, Waverr>>,
    ) -> Result<InMemWave, Waverr> {
        let mut signal_content = Vec::new();
        //TODO: can parallelize
        for bucket in buckets {
            match bucket {
                Ok(mut bucket) => signal_content.append(&mut bucket.sig_dumps),
                Err(Waverr::MissingID(_)) => (),
                Err(bucket_err) => return Err(bucket_err.clone()),
            }
        }

        let st = SigType::from_width(signal_content.first().unwrap().1.len());
        Ok(InMemWave {
            name: name_str.into(),
            signal_content: signal_content,
            sig_type: st,
        })
    }
}

///Chunk of a signal that is stored in wave2 db; on disk signal data structure
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Bucket {
    timestamp_range: (u32, u32),
    id: u32,
    sig_dumps: Vec<(u32, ParsedVec)>,
}

impl Default for Bucket {
    fn default() -> Self {
        Bucket {
            timestamp_range: (0, 10000),
            id: 0,
            sig_dumps: Vec::new(),
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
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ParsedVec(BitVec, Option<BitVec>);
impl ParsedVec {
    fn len(&self) -> u32 {
        self.0.len() as u32
    }
    pub fn get_bv(&self) -> bool {
        self.0.get(7).unwrap()
    }
}

impl From<u8> for ParsedVec {
    fn from(vec_val: u8) -> ParsedVec {
        ParsedVec(BitVec::from_bytes(&[vec_val]), None)
    }
}

impl From<Vec<Value>> for ParsedVec {
    fn from(vec_val: Vec<Value>) -> ParsedVec {
        let mut parsed_vec =
            ParsedVec(BitVec::from_elem(vec_val.len(), false), Option::None);
        let ref mut option_vec = parsed_vec.1;
        for (bidx, bit) in vec_val.iter().enumerate() {
            match bit {
                Value::V1 => parsed_vec.0.set(bidx, true),
                Value::X => {
                    parsed_vec.0.set(bidx, true);
                    if *option_vec == Option::None {
                        *option_vec =
                            Some(BitVec::from_elem(vec_val.len(), false));
                    }
                    option_vec.as_mut().unwrap().set(bidx, true);
                }
                Value::Z => {
                    if *option_vec == Option::None {
                        *option_vec =
                            Some(BitVec::from_elem(vec_val.len(), false));
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
        Bucket {
            timestamp_range: stamps,
            id: id_,
            sig_dumps: Vec::new(),
        }
    }

    fn add_new_signal(&mut self, timestamp: u32, val_vec: Vec<Value>) {
        self.sig_dumps.push((timestamp, ParsedVec::from(val_vec)));
    }

    // fn add_dump(&mut self, timestamp: u32,  {
}

#[cfg(test)]
mod tests {
    use crate::*;
    use bit_vec::BitVec;
    use std::path::*;
    use wavedb::WaveDB;
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
            Err(errors::Waverr::VCDErr(vcdmess)) => {
                panic!("{} is the vcd error message", vcdmess)
            }
            Err(Waverr::GenericErr(message)) => {
                panic!("Unhandled error case: {} ", message)
            }
            Err(_) => panic!("Unhandled error case"),
        };
        let var = actualdb.get_imw("logic.data").unwrap();
        assert_eq!(var.sig_type, SigType::Vector(8));
    }
}
