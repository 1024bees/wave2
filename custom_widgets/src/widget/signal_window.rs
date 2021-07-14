use iced_native::{
    event, layout, mouse, overlay, scrollable, text, Clipboard, Element, Event, Hasher, Layout,
    Length, Point, Rectangle, Size, Widget,
};

use super::core::DisplayedWave;

/// A widget to represent a singular "SignalWindow"
///
/// This is the core widget on which most components are built on. add doc comments in sooner
/// rather than later
#[allow(missing_debug_implementations)]
pub struct SignalWindow<'a, Message: 'static, Renderer: self::Renderer> {
    window_state: &'a mut menu::State,
    waves: &'a [DisplayedWave],
    width: Length,
    padding: u16,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: <Renderer as self::Renderer>::Style,
}

/// The local state of a [`SignalWindow`].
///
/// [`SignalWindow`]: struct.SignalWindow.html
#[derive(Debug, Clone, Default)]
pub struct State {
    start_time: u32,
    end_time: u32,
    ns_per_unit: f32,
    cursor_location: u32,
    offset: f32,
    hovered_position: f32,
}

impl<'a, Message, Renderer: self::Renderer> SignalWindow<'a, Message, Renderer> {
    /// Creates a new [`SignalWindow`] with the given [`State`], a list of options,
    /// the current selected value(s), and the message to produce when option(s) is / are
    /// selected.
    ///
    /// [`SignalWindow`]: struct.SignalWindow.html
    /// [`State`]: struct.State.html
    pub fn new(window_state: &'a mut State, waves: &'a [DisplayedWave]) -> Self {
        Self {
            window_state,
            waves,
            width: Length::Fill,
            padding: Renderer::DEFAULT_PADDING,
            text_size: None,
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
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for SignalWindow<'a, Message, Renderer>
where
    Message: 'static,
    Renderer: self::Renderer + scrollable::Renderer + 'a,
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

        let width = 10000;
        let height = 10000;

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
        messages: &mut Vec<Message>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let text_size = self.text_size.unwrap_or(renderer.default_size());

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                self.state.hovered_position = cursor_position.x;
            }

            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {}

            _ => {}
        }
        event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        //TODO: redo this all
        self::Renderer::draw(
            renderer,
            layout.bounds(),
            self.waves,
            self.padding,
            self.text_size.unwrap_or(renderer.default_size()),
            self.font,
            &self.style,
        )
    }

    fn overlay(&mut self, layout: Layout<'_>) -> Option<overlay::Element<'_, Message, Renderer>> {
        None
    }
}

/// The renderer of a [`SignalWindow`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`SignalWindow`] in your user interface.
///
/// [`SignalWindow`]: struct.SignalWindow.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: text::Renderer {
    /// The default padding of a [`SignalWindow`].
    ///
    /// [`SignalWindow`]: struct.SignalWindow.html
    const DEFAULT_PADDING: u16;

    /// The [`SignalWindow`] style supported by this renderer.
    ///
    /// [`SignalWindow`]: struct.SignalWindow.html
    type Style: Default;

    
    /// Draws a [`SignalWindow`].
    ///
    /// [`SignalWindow`]: struct.SignalWindow.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        item: &[DisplatedWave],
        padding: u16,
        text_size: u16,
        font: Self::Font,
        style: &<Self as Renderer>::Style,
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
