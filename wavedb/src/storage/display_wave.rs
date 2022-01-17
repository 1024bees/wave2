use crate::formatting::{format_payload, WaveFormat};
use crate::puddle::Toffset;
use crate::storage::in_memory::InMemWave;
use std::sync::Arc;

/// Mininum x_delta between two "value" changes that must occur before we consider writing the
/// wave's value on the line
//const TEXT_THRESHOLD: f32 = 12.0;
//
//const TEXT_SIZE: f32 = 12.0;

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
    pub val_under_cursor: Option<String>,
    pub display_conf: WaveDisplayOptions,
}

//FIXME: for testing only; this should be removed once sigwindow is stable
impl Default for DisplayedWave {
    fn default() -> Self {
        DisplayedWave {
            wave_content: Arc::new(InMemWave::default()),
            val_under_cursor: None,
            display_conf: WaveDisplayOptions::default(),
        }
    }
}

impl DisplayedWave {
    pub fn get_wave(&self) -> &Arc<InMemWave> {
        &self.wave_content
    }
    pub fn get_color(&self) -> WaveColors {
        self.display_conf.color
    }

    /// Formats string into the [`WaveFormat`](WaveFormat) at the time provided
    /// by the Toffset argument
    pub fn value_at_time(&self, time: Toffset) -> String {
        let droplet = self
            .wave_content
            .get_droplet_at(time)
            .expect("Value is not defined yet");

        format_payload(
            droplet,
            self.display_conf.format,
            self.wave_content.get_width(),
            500,
        )
    }
}

impl From<Arc<InMemWave>> for DisplayedWave {
    fn from(imw: Arc<InMemWave>) -> Self {
        DisplayedWave {
            wave_content: imw,
            val_under_cursor: None,
            display_conf: WaveDisplayOptions::default(),
        }
    }
}

impl std::fmt::Display for DisplayedWave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.wave_content.fmt(f)?;
        if let Some(ref value) = self.val_under_cursor {
            write!(f, " = {}", value)?;
        }
        Ok(())
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

#[derive(Clone, Debug, PartialEq)]
/// Wave state for single bit signals
///
/// Used when iterating across an in memory wave to decide coloring state
pub enum SBWaveState {
    Beginning,
    EndSentinel,
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
