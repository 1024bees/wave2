use crate::{InMemWave};


/// Interface provided to wave2 for querying signal hierarchy
pub struct WdbAPI {
    temp_field : Vec<String>,
}




///External API to use when interacting with WaveDB instances
impl WdbAPI {
    

    pub fn open_from_file(path_to_file) -> Self {
        unimplemented!()
    }

    /// Get the signal content associated with this path
    pub fn get_signal_content(&self, sig_path : String) -> InMemWave {
        unimplemented!()
    }


    /// Get the names of all signals that exist within this module (that are visible to wavedb)
    pub fn get_signal_names(&self, module_path : String) -> &[String] {
        unimplemented!()
    }

    /// Get module names underneath module_path
    /// TODO: encode if there is a submodule here
    pub fn get_submodules(&self, module_path : String) -> &[String] {
        unimplemented!(..)
    }



}
