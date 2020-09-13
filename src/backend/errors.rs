use sled;


pub enum Waverr {
//    SledErr(sled::Error),
    GenericErr(String),
}



//impl From<sled::Error> for Waverr{
//    #[inline]
//    fn from(sled_err : sled::Error) -> Self {
//        Waverr::SledErr(sled_err)
//    }
//}


impl<T: ToString> From<T> for Waverr {
    #[inline]
    fn from(gen_err: T) -> Self {
        Waverr::GenericErr(gen_err.to_string())
    }
}

