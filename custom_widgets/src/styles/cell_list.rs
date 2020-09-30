use iced_core::{Background, Color};
use iced_style::menu::Style as MenuStyle;
/// The appearance of a menu.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub text_color: Color,
    pub background: Background,
    pub border_width: u16,
    pub border_color: Color,
    pub selected_text_color: Color,
    pub selected_background: Background,
    pub heading_background : Background,
}

impl std::default::Default for Style {
    fn default() -> Self {
        Self {
            text_color: Color::BLACK,
            background: Background::Color([0.87, 0.87, 0.87].into()),
            border_width: 1,
            border_color: [0.7, 0.7, 0.7].into(),
            selected_text_color: Color::WHITE,
            selected_background: Background::Color([0.4, 0.4, 1.0].into()),
            heading_background: Background::Color([121.0 / 255.0, 132.0 /255.0, 143.0/ 255.0].into()),
        }
    }
}
/// A set of rules that dictate the style of a container.
pub trait StyleSheet {
    fn options(&self) -> MenuStyle;

    fn active(&self) -> Style;

    /// Produces the style of a container.
    fn hovered(&self) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn options(&self) -> MenuStyle {
        MenuStyle::default()
    }

    fn active(&self) -> Style {
        Style::default()
    }

    fn hovered(&self) -> Style {
        Style {
            border_color: Color::BLACK,
            ..self.active()
        }
    }
}

impl std::default::Default for Box<dyn StyleSheet> {
    fn default() -> Self {
        Box::new(Default)
    }
}

impl<T> From<T> for Box<dyn StyleSheet>
where
    T: 'static + StyleSheet,
{
    fn from(style: T) -> Self {
        Box::new(style)
    }
}

