use crate::errors::Waverr;
use crate::signals::{SigType,ParsedVec};
use super::bucket::Bucket;

#[derive(Debug)]
pub struct InMemWave {
    name: String,
    signal_content: Vec<(u32, ParsedVec)>,
    pub sig_type: SigType,
}

impl Default for InMemWave {
    fn default() -> Self {
        InMemWave {
            name: String::from("PlaceholderWave"),
            signal_content: vec![
                (0, ParsedVec::from(0)),
                (10, ParsedVec::from(1)),
                (20, ParsedVec::from(0)),
                (30, ParsedVec::from(1)),
                (50, ParsedVec::from(0)),
                (500, ParsedVec::from(1)),
            ],
            sig_type: SigType::Bit,
        }
    }
}

impl std::fmt::Display for InMemWave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.as_str())
    }
}

///In memory DS for wave content; created from a list of Buckets
impl InMemWave {
    pub fn default_vec() -> Self {
        InMemWave {
            sig_type: SigType::Vector(4),
            ..InMemWave::default()
        }
    }
    pub fn first_change(&self) -> ParsedVec {
        self
            .signal_content
            .first()
            .expect("Empty signal found")
            .1
            .clone()
    }

    pub fn changes(&self) -> std::slice::Iter<'_, (u32, ParsedVec)> {
        self.signal_content.iter()
    }

    pub fn new(
        name_str: String,
        buckets: Vec<Result<Bucket, Waverr>>,
    ) -> Result<InMemWave, Waverr> {
        let mut signal_content = Vec::new();
        //TODO: can parallelize
        let mut st = None;
        for bucket in buckets {
            match bucket {
                Ok(mut bucket) => {
                    signal_content.append(bucket.signals());
                    if st.is_none() {
                        st = Some(bucket.signal_type())
                    }
                }
                Err(Waverr::BucketErr { .. }) => (),
                Err(bucket_err) => (return Err(bucket_err)),
            }
        }

        Ok(InMemWave {
            name: name_str.into(),
            signal_content: signal_content,
            sig_type: st.unwrap(),
        })
    }
}



