use crate::errors::Waverr;
use crate::puddle::{Puddle,SignalId,PCursor,Toffset};
use std::sync::Arc;

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





///In memory DS for wave content; created from a list of Buckets
impl<'a> InMemWave<'a> {

    //pub fn first_change(&self) -> ParsedVec {
    //    self
    //        .signal_content
    //        .first()
    //        .expect("Empty signal found")
    //        .1
    //        .clone()
    //}
    //
    //

    pub fn all_data(&'a self) -> Box<dyn Iterator<Item=(u32, &'a[u8])> + 'a> {
        let sigid = self.signal_id;
        Box::new(self.puddles.iter()
            .map(move |puddle| (puddle.get_cursor(sigid).unwrap(), puddle.puddle_base()))
            .flat_map(|(cursor, base)| cursor.into_iter().zip(std::iter::repeat(base)))
            .map(|(droplet, base)| ( base + droplet.get_timestamp() as Toffset,  droplet.take_data())
            ))
    }
    
    //fn init_cursor(&'a mut self) -> Result<(),Waverr> {
    //    self.live_cursor = self.puddles
    //        .get(0)
    //        .map(|puddle| puddle.get_cursor(self.signal_id))
    //        .map_or(Ok(None), |cursor| cursor.map(Some))?;
    //        
    //    Ok(())

    //}

    pub fn new(
        name_str: String,
        signal_id: SignalId,
        puddles: Vec<Arc<Puddle>>,
    ) -> Result<InMemWave<'a>, Waverr> {
        let wave = InMemWave {
            name: name_str,
            signal_id,
            puddles: puddles,
            ref_holder: std::marker::PhantomData::default()
        };
        Ok(wave)
    }
}






