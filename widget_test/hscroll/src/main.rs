mod style;

use iced::{
    Column, Container, Element, Length, Radio, Row, Rule, Sandbox,
    Settings, Space, Text,
};

use wave2_custom_widgets::widget::hscroll;
use wave2_custom_widgets::widget::hscroll::HScroll;


pub fn main() -> iced::Result {
    pretty_env_logger::init();
    HScrollDemo::run(Settings::default())
}

struct HScrollDemo {
    theme: style::Theme,
    variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(style::Theme),
}

impl Sandbox for HScrollDemo {
    type Message = Message;

    fn new() -> Self {
        HScrollDemo {
            theme: Default::default(),
            variants: Variant::all(),
        }
    }

    fn title(&self) -> String {
        String::from("HScroll - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => self.theme = theme,
        }
    }

    fn view(&mut self) -> Element<Message> {
        let HScrollDemo {
            theme, variants, ..
        } = self;

        let choose_theme = style::Theme::ALL.iter().fold(
            Column::new().spacing(10).push(Text::new("Choose a theme:")),
            |column, option| {
                column.push(
                    Radio::new(
                        *option,
                        &format!("{:?}", option),
                        Some(*theme),
                        Message::ThemeChanged,
                    )
                    .style(*theme),
                )
            },
        );

        let hscroll_row = Column::with_children(
            variants
                .iter_mut()
                .map(|variant| {
                    let mut hscroll = HScroll::new(&mut variant.state)
                        .padding(10)
                        .width(Length::Shrink)
                        .height(Length::Fill)
                        .style(*theme)
                        .push(Text::new(variant.title));

                    if let Some(scrollbar_width) = variant.scrollbar_width {
                        hscroll = hscroll
                            .scrollbar_width(scrollbar_width)
                            .push(Text::new(format!(
                                "scrollbar_width: {:?}",
                                scrollbar_width
                            )));
                    }

                    if let Some(scrollbar_margin) = variant.scrollbar_margin {
                        hscroll = hscroll
                            .scrollbar_margin(scrollbar_margin)
                            .push(Text::new(format!(
                                "scrollbar_margin: {:?}",
                                scrollbar_margin
                            )));
                    }

                    if let Some(scroller_width) = variant.scroller_width {
                        hscroll = hscroll
                            .scroller_width(scroller_width)
                            .push(Text::new(format!(
                                "scroller_width: {:?}",
                                scroller_width
                            )));
                    }

                    hscroll = hscroll
                        .push(Space::with_width(Length::Units(100)))
                        .push(Text::new(
                            "Some content that should wrap within the \
                            hscroll. Let's output a lot of short words, so \
                            that we'll make sure to see how wrapping works \
                            with these scrollbars.",
                        ))
                        .push(Space::with_width(Length::Units(1200)))
                        .push(Text::new("Middle"))
                        .push(Space::with_width(Length::Units(1200)))
                        .push(Text::new("The End."));

                    Container::new(hscroll)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(*theme)
                        .into()
                })
                .collect(),
        )
        .spacing(20)
        .width(Length::Fill)
        .height(Length::Fill);

        let content = Column::new()
            .spacing(20)
            .padding(20)
            .push(choose_theme)
            .push(Rule::horizontal(20).style(self.theme))
            .push(hscroll_row);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(self.theme)
            .into()
    }
}

/// A version of a hscroll
struct Variant {
    title: &'static str,
    state: hscroll::State,
    scrollbar_width: Option<u16>,
    scrollbar_margin: Option<u16>,
    scroller_width: Option<u16>,
}

impl Variant {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                title: "Default Scrollbar",
                state: hscroll::State::new(),
                scrollbar_width: None,
                scrollbar_margin: None,
                scroller_width: None,
            },
            Self {
                title: "Slimmed & Margin",
                state: hscroll::State::new(),
                scrollbar_width: Some(4),
                scrollbar_margin: Some(3),
                scroller_width: Some(4),
            },
            Self {
                title: "Wide Scroller",
                state: hscroll::State::new(),
                scrollbar_width: Some(4),
                scrollbar_margin: None,
                scroller_width: Some(10),
            },
            Self {
                title: "Narrow Scroller",
                state: hscroll::State::new(),
                scrollbar_width: Some(10),
                scrollbar_margin: None,
                scroller_width: Some(4),
            },
        ]
    }
}
