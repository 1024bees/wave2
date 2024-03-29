use crate::errors::Waverr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::Iterator;

pub mod builder;
pub mod testing_utils;
pub mod utils;

pub type SignalId = u32;
/// offset into a puddle
pub type Poffset = usize;
/// Time offset; describes what puddle to look at
pub type Toffset = u32;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PMeta {
    /// offset into the payload when this signal starts
    offset: Poffset,
    /// number of items in the payload
    len: u16,
    /// Signal type information
    width: usize,
    /// if this slice of the puddle has variable length data
    /// variable length data happens zx bits are present, etc
    var_len: bool,
}

impl PMeta {
    fn width(&self) -> usize {
        self.width
    }
    fn drop_len(&self) -> Option<usize> {
        if self.var_len {
            Some(self.width() as usize + Droplet::header_width())
        } else {
            None
        }
    }
}

///Chunk of a signal that is stored in wave2 db; on disk signal data structure
#[derive(Serialize, Deserialize, Debug)]
pub struct Puddle {
    offset_map: HashMap<SignalId, PMeta>,
    next_sig_map: HashMap<SignalId, Toffset>,
    prev_sig_map: HashMap<SignalId, Toffset>,
    ///Base time offset of this puddle;
    base: Toffset,
    base_sigid: SignalId,
    payload: Vec<u8>,
}

impl Puddle {
    /// The time width of a puddle; currently statically set, maybe worth setting as part of some
    /// configuration for wavedb
    const TIMESTAMP_BITS: u32 = 12;
    pub const fn max_puddle_length() -> Toffset {
        1 << Puddle::TIMESTAMP_BITS
    }

    ///TODO: this should be some configuration part of wavedb
    pub const fn signals_per_puddle() -> SignalId {
        50
    }

    pub fn puddle_end(&self) -> Toffset {
        self.base + Puddle::max_puddle_length()
    }

    //TODO: get rid of this god damn it, merge with puddle_base
    pub fn get_btree_idx(&self) -> Toffset {
        self.base
    }

    pub fn puddle_base(&self) -> Toffset {
        self.base
    }

    pub fn get_base_sigid(&self) -> SignalId {
        self.base_sigid
    }

    pub fn get_droplet(&self, signal_id: SignalId, poffset: Poffset) -> Result<Droplet, Toffset> {
        let offset_data = self.offset_map.get(&signal_id);

        if offset_data.is_none() {
            let toffset = *self.next_sig_map.get(&signal_id).expect(
                "next_sig_map is missing a signal id. TODO: maybe downgrade to recoverable error",
            );
            return Err(toffset);
        }

        let pmeta = offset_data.unwrap();

        if pmeta.var_len {
            unimplemented!("i dont wanna deal with this yet")
        } else {
            let lbound = poffset + pmeta.offset;
            let rbound = lbound + pmeta.drop_len().expect("Must be statically sized");

            Ok(Droplet {
                content: &self.payload[lbound..rbound],
            })
        }
    }

    pub fn get_signal_width(&self, sig_id: SignalId) -> Option<usize> {
        self.offset_map.get(&sig_id).map(|pmeta| pmeta.width)
    }

    pub fn get_cursor(&self, sig_id: SignalId) -> Result<PCursor<'_>, Waverr> {
        let meta_handle = self.offset_map.get(&sig_id).ok_or(Waverr::PCursorErr {
            id: sig_id,
            context: "No content for this signal",
        })?;
        Ok(PCursor::new(sig_id, meta_handle, self))
    }
}

#[derive(Debug)]
pub struct PCursor<'a> {
    sig_id: SignalId,
    /// Offset into the payload
    poffset: Poffset,
    /// index into the current puddle; keeps track if we need to go to the next puddle
    pidx: u16,
    pidx_back: u16,
    /// length of the puddle; if pidx equals this number, we have to go to the next puddle
    plen: u16,
    /// this slice should contain the ENTIRE puddle payload
    payload_handle: &'a [u8],
    meta_handle: &'a PMeta,
    puddle_handle: &'a Puddle,
}

impl<'a> Iterator for PCursor<'a> {
    type Item = Droplet<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pidx >= self.pidx_back {
            None
        } else {
            let drop = self.set_front_index(self.pidx);
            self.poffset += self.get_sigwidth();
            self.pidx += 1;
            drop
        }
    }
}

impl<'a> DoubleEndedIterator for PCursor<'a> {
    fn next_back(&mut self) -> Option<Droplet<'a>> {
        if self.pidx_back <= self.pidx {
            None
        } else {
            self.pidx_back -= 1;
            self.set_back_index(self.pidx_back)
        }
    }
}

/**
 * By default, signals are directly represented in a puddle by numeric values. However we want to
 * support 4 state simulations with x's (unknown values) and z's (undriven signals)
 *
 *
 * We do this by having an optional 2bit encoding for signals, with the following mapping
 *
 * 00 -> 0
 * 01 -> 1
 * 10 -> Z
 * 11 -> X
 *
 *
 * TwoBitSignal wraps this
 *
 **/

pub enum TwoBitSignal {
    Zero,
    One,
    Z,
    X,
}

impl From<(bool, bool)> for TwoBitSignal {
    fn from(zx_and_sig: (bool, bool)) -> TwoBitSignal {
        match zx_and_sig {
            (false, false) => TwoBitSignal::Zero,
            (true, false) => TwoBitSignal::One,
            (false, true) => TwoBitSignal::Z,
            (true, true) => TwoBitSignal::X,
        }
    }
}

impl From<TwoBitSignal> for char {
    fn from(tbs: TwoBitSignal) -> char {
        match tbs {
            TwoBitSignal::One => '1',
            TwoBitSignal::Zero => '0',
            TwoBitSignal::X => 'x',
            TwoBitSignal::Z => 'z',
        }
    }
}

/**
Droplet structure.

2 bytes of header; header structure is as follows (little endian) starting from LSB:

* Timestamp(12 bits): offset from start of the drop.
* Optional (2 bits): Unallocated
* Variable length signal (1 bit): this bit is set if the signal has variable length; if this is the case
* ZX Bit (1bit) : This bit is set if there are any undefined (X) or undriven (HiZ) bits of this signal. If this is high, the payload portion of the Drop will be twice as long.

if the zx bit is set, we have two "parallel" bit vectors that encode the state of the payload.


We have the original payload from bytes 0..N-1, and then the "zx" payload from bytes N to 2N-1

e.g. if we have a signal that is normally 4 bits wide, then we would have a payload that looks like the following (in binary)

0100 0001, where the "original" signal is 0100 and the zx signal is 0001.

we then treat each bit of the original as a two bit signal, where the MSB comes from the zx signal and the LSB comes from the original signal.

These two bit signals have the following mapping.
00 -> 0
01 -> 1
10 -> Z
11 -> X


so in the case alluded above we have 00, 01, 00 and 10, which maps to 010z


*/
pub struct Droplet<'a> {
    content: &'a [u8],
}

impl<'a> Droplet<'a> {
    const fn header_width() -> usize {
        2
    }

    fn new(payload: &'a [u8], poffset: Poffset, len: Poffset) -> Self {
        Droplet {
            content: &payload[poffset..poffset + len + Droplet::header_width()],
        }
    }

    pub fn get_timestamp(&self) -> u16 {
        (((self.content[1] & 0x0f) as u16) << 8) | self.content[0] as u16
    }

    fn is_zx_from_bytes(payload: &'a [u8]) -> bool {
        (payload[1] & 0x80) != 0
    }

    fn is_var_from_bytes(payload: &'a [u8]) -> bool {
        (payload[1] & 0x40) != 0
    }

    pub fn is_zx(&self) -> bool {
        (self.content[1] & 0x80) != 0
    }

    pub fn take_data(self) -> &'a [u8] {
        &self.content[2..]
    }
    pub fn get_data(&self) -> &[u8] {
        &self.content[2..]
    }
}

impl<'a> PCursor<'a> {
    pub fn new(sig_id: SignalId, meta_handle: &'a PMeta, puddle_handle: &'a Puddle) -> Self {
        PCursor {
            sig_id,
            pidx: 0,
            pidx_back: meta_handle.len,
            poffset: meta_handle.offset,
            plen: meta_handle.len,
            meta_handle,
            payload_handle: &puddle_handle.payload[..],
            puddle_handle,
        }
    }

    fn get_droplet(&self, sig_width: usize) -> Option<Droplet<'a>> {
        Some(Droplet {
            content: &self.payload_handle[self.poffset..self.poffset + sig_width],
        })
    }

    fn set_index(&mut self, pidx: u16, starting_pidx: u16) -> Option<Droplet<'a>> {
        let mut starting_pidx = starting_pidx;
        if self.meta_handle.var_len {
            if pidx < self.pidx {
                starting_pidx = 0;
                self.poffset = self.meta_handle.offset;
            }
            while pidx != starting_pidx {
                self.poffset += self.get_sigwidth();
                starting_pidx += 1;
            }
            let sig_width = self.get_sigwidth();
            self.get_droplet(sig_width)
        } else {
            let sig_width = self.get_sigwidth();
            self.poffset = self.meta_handle.offset + (pidx as usize * sig_width);
            self.get_droplet(sig_width)
        }
    }

    /// Set the back index of cursor, get droplet at that index
    /// used by next_back of double ended iterator
    pub fn set_back_index(&mut self, pidx: u16) -> Option<Droplet<'a>> {
        if pidx >= self.plen || pidx < self.pidx {
            self.pidx_back = pidx;
            return None;
        }
        let starting_index = self.pidx_back;
        let drop = self.set_index(pidx, starting_index);
        self.pidx_back = pidx;
        drop
    }

    /// Set the back index of cursor, get droplet at that index
    /// used by next_back of double ended iterator
    pub fn set_front_index(&mut self, pidx: u16) -> Option<Droplet<'a>> {
        if pidx >= self.plen || pidx > self.pidx_back {
            self.pidx = pidx;
            return None;
        }
        let starting_index = self.pidx;
        let drop = self.set_index(pidx, starting_index);
        self.pidx = pidx;
        drop
    }

    /// Move the cursor to point to the next droplet
    pub fn next_change(&mut self) -> Result<Droplet, Toffset> {
        if self.meta_handle.var_len {
            unimplemented!()
        }
        self.pidx += 1;
        if self.pidx < self.plen {
            self.poffset += self.meta_handle.width();
            Ok(Droplet::new(
                self.payload_handle,
                self.poffset,
                self.meta_handle.width() as Poffset,
            ))
        } else {
            Err(*self.puddle_handle.next_sig_map.get(&self.sig_id).unwrap())
        }
    }
    fn get_sigwidth(&mut self) -> usize {
        if Droplet::is_var_from_bytes(self.payload_handle) {
            unimplemented!()
        } else if Droplet::is_zx_from_bytes(
            &self.payload_handle[self.poffset..self.poffset + Droplet::header_width()],
        ) {
            2 * (self.meta_handle.width() / 8
                + if self.meta_handle.width() % 8 != 0 {
                    1
                } else {
                    0
                })
                + Droplet::header_width()
        } else {
            self.meta_handle.width() / 8
                + Droplet::header_width()
                + if self.meta_handle.width() % 8 != 0 {
                    1
                } else {
                    0
                }
        }
    }
}
