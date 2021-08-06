use iced_core::{Background, Color};
/// The appearance of a menu.
#[derive(Debug, Clone, Copy)]
pub struct Style;

impl std::default::Default for Style {
    fn default() -> Self {
        Self
    }
}
/// A set of rules that dictate the style of a container.
pub trait StyleSheet {
    fn active(&self) -> Style;

    /// Produces the style of a container.
    fn hovered(&self) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn active(&self) -> Style {
        Style::default()
    }

    fn hovered(&self) -> Style {
        Style::default()
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
