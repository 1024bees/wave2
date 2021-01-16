use iced::{
    scrollable, Align, Container, Element, Length, Sandbox, Scrollable,
    Settings, Space, Text,
};

use env_logger;
use log::info;
use wave2_custom_widgets::widget::menu_bar::{self,MenuBar,MenuBarOption,MenuOption};
use strum_macros;

pub fn main() -> Result<(), iced::Error> {
    env_logger::init();
    Example::run(Settings::default())
}


#[derive(Debug,Clone,Copy,strum_macros::Display)]
pub enum TopMenu {
    Edit(EditMenu),
    View(ViewMenu),
}

#[derive(Debug,Clone,Copy,strum_macros::Display)]
pub enum ViewMenu {
    Window1,
    Window2,
}

#[derive(Debug,Clone,Copy,strum_macros::Display)]
pub enum EditMenu {
    Copy,
    Delete,
    Paste
}

impl EditMenu {
    const fn base() -> Self {
        EditMenu::Copy
    }
}


impl TopMenu {
    const ALL: [TopMenu; 2] = [TopMenu::Edit(EditMenu::base()), TopMenu::View(ViewMenu::Window1),];
}

impl MenuOption for ViewMenu {
    type Message = TopMenu;

    fn to_message(&self) -> Self::Message {
        TopMenu::View(self.clone())
    }

    fn all(&self) -> &'static [&dyn MenuOption<Message=Self::Message>]
    {
        &ViewMenu::ALL
    }
}

impl MenuOption for EditMenu {
    type Message = TopMenu;

    fn to_message(&self) -> Self::Message{
        TopMenu::Edit(self.clone())
    }

    fn all(&self) -> &'static [&dyn MenuOption<Message=Self::Message>] {
        &EditMenu::ALL
    }
}


impl ViewMenu {
    const ALL: [&'static dyn MenuOption<Message=<Self as MenuOption>::Message>; 2] =  [&ViewMenu::Window1, &ViewMenu::Window2]; 

}


impl EditMenu {
    const ALL: [&'static dyn MenuOption<Message=TopMenu>; 3] = [&EditMenu::Copy, &EditMenu::Delete, &EditMenu::Paste];
}



impl MenuBarOption for TopMenu {
    type Message = TopMenu;
    fn all() -> &'static [Self] {
        &Self::ALL
    }
    fn get_children(&self) -> &'static [&dyn MenuOption<Message=TopMenu>] {
        match self {
            TopMenu::Edit(default) => {
                default.all()
            }
            TopMenu::View(default) => {
                default.all()
            }
        }
    }
}






#[derive(Default)]
struct Example {
    scroll: scrollable::State,
    menubar : menu_bar::State,
    selected_language: Language,
}

type Message = TopMenu;
impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Cell list primitive")
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
            .push(Text::new("Which is your favorite language?"));
            

        content = content.push(Space::with_height(Length::Units(600)));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Elm,
    Ruby,
    Haskell,
    C,
    Javascript,
    Other,
}

impl Language {
    const ALL: [Language; 7] = [
        Language::C,
        Language::Elm,
        Language::Ruby,
        Language::Haskell,
        Language::Rust,
        Language::Javascript,
        Language::Other,
    ];
}

impl Default for Language {
    fn default() -> Language {
        Language::Rust
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Language::Rust => "Rust",
                Language::Elm => "Elm",
                Language::Ruby => "Ruby",
                Language::Haskell => "Haskell",
                Language::C => "C",
                Language::Javascript => "Javascript",
                Language::Other => "Some other language",
            }
        )
    }
}
