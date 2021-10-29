use iced::{
    button, text_input, Align, Button, Command, Container, Element, Font, HorizontalAlignment,
    Length, Row, Space, Text, TextInput,
};

use crate::signals::Bound;
pub use crate::signals::{IconBarMessage, Message};

const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("./fonts/bootstrap-icons.ttf"),
};

fn icon_from_char(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .font(ICONS)
        .size(20)
        .horizontal_alignment(HorizontalAlignment::Center)
}

fn time_to_text(time: u32) -> Text {
    Text::new(format!("{} ns", time))
        .size(12)
        .horizontal_alignment(HorizontalAlignment::Center)
}

struct Icon {
    button: button::State,
    icon_char: char,
    message: Message,
}

impl Icon {
    fn new(icon_char: char, message: Message) -> Self {
        Icon {
            button: button::State::default(),
            icon_char,
            message,
        }
    }
}

impl Default for IconBar {
    fn default() -> Self {
        IconBar {
            buttons_state: vec![
                Icon::new('\u{f5b0}', Message::ZoomIn),
                Icon::new('\u{f5b1}', Message::ZoomOut),
                Icon::new('\u{f264}', Message::GoToStart),
                Icon::new('\u{f265}', Message::GoToEnd),
                Icon::new('\u{f26d}', Message::Prev),
                Icon::new('\u{f26e}', Message::Next),
            ],
            bounds_state: (text_input::State::default(), text_input::State::default()),
            bounds_str: (String::default(), String::default()),
            curor_location: None,
        }
    }
}

pub struct IconBar {
    buttons_state: Vec<Icon>,
    bounds_state: (text_input::State, text_input::State),
    bounds_str: (String, String),
    curor_location: Option<String>,
}

impl IconBar {
    pub fn update(&mut self, message: IconBarMessage) -> Command<Message> {
        match message {
            IconBarMessage::TIUpdate(bound, val) => match bound {
                Bound::Left => self.bounds_str.0 = val,
                Bound::Right => self.bounds_str.1 = val,
            },
            IconBarMessage::BoundsUpdate(bound) => {
                let str_ref = if matches!(bound, Bound::Left) {
                    std::mem::replace(&mut self.bounds_str.0, String::new())
                } else {
                    std::mem::replace(&mut self.bounds_str.1, String::new())
                };
                if let Ok(value) = str_ref.parse::<u32>() {

                }
            }

            _ => {
                panic!(
                    "Message: {:?} is being sent to the Icon Bar when it shouldnt be.. we die!",
                    message
                )
            }
        }
        Command::none()
    }

    pub fn view(&mut self) -> Element<Message> {
        let buttons: Vec<Element<Message>> = self
            .buttons_state
            .iter_mut()
            .map(|icon| {
                Button::new(&mut icon.button, icon_from_char(icon.icon_char))
                    .on_press(icon.message.clone())
                    .style(style::Button)
                    .into()
            })
            .collect();

        let left_bounds = TextInput::new(
            &mut self.bounds_state.0,
            "0",
            self.bounds_str.0.as_str(),
            |val| Message::IconBarMessage(IconBarMessage::TIUpdate(Bound::Left, val)),
        )
        .on_submit(Message::IconBarMessage(IconBarMessage::BoundsUpdate(
            Bound::Left,
        )))
        .size(15)
        .width(Length::Units(150));
        let right_bounds = TextInput::new(
            &mut self.bounds_state.1,
            "0",
            self.bounds_str.1.as_str(),
            |val| Message::IconBarMessage(IconBarMessage::TIUpdate(Bound::Right, val)),
        )
        .on_submit(Message::IconBarMessage(IconBarMessage::BoundsUpdate(
            Bound::Right,
        )))
        .size(15)
        .width(Length::Units(150));

        Container::new(
            Row::with_children(buttons)
                //push barrier
                .push(left_bounds)
                .push(right_bounds)
                //push barrier
                .push(time_to_text(5))
                .push(time_to_text(6))
                .push(Space::with_width(Length::Fill))
                .height(Length::Shrink)
                .width(Length::Fill)
                .align_items(Align::Center),
        )
        .style(style::Container)
        .into()
    }
}

mod style {
    use iced::{button, container, Background, Color};
    use wave2_custom_widgets::utils::color_from_str;

    pub struct Container;

    impl container::StyleSheet for Container {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(color_from_str("#dedede"))),
                text_color: Some(Color::BLACK),
                ..container::Style::default()
            }
        }
    }

    pub struct Button;

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(color_from_str("#dedede"))),
                //border_radius: 3.0,
                text_color: Color::BLACK,
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(color_from_str("#c8c8c8"))),
                ..self.active()
            }
        }

        fn pressed(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(color_from_str("#a8a8a8"))),
                ..self.hovered()
            }
        }
    }
}
