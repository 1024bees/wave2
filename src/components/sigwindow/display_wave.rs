use iced::Color;
use iced::canvas::Text;
use std::sync::Arc;
use wave2_wavedb::InMemWave;
use wave2_wavedb::{ParsedVec,WaveFormat};


#[derive(Clone, Copy, Debug)]
pub struct WaveDisplayOptions {
    color: WaveColors,
    format: WaveFormat,
}


impl Default for WaveDisplayOptions { 
    fn default() -> WaveDisplayOptions {
        WaveDisplayOptions {
            color: WaveColors::Green,
            format: WaveFormat::Hex, 
        }
    }
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


/// Utility for converting value -> canvas based text.
/// The text that we are generating exists in the margins between two "wave deltas", so we have to
/// truncate that value occasionally
pub fn generate_canvas_text(text_space : f32, width : u32, data: ParsedVec,format: Option<WaveDisplayOptions>) -> Text {
    let str_format = format.unwrap_or(WaveDisplayOptions::default()).format;

    



    let ValueStr = match str_format {
        WaveFormat::Decimal => {
            
            
        }
        WaveFormat::Hex => {

        }
        WaveFormat::SDecimal => {
            unimplemented!("Need to impliment SDecimal canvas rep")
        }
        WaveFormat::Octal => {
            unimplemented!("On the record.. fuck Octal")
        }


    };


    Text::from("Dog!")
}




#[derive(Clone, Debug)]
/// Wave state for single bit signals
///
/// Used when iterating across an in memory wave to decide coloring state
pub enum SBWaveState {
    Beginning,
    Low,
    High,
    X,
    Z
}



#[derive(Clone,Copy, Debug)]
enum WaveColors {
    Green,
    Red,
    Blue,
}


