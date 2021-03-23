use super::{Toffset, SignalId,Poffset,Puddle,PMeta};
use std::collections::HashMap;
use vcd::{Command,Value};
use crate::errors::Waverr;
use std::convert::TryFrom;
use super::utils::get_id;
use crate::signals::SigType;
use log::info;
use std::convert::TryInto;

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
        info!("data size is {}",data_size);
        let mut zx_base : Option<usize>= None;
        //FIXME: SINFUL BEYOND COMPARE. BY GOD THIS WILL BE PAINFUL ONE DAY
        self.data.resize(self.data.len() + data_size,0);
        let base = self.data.len()-1;
        let (mut bit_offset, mut byte_offset) = (0,0);


        // Vec<Value> are organized as MSB first. This is similar to big endian, but values are
        // recorded at bit granularity, so they must be reversed as well. 
        for (bidx,value) in iter.into_iter().enumerate() {
            bit_offset = !bidx & 0x7;
            byte_offset = base - (bidx >> 3);
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
        if bit_offset != 0 {
            info!("bit offset is {}, data is currently, data: {}", bit_offset, self.data[byte_offset]);
            
            self.data[byte_offset] >>= bit_offset;
            info!("bit offset is {}, data is currently, data: {}", bit_offset, self.data[byte_offset]);

            if zx_base.is_some() {
                self.data[byte_offset + data_size] >>= bit_offset;
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
        running_pload.num_items += 1;
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
                info!("num items is {}",payload.num_items);
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


#[cfg(test)]
#[allow(dead_code, unused_macros, unused_imports, unused_variables)]
mod tests {
    use super::*;
    use crate::puddle::Droplet;
    fn num_to_vec(in_value: u64, len: u8) -> Vec<Value> {
        (0..len)
            .filter_map(|bit| ((in_value & (1 << bit)) != 0 ).then(||Value::V1).or_else(|| Some(Value::V0)))
            .rev()
            .collect()
    }

    fn zx_vec_builder(valid_val: u64, x_val: u64, z_val: u64, len: u8 ) -> Vec<Value> {
        (0..len)
            .map(|bit| {
                match (((valid_val& (1 << bit)) != 0 ), ((x_val & (1 << bit)) != 0 ), ((z_val & (1 << bit)) != 0 )) {
                    (_, true, _) => Value::X,
                    (_, false,true) => Value::Z,
                    (false, _, _) => Value::V0,
                    (true, _, _) => Value::V1,
                }

            })
            .rev()
            .collect()

    }



    fn init_test_logger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init(); 
    }





    #[test]
    #[allow(unused_must_use)]
    fn puddle_builder_single_bit() { 
        init_test_logger();

        let mut pb = PuddleBuilder::new(0);
        let mut clock_sig = 0;
        let small_range = 400;
        for i in 0..small_range {

            pb.add_signal(Command::ChangeVector((0 as u32).into(),num_to_vec(clock_sig,1)),i).unwrap();
            clock_sig = !clock_sig & 0x1;
        }
        let puddle : Puddle = pb.into();
        let droplet_vec : Vec<Droplet>= puddle.get_cursor(0).expect("This cursor should exist").into_iter().collect();
        assert_eq!(droplet_vec.len(), small_range as usize, "Missing values inside droplet_vec");
        for (time, droplet) in droplet_vec.iter().enumerate() {
            info!("data len is {}", droplet.get_data().len());
            let data = u8::from_le_bytes(droplet.get_data().try_into().unwrap()) as usize;
            assert_eq!(time % 2, data);
            assert_eq!(time,droplet.get_timestamp() as usize);
        }




    }

    #[test]
    #[allow(unused_must_use)]
    fn puddle_builder_wide() { 
        init_test_logger();

        let mut pb = PuddleBuilder::new(0);
        let large_range = 500;


        for i in 0..large_range {
            pb.add_signal(Command::ChangeVector((0 as u32).into(),num_to_vec(0xdeadbeefdeadbeef,64)),i).unwrap();
        }
        let puddle : Puddle = pb.into();
        let droplet_vec : Vec<Droplet>= puddle.get_cursor(0).expect("This cursor should exist").into_iter().collect();
        assert_eq!(droplet_vec.len(), large_range as usize, "Missing values inside droplet_vec");
        for (time, droplet) in droplet_vec.iter().enumerate() {
            let data = u64::from_le_bytes(droplet.get_data().try_into().unwrap());
            assert_eq!(0xdeadbeefdeadbeef, data);
            assert_eq!(time,droplet.get_timestamp() as usize);
        }

    }

    #[test]
    #[allow(unused_must_use)]
    fn puddle_builder_x_single() { 
        init_test_logger();

        let mut pb = PuddleBuilder::new(0);
        let mut x_clock_sig = 0;
        let large_range = 500;


        for i in 0..large_range {
            pb.add_signal(Command::ChangeVector((0 as u32).into(),zx_vec_builder(0,x_clock_sig,0,1)),i).unwrap();
            x_clock_sig = !x_clock_sig & 0x1;
        }
        let puddle : Puddle = pb.into();
        let droplet_vec : Vec<Droplet>= puddle.get_cursor(0).expect("This cursor should exist").into_iter().collect();
        assert_eq!(droplet_vec.len(), large_range as usize, "Missing values inside droplet_vec");

    }

    #[test]
    #[allow(unused_must_use)]
    fn puddle_builder_z_single() { 
        init_test_logger();

        let mut pb = PuddleBuilder::new(0);
        let mut z_clock_sig = 0;
        let large_range = 500;


        for i in 0..large_range {
            pb.add_signal(Command::ChangeVector((0 as u32).into(),zx_vec_builder(0,0,z_clock_sig,1)),i).unwrap();
            z_clock_sig = !z_clock_sig & 0x1;
        }
        let puddle : Puddle = pb.into();
        let droplet_vec : Vec<Droplet>= puddle.get_cursor(0).expect("This cursor should exist").into_iter().collect();
        assert_eq!(droplet_vec.len(), large_range as usize, "Missing values inside droplet_vec");

    }

}

