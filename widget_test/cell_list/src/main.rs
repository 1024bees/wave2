use iced::{
    scrollable, Align, Container, Element, Length, Sandbox, Scrollable,
    Settings, Space, Text,
};

use iced::{pick_list, PickList};

use wave2_custom_widgets::cell_list;
use wave2_custom_widgets::cell_list::CellList;
use env_logger;
use log::info;
pub fn main() -> Result<(), iced::Error>{
    env_logger::init();
    info!("TEST");
    Example::run(Settings::default())
}

#[derive(Clone)]
enum Menu {
    Test1,
    Test2
}

impl Menu {
    const ALL : [Menu; 2] = [
        Menu::Test1,
        Menu::Test2
    ];

}
impl ToString for Menu {
    fn to_string(&self) -> String {
        match self {
            Menu::Test1 => "Test1!".into(),
            Menu::Test2 => "Test2!".into(),
            _ => "Unlabeled".into(),
        }
    }

}


#[derive(Default)]
struct Example {
    scroll: scrollable::State,
    pick_list: cell_list::State<Menu>,
    selected_language: Language,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    LanguageSelected(Language),
}

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
            Message::LanguageSelected(language) => {
                self.selected_language = language;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let pick_list = CellList::new(
            &mut self.pick_list,
            &Language::ALL[..],
            &Menu::ALL,
            Message::LanguageSelected,
        )
        .heading("Dog!".into())
        .heading_size(8);

        let mut content = Scrollable::new(&mut self.scroll)
            .width(Length::Fill)
            .align_items(Align::Center)
            .spacing(10)
            .push(Space::with_height(Length::Units(600)))
            .push(Text::new("Which is your favorite language?"))
            .push(pick_list);

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
