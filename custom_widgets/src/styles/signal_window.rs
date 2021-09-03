use iced_core::Color;

use crate::utils::color_from_str;

/// Style Configuration for signal window
///
#[derive(Debug, Clone, Copy)]
pub struct SignalWindow {
    pub hscroll_cursor_color: Color,
    pub background_color: Color,
}

impl std::default::Default for SignalWindow {
    fn default() -> Self {
        SignalWindow {
            hscroll_cursor_color: color_from_str("#797986"),
            background_color: Color::BLACK,
        }
    }
}
/// A set of rules that dictate the style of a container.
pub trait StyleSheet {
    fn default(&self) -> SignalWindow;
}

struct Default;

impl StyleSheet for Default {
    fn default(&self) -> SignalWindow {
        SignalWindow::default()
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
