use iced::Color;
use std::sync::Arc;
use wave2_wavedb::formatting::{format_payload, WaveFormat};
use wave2_wavedb::storage::in_memory::InMemWave;

use wave2_wavedb::puddle::Droplet;

/// Mininum x_delta between two "value" changes that must occur before we consider writing the
/// wave's value on the line
const TEXT_THRESHOLD: f32 = 12.0;

const TEXT_SIZE: f32 = 12.0;

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
        self.wave_content.fmt(f)
   }
}

impl WaveColors {
    pub const ALL: [WaveColors; 3] = [WaveColors::Green, WaveColors::Red, WaveColors::Blue];
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

///// Utility for converting value -> canvas based text.
///// The text that we are generating exists in the margins between two "wave deltas", so we have to
///// truncate that value occasionally
//pub fn generate_canvas_text(
//    data: Droplet,
//    display_options: WaveDisplayOptions,
//    bitwidth: usize,
//    space: f32,
//) -> Option<Text> {
//    let str_format = display_options.format;
//    if space < TEXT_SIZE {
//        return None;
//    }
//    let visible_chars = (space / TEXT_SIZE).ceil() as usize;
//    log::info!("payload is {:?}", data.get_data());
//
//    let value = format_payload(data, str_format,bitwidth,visible_chars);
//   log::info!("string value is {}",value);
//    Some(Text::from(value))
//}

#[derive(Clone, Debug)]
/// Wave state for single bit signals
///
/// Used when iterating across an in memory wave to decide coloring state
pub enum SBWaveState {
    Beginning,
    Low,
    High,
    X,
    Z,
}

#[derive(Clone, Copy, Debug)]
enum WaveColors {
    Green,
    Red,
    Blue,
}
