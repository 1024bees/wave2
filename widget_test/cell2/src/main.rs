use iced::{
    scrollable, Align, Container, Element, Length, Sandbox, Scrollable, Settings, Space, Text,
};

use env_logger;
use log::info;
use wave2_custom_widgets::traits::CellOption;
use wave2_custom_widgets::widget::cell2::{self, Cell2, Entry};
pub fn main() -> Result<(), iced::Error> {
    env_logger::init();
    info!("TEST");
    Example::run(Settings::default())
}

#[derive(Clone)]
enum Menu {
    Test1,
    Test2,
}

impl CellOption for Menu {
    type Message = Message;

    fn all() -> &'static [Menu] {
        &Self::ALL
    }

    fn to_message(&self) -> Self::Message {
        match self {
            Menu::Test1 => Message::Test1,
            Menu::Test2 => Message::Test2,
        }
    }
}

impl Menu {
    const ALL: [Menu; 2] = [Menu::Test1, Menu::Test2];
}
impl std::fmt::Display for Menu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Menu::Test1 => write!(f, "Test1"),
            Menu::Test2 => write!(f, "Test1"),
        }
    }
}

#[derive(Default)]
struct Example {
    scroll: scrollable::State,
    pick_list: cell2::State,
    selected_language: Language,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    LanguageSelected(Language),
    Test1,
    Test2,
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
            _ => {
                println!("Test!");
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let cell = Cell2::with_entries(
            Text::new("Wassup").into(),
            &mut self.pick_list,
            vec![Entry::Item(
                Text::new("Test1").into(),
                Some(Message::Test1),
            )],
        );

        let container = Container::new(cell).width(Length::Units(400));

        let mut content = Scrollable::new(&mut self.scroll)
            .width(Length::Fill)
            .align_items(Align::Center)
            .spacing(10)
            .push(Space::with_height(Length::Units(600)))
            .push(Text::new("Which is your favorite language?"))
            .push(container);

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
