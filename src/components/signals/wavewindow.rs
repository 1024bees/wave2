use iced::{Command, Container, Element, Length};

use super::beach::BEACH_PADDING;
use super::Message;
use wave2_custom_widgets::widget::signal_window;
use wave2_wavedb::storage::display_wave::DisplayedWave;

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
        .padding(BEACH_PADDING)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::InitBounds(bounds) => {
                self.widget_state.init_bounds(bounds);
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
