use super::nfd_wrapper;
use crate::api::WdbApi;
use crate::errors::Waverr;
use crate::wavedb::WaveDb;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path,PathBuf};
use std::sync::Arc;

///Helper to hash type -> String
fn quick_hash<T: Hash>(t: &T) -> String {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    let mut rs = format!("{:x}", s.finish());
    rs.truncate(10);
    rs
}

pub async fn load_vcd() -> Result<Arc<WdbApi>, Waverr> {
    let path = match nfd_wrapper::open().await {
        Ok(path) => path,
        Err(error) => return Err(Waverr::IoErr(error)),
    };

    let output_path = format!("/tmp/wave2/{}", quick_hash(&path));
    // i am going to be fucking sick
    let wdb = async { WaveDb::from_vcd(path, Path::new(&output_path)) }.await?;

    Ok(Arc::new(WdbApi::from(wdb)))
}

pub async fn load_vcd_from_path(path: PathBuf) -> Option<Arc<WdbApi>> {
    let output_path = format!("/tmp/wave2/{}", quick_hash(&path));
    // i am going to be fucking sick
    let wdb = async { WaveDb::from_vcd(path, Path::new(&output_path)) }.await.ok();
    if wdb.is_some() {
        Some(Arc::new(WdbApi::from(wdb.unwrap())))
    } else {
        None
    }
}


