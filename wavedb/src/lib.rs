/*! # `wavedb` - Wave2 signaldump API

`wavedb` provides an on-disk format to represent signal dumps from digital logic simulations and an API for the wave2 application to use to access signals from that 


*/





pub mod api;
pub mod errors;
pub mod storage;
pub mod signals;
pub mod hier_map;
pub mod inout;
pub mod puddle;
mod vcd_parser;
pub mod wavedb;
pub mod formatting;

//TODO: maybe replace this eventually
const MAX_PUDDLE_WIDTH: u32 = puddle::Puddle::max_puddle_width();

