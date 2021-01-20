use iced::{
    scrollable, Align, Container, Element, Length, Sandbox, Scrollable,
    Settings, Space, Text,
};

use env_logger;
use log::info;
use wave2_custom_widgets::widget::menu_bar::{self,MenuBar,MenuBarOption,MenuOption};
use wave2_custom_widgets::traits::{MenuOption,MenuBarOption};
use strum_macros;

pub fn main() -> Result<(), iced::Error> {
    env_logger::init();
    Example::run(Settings::default())
}


#[derive(MenuBarOption,strum_macros::Display,Debug,Clone,Copy,)]
pub enum TopMenu {
    Edit(EditMenu),
    View(ViewMenu),
}

#[derive(MenuOption,strum_macros::Display,Debug,Clone,Copy,)]
pub enum ViewMenu {
    Window1,
    Window2,
}

#[derive(MenuOption,strum_macros::Display,Debug,Clone,Copy,)]
pub enum EditMenu {
    Copy,
    Delete,
    Paste
}





#[derive(Default)]
struct Example {
    scroll: scrollable::State,
    menubar : menu_bar::State,
}

type Message = TopMenu;
impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Menu bar primitive")
    }

    fn update(&mut self, message: Message) {
        match message {
            _ => {
                info!("help!")
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        
        let menu_bar : MenuBar<TopMenu,Message,_> = MenuBar::new(&mut self.menubar,Message::all());


        let mut content = Scrollable::new(&mut self.scroll)
            .width(Length::Fill)
            .align_items(Align::Center)
            .spacing(10)
            .push(menu_bar)
            .push(Space::with_height(Length::Units(600)))
            .push(Text::new("Play with the menu bar! FIXME: impl messages that do things"));
            

        content = content.push(Space::with_height(Length::Units(600)));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}


