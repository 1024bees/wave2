use iced::{
    button, Align, Button, Container, Element, Font, HorizontalAlignment, Length, Row, Text,
};
#[derive(Debug, Clone)]
pub enum Message {
    Next,
    Prev,
}

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
                Icon::new('\u{f26d}', Message::Prev),
                Icon::new('\u{f26e}', Message::Next),
            ],
        }
    }
}

pub struct IconBar {
    buttons_state: Vec<Icon>,
}

impl IconBar {
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
        Container::new(
            Row::with_children(buttons)
                .height(Length::Shrink)
                .width(Length::Fill)
                .spacing(20)
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
                border_color: Color::BLACK,
                ..container::Style::default()
            }
        }
    }

    pub struct Button;

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(color_from_str("#dedede"))),
                border_radius: 3.0,
                text_color: Color::BLACK,
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(color_from_str("#c8c8c8"))),
                text_color: Color::BLACK,
                ..self.active()
            }
        }

        fn pressed(&self) -> button::Style {
            button::Style {
                border_width: 1.0,
                background: Some(Background::Color(color_from_str("#a8a8a8"))),

                border_color: Color::BLACK,
                ..self.hovered()
            }
        }
    }
}
