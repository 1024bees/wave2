use super::{Toffset, SignalId};
use std::collections::HashMap;
use vcd::{Command,Value};
use crate::errors::Waverr;
use std::convert::TryFrom;
use super::utils::get_id;
use super::Puddle;
use crate::signals::SigType;

/// Transient payload; this is the temporary container that is accumulated into as 
/// a vcd is parsed; TODO: make this sync? in the future it would be nice to build these out in
/// parallel
#[derive(Default)]
struct RunningPayload  {
    data : Vec<u8>,
    num_items: u16,
    sig_type: SigType,
    variable_len: bool,
    
}

#[derive(Default)]
pub struct PuddleBuilder {
    base : Toffset,
    payloads: HashMap<SignalId,RunningPayload>,
}

impl Extend<u8> for RunningPayload {
    fn extend<T: IntoIterator<Item=u8>>(&mut self, iter: T) { 
        self.data.extend(iter)
    }
}


impl Extend<Value> for RunningPayload {
    fn extend<T: IntoIterator<Item=Value>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let data_size = (iter.size_hint().0 as f32 / 8.0).ceil() as usize;
        let mut zx_base : Option<usize>= None;
        let base = self.data.len();
        self.data.resize(self.data.len() + data_size,0);


        for (bidx,value) in iter.into_iter().enumerate() {
            let bit_offset = bidx & 0x7;
            let byte_offset = base + bidx >> 3;
            match value {
                Value::V1 => self.data[byte_offset] |= 1 << bit_offset, 
                Value::X | Value::Z => {
                    if value == Value::X {
                        self.data[byte_offset] |= 1 << bit_offset;
                    };
                    if zx_base.is_none() {
                        zx_base = Some(self.data.len());
                        self.data.resize(self.data.len() + data_size,0);
                        self.variable_len = true;
                    }
                    let zx_byte_offset = byte_offset + data_size;
                    self.data[zx_byte_offset] |= 1 << bit_offset;
                }
                Value::V0 => (),
            }
        }
    }
}

impl PuddleBuilder {
    fn new(base: Toffset) -> Self {
        PuddleBuilder {
            base,
            ..PuddleBuilder::default()
        }
    }
    


    pub fn add_signal(&mut self, command: Command,timestamp: Toffset) -> Result<(),Waverr> {
        let time_delta = u16::try_from(timestamp - self.base).expect("Puddles are much too large; probably an error with how this is called") & 0xfff;
        let id = get_id(&command)?;
        let running_pload = self.payloads.entry(id as u32).or_insert(RunningPayload::default());
        match command {
            Command::ChangeScalar(id,val) => {
                //TODO: optimize
                running_pload.extend(vec![val]);
            },
            Command::ChangeVector(id,val) => {
                running_pload.extend(val);
            },
            Command::ChangeReal(id,val) => {
                running_pload.extend(val.to_le_bytes().into_iter().cloned());
            }, 
            Command::ChangeString(id,string) => {
                running_pload.extend(string.as_bytes().into_iter().cloned());
            },
            _ => {
                return Err(Waverr::VcdCommandErr(command))
            }
        }
        Ok(())
    }

}

//impl Into<Puddle> for PuddleBuilder {
//    fn into(self) -> Puddle { 
//
//    }
//
//}

