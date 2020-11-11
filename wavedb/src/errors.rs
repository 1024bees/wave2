

use std::io;
use thiserror::Error;



#[derive(Debug,Error)]
pub enum Waverr {
    //Parse errors 
    #[error("VCDError found, issue is `{0}`. TODO: make a better error enum here!")]
    VCDErr(&'static str),
    #[error("Wdb Bucket error for bucket id : {id:?}, ts : {ts:?}")]
    BucketErr{
       id: u32,
       ts: u32,
    },
    #[error("MissingID found, payload is `{0}` TODO: make a better error type!")]
    SledError(#[from] sled::Error),
    #[error("Payload Serialization (non-config) fail; from  bincode: `{0}`")]
    DataCorrupt(#[from] Box<bincode::ErrorKind>),
    #[error("Config Serialization  fail; from  toml: `{0}`")]
    SerConfig(#[from] toml::ser::Error),
    #[error("Config deserialization failed; issue is `{0}`")]
    DeserConfig(#[from] toml::de::Error),
    #[error("MissingID found, payload is `{0}` TODO: make a better error type!")]
    HierMapError(&'static str),
    #[error("Some IOErr `{0}`")]
    IOError(io::Error),
    #[error("Some cfg err `{0}`")]
    WDBCfgErr(&'static str),
    #[error("Generic error. This should be removed. Refactor this now")]
    GenericErr(&'static str), 
}

