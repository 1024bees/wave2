use crate::errors::Waverr;
use crate::puddle::PCursor;

#[derive(Debug)]
pub struct InMemWave<'a> {
    name: String,
    cursors: Vec<PCursor<'a>>,
}



///In memory DS for wave content; created from a list of Buckets
impl<'a> InMemWave<'a> {
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
        cursor: PCursor<'a>,
    ) -> Result<InMemWave, Waverr> {
        Ok(InMemWave {
            name: name_str,
            cursors: vec![cursor],
        })
    }
}



