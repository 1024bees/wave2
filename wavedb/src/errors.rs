/*! This module creates a generic error type for any error that can manifest across wavedb.

Uses [`thiserror`] to derive different error types as they appear



[`thiserror`]: thiserror
!*/
use std::io;
use thiserror::Error;
use crate::puddle::{SignalId,Toffset};

///Generic error type for any error that can manifest within wavedb or wave2
#[derive(Debug, Error)]
pub enum Waverr {
    /// Error during VCD parsing
    #[error(
        "VCDError found, issue is `{0}`. TODO: make a better error enum here!"
    )]
    VCDErr(&'static str),
    ///TODO: depricated, remove
    #[error("Wdb Bucket error for bucket id : {id:?}, ts : {ts_range:?}. context: {context:?}")]
    BucketErr {
        id: u32,
        ts_range: String,
        context: &'static str,
    },
    #[error("Puddle error: Puddle time : {time:?}, base_sigid: {base_sigid:?}.\n
        context : {context:?}")]
    PuddleErr {
        time: u32,
        base_sigid: SignalId,
        context: &'static str,
    },
    #[error("Unhandled comand found when building puddle; command is {0:?}")]
    VcdCommandErr(vcd::Command),
    #[error(
        "MissingID found, payload is `{0}` TODO: make a better error type!"
    )]
    SledError(#[from] sled::Error),
    ///Serde failure. TODO: move this to bincode whenever possible
    #[error("Problem ser/deser bucket is {0}. TODO: Depricate this")]
    BuckerSerdeErr(#[from] serde_json::Error),

    #[error("Payload Serialization (non-config) fail; from  bincode: `{0}`")]
    DataCorrupt(#[from] Box<bincode::ErrorKind>),
    #[error("Config Serialization  fail; from  toml: `{0}`")]
    SerConfig(#[from] toml::ser::Error),
    #[error("Config deserialization failed; issue is `{0}`")]
    DeserConfig(#[from] toml::de::Error),
    #[error(
        "MissingID found, payload is `{0}` TODO: make a better error type!"
    )]
    HierMapError(&'static str),
    #[error("Some IOErr `{0}`")]
    IOError(io::Error),
    #[error("Some cfg err `{0}`")]
    WDBCfgErr(&'static str),
    #[error("Generic error. This should be removed. Refactor this now")]
    GenericErr(&'static str),
}
