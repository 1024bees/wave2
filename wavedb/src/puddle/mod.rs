use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::signals::{SigType,ParsedVec};
use std::sync::Arc;
use std::iter::Iterator;
type SignalId = u32;
/// offset into a puddle
type Poffset= usize;
/// Time offset; describes what puddle to look at
type Toffset= u32;
pub mod builder;
mod utils;

#[derive(Debug,Serialize,Deserialize)]
pub struct PMeta {
    /// offset into the payload when this signal starts
    offset: u32,
    /// number of items in the payload
    len: u16,
    /// Signal type information
    sig_type: SigType,
    /// if this slice of the puddle has variable length data
    /// variable length data happens zx bits are present, etc
    variable_payload: bool,
}

impl PMeta {
    fn drop_len(&self) -> Option<usize> {
        if self.variable_payload {
            Some(self.sig_type.width() + Droplet::header_width())
        } else {
            None
        }
    }
}





///Chunk of a signal that is stored in wave2 db; on disk signal data structure
#[derive(Serialize, Deserialize, Debug)]
pub struct Puddle {
    offset_map: HashMap<SignalId,PMeta>,
    next_sig_map: HashMap<SignalId, Toffset>,
    prev_sig_map: HashMap<SignalId,Toffset>,
    ///Base time offset of this puddle; 
    base : Toffset,
    payload : Vec<u8>,
}

impl Puddle {
    /// The time width of a puddle; currently statically set, maybe worth setting as part of some
    /// configuration 
    pub const fn puddle_width() -> Toffset {
        10000
    }

    pub fn puddle_end(&self) -> Toffset {
        self.base + Puddle::puddle_width()
    }

    fn is_variable(&self, signal_id: SignalId) -> Option<bool> {
        self.offset_map.get(&signal_id)
            .map(|meta_data| meta_data.variable_payload)
    }

    pub fn get_droplet(&self,signal_id: SignalId, poffset : Toffset) -> Result<Droplet, Toffset> {
        let offset_data = self.offset_map.get(&signal_id);

        if offset_data.is_none() {
            let toffset = self.next_sig_map.get(&signal_id)
                .expect("next_sig_map is missing a signal id. TODO: maybe downgrade to recoverable error")
                .clone();
            return Err(toffset);
        }

        let pmeta = offset_data.unwrap();


        if pmeta.variable_payload {
            unimplemented!("i dont wanna deal with this yet")
        } else {
            let lbound = pmeta.offset as usize;
            let rbound = lbound + pmeta.drop_len().expect("Must be statically sized");

            Ok(Droplet{ content: &self.payload[lbound..rbound]} )

        }
    }
}


pub struct PCursor<'a> {
    sig_id: SignalId,
    /// Time offset of the cursor 
    curr_off : Toffset,
    /// Offset into the payload
    poffset: Poffset,
    /// index into the current puddle; keeps track if we need to go to the next puddle
    pidx: u16,
    /// length of the puddle; if pidx equals this number, we have to go to the next puddle 
    plen: u16,
    payload_handle: &'a[u8],
    meta_handle: &'a PMeta,
    puddle_handle: Arc<Puddle>,

}
impl<'a> Iterator for PCursor<'a> {
    type Item = Droplet<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.pidx +=1;

        if self.pidx >= self.plen {
            return None
        } else {
            if  self.meta_handle.variable_payload {
                unimplemented!()
            } else {
                let sig_width = self.meta_handle.sig_type.width() / 8;
                self.poffset += sig_width;
                return Some(Droplet{content: &self.payload_handle[self.poffset..self.poffset + sig_width]})
            }
        }

    }

}
pub struct Droplet<'a> {
    ///
    /// Droplet structure:
    ///
    ///     2 bytes of header; header structure is as follows (little endian):
    ///         starting from LSB 
    ///         Timestamp(12 bits): offset from start of the drop.
    ///         Optional (3 bits): Length info? TBD 
    ///         ZX Bit (1bit) : This bit is set if there are any undefined (X) or undriven (HiZ) bits of this signal. If this is high, the payload portion of the Drop will be twice as long.
    ///     N bytes of payload; N should either be;
    ///         C; statically set by 
    ///
    content: &'a[u8],
}


impl<'a> Droplet<'a> {
    const fn header_width()->  usize { 2 }
    
    fn new(payload: &'a[u8], poffset: Poffset, len: Poffset) -> Self {
        Droplet {
            content : &payload[poffset..poffset+len]
        }
    }

    fn timestamp_from_bytes(payload: &'a[u8], poffset: Poffset) -> u16 {
        (((payload[poffset+1] & 0x0f) as u16) << 8) | payload[poffset] as u16

    }
    fn get_timestamp(&self) -> u16 {
        (((self.content[1] & 0x0f) as u16) << 8) | self.content[0] as u16
    }
    fn get_data(&self) -> &[u8] {
        &self.content[2..]
    }

}



impl<'a> PCursor<'a> {
    pub fn set_time(&mut self, offset: Toffset) -> Result<(),Toffset> {
        if offset > self.puddle_handle.as_ref().puddle_end() {
            let next_signal = self.puddle_handle.next_sig_map.get(&self.sig_id).expect("TODO: Message").clone();
            if next_signal >= offset {
                //move to last signal in current puddle
                return Ok(());
            } else {
                return Err(next_signal)
            }
        } else if offset < self.puddle_handle.as_ref().base {
            let prev_signal = self.puddle_handle.prev_sig_map.get(&self.sig_id).expect("TODO: Message").clone();
            if prev_signal >= offset {
                //move to last signal in current puddle
                return Ok(());
            } else {
                return Err(prev_signal)
            }
        } else {
            if self.meta_handle.variable_payload {
                unimplemented!()
            }
            //TODO: this could be potentially sped up; current impl is linear
            self.pidx = 0;
            self.poffset = self.meta_handle.offset as usize;
            let sig_width = self.meta_handle.sig_type.width();
            loop {
                let next_time= self.puddle_handle.base + Droplet::timestamp_from_bytes(self.payload_handle,self.poffset + sig_width) as u32;
                if next_time <= offset && self.pidx < self.plen {
                    self.pidx +=1;
                    self.poffset += sig_width;
                } else {
                    break;
                }
            }
            Ok(())

        }
    }

    /// Get the droplet that is currently pointed to by the cursor 
    pub fn get_current_signal(&self) -> Option<Droplet> {
        if self.pidx < self.plen {
            Some(Droplet::new(self.payload_handle,self.poffset,self.meta_handle.offset as Poffset))
        } else {
            None
        }
    }

    /// Move the cursor to point to the next droplet
    pub fn next_change(&self) -> Result<Droplet,Toffset> {
       
        if self.meta_handle.variable_payload { 
            unimplemented!()
        }
        self.pidx += 1;
        if self.pidx < self.plen {
            self.poffset += self.meta_handle.sig_type.width();
            Ok(Droplet::new(self.payload_handle,self.poffset,self.meta_handle.sig_type.width() as Poffset))
        } else {
            Err(self.puddle_handle.next_sig_map.get(&self.sig_id).unwrap().clone())
        }
    }

    /// Move the cursor to point to the next droplet
    pub fn prev_change(&self) -> Result<Droplet,Toffset> {
        if self.meta_handle.variable_payload { 
            unimplemented!()
        }
        else if self.pidx != 0 {
            self.pidx -= 1;
            self.poffset -= self.meta_handle.sig_type.width();
            Ok(Droplet::new(self.payload_handle,self.poffset,self.meta_handle.sig_type.width() as Poffset))
        } else {
            Err(self.puddle_handle.next_sig_map.get(&self.sig_id).unwrap().clone())
        }
    }

}

