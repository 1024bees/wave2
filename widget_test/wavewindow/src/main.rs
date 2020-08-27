use iced::{
    scrollable, Align, Column, Container, Element, Length, Sandbox, Scrollable, Settings, Text,
};
use wave2::frontend::wavewindow;
use wave2::frontend::wavewindow::*;

use iced::{button, text_input, Button, Rectangle, TextInput};

pub fn main() {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    Signals: Vec<Wave>,
    GlobalCursorState: CursorState,
    wavewindow: wavewindow::WaveWindowState,
    scroll: scrollable::State,
    button: button::State,
    vec_button: button::State,
    clear_button: button::State,
}

#[derive(Debug, Clone)]
enum Message {
    ClearWaves,
    AddDummy,
    AddDummyVec,
    UpdateCursor(CursorState),
}
impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Example::default()
    }

    fn title(&self) -> String {
        String::from("Wavewindow widget example app")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::AddDummy => {
                self.Signals.push(Wave::default());
                self.wavewindow.request_redraw();
            }
            Message::AddDummyVec => {
                self.Signals.push(Wave::default_vec());
                self.wavewindow.request_redraw();
            }
            Message::ClearWaves => {
                self.Signals.clear();
                self.wavewindow = wavewindow::WaveWindowState::default();
            }
            Message::UpdateCursor(state) => self.GlobalCursorState = state,
        }
    }

    fn view(&mut self) -> Element<Message> {
        let button = Button::new(&mut self.button, Text::new("AddWave"))
            .padding(10)
            .on_press(Message::AddDummy);

        Column::new()
            .padding(20)
            .spacing(20)
            .align_items(Align::Center)
            .max_height(1000)
            .push(Text::new("Wavewindow example").width(Length::Shrink).size(50))
            .push(
                self.wavewindow
                    .view(&self.Signals, self.GlobalCursorState)
                    .map(Message::UpdateCursor),
            )
            .push(button)
            .push(
                Button::new(&mut self.clear_button, Text::new("Clear Waves"))
                    .padding(10)
                    .on_press(Message::ClearWaves),
            )
            .push(
                Button::new(&mut self.vec_button, Text::new("AddVecWave"))
                    .padding(10)
                    .on_press(Message::AddDummyVec),
            )
            .push(
                Text::new(format!("Cursor pos: {}", self.GlobalCursorState.cursor_location))
                    .width(Length::Shrink)
                    .size(50),
            )
            .into()
    }
}
