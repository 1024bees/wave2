use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
///Signal type enum; describes bitwidth for vectored signals
pub enum SigType {
    Bit,
    Float,
    Vector(usize),
    Str(usize),
}

