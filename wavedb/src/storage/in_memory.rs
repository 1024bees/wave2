use crate::errors::Waverr;
use crate::puddle::{Puddle,SignalId};
use std::sync::Arc;

#[derive(Debug)]
pub struct InMemWave {
    name: String,
    signal_id: SignalId,
    cursors: Vec<Arc<Puddle>>
}



///In memory DS for wave content; created from a list of Buckets
impl InMemWave {
    //pub fn default_vec() -> Self {
    //    InMemWave {
    //        sig_type: SigType::Vector(4),
    //        ..InMemWave::default()
    //    }
    //}
    //pub fn first_change(&self) -> ParsedVec {
    //    self
    //        .signal_content
    //        .first()
    //        .expect("Empty signal found")
    //        .1
    //        .clone()
    //}

    //pub fn changes(&self) -> std::slice::Iter<'_, (u32, ParsedVec)> {
    //    self.signal_content.iter()
    //}

    pub fn new(
        name_str: String,
        signal_id: SignalId,
        cursor: Vec<Arc<Puddle>>,
    ) -> Result<InMemWave, Waverr> {
        Ok(InMemWave {
            name: name_str,
            signal_id,
            cursors: cursor,
        })
    }
}



