use iced::{button, Button, Element, Text};
use wave2_custom_widgets::widget::menu_bar::{self, MenuBar as MenuBarWidget};

#[derive(Debug, Clone)]
pub enum Message {
    OpenFile,
}

#[derive(Debug, Clone, Default)]
pub struct MenuBar {
    open_file: button::State,
    pending_file: bool,
}

impl MenuBar {
    pub fn set_pending_file(&mut self, pend_flag: bool) {
        self.pending_file = pend_flag;
    }
    pub fn get_pending_file(&self) -> bool {
        self.pending_file
    }

    pub fn view(&mut self) -> Element<Message> {
        Button::new(&mut self.open_file, Text::new("Open File"))
            .on_press(Message::OpenFile)
            .into()
    }
}
