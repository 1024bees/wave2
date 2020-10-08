#[derive(Debug, Clone)]
pub enum Waverr {
    //    SledErr(sled::Error),
    //
    VCDErr(&'static str),
    MissingID(&'static str),
    SledError(&'static str),
    HierMapError(&'static str),
    GenericErr(String),
}

//impl fmt::Display for Waverr {
//    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
//        match self {
//            Waverr::VCDErr(message) => {
//                write!(f,"There was a parsing error at the VCD factory... Here's the message we got: {}",message)
//            },
//            Waverr::MissingID(message) => {
//                write!(f,"There was a failure when looking up a bucket.. message is: {}", message)
//            },
//            Waverr::GenericErr(string) => {
//                write!(f,"Generic error: This should probably be made into its own error type: {}",string)
//            }
//        }
//    }
//}

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
