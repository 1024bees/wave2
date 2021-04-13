/*! Interfaces required by custom widgets.


[`Event`]: iced_native::event::Event;
[`widget`]: crate::widget;
[`Cell`]: crate::widget::cell::Cell;
!*/



/** Trait for generation options that are displayed when a [`Cell`] widget is right clicked.

The string generated from std::fmt::Display is what will be shown in the overlay menu 

**/
pub trait CellOption: std::fmt::Display + Clone + 'static {
    type Message: 'static;
    ///Return a slice of all option variants
    fn all() -> &'static [Self];

    ///
    fn to_message(&self) -> Self::Message;
}


pub use wave2_widget_derives::{MenuOption,MenuBarOption};
