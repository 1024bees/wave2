//! A cell with a nested menu
//!
use std::hash::Hash;

use iced_graphics::{Point, Size};
use iced_native::{
    event, layout, mouse, overlay, row, touch, Clipboard, Element, Event, Layout, Length, Row,
    Widget,
};

use iced_aw::core::renderer::DrawEnvironment;

use super::overlay::cell_overlay::Cell2Overlay;

/// A cell with a nested menu
///
/// # Example
/// ```
/// # use iced_aw::menu::{State, Section, Entry};
/// # use iced_native::{Text, renderer::Null};
/// #
///
/// #[derive(Clone, Debug)]
/// enum Message {
///     Entry1,
///     Entry2,
///     Entry3,    
/// }
///
/// let mut menu_state = State::new();
///
/// let menu = Cell2::new(&mut menu_state)
///     .push(Section::new(
///         Text::new("Section 1"),
///         vec![
///             Entry::Item(Text::new("Entry 1").into(), Some(Message::Entry1)),
///             Entry::Item(Text::new("Entry 2").into(), Some(Message::Entry2)),
///         ]
///     ))
///     .push(Section::new(
///         Text::new("Section2"),
///         vec![
///             Entry::Item(Text::new("Entry 3").into(), Some(Message::Entry3)),
///         ]
///     ));
/// ```
#[allow(missing_debug_implementations)]
pub struct Cell2<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    /// The state of the [`Cell2`](Cell2).
    state: &'a mut State,
    /// A vector containing the [`Section`](Section)s of the [`Cell2`](Cell2).
    overlay_entries: Vec<Entry<'a, Message, Renderer>>,
    /// The width of the [`Cell2`](Cell2).
    width: Length,
    /// The height of the [`Cell2`](Cell2).
    height: Length,
    /// The space between the [`Section`](Section)s of the [`Cell2`](Cell2).
    /// Text to be rendered by the [`Cell2`](Cell2)
    item: Element<'a, Message, Renderer>,
    /// Message to be generated by the [`Cell2`](Cell2) when a [`mouse::click::Kind::Single`]
    /// occurs
    on_click: Option<Box<dyn Fn() -> Message>>,
    /// Message to be generated by the [`Cell2`](Cell2) when a double click [`mouse::click::Kind::Double`]
    on_double_click: Option<Box<dyn Fn() -> Message>>,
    /// The [`Style`](crate::style::menu::Style) of the [`Cell2`](Cell2).
    style: Renderer::Style,
}

impl<'a, Message, Renderer> Cell2<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: self::Renderer,
{
    /// Creates a new [`Cell2`](Cell2) with an empty list of sections.
    ///
    /// It expects:
    ///     * a mutable reference to the [`Cell2`](Cell2)'s [`State`](State).
    pub fn new(item: Element<'a, Message, Renderer>, state: &'a mut State) -> Self {
        Cell2::with_entries(item, state, Vec::new())
    }

    /// Creates a new [`Cell2`](Cell2) with the given list of sections.
    ///
    /// It expects:
    ///     * a mutable reference to the [`Cell2`](Cell2)'s [`State`](State).
    ///     * a vector containing the sections.
    pub fn with_entries(
        item: Element<'a, Message, Renderer>,
        state: &'a mut State,
        sections: Vec<Entry<'a, Message, Renderer>>,
    ) -> Self {
        Self {
            state,
            overlay_entries: sections,
            width: Length::Fill,
            height: Length::Shrink,
            item,
            style: Renderer::Style::default(),
            on_click: None,
            on_double_click: None,
        }
    }

    /// Sets the style of the [`Cell2`](Cell2).
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the callback for a single click of the [`Cell2`](Cell2).
    pub fn set_single_click<F: Fn() -> Message + 'static>(mut self, callback: F) -> Self {
        self.on_click = Some(Box::new(callback));
        self
    }
    /// Sets the callback for a single click of the [`Cell2`](Cell2).
    pub fn set_double_click<F: Fn() -> Message + 'static>(mut self, callback: F) -> Self {
        self.on_double_click = Some(Box::new(callback));
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Cell2<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: self::Renderer
        + row::Renderer
        + iced_native::text::Renderer
        + crate::widget::overlay::cell_overlay::Renderer, //crate::native::overlay::menu::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &iced_native::layout::Limits) -> layout::Node {
        self.item.layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let children = layout.children();

        if !self.state.menu_open && layout.bounds().contains(cursor_position) {
            let no_entries = self.overlay_entries.is_empty();
            let status = match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                    if !no_entries {
                        log::info!("wassup we are right clicking");
                        self.state.menu_open = true;
                        event::Status::Captured
                    } else {
                        event::Status::Ignored
                    }
                }
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    if let Some(ref callback) = self.on_click {
                        messages.push(callback());
                    }

                    event::Status::Captured
                }

                _ => event::Status::Ignored,
            };

            status
        } else {
            if layout.bounds().contains(cursor_position) {
                match event {
                    _ => event::Status::Ignored,
                }
            } else {
                event::Status::Ignored
            }
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: iced_native::Layout<'_>,
        cursor_position: iced_graphics::Point,
        viewport: &iced_graphics::Rectangle,
    ) -> Renderer::Output {
        <Renderer as self::Renderer>::draw(
            renderer,
            self.state,
            DrawEnvironment {
                defaults,
                layout,
                cursor_position,
                style_sheet: &self.style,
                viewport: Some(viewport),
                focus: (),
            },
            &self.item,
        )
    }

    fn hash_layout(&self, state: &mut iced_native::Hasher) {
        #[allow(clippy::missing_docs_in_private_items)]
        self.item.hash_layout(state);
    }

    fn overlay(&mut self, layout: Layout<'_>) -> Option<overlay::Element<'_, Message, Renderer>> {
        if !self.state.menu_open {
            return None;
        }

        let bounds = layout.bounds();

        let position = Point::new(bounds.x, bounds.y + bounds.height);

        Some(Cell2Overlay::new(&mut self.state, &self.overlay_entries, position).overlay())
    }
}

/// The renderer of  a [`Cell2`](Cell2).
///
/// Your renderer will need to implement this trait before being
/// able to use a [`Cell2`](Cell2) in your user interface.
pub trait Renderer: iced_native::Renderer {
    /// The style supported by this renderer.
    type Style: Default;

    /// Draws a [`Cell2`](Cell2).
    fn draw<Message>(
        &mut self,
        state: &State,
        env: DrawEnvironment<'_, Self::Defaults, Self::Style, ()>,
        item: &Element<'_, Message, Self>,
    ) -> Self::Output;
}

impl Renderer for iced_native::renderer::Null {
    type Style = ();

    fn draw<Message>(
        &mut self,
        state: &State,
        _env: DrawEnvironment<'_, Self::Defaults, Self::Style, ()>,
        item: &Element<'_, Message, Self>,
    ) -> Self::Output {
    }
}

/// The state of the [`Cell2`](Cell2).
#[derive(Debug, Default)]
pub struct State {
    /// The stack containing the indices that build a path to the opened [`Entry`](Entry).
    pub(crate) stack: Vec<usize>,
    pub selected: bool,
    pub(crate) menu_open: bool,
}

impl State {
    /// Creates a new [`State`](State).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stack: Vec::new(),
            selected: false,
            menu_open: false,
        }
    }
}

/// An [`Entry`](Entry) of a [`Section`](Section) or `[Entry](Entry)::Group`.
#[allow(missing_debug_implementations)]
pub enum Entry<'a, Message, Renderer> {
    /// An [`Entry`] item holding an [`Element`](iced_native::Element) for it's label
    /// and a message that is send when the item is pressed.
    /// If the message is none the item will be disabled.
    Item(Element<'a, Message, Renderer>, Option<Message>),
    /// An [`Entry`] item that can be toggled.
    Toggle(
        Element<'a, Message, Renderer>,
        bool,
        Option<Box<dyn Fn(bool) -> Message + 'static>>,
    ),
    /// A group of [`Entry`](Entry)s holding an [`Element`](iced_native::Element) for
    /// it's label.
    /// If the vector is empty the group will be disabled.
    Group(
        Element<'a, Message, Renderer>,
        Vec<Entry<'a, Message, Renderer>>,
    ),
    /// A separator.
    Separator,
}

impl<'a, Message, Renderer: iced_native::Renderer> Entry<'a, Message, Renderer> {
    /// Applies a transformation to the produced message of the [`Element`](Element).
    ///
    /// Take a look into the [`Element`](iced_native::Element) documentation for
    /// more information.
    pub fn map<F, B>(self, f: F) -> Entry<'a, B, Renderer>
    where
        Message: 'static,
        Renderer: 'a,
        B: 'static,
        F: 'static + Copy + Fn(Message) -> B,
    {
        match self {
            Entry::Item(label, message) => Entry::Item(label.map(f), message.map(f)),
            Entry::Toggle(label, toggled, message) => Entry::Toggle(
                label.map(f),
                toggled,
                message.map(|m| {
                    // TODO: I can't believe that this actually works...
                    Box::new(move |b: bool| f(m(b))) as Box<dyn Fn(bool) -> B>
                }),
            ),
            Entry::Group(label, entries) => Entry::Group(
                label.map(f),
                entries.into_iter().map(|entry| entry.map(f)).collect(),
            ),
            Entry::Separator => Entry::Separator,
        }
    }
}

impl<'a, Message, Renderer> From<Cell2<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a
        + self::Renderer
        + row::Renderer
        + iced_native::text::Renderer
        + crate::widget::overlay::cell_overlay::Renderer,
{
    fn from(menu: Cell2<'a, Message, Renderer>) -> Self {
        Element::new(menu)
    }
}
