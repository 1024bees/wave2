#[derive(Clone,Copy, Debug)]
///Represents ways to format ParsedVec into String
pub enum WaveFormat {
    Decimal,
    Hex,
    Octal,
    SDecimal,
}


impl WaveFormat {
    ///The number of bits per digit for this particular radix
    fn num_bits(&self) -> f32 {
        match self {
            WaveFormat::Hex => 4.0,
            WaveFormat::Octal => 3.0,
            WaveFormat::Decimal | WaveFormat::SDecimal => 3.32,
        }
    }
}

pub fn format_payload(payload : &[u8], format: WaveFormat) -> String {
    String::from("deadbeef")
}
