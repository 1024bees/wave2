use iced::canvas::Text;
use wave2_wavedb::formatting::format_payload;
use wave2_wavedb::puddle::Droplet;
use wave2_wavedb::storage::display_wave::WaveDisplayOptions;

/// Mininum x_delta between two "value" changes that must occur before we consider writing the
/// wave's value on the line
const TEXT_THRESHOLD: f32 = 12.0;

const TEXT_SIZE: f32 = 12.0;

/// Utility for converting value -> canvas based text.
/// The text that we are generating exists in the margins between two "wave deltas", so we have to
/// truncate that value occasionally
pub fn generate_canvas_text(
    data: Droplet,
    display_options: WaveDisplayOptions,
    bitwidth: usize,
    space: f32,
) -> Option<Text> {
    let str_format = display_options.format;
    if space < TEXT_SIZE {
        return None;
    }
    let visible_chars = (space / TEXT_SIZE).ceil() as usize;
    log::info!("payload is {:?}", data.get_data());

    let value = format_payload(data, str_format, bitwidth, visible_chars);
    log::info!("string value is {}", value);
    Some(Text::from(value))
}
