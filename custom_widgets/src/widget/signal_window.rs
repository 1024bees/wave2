use iced_native::{
    event, layout, mouse, overlay, text, Clipboard, Element, Event, Hasher, Layout, Length, Point,
    Rectangle, Size, Widget,
};

use crate::core::signal_window::DisplayedWave;

use log;
/// A widget to view signals. The signal window is the core of the wave2 app. This is the widget
/// that does the heavy lifting of vizualising waves and displaying values of vectors
///
/// Horizontal scrolling is built in as part of the widget.
///
#[allow(missing_debug_implementations)]
pub struct SignalWindow<'a, Message: 'static, Renderer: self::Renderer> {
    waves: &'a [DisplayedWave],
    state: &'a mut State,
    width: Length,
    padding: u16,
    on_click: Option<Box<dyn Fn(u32) -> Message>>,
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
#[derive(Debug, Clone)]
pub struct State {
    pub start_time: u32,
    pub end_time: u32,
    pub ns_per_pixel: f32,
    pub cursor_location: u32,
    pub offset: f32,
    pub hovered_position: f32,
    zoom: i32,
    pub(crate) ppf: f64,
    pub(crate) ns_per_frame: f64,
    scroller_grabbed_at: Option<f32>,
}

impl Default for State {
    fn default() -> State {
        State {
            start_time: 0,
            end_time: 4,
            ns_per_pixel: 0.005,
            cursor_location: 0,
            offset: 0.0,
            hovered_position: 0.0,
            scroller_grabbed_at: None,
            zoom: 0,
            ppf: 200.0,
            ns_per_frame: 1.0,
        }
    }
}

impl State {
    /// Display
    fn ns_per_screen(&self, bounds: Rectangle) -> f32 {
        self.ns_per_pixel * bounds.width
    }

    /// Creates a new [`State`] with the scrollbar located at the left.
    pub fn new() -> Self {
        State::default()
    }

    pub fn init_bounds(&mut self, bounds: (u32, u32)) {
        self.start_time = bounds.0;
        self.end_time = bounds.1;
    }

    pub fn set_bounds(&mut self, maybe_bounds: [Option<u32>; 2]) {
        for (idx, val) in maybe_bounds.iter().enumerate() {
            if val.is_some() {}
        }
    }

    /// Apply a scrolling offset to the current [`State`], given the bounds of
    /// the [`SignalWindow`] and its contents.
    pub fn scroll(&mut self, delta_x: f32, bounds: Rectangle) {
        self.offset = (self.offset - delta_x * self.ns_per_pixel)
            .max(0.0)
            .min((self.end_time as f32) - self.ns_per_screen(bounds));
    }

    /// Moves the scroll position to a relative amount, given the bounds of
    /// the [`SignalWindow`] and its contents.
    ///
    /// `0` represents scrollbar at the top, while `1` represents scrollbar at
    /// the bottom.
    pub fn scroll_to(&mut self, percentage: f32, bounds: Rectangle) {
        self.offset = ((((self.end_time as f32) - self.ns_per_screen(bounds))
            - self.start_time as f32) as f32
            * percentage)
            .max(0.0)
    }

    /// Calculates the zoom factor of the [`SignalWindow`]. This logic is mirrored from GtkWave's
    /// calczoom function
    pub fn calczoom(&mut self, zoom_factor: i32) {
        self.zoom += zoom_factor;
        self.zoom = self.zoom.clamp(0, 63);
        log::info!("zoomfactor is {}", self.zoom);
        let lnspf: usize = 1 << self.zoom;
        if self.zoom <= 3 {
            self.ppf = 200.0;
            self.ns_per_frame = lnspf as f64;
            log::info!("state is {:#?}", self);
        } else {
            let nspf = lnspf as f64;
            let pow_base10 = nspf.log10().round() as i32;

            let nsperframe2 = 10.0_f64.powi(pow_base10);
            self.ppf = 200.0 * nsperframe2 / nspf;
            log::info!("scale is {}", nsperframe2 / nspf);
            self.ns_per_frame = nsperframe2;
            log::info!("nsperframe is {}", self.ns_per_frame);
        }
        self.ns_per_pixel = (self.ns_per_frame / self.ppf) as f32;
    }
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
            scrollbar_width: 20,
            scroller_width: 20,
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

    pub fn on_click(mut self, on_click: impl Fn(u32) -> Message + 'static) -> Self {
        self.on_click = Some(Box::new(on_click));
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

    /// Sets the scrollbar width of the [`SignalWindow`] .
    ///
    /// Silently enforces a minimum value of 1.
    pub fn scrollbar_width(mut self, scrollbar_width: u16) -> Self {
        self.scrollbar_width = scrollbar_width.max(1);
        self
    }

    /// Sets the scrollbar margin of the [`SignalWindow`] .
    pub fn scrollbar_margin(mut self, scrollbar_margin: u16) -> Self {
        self.scrollbar_margin = scrollbar_margin;
        self
    }

    /// Sets the scroller width of the [`SignalWindow`] .
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
        messages: &mut Vec<Message>,
    ) -> event::Status {
        let bounds = layout.bounds();
        //let text_size = self.text_size.unwrap_or(renderer.default_size());

        let is_mouse_over = bounds.contains(cursor_position);

        let scrollbar = renderer.wave_scrollbar(
            bounds,
            self.state,
            self.scrollbar_width,
            self.scrollbar_margin,
            self.scroller_width,
        );
        let is_mouse_over_scrollbar = scrollbar
            .as_ref()
            .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
            .unwrap_or(false);

        if is_mouse_over {
            match event {
                Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                    match delta {
                        // FIXME: As I currently understand, ScrollDelta captures the movement of
                        // the scroller of the mouse; this movement is still being grabbed in the y dimension,
                        // I don't have a mouse that can scroll in x ...?
                        mouse::ScrollDelta::Lines { y, .. } => {
                            // TODO: Configurable speed (?)
                            self.state.scroll(y * 60.0, bounds);
                        }
                        mouse::ScrollDelta::Pixels { .. } => {
                            //self.state.scroll(y, bounds);
                        }
                    }

                    return event::Status::Captured;
                }
                Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    self.state.hovered_position =
                        self.state.offset + self.state.ns_per_pixel * (position.x - bounds.x);
                }
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    self.state.cursor_location = self.state.hovered_position as u32;
                    if let Some(ref click_fn) = self.on_click {
                        messages.push(click_fn(self.state.cursor_location));
                    }
                }

                _ => {}
            }
        }

        if self.state.scroller_grabbed_at.is_some() {
            match event {
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    self.state.scroller_grabbed_at = None;

                    return event::Status::Captured;
                }
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    if let (Some(scrollbar), Some(scroller_grabbed_at)) =
                        (scrollbar, self.state.scroller_grabbed_at)
                    {
                        self.state.scroll_to(
                            scrollbar.scroll_percentage(scroller_grabbed_at, cursor_position),
                            bounds,
                        );

                        return event::Status::Captured;
                    }
                }
                _ => {}
            }
        } else if is_mouse_over_scrollbar {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    if let Some(scrollbar) = scrollbar {
                        if let Some(scroller_grabbed_at) = scrollbar.grab_scroller(cursor_position)
                        {
                            self.state.scroll_to(
                                scrollbar.scroll_percentage(scroller_grabbed_at, cursor_position),
                                bounds,
                            );

                            self.state.scroller_grabbed_at = Some(scroller_grabbed_at);

                            return event::Status::Captured;
                        }
                    }
                }
                _ => {}
            }
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
            self.font,
            &self.style,
        )
    }

    fn overlay(&mut self, _layout: Layout<'_>) -> Option<overlay::Element<'_, Message, Renderer>> {
        None
    }
}

/// The scrollbar of a [`SignalWindow`].
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
        let percentage =
            (cursor_position.x - self.bounds.x - self.scroller.bounds.width * grabbed_at)
                / (self.bounds.width - self.scroller.bounds.width);
        let percentage = percentage.clamp(0.0, 1.0);
        log::info!("scroll percentage {}", percentage);
        percentage
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
    /// [`SignalWindow`].
    fn wave_scrollbar(
        &self,
        bounds: Rectangle,
        state: &State,
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
        state: &State,
        scrollbar: Option<Scrollbar>,
        padding: u16,
        text_size: u16,
        font: Self::Font,
        style: &Self::Style,
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
