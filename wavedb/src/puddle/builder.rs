use super::{Toffset, SignalId,Poffset,Puddle,PMeta};
use std::collections::HashMap;
use vcd::{Command,Value};
use crate::errors::Waverr;
use std::convert::TryFrom;
use super::utils::get_id;
use crate::signals::SigType;
use log::info;

/// Transient payload; this is the temporary container that is accumulated into as 
/// a vcd is parsed; TODO: make this sync? in the future it would be nice to build these out in
/// parallel
#[derive(Default)]
struct RunningPayload  {
    data : Vec<u8>,
    num_items: u16,
    width: usize,
    var_len: bool,
    
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
                        self.var_len = true;
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
    pub fn new(base: Toffset) -> Self {
        PuddleBuilder {
            base,
            ..PuddleBuilder::default()
        }
    }



    pub fn add_signal(&mut self, command: Command,timestamp: Toffset) -> Result<(),Waverr> {
        let time_delta = u16::try_from(timestamp - self.base).expect("Puddles are much too large; probably an error with how this is called") & 0xfff;
        let id = get_id(&command)?;
        let running_pload = self.payloads.entry(id as u32).or_insert(RunningPayload::default());
        running_pload.extend(time_delta.to_le_bytes().iter().cloned());
        match command {
            Command::ChangeScalar(..,val) => {
                //TODO: optimize
                running_pload.extend(vec![val]);
            },
            Command::ChangeVector(..,val) => {
                running_pload.width = usize::try_from(val.len()).unwrap();
                running_pload.extend(val);
            },
            Command::ChangeReal(..,val) => {
                running_pload.extend(val.to_le_bytes().iter().cloned());
            }, 
            Command::ChangeString(..,string) => {
                running_pload.extend(string.as_bytes().into_iter().cloned());
            },
            _ => {
                return Err(Waverr::VcdCommandErr(command))
            }
        }
        Ok(())
    }

}

impl Into<Puddle> for PuddleBuilder {
    fn into(self) -> Puddle { 
        let mut offset : Poffset = 0;
        let mut offset_map = HashMap::default();
        let prev_sig_map = HashMap::default();
        let next_sig_map = HashMap::default();

        //TODO: merge this in with the payloads into iter. me just lazy hehe!
        let base_sigid = self.payloads.iter()
            .min_by_key(|entry| entry.0)
            .map(|entry| entry.0 - entry.0 % Puddle::signals_per_puddle())
            .unwrap();

        let payload = self.payloads.into_iter()
            .flat_map(|(key, payload)| {
                let droplet_descriptor = PMeta {
                    offset,
                    len: payload.num_items,
                    width: payload.width,
                    var_len: payload.var_len,
                    ..PMeta::default()
                };
                offset_map.insert(key, droplet_descriptor);
                offset += payload.data.len();
                payload.data.into_iter()
            })
            .collect::<Vec<u8>>();
        info!("Base sigid is {}",base_sigid);
        Puddle {
            offset_map,
            prev_sig_map,
            next_sig_map,
            base: self.base,
            base_sigid,
            payload
        }
    }

}

