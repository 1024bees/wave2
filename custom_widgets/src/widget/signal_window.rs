use iced_native::{
    event, layout, mouse, overlay, scrollable, text, Clipboard, Element, Event, Hasher, Layout,
    Length, Point, Rectangle, Size, Widget,
};

use crate::core::signal_window::DisplayedWave;

/// A widget to represent a singular "SignalWindow"
///
/// This is the core widget on which most components are built on. add doc comments in sooner
/// rather than later
#[allow(missing_debug_implementations)]
pub struct SignalWindow<'a, Message: 'static, Renderer: self::Renderer> {
    waves: &'a [DisplayedWave],
    state: &'a mut State,
    width: Length,
    padding: u16,
    on_click: Option<Message>,
    text_size: Option<u16>,
    font: Renderer::Font,
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    style: <Renderer as self::Renderer>::Style,
}

/// The local state of a [`SignalWindow`].
///
/// [`SignalWindow`]: struct.SignalWindow.html
#[derive(Debug, Clone, Default)]
pub struct State {
    pub(crate) start_time: u32,
    pub(crate) end_time: u32,
    pub(crate) ns_per_unit: f32,
    pub(crate) cursor_location: u32,
    pub(crate) offset: f32,
    pub(crate) hovered_position: f32,
}

impl<'a, Message, Renderer: self::Renderer> SignalWindow<'a, Message, Renderer> {
    /// Creates a new [`SignalWindow`] with the given [`State`], a list of options,
    /// the current selected value(s), and the message to produce when option(s) is / are
    /// selected.
    ///
    /// [`SignalWindow`]: struct.SignalWindow.html
    /// [`State`]: struct.State.html
    pub fn new(waves: &'a [DisplayedWave], state: &'a mut State) -> Self {
        Self {
            waves,
            state,
            width: Length::Fill,
            padding: Renderer::DEFAULT_PADDING,
            text_size: None,
            on_click: None,
            scrollbar_margin: 0,
            scrollbar_width: 100,
            scroller_width: 10,
            font: Default::default(),
            style: Default::default(),
        }
    }

    /// Sets the width of the [`SignalWindow`].
    ///
    /// [`SignalWindow`]: struct.SignalWindow.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the padding of the [`SignalWindow`].
    ///
    /// [`SignalWindow`]: struct.SignalWindow.html
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the text size of the [`SignalWindow`].
    ///
    /// [`SignalWindow`]: struct.SignalWindow.html
    pub fn text_size(mut self, size: u16) -> Self {
        self.text_size = Some(size);
        self
    }

    /// Sets the font of the [`SignalWindow`].
    ///
    /// [`SignalWindow`]: struct.SignalWindow.html
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`SignalWindow`].
    ///
    /// [`SignalWindow`]: struct.SignalWindow.html
    pub fn style(mut self, style: impl Into<<Renderer as self::Renderer>::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the scrollbar width of the [`HScroll`] .
    ///
    /// Silently enforces a minimum value of 1.
    pub fn scrollbar_width(mut self, scrollbar_width: u16) -> Self {
        self.scrollbar_width = scrollbar_width.max(1);
        self
    }

    /// Sets the scrollbar margin of the [`HScroll`] .
    pub fn scrollbar_margin(mut self, scrollbar_margin: u16) -> Self {
        self.scrollbar_margin = scrollbar_margin;
        self
    }

    /// Sets the scroller width of the [`HScroll`] .
    /// Silently enforces a minimum value of 1.
    pub fn scroller_width(mut self, scroller_width: u16) -> Self {
        self.scroller_width = scroller_width.max(1);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for SignalWindow<'a, Message, Renderer>
where
    Message: 'static,
    Renderer: self::Renderer + 'a,
{
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        use std::f32;

        let limits = limits.width(Length::Fill).height(Length::Shrink);
        let text_size = self.text_size.unwrap_or(renderer.default_size());

        let bounds = limits.max();

        let width = 10000.0;
        let height = 10000.0;

        let size = limits.resolve(Size::new(width, height));

        layout::Node::new(size)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash as _;

        self.width.hash(state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _messages: &mut Vec<Message>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let text_size = self.text_size.unwrap_or(renderer.default_size());

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                self.state.hovered_position =
                    self.state.offset + self.state.ns_per_unit * cursor_position.x;
            }

            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                unimplemented!()
            }

            _ => {}
        }
        event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();
        let offset = self.state.offset;
        let scrollbar = renderer.wave_scrollbar(
            bounds,
            self.state, 
            self.scrollbar_width,
            self.scrollbar_margin,
            self.scroller_width,
        );

        self::Renderer::draw(
            renderer,
            layout.bounds(),
            self.waves,
            self.state, 
            scrollbar,
            self.padding,
            self.text_size.unwrap_or(renderer.default_size()),
            self.font
        )
    }

    fn overlay(&mut self, layout: Layout<'_>) -> Option<overlay::Element<'_, Message, Renderer>> {
        None
    }
}

/// The scrollbar of a [`HScroll`].
#[derive(Debug)]
pub struct Scrollbar {
    /// The outer bounds of the scrollable, including the [`Scrollbar`] and
    /// [`Scroller`].
    pub outer_bounds: Rectangle,

    /// The bounds of the [`Scrollbar`].
    pub bounds: Rectangle,

    /// The margin within the [`Scrollbar`].
    pub margin: u16,

    /// The bounds of the [`Scroller`].
    pub scroller: Scroller,
}

impl Scrollbar {
    fn is_mouse_over(&self, cursor_position: Point) -> bool {
        self.outer_bounds.contains(cursor_position)
    }

    fn grab_scroller(&self, cursor_position: Point) -> Option<f32> {
        if self.outer_bounds.contains(cursor_position) {
            Some(if self.scroller.bounds.contains(cursor_position) {
                (cursor_position.x - self.scroller.bounds.x) / self.scroller.bounds.width
            } else {
                0.5
            })
        } else {
            None
        }
    }

    fn scroll_percentage(&self, grabbed_at: f32, cursor_position: Point) -> f32 {
        (cursor_position.x - self.bounds.x - self.scroller.bounds.width * grabbed_at)
            / (self.bounds.width - self.scroller.bounds.width)
    }
}

/// The handle of a [`Scrollbar`].
#[derive(Debug, Clone, Copy)]
pub struct Scroller {
    /// The bounds of the [`Scroller`].
    pub bounds: Rectangle,
}

/// The renderer of a [`SignalWindow`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`SignalWindow`] in your user interface.
///
/// [`SignalWindow`]: struct.SignalWindow.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: text::Renderer + Sized {
    /// The default padding of a [`SignalWindow`].
    ///
    /// [`SignalWindow`]: struct.SignalWindow.html
    const DEFAULT_PADDING: u16;

    /// The [`SignalWindow`] style supported by this renderer.
    ///
    /// [`SignalWindow`]: struct.SignalWindow.html
    type Style: Default;

    /// Returns the [`Scrollbar`] given the bounds and content bounds of a
    /// [`HScroll`].
    fn wave_scrollbar(
        &self,
        bounds: Rectangle,
        state:  &State,
        scrollbar_width: u16,
        scrollbar_margin: u16,
        scroller_width: u16,
    ) -> Option<Scrollbar>;

    /// Draws a [`SignalWindow`].
    ///
    /// [`SignalWindow`]: struct.SignalWindow.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        waves: &[DisplayedWave],
        state: & State,
        scrollbar: Option<Scrollbar>,
        padding: u16,
        text_size: u16,
        font: Self::Font,
        //style: &<Self as Renderer>::Style,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for SignalWindow<'a, Message, Renderer>
where
    Renderer: self::Renderer + 'a,
    Message: 'static,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
