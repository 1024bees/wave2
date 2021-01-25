

///Chunk of a signal that is stored in wave2 db; on disk signal data structure
#[derive(Serialize, Deserialize, Debug)]
struct Bucket {
    timestamp_range: (u32, u32),
    id: u32,
    sig_type: SigType,
    sig_dumps: Vec<(u32, ParsedVec)>,
}

impl Default for Bucket {
    fn default() -> Self {
        Bucket {
            timestamp_range: (0, 10000),
            id: 0,
            sig_type: SigType::Vector(4),
            sig_dumps: vec![(0, ParsedVec::from(7)), (4, ParsedVec::from(3))],
        }
    }
}

