use iced::{Color, Column, Command, Container, Element, Length, Row};

use super::state;
use super::Message;
use wave2_custom_widgets::widget::signal_window;
use wave2_wavedb::storage::display_wave::DisplayedWave;

pub const BUFFER_PX: f32 = 1.5;
pub const WAVEHEIGHT: f32 = 16.0;
pub const VEC_SHIFT_WIDTH: f32 = 4.0;
pub const TS_FONT_SIZE: f32 = 12.0;

/// If we try to put a timestamp too close to the start of the wave window
/// it clips the black bounding box of the wave window and looks bad
const TS_CLIP_RANGE: f32 = 5.0;

#[derive(Default)]
pub struct WaveWindowState {
    pub widget_state: signal_window::State,
}

impl WaveWindowState {
    pub fn view2<'a>(&'a mut self, waves: &'a [DisplayedWave]) -> Element<Message> {
        Container::new(
            signal_window::SignalWindow::new(waves, &mut self.widget_state)
                .on_click(Message::UpdateCursor),
        )
        .width(Length::Shrink)
        .height(Length::Fill)
        .padding(10)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::UpdateBounds(bounds) => {
                self.widget_state.set_bounds(bounds);
            }
            //Message::AddWave(imw) => match imw {
            //    Ok(wave) => {
            //        self.live_waves.push(DisplayedWave::from(wave));
            //    }
            //    Err(err) => log::info!("Failed to add wave with err {:?}", err),
            //},
            Message::UpdateCursor(cursor_loc) => {
                self.widget_state.cursor_location = cursor_loc;
            }
            Message::ZoomIn => {
                self.widget_state.calczoom(-1);
            }

            Message::ZoomOut => {
                self.widget_state.calczoom(1);
            }

            _ => {
                log::info!("Not covered");
            }
        }
        Command::none()
    }
}
