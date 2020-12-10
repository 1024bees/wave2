use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use vcd::Value;
pub mod api;
pub mod errors;
pub mod hier_map;
pub mod inout;
mod vcd_parser;
pub mod wavedb;
use errors::Waverr;
const DEFAULT_SLIZE_SIZE: u32 = 10000;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum SigType {
    Bit,
    Vector(usize),
}

impl SigType {
    fn from_width(width: usize) -> SigType {
        match width {
            1 => SigType::Bit,
            bw => SigType::Vector(bw),
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
        name_str: String,
        buckets: Vec<Result<Bucket, Waverr>>,
    ) -> Result<InMemWave, Waverr> {
        let mut signal_content = Vec::new();
        //TODO: can parallelize
        let mut st = None;
        for bucket in buckets {
            match bucket {
                Ok(mut bucket) => {
                    signal_content.append(&mut bucket.sig_dumps);
                    if st.is_none() {
                        st = Some(bucket.sig_type)
                    }
                }
                Err(Waverr::BucketErr { .. }) => (),
                Err(bucket_err) => (return Err(bucket_err)),
            }
        }

        Ok(InMemWave {
            name: name_str.into(),
            signal_content: signal_content,
            sig_type: st.unwrap(),
        })
    }
}

///Chunk of a signal that is stored in wave2 db; on disk signal data structure
#[derive(Serialize, Deserialize, Debug)]
struct Bucket {
    timestamp_range: (u32, u32),
    id: u32,
    sig_type: SigType,
    sig_dumps: Vec<(u32, ParsedVec)>,
}

impl Default for Bucket {
    fn default() -> Self {
        Bucket {
            timestamp_range: (0, 10000),
            id: 0,
            sig_type: SigType::Vector(4),
            sig_dumps: vec![(0, ParsedVec::from(7)), (4, ParsedVec::from(3))],
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
#[derive(Debug, Serialize, Deserialize)]
pub enum ParsedVec {
    WordVec(FourStateBitArr),
    WideVec(FourStateBitVec),
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct FourStateBitArr {
    value_bits: BitArray<LocalBits, [usize; 1]>,
    zx_bits: Option<BitArray<LocalBits, [usize; 1]>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct FourStateBitVec {
    value_bits: BitVec<LocalBits>,
    zx_bits: Option<BitVec<LocalBits>>,
}

macro_rules! from_vcd_vec {
    ($([$t:ident,$ut:ident]),*) => {
        $(impl From<Vec<Value>> for $t {

            fn from(vec_val : Vec<Value>) -> $t {
                let mut vb  = $ut::default();
                let mut zx = None;

                for (bidx, bit) in vec_val.iter().enumerate() {
                    match bit {
                        Value::V1 => vb.set(bidx, true),
                        Value::X => {
                            vb.set(bidx, true);
                            if zx == Option::None {
                                zx =
                                    Some($ut::default());
                            }
                            zx.as_mut().unwrap().set(bidx, true);
                        }
                        Value::Z => {
                            if zx == Option::None {
                                zx =
                                    Some($ut::default());
                            }
                            zx.as_mut().unwrap().set(bidx, true);
                        }
                        Value::V0 => (),
                    }
                }
            $t { value_bits : vb, zx_bits: zx}
            }
        })*
    };
}

from_vcd_vec!([FourStateBitArr, BitArray], [FourStateBitVec, BitVec]);

//impl From<Vec<Value>> for FourStateBitArr {
//    fn from(vec_val : Vec<Value>) -> FourStateBitArr {
//        let mut rv = FourStateBitArr::default();
//        let mut vb  = BitArray::default();
//        let mut zx = None;
//
//        for (bidx, bit) in vec_val.iter().enumerate() {
//            match bit {
//                Value::V1 => vb.set(bidx, true),
//                Value::X => {
//                    vb.set(bidx, true);
//                    if zx == Option::None {
//                        zx =
//                            Some(BitArray::default());
//                    }
//                    zx.as_mut().unwrap().set(bidx, true);
//                }
//                Value::Z => {
//                    if zx == Option::None {
//                        zx =
//                            Some(BitArray::default());
//                    }
//                    zx.as_mut().unwrap().set(bidx, true);
//                }
//                Value::V0 => (),
//            }
//        }
//
//    FourStateBitArr{ value_bits : vb, zx_bits: zx}
//    }
//}

impl ParsedVec {
    pub fn get_bv(&self) -> Option<bool> {
        match self {
            ParsedVec::WordVec(payload) => {
                let FourStateBitArr {
                    value_bits,
                    zx_bits,
                } = payload;
                if let Some(_) = zx_bits {
                    None
                } else {
                    Some(value_bits.get(0).unwrap().clone())
                }
            }
            _ => None,
        }
    }
}

impl From<u8> for ParsedVec {
    fn from(vec_val: u8) -> ParsedVec {
        let mut fbv = FourStateBitArr::default();
        fbv.value_bits = [vec_val as usize].into();
        ParsedVec::WordVec(fbv)
    }
}

impl From<Vec<Value>> for ParsedVec {
    fn from(vec_val: Vec<Value>) -> ParsedVec {
        match vec_val.len() {
            1..=32 => ParsedVec::WordVec(FourStateBitArr::from(vec_val)),
            _ => ParsedVec::WideVec(FourStateBitVec::from(vec_val)),
        }
    }
}

impl Bucket {
    fn get_db_idx(&self) -> String {
        format!("{}-{}", self.timestamp_range.0, self.timestamp_range.1)
    }

    fn new(id_: u32, width: usize, stamps: (u32, u32)) -> Bucket {
        Bucket {
            timestamp_range: stamps,
            id: id_,
            sig_type: SigType::from_width(width),
            sig_dumps: Vec::new(),
        }
    }

    fn add_new_signal(&mut self, timestamp: u32, val_vec: Vec<Value>) {
        self.sig_dumps.push((timestamp, ParsedVec::from(val_vec)));
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::mem::drop;
    use std::path::Path;
    use std::path::*;
    use wavedb::WaveDB;

    #[test]
    fn serde_4bit_arr() {
        let mut fbv = FourStateBitArr::default();
        fbv.value_bits = [0xffff as usize].into();
        let bytes = serde_json::to_string(&fbv).unwrap();
        serde_json::from_str::<FourStateBitArr>(bytes.as_ref()).expect(
            format!("failed to deserialize, bytes are {:#?}", bytes).as_str(),
        );
    }

    //#[test]
    //fn serde_parsed_vec() {
    //    let pv = ParsedVec::from(0xFF);
    //    let bytes = bincode::serialize(&pv).unwrap();
    //    match bincode::deserialize::<ParsedVec>(bytes.as_ref()) {
    //        Ok(pv) => (),
    //        Err(err) => panic!("err is {}, failed deserialize! bytes are {:#?}",err,bytes)
    //    }
    //}

    #[test]
    #[allow(unused_must_use)]
    fn wdb_from_wikivcd() {
        let mut path_to_wikivcd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_to_wikivcd.push("test_vcds/wikipedia.vcd");
        //bad but hey... is what it is
        std::fs::remove_dir_all("/tmp/rng").expect("could not clean wavedb");
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
        std::fs::remove_dir_all("/tmp/vcddb")
            .expect("could not clear old wavedb");
        let wdb = WaveDB::from_vcd(path_to_wikivcd, Path::new("/tmp/vcddb"))
            .expect("could not create wavedb");

        let var = wdb.get_imw("TOP.vga_g_DAC".into()).unwrap();
        assert_eq!(var.as_ref().sig_type, SigType::Vector(10));

        std::fs::remove_dir_all("/tmp/vcddb");
    }
}
