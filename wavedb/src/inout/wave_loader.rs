use crate::wavedb::WaveDB;
use crate::errors::Waverr;
use super::nfd_wrapper;


pub async fn load_vcd() -> Result<WaveDB,Waverr> {
    let path = match nfd_wrapper::open().await {
        Ok(path) => path,
        Err(error) => return Err(Waverr::IOError(error)),
    };

    // i am going to be fucking sick
    let vcd_file = async { WaveDB::from_vcd(path.into_os_string().into_string().unwrap(), "/tmp/garbage") }.await?;
    Ok(vcd_file)
}



