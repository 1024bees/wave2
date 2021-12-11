use iced::{
    scrollable, Align, Container, Element, Length, Sandbox, Scrollable, Settings, Space, Text,
};

use env_logger;
use log::info;
use wave2_custom_widgets::traits::CellOption;
use wave2_custom_widgets::widget::cell2::{self, Cell2, LazyEntry};
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

struct Example {
    scroll: scrollable::State,
    cell_state: cell2::State,
    entries: Vec<LazyEntry<Message>>,
}

impl Default for Example {
    fn default() -> Self {
        Example {
            scroll: scrollable::State::default(),
            cell_state: cell2::State::default(),
            entries: vec![
                LazyEntry::Item("Test1".into(), Some(Message::Test1)),
                LazyEntry::Group(
                    "submenu".into(),
                    vec![
                        LazyEntry::Item("Test2".into(), Some(Message::Test2)),
                        LazyEntry::Item("Test3".into(), Some(Message::Test3)),
                        LazyEntry::Group(
                            "submenu".into(),
                            vec![
                                LazyEntry::Item("Test2".into(), Some(Message::Test2)),
                                LazyEntry::Item("Test3".into(), Some(Message::Test3)),
                            ],
                        ),
                    ],
                ),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Test1,
    Test2,
    Test3,
    Toggle,
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
            Message::Toggle => {
                self.cell_state.selected = !self.cell_state.selected;
                log::info!("selected is {:?}", self.cell_state.selected);
            }
            _ => {
                println!("{:?}", message);
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let cell = Cell2::with_entries(
            Text::new("Wassup").width(Length::Fill).into(),
            &mut self.cell_state,
            &self.entries,
        )
        .set_single_click(|| Message::Toggle);

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
