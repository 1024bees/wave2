use crate::errors::Waverr;
use crate::wavedb::WaveDB;
use crate::storage::in_memory::InMemWave;

use crate::hier_map::{HierMap, SignalItem};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;

/// Interface provided to wave2 for querying signal hierarchy
#[derive(Debug)]
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

impl From<WaveDB> for WdbAPI {
    fn from(indb: WaveDB) -> WdbAPI {
        WdbAPI { wdb: indb }
    }
}

///External API for WaveDB instances
impl WdbAPI {
    /// We clone self when calling
    pub fn open_from_vcd(path_to_vcd: &str) -> Result<WdbAPI, Waverr> {
        let wdb_path = format!("/tmp/wavedb/{}/wdb", quick_hash(&path_to_vcd));
        Ok(WdbAPI {
            wdb: WaveDB::from_vcd(
                path_to_vcd.into(),
                Path::new(wdb_path.as_str()),
            )?,
        })
    }

    pub fn get_hier_map(&self) -> Arc<HierMap> {
        self.wdb.get_hier_map().clone()
    }

    pub async fn get_signals<'a>(
        api: Arc<WdbAPI>,
        signal: SignalItem,
    ) -> Result<Arc<InMemWave>, Arc<Waverr>> {
        let (sig_name, sig_id) = SignalItem::destructure(signal);
        api.wdb.get_imw_id(sig_name, sig_id)
    }

    /// Get the names of all signals that exist within this module (that are visible to wavedb)
    pub async fn get_module_signals(
        api: Arc<WdbAPI>,
        module_idx: usize,
    ) -> Arc<Vec<SignalItem>> {
        Arc::new(api.as_ref().wdb.hier_map.get_module_signals_vec(module_idx))
    }


    /// Get the starting and ending time of the signal dump represented by this WaveDB
    pub async fn bounds(
        api: Arc<WdbAPI>,
    ) -> (u32,u32) {
        api.wdb.get_bounds()
    }





    /// Get module names underneath module_path
    /// TODO: encode if there is a submodule here
    pub fn get_submodules(&self) -> &[String] {
        unimplemented!()
    }
}
