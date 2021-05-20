use iced::{Container, Element};

use wave2_custom_widgets::widget::menu_bar::{self,MenuBar,MenuBarOption,MenuOption};
use wave2_custom_widgets::traits::{MenuOption,MenuBarOption};
use strum_macros;

#[derive(MenuBarOption, strum_macros::Display, Debug, Clone)]
pub enum Message {
    File(FileMenu),
    View(ViewMenu),

}

#[derive(MenuOption, strum_macros::Display, Debug, Clone)]
pub enum FileMenu {
    Open,
}

#[derive(MenuOption, strum_macros::Display, Debug, Clone)]
pub enum ViewMenu {
    ImplMe
}

#[derive(Debug, Clone, Default)]
pub struct GlobalMenuBar {
    menu_bar: menu_bar::State,
    pending_file: bool,
}

impl GlobalMenuBar {
    pub fn set_pending_file(&mut self, pend_flag: bool) {
        self.pending_file = pend_flag;
    }
    pub fn get_pending_file(&self) -> bool {
        self.pending_file
    }

    pub fn view(&mut self) -> Element<Message> { 
        Container::new(MenuBar::new(&mut self.menu_bar,Message::all()))
        .width(iced::Length::Fill)
        .center_x()
        .into()
    
    }
}
