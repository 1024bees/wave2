/*! Interfaces required by custom widgets.


[`Event`]: iced_native::event::Event;
[`widget`]: crate::widget;
[`Cell`]: crate::widget::cell::Cell;
[`CellConfig`]: crate::traits::CellConfig;
!*/


/** Configuration trait for the [`Cell`] widget.

Each message [`Cell`] emits should be a thin wrapper around an [Event]. Broadly, messages can be optionally emitted for the following events.

- Click
- Double click
- Shift key press/release
- Ctrl key press/release

[`CellConfig`] also allows for an enum of Options to be specified. These options sho

**/
pub trait CellConfig {
    type Option: CellOption;
    type Payload;
}


/** Trait for generation options that are displayed when a [`Cell`] widget is right clicked.

The generated string from Display is what will

**/
pub trait CellOption: std::fmt::Display + Clone + 'static {
    type Message: 'static;
    ///Return a slice of all option variants
    fn all() -> &'static [Self];

    ///
    fn to_message(&self) -> Self::Message;
}
