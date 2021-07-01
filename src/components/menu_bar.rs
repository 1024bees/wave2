use strum_macros;
use wave2_custom_widgets::traits::{MenuBarOption, MenuOption};
use wave2_custom_widgets::widget::menu_bar::{self, MenuBar, MenuBarOption, MenuOption};

use iced::{Column, Container, Element, Length, Text};
use iced_aw::{menu, Menu};
use menu::{Entry, Section};



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
    ImplMe,
}

#[derive(Debug, Default)]
pub struct GlobalMenuBar {
    menu_bar: menu_bar::State,
    menu: menu::State,
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
        let menu : Element<Message> = Menu::new(&mut self.menu)
            .push(
                Section::new(
                    Text::new("File"),
                    vec![Entry::Item(
                        Text::new("New File").into(),
                        Some(FileMenu::Open),
                    )],
                )
                .map(Message::File),
            )
            .push(
                Section::new(
                    Text::new("View"),
                    vec![Entry::Item(
                        Text::new("ImplMe").into(),
                        Some(ViewMenu::ImplMe),
                    )],
                )
                .map(Message::View),
            )
            .into();

        Container::new(menu)
            .width(iced::Length::Fill)
            .center_x()
            .into()
    }
}
