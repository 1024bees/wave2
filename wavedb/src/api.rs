use crate::errors::Waverr;
use crate::wavedb::WaveDb;
use crate::storage::in_memory::InMemWave;

use crate::hier_map::{HierMap, SignalItem};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;

/// Interface provided to wave2 for querying signal hierarchy
#[derive(Debug)]
pub struct WdbApi {
    wdb: WaveDb,
}

///Helper to hash type -> String
fn quick_hash<T: Hash>(t: &T) -> String {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    let mut rs = format!("{:x}", s.finish());
    rs.truncate(10);
    rs
}

impl From<WaveDb> for WdbApi {
    fn from(indb: WaveDb) -> WdbApi {
        WdbApi { wdb: indb }
    }
}

///External API for WaveDB instances
impl WdbApi {
    /// We clone self when calling
    pub fn open_from_vcd(path_to_vcd: &str) -> Result<WdbApi, Waverr> {
        let wdb_path = format!("/tmp/wavedb/{}/wdb", quick_hash(&path_to_vcd));
        Ok(WdbApi {
            wdb: WaveDb::from_vcd(
                path_to_vcd.into(),
                Path::new(wdb_path.as_str()),
            )?,
        })
    }

    pub fn get_hier_map(&self) -> Arc<HierMap> {
        self.wdb.get_hier_map()
    }

    pub async fn get_signal(
        api: Arc<WdbApi>,
        signal: SignalItem,
    ) -> Result<Arc<InMemWave>, Arc<Waverr>> {
        api.wdb.get_imw_sigitem(signal)
    }

    /// Get the names of all signals that exist within this module (that are visible to wavedb)
    pub async fn get_module_signals(
        api: Arc<WdbApi>,
        module_idx: usize,
    ) -> Arc<Vec<SignalItem>> {
        Arc::new(api.as_ref().wdb.hier_map.get_module_signals_vec(module_idx))
    }


    /// Get the starting and ending time of the signal dump represented by this WaveDB
    pub async fn bounds(
        api: Arc<WdbApi>,
    ) -> (u32,u32) {
        api.wdb.get_bounds()
    }





    /// Get module names underneath module_path
    /// TODO: encode if there is a submodule here
    pub fn get_submodules(&self) -> &[String] {
        unimplemented!()
    }
}
