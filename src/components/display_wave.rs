use iced::Color;
use std::sync::Arc;
use wave2_wavedb::InMemWave;

#[derive(Clone, Debug)]
pub struct WaveDisplayOptions {
    color: WaveColors,
    format: WaveFormat,
}

pub const fn to_color(opts: &WaveDisplayOptions) -> Color {
    match opts.color {
        WaveColors::Green => Color::from_rgba(0.0, 1.0, 0.0, 1.0),
        WaveColors::Red => Color::from_rgba(1.0, 0.0, 0.0, 1.0),
        WaveColors::Blue => Color::from_rgba(0.0, 0.0, 1.0, 1.0),
    }
}

#[derive(Clone, Debug)]
pub struct DisplayedWave {
    wave_content: Arc<InMemWave>,
    display_conf: Option<WaveDisplayOptions>,
}

//FIXME: for testing only; this should be removed once sigwindow is stable
impl Default for DisplayedWave {
    fn default() -> Self {
        DisplayedWave {
            wave_content: Arc::new(InMemWave::default()),
            display_conf: Option::default(),
        }
    }
}

impl DisplayedWave {
    pub fn get_wave(&self) -> &Arc<InMemWave> {
        &self.wave_content
    }
}

impl From<Arc<InMemWave>> for DisplayedWave {
    fn from(imw: Arc<InMemWave>) -> Self {
        DisplayedWave {
            wave_content: imw,
            display_conf: Option::default(),
        }
    }
}

impl std::fmt::Display for DisplayedWave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.wave_content)
    }
}

impl WaveColors {
    pub const ALL: [WaveColors; 3] =
        [WaveColors::Green, WaveColors::Red, WaveColors::Blue];
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

#[derive(Clone, Debug)]
enum WaveColors {
    Green,
    Red,
    Blue,
}

#[derive(Clone, Debug)]
enum WaveFormat {
    Decimal,
    Hex,
    Octal,
    SDecimal,
}
