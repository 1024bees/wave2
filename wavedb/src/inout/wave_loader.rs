use super::nfd_wrapper;
use crate::api::WdbAPI;
use crate::errors::Waverr;
use crate::wavedb::WaveDB;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;

///Helper to hash type -> String
fn quick_hash<T: Hash>(t: &T) -> String {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    let mut rs = format!("{:x}", s.finish());
    rs.truncate(10);
    rs
}

pub async fn load_vcd() -> Result<Arc<WdbAPI>, Waverr> {
    let path = match nfd_wrapper::open().await {
        Ok(path) => path,
        Err(error) => return Err(Waverr::IOError(error)),
    };

    let output_path = format!("/tmp/wave2/{}", quick_hash(&path));
    // i am going to be fucking sick
    let wdb = async { WaveDB::from_vcd(path, Path::new(&output_path)) }.await?;

    Ok(Arc::new(WdbAPI::from(wdb)))
}
