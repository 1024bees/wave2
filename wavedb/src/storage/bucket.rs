use serde::{Deserialize, Serialize};
use vcd::Value;
use crate::signals::{SigType,ParsedVec};

///Chunk of a signal that is stored in wave2 db; on disk signal data structure
#[derive(Serialize, Deserialize, Debug)]
pub struct Bucket {
    timestamp_range: (u32, u32),
    pub id: u32,
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



impl Bucket {
    pub fn get_db_idx(&self) -> String {
        format!("{}-{}", self.timestamp_range.0, self.timestamp_range.1)
    }

    pub fn signals(&mut self) -> &mut Vec<(u32,ParsedVec)> {
        &mut self.sig_dumps
    }

    pub fn signal_id(&self) -> u32 { self.id}

    pub fn signal_type(&self) -> SigType {
        self.sig_type
    }

    pub fn new(id_: u32, width: usize, stamps: (u32, u32)) -> Bucket {
        Bucket {
            timestamp_range: stamps,
            id: id_,
            sig_type: SigType::from_width(width),
            sig_dumps: Vec::new(),
        }
    }

    pub fn add_new_signal(&mut self, timestamp: u32, val_vec: Vec<Value>) {
        self.sig_dumps.push((timestamp, ParsedVec::from(val_vec)));
    }
}

