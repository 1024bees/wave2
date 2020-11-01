use crate::wavedb::WaveDB;
use crate::errors::Waverr;
use std::sync::Arc;
use super::nfd_wrapper;
use std::path::Path;


pub async fn load_vcd() -> Result<Arc<WaveDB>,Waverr> {
    let path = match nfd_wrapper::open().await {
        Ok(path) => path,
        Err(error) => return Err(Waverr::IOError(error)),
    };

    // i am going to be fucking sick
    let vcd_file = async { WaveDB::from_vcd(path, Path::new("/tmp/garbage")) }.await?;
    Ok(Arc::new(vcd_file))
}
