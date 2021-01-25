use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use vcd::Value;


#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
///Signal type enum; describes bitwidth for vectored signals
pub enum SigType {
    Bit,
    Vector(usize),
}

impl SigType {
    pub fn from_width(width: usize) -> SigType {
        match width {
            1 => SigType::Bit,
            bw => SigType::Vector(bw),
        }
    }
}




#[derive(Clone,Copy, Debug)]
///Represents ways to format ParsedVec into String
pub enum WaveFormat {
    Decimal,
    Hex,
    Octal,
    SDecimal,
}


impl WaveFormat {
    ///The number of bits per digit for this particular radix
    fn num_bits(&self) -> f32 {
        match self {
            WaveFormat::Hex => 4.0,
            WaveFormat::Octal => 3.0,
            WaveFormat::Decimal | WaveFormat::SDecimal => 3.32,
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
#[derive(Debug, Clone,Serialize, Deserialize)]
pub enum ParsedVec {
    WordVec(FourStateBitArr),
    WideVec(FourStateBitVec),
}


/// When signals are 64 bits or less, we use BitArrays to represent value bits and zx_bits
#[derive(Default,Clone, Debug, Serialize, Deserialize)]
pub struct FourStateBitArr {
    value_bits: BitArray<LocalBits, [u32; 1]>,
    zx_bits: Option<BitArray<LocalBits, [u32; 1]>>,
}


/// When signal width is 64 bits or larger, we use BitArrays to represent value bits and zx_bits
#[derive(Default,Clone, Debug, Serialize, Deserialize)]
pub struct FourStateBitVec {
    value_bits: BitVec<LocalBits,u32>,
    zx_bits: Option<BitVec<LocalBits,u32>>,
}

/// Trait for serializing ParsedVecs
pub trait SignalRepr {
    fn to_string(&self, format: WaveFormat, bit_width: usize) -> Option<String>;
}

impl SignalRepr for ParsedVec {
    fn to_string(&self, format: WaveFormat, bit_width: usize) -> Option<String> {
        match self {
            ParsedVec::WordVec(bit_arr) => bit_arr.to_string(format,bit_width),
            ParsedVec::WideVec(bit_vec) => bit_vec.to_string(format,bit_width),
        }
    }
}


impl FourStateBitArr {
    ///number of bits per 
    const fn width() -> usize {
        std::mem::size_of::<u32>() * 8
    }
}



//TODO: FourStateBitArr can be optimized further
impl SignalRepr for FourStateBitArr {
    fn to_string(&self, format : WaveFormat, bit_width: usize) -> Option<String> {
        let FourStateBitArr { value_bits, zx_bits} = self;
        if let Some(_) = zx_bits {
            match format {
                _ => None
            }
        } else {
            let mut need_padding = false;
            //FIXME: move to some const value that is a part of BitArr
            let width = ((bit_width as f32 / format.num_bits()).ceil() as usize).min(Self::width());
            let vstr: String = value_bits
                .domain()
                .enumerate()
                .rev()
                .map(|(_,value)| {
                    match format {
                        WaveFormat::Hex => {
                            format!("{:0>width$X}",value, width = width)
                        },
                        WaveFormat::Octal => {
                            format!("{:0>width$o}",value, width = width)
                        },
                        WaveFormat::Decimal => {
                            if need_padding {
                                format!("{:0>width$}",value, width =  width)
                            } else {
                                if value == 0 {
                                    String::from("")
                                }  else {
                                need_padding = true;
                                format!("{}",value)
                                }
                            }

                        }
                        _ => unimplemented!("Format unsupported! Time to die!")

                    }
                }).collect();

                    Some(vstr)
            }
        }
}




impl SignalRepr for FourStateBitVec {
    fn to_string(&self, format: WaveFormat, bit_width: usize) -> Option<String> {
        let FourStateBitVec { value_bits, zx_bits} = self;
        if let Some(_) = zx_bits {
            match format {
                _ => None
            }
        } else {

            let domain_width = 32;
            let mut octal_extra_bits = 0;
            let mut octal_extra_value = 0;
            let vstr: String = value_bits
                .domain()
                .enumerate()
                .rev()
                .map(|(idx,value)| {
                    let width = ((bit_width - idx * domain_width).min(32) as f32 / format.num_bits()).ceil() as usize;

                    match format {
                        WaveFormat::Hex => {
                            println!("{:0>width$X}",value, width = width);
                            format!("{:0>width$X}",value, width = width)
                        },
                        WaveFormat::Octal => {
                            let (printed_payload,printed_width) = if  idx == value_bits.domain().len() -1 {
                                let lent_bits  = (value_bits.domain().len() -1) % 3;
                                let print_width = (((bit_width - idx * domain_width).min(32) - lent_bits) as f32 / format.num_bits()).ceil() as usize;
                                octal_extra_bits = lent_bits;
                                octal_extra_value = value & (1 << lent_bits) -1;
                                ((value >> lent_bits) as u64 , print_width)

                            } else {
                                let lent_bits = (domain_width + octal_extra_bits) % 3;
                                let ret_val = (value.overflowing_shr(lent_bits as u32).0 ) as u64 
                                    | ((octal_extra_value as u64) << (32 - lent_bits)) as u64;
                                let print_width =(domain_width + octal_extra_bits - lent_bits) / 3;
                                octal_extra_bits = lent_bits;
                                octal_extra_value = value & (1 << lent_bits) -1;
                                (ret_val, print_width )

                            };

                            println!("value is {:0>8o}, orig is {:0>8o},width is {}",printed_payload,value,printed_width);
                            println!("{:0>width$o}",printed_payload, width = printed_width);
                            format!("{:0>width$o}",printed_payload, width = printed_width)
                        },
                        _ => unimplemented!("Format unsupported! Time to die!")
                    }

                }).collect();

                    Some(vstr)
            }


        }


        
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


impl From<u32> for FourStateBitArr {
    fn from(in_val: u32) -> Self {
        FourStateBitArr {
            value_bits: BitArray::from([in_val as u32]),
            zx_bits: None,
        }
    }
}


impl From<Vec<u32>> for FourStateBitVec {
    fn from(in_val: Vec<u32>) -> Self {
        FourStateBitVec {
            value_bits: BitVec::from_vec(in_val),
            zx_bits: None,
        }
    }
}





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
        fbv.value_bits = [vec_val as u32].into();
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


#[cfg(test)]
mod tests {
    const BITARR_FORMATS : [WaveFormat; 3] = [WaveFormat::Decimal, WaveFormat::Hex, WaveFormat::Octal];
    const BITVEC_FORMATS : [WaveFormat; 2] = [WaveFormat::Hex, WaveFormat::Octal];

    use super::*;
    use std::convert::TryInto;

    fn serialize_valid_bitvec(value: [u32; 2], value_width: usize) {
        let flat_val : u64  = value
            .iter()
            .cloned()
            .enumerate()
            .fold(0,|acc, (idx,val)| acc | (val as u64) << (32 * idx));
        //let combined_val : u64 = u64::from_be_bytes(
        let hex_width = (value_width as f32/4.0).ceil() as usize;
        let oct_width = (value_width as f32/3.0).ceil() as usize;
        let bitarr = FourStateBitVec::from(Vec::from(value));
        let formatted_bitarr : Vec<String> =BITVEC_FORMATS 
            .iter()
            .filter_map(|format| bitarr.to_string(*format,value_width))
            .collect();

        let ground_truth_strings = vec![format!("{:0>width$X}",flat_val,width=hex_width),format!("{:0>width$o}",flat_val,width = oct_width)];

        formatted_bitarr.into_iter().zip(ground_truth_strings).for_each(|(gen, ground_truth)| { assert_eq!(gen,ground_truth)});




    }

    fn serialize_valid_bitarr(value: u32, value_width: usize) {
        let hex_width = (value_width as f32/4.0).ceil() as usize;
        let oct_width = (value_width as f32/3.0).ceil() as usize;
        let bitarr = FourStateBitArr::from(value);
        let formatted_bitarr : Vec<String> =BITARR_FORMATS 
            .iter()
            .filter_map(|format| bitarr.to_string(*format,value_width))
            .collect();

        let ground_truth_strings = vec![format!("{}",value), format!("{:0>width$X}",value,width=hex_width),format!("{:0>width$o}",value,width = oct_width)];

        formatted_bitarr.into_iter().zip(ground_truth_strings).for_each(|(gen, ground_truth)| { assert_eq!(gen,ground_truth)});
    }


    #[test]
    fn serde_4bit_arr() {
        let mut fbv = FourStateBitArr::default();
        fbv.value_bits = [0xffff as u32].into();
        let bytes = serde_json::to_string(&fbv).unwrap();
        serde_json::from_str::<FourStateBitArr>(bytes.as_ref()).expect(
            format!("failed to deserialize, bytes are {:#?}", bytes).as_str(),
        );
    }

    #[test]
    fn serialize_full_bitarr() {

        let full_values : Vec<u32> = vec![0xdeadbeef,0x01010,0x34141414,0x0044feed,0x0dad0dad];

        full_values.into_iter().for_each(|value| serialize_valid_bitarr(value,32));
    }

    #[test]
    fn serialize_partial_bitarr() { 

        let full_values : Vec<(u32,usize)> = vec![(0xbeef,16),(0xa,5),(0xff,7),(0xff,8), (0x3f,6)];

        full_values.into_iter().for_each(|(value,width)| serialize_valid_bitarr(value,width));
    }

    #[test]
    fn test_bitvec_tostr_full() {

        let full_values : Vec<[u32; 2]> = vec![[0xdeadbeef,0x11beef11],[0x01010,0x34141414],[0x0044feed,0x0dad0dad]];

        full_values.into_iter().for_each(|value| serialize_valid_bitvec(value,64));


    }


}
