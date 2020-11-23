use iced::{
    button, scrollable, text_input, Align, Button, Column, Container, Element,
    Length, Row, Scrollable, Text, TextInput,
};

use log::error;
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use wave2_wavedb::hier_map::{HierMap, ModuleItem, SignalItem};

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
