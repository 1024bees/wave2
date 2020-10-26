use iced_native::{
    keyboard, layout, mouse, overlay,
    overlay::menu::{self, Menu},
    scrollable, text, Clipboard, Element, Event, Hasher, Layout, Length, Point,
    Rectangle, Size, Widget,
};

use log::info;

/// A widget to represent a singular "Cell"
///
/// This is the core widget on which most components are built on. add doc comments in sooner
/// rather than later
#[allow(missing_debug_implementations)]
pub struct Cell<'a, T, O, Message, Renderer: self::Renderer>
where
    T: ToString + Clone,
    O: ToString + Clone + 'static,
{
    menu: &'a mut menu::State,
    menu_open: &'a mut bool,
    menu_point: &'a mut Point,
    hovered_option: &'a mut bool,
    last_selection: &'a mut bool,
    menu_hovered_option: &'a mut Option<usize>,
    menu_last_selection: &'a mut Option<O>,
    on_selected: Box<dyn Fn(T) -> Message>,
    item: &'a T,
    options: &'static [O],
    width: Length,
    padding: u16,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: <Renderer as self::Renderer>::Style,
}

/// The local state of a [`Cell`].
///
/// [`Cell`]: struct.Cell.html
#[derive(Debug, Clone)]
pub struct State<O> {
    menu: menu::State,
    menu_open: bool,
    menu_point: Point,
    hovered_option: bool,
    last_selection: bool,
    menu_hovered_option: Option<usize>,
    menu_last_selection: Option<O>,
}

impl<O> Default for State<O> {
    fn default() -> Self {
        Self {
            menu: menu::State::default(),
            menu_open: bool::default(),
            menu_point: Point::default(),
            hovered_option: bool::default(),
            last_selection: bool::default(),
            menu_hovered_option: Option::default(),
            menu_last_selection: Option::default(),
        }
    }
}

impl<'a, T: 'a, O: 'a, Message, Renderer: self::Renderer>
    Cell<'a, T, O, Message, Renderer>
where
    T: ToString + Clone,
    O: ToString + Clone + 'static,
{
    /// Creates a new [`Cell`] with the given [`State`], a list of options,
    /// the current selected value(s), and the message to produce when option(s) is / are
    /// selected.
    ///
    /// [`Cell`]: struct.Cell.html
    /// [`State`]: struct.State.html
    pub fn new(
        state: &'a mut State<O>,
        item: &'a T,
        menu_options: &'static [O],
        on_selected: impl Fn(T) -> Message + 'static,
    ) -> Self {
        let State {
            menu,
            menu_open,
            menu_point,
            hovered_option,
            last_selection,
            menu_hovered_option,
            menu_last_selection,
        } = state;

        Self {
            menu,
            menu_open,
            menu_point,
            hovered_option,
            last_selection,
            item: item,
            options: menu_options,
            menu_hovered_option,
            menu_last_selection,
            on_selected: Box::new(on_selected),
            width: Length::Shrink,
            text_size: None,
            padding: Renderer::DEFAULT_PADDING,
            font: Default::default(),
            style: Default::default(),
        }
    }

    /// Sets the width of the [`Cell`].
    ///
    /// [`Cell`]: struct.Cell.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the padding of the [`Cell`].
    ///
    /// [`Cell`]: struct.Cell.html
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the text size of the [`Cell`].
    ///
    /// [`Cell`]: struct.Cell.html
    pub fn text_size(mut self, size: u16) -> Self {
        self.text_size = Some(size);
        self
    }

    /// Sets the font of the [`Cell`].
    ///
    /// [`Cell`]: struct.Cell.html
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`Cell`].
    ///
    /// [`Cell`]: struct.Cell.html
    pub fn style(
        mut self,
        style: impl Into<<Renderer as self::Renderer>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, T: 'a, O: 'a, Message, Renderer> Widget<Message, Renderer>
    for Cell<'a, T, O, Message, Renderer>
where
    T: Clone + ToString,
    O: Clone + ToString + 'static,
    Message: 'static,
    Renderer: self::Renderer + scrollable::Renderer + 'a,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        use std::f32;

        let limits = limits.width(Length::Fill).height(Length::Shrink);
        let text_size = self.text_size.unwrap_or(renderer.default_size());

        let size = {
            let intrinsic =
                Size::new(0.0, f32::from(text_size + self.padding * 2));

            limits.resolve(intrinsic)
        };

        layout::Node::new(size)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash as _;

        match self.width {
            Length::Shrink => {
                self.item.to_string().hash(state);
            }
            _ => {
                self.width.hash(state);
            }
        }
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if *self.menu_open {
                    if let Some(selection) = self.menu_last_selection {
                        info!("Selected {} from menu", selection.to_string());
                        *self.menu_open = false;
                    } else {
                        *self.menu_open = false;
                        *self.menu_last_selection = None;
                    }
                } else if bounds.contains(cursor_position) {
                    if *self.hovered_option {
                        *self.last_selection = !*self.last_selection;
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if bounds.contains(cursor_position) {
                    if *self.last_selection {
                        *self.menu_open = !*self.menu_open;
                        *self.menu_point = cursor_position;
                        info!(
                            "Opening menu at position x: {}, y: {}",
                            cursor_position.x, cursor_position.y
                        );
                        *self.menu_last_selection = None;
                        *self.menu_last_selection = None;
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let text_size =
                    self.text_size.unwrap_or(renderer.default_size());

                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    *self.hovered_option = true;
                }
            }

            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        //TODO: redo this all
        self::Renderer::draw(
            renderer,
            layout.bounds(),
            cursor_position,
            self.item,
            *self.last_selection,
            self.padding,
            self.text_size.unwrap_or(renderer.default_size()),
            self.font,
            &self.style,
        )
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        if *self.menu_open {
            let bounds = layout.bounds();

            let mut menu = Menu::new(
                &mut self.menu,
                &self.options,
                &mut self.menu_hovered_option,
                &mut self.menu_last_selection,
            )
            .width(bounds.width.round() as u16)
            .padding(self.padding)
            .font(self.font)
            .style(Renderer::menu_style(&self.style));

            if let Some(text_size) = self.text_size {
                menu = menu.text_size(text_size);
            }

            //FIXME: this is some bullshit default; if we dont set text this is broken as hell
            let text_size = self.text_size.unwrap_or(8);
            info!("Bounds height is {}", bounds.height);
            Some(menu.overlay(*self.menu_point, 0.0)) //(self.options.len() * ( 2 * self.padding + text_size ) as usize) as f32))
        } else {
            None
        }
    }
}

/// The renderer of a [`Cell`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Cell`] in your user interface.
///
/// [`Cell`]: struct.Cell.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: text::Renderer + menu::Renderer {
    /// The default padding of a [`Cell`].
    ///
    /// [`Cell`]: struct.Cell.html
    const DEFAULT_PADDING: u16;

    /// The [`Cell`] style supported by this renderer.
    ///
    /// [`Cell`]: struct.Cell.html
    type Style: Default;

    /// Returns the style of the [`Menu`] of the [`Cell`].
    ///
    /// [`Menu`]: ../../overlay/menu/struct.Menu.html
    /// [`Cell`]: struct.Cell.html
    fn menu_style(
        style: &<Self as Renderer>::Style,
    ) -> <Self as menu::Renderer>::Style;

    /// Draws a [`Cell`].
    ///
    /// [`Cell`]: struct.Cell.html
    fn draw<T: ToString>(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        item: &T,
        selected: bool,
        padding: u16,
        text_size: u16,
        font: Self::Font,
        style: &<Self as Renderer>::Style,
    ) -> Self::Output;
}

impl<'a, T: 'a, O: 'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for Cell<'a, T, O, Message, Renderer>
where
    T: Clone + ToString,
    O: ToString + Clone + 'static,

    Renderer: self::Renderer + 'a,
    Message: 'static,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
