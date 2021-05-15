use super::TwoBitSignal;
use crate::errors::Waverr;
use vcd::Command;

pub fn get_id(command: &Command) -> Result<u32, Waverr> {
    match command {
        Command::ChangeScalar(id, ..)
        | Command::ChangeVector(id, ..)
        | Command::ChangeReal(id, ..)
        | Command::ChangeString(id, ..) => Ok(id.0 as u32),
        _ => Err(Waverr::VcdCommandErr(command.clone())),
    }
}

pub struct ZxIter {
    zx_bits: u8,
    payload: u8,
    bit_index: u8,
}

impl ZxIter {
    pub fn new(zx_bits :u8, payload: u8, bit_index: u8) -> Self {
        ZxIter {
            zx_bits : zx_bits >> bit_index,
            payload: payload >> bit_index,
            bit_index,
        }
    }

}

impl Iterator for ZxIter {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        let ZxIter {
            zx_bits,
            payload,
            bit_index,
        } = self;
        if *bit_index >= 8 {
            None
        } else {
            let signal = (*payload & 0x1 == 1, *zx_bits & 0x1 == 1);
            let tbs = TwoBitSignal::from(signal);
            *zx_bits >>= 1;
            *payload >>= 1;
            *bit_index +=1;
            Some(tbs.into())
        }
    }
}


