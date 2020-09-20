use iced::{Color};

#[derive(Clone,Debug)]
pub struct WaveDisplayOptions {
    color : WaveColors,
    format : WaveFormat,
}

pub const fn to_color(opts : &WaveDisplayOptions) -> Color {
    match opts.color {
        WaveColors::Green => Color::from_rgba(0.0,1.0,0.0,1.0),
        WaveColors::Red => Color::from_rgba(1.0,0.0,0.0,1.0),
        WaveColors::Blue=> Color::from_rgba(0.0,0.0,1.0,1.0),
    }

}

impl WaveColors {
    pub const ALL: [WaveColors; 3] = [
        WaveColors::Green,
        WaveColors::Red,
        WaveColors::Blue,
    ];
}


impl std::fmt::Display for WaveColors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
            WaveColors::Green => "Green",
            WaveColors::Red => "Red",
            WaveColors::Blue => "Blue",
            }
        )
    }
}

#[derive(Clone,Debug)]
enum WaveColors {
    Green,
    Red,
    Blue,
}

#[derive(Clone,Debug)]
enum WaveFormat {
    Decimal,
    Hex,
    Octal,
    SDecimal,
}
