use iced::Color;
use iced::canvas::Text;
use std::sync::Arc;
use wave2_wavedb::storage::in_memory::InMemWave;
use wave2_wavedb::formatting::{WaveFormat,format_payload};

/// Mininum x_delta between two "value" changes that must occur before we consider writing the
/// wave's value on the line
const TEXT_THRESHOLD: f32 = 20.0;

const TEXT_SIZE: f32 = 15.0;

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
    pub display_conf: Option<WaveDisplayOptions>,
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
        write!(f, "{}", self.wave_content.get_name())
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
pub fn generate_canvas_text(data: &[u8],display_options: WaveDisplayOptions, bitwidth: usize, space: f32) -> Option<Text> {
    let str_format = display_options.format;
    if space < TEXT_THRESHOLD {
        return None
    }
    let value = format_payload(data,str_format);
    let visible_chars = (space / TEXT_SIZE).ceil() as usize;
    let printed_str : &str = if visible_chars < value.len() {
        value.get(0..visible_chars)
            .expect("Truncating string improperly when generating wavewindow canvas text")
    } else {
            value.as_str()
        };
    Some(Text::from(printed_str))

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


