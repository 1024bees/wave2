use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
///Signal type enum; describes bitwidth for vectored signals
pub enum SigType {
    Bit,
    Float,
    Vector(usize),
    Str(usize),
}


impl Default for SigType {
    fn default() -> Self {
        SigType::Bit
    }
}
impl SigType {
    pub fn from_width(width: usize) -> SigType {
        match width {
            1 => SigType::Bit,
            bw => SigType::Vector(bw),
        }
    }

    pub fn width(&self) -> usize {
        match self {
            SigType::Bit => 1,
            SigType::Float => 64,
            SigType::Vector(width) => width.clone(),
            SigType::Str(width) => width.clone()


        }
    }


}







