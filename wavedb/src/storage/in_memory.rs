use crate::errors::Waverr;
use crate::puddle::{Puddle,SignalId,PCursor,Toffset};
use std::sync::Arc;
use log::info;

#[derive(Debug)]
pub struct InMemWave<'a> {
    name: String,
    signal_id: SignalId,
    puddles: Vec<Arc<Puddle>>,
    ///// The index into the puddles variable for deciding which puddle we should convert into a
    ///// cursor
    ref_holder: std::marker::PhantomData<&'a u8>
    //live_cursor: Option<PCursor<'a>>
}





///In memory DS for wave content; created from a Vector of Arcs to puddles
impl<'a> InMemWave<'a> {
    pub fn all_data(&'a self) -> Box<dyn Iterator<Item=(u32, &'a[u8])> + 'a> {
        let sigid = self.signal_id;
        Box::new(self.puddles.iter()
            .filter_map(move |puddle| puddle.get_cursor(sigid).ok().map(|cursor| (cursor, puddle.puddle_base())))
            .flat_map(|(cursor, base)| cursor.into_iter().zip(std::iter::repeat(base)))
            .map(|(droplet, base)| ( base + droplet.get_timestamp() as Toffset,  droplet.take_data()))
            )
    }
    
    pub fn data_in_range(&'a self, begin : Toffset, end: Toffset) -> Box<dyn Iterator<Item=(Toffset, &'a[u8])> + 'a>  {
        let sigid = self.signal_id;
        Box::new(self.puddles.iter()
            .filter(move |puddle| begin < puddle.puddle_end()  &&  end > puddle.puddle_base())
            
            .filter_map(move |puddle| puddle.get_cursor(sigid).ok().map(|cursor| (cursor, puddle.puddle_base())))
            .flat_map(|(cursor, base)| cursor.into_iter().zip(std::iter::repeat(base)))
            .map(|(droplet, base)| ( base + droplet.get_timestamp() as Toffset,  droplet.take_data()))
            //TODO: consider pulling this out
            .filter(move |(time, _)| *time >= begin && *time < end)
            )
    }


    pub fn new(
        name_str: String,
        signal_id: SignalId,
        puddles: Vec<Arc<Puddle>>,
    ) -> Result<InMemWave<'a>, Waverr> {
        Ok(InMemWave {
            name: name_str,
            signal_id,
            puddles,
            ref_holder: std::marker::PhantomData::default()
        })
    }
}

#[cfg(test)]
#[allow(dead_code, unused_macros, unused_imports, unused_variables)]
mod tests {
    use super::*;
    use crate::puddle::Droplet;
    use crate::puddle::builder::tests::build_dummy_puddles;
    use log::info;
    use std::convert::TryInto;

    fn init_test_logger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init(); 
    }

    #[test]
    fn sanity_imw() { 
        let puddles : Vec<Arc<Puddle>>= (0..5).into_iter().map(|idx| build_dummy_puddles(idx * Puddle::max_puddle_width(),20,16)).collect();
        let imw_0 = InMemWave::new("sig_0".into(),0,puddles.clone()).unwrap();
        let first_puddle_fragment : Vec<(u32, &[u8])> =  imw_0.data_in_range(0,1000).collect();
        
        assert_eq!(first_puddle_fragment.len(),1000);
        for (time, payload) in first_puddle_fragment {
            let value = u16::from_le_bytes(payload.try_into().expect("should be u16"));
            assert_eq!(time as u16,value);
        }
    }



}

