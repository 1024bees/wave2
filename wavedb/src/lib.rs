/*! # `wavedb` - Wave2 signaldump API

`wavedb` provides an on-disk format to represent signal dumps from digital logic simulations and an API for the wave2 application to use to access signals from that 


*/





pub mod api;
pub mod errors;
pub mod storage;
pub mod signals;
pub mod hier_map;
pub mod inout;
mod vcd_parser;
pub mod wavedb;

const DEFAULT_SLIZE_SIZE: u32 = 10000;

