use std::sync::Arc;
use crate::formatting::{format_payload, WaveFormat};
use crate::storage::in_memory::InMemWave;

use crate::puddle::Droplet;

/// Mininum x_delta between two "value" changes that must occur before we consider writing the
/// wave's value on the line
const TEXT_THRESHOLD: f32 = 12.0;

const TEXT_SIZE: f32 = 12.0;

#[derive(Clone, Copy, Debug)]
pub struct WaveDisplayOptions {
    pub color: WaveColors,
    pub format: WaveFormat,
}

impl Default for WaveDisplayOptions {
    fn default() -> WaveDisplayOptions {
        WaveDisplayOptions {
            color: WaveColors::Green,
            format: WaveFormat::Hex,
        }
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
pub enum WaveColors {
    Green,
    Red,
    Blue,
}
