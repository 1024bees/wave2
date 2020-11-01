use crate::errors::Waverr;
use crate::wavedb::WaveDB;
use crate::InMemWave;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Interface provided to wave2 for querying signal hierarchy
pub struct WdbAPI {
    wdb: WaveDB,
}

///Helper to hash type -> String
fn quick_hash<T: Hash>(t: &T) -> String {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    let mut rs = format!("{:x}", s.finish());
    rs.truncate(10);
    rs
}

///External API to use when interacting with WaveDB instances
impl WdbAPI {
    pub fn open_from_vcd(path_to_vcd: &str) -> Result<WdbAPI, Waverr> {
        let wdb_path = format!("/tmp/wavedb/{}/wdb", quick_hash(&path_to_vcd));
        Ok(WdbAPI {
            wdb: WaveDB::from_vcd(path_to_vcd.into(), Path::new(wdb_path.as_str()))?,
        })
    }

    /// Get the signal content associated with this path
    pub fn get_signal_content(&self, sig_path: String) -> InMemWave {
        unimplemented!()
    }

    /// Get the names of all signals that exist within this module (that are visible to wavedb)
    pub fn get_signal_names(&self, module_path: String) -> &[String] {
        unimplemented!()
    }

    /// Get module names underneath module_path
    /// TODO: encode if there is a submodule here
    pub fn get_submodules(&self, module_path: String) -> &[String] {
        unimplemented!()
    }
}
