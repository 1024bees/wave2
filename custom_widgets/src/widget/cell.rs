use iced_native::{
    event, layout, mouse, overlay,
    overlay::menu::{self, Menu},
    text, Clipboard, Element, Event, Hasher, Layout, Length, Point, Rectangle, Size, Widget,
};

use crate::traits::CellOption;
use log::info;
use std::marker::PhantomData;

/// A widget to represent a singular "Cell"
///
/// This is the core widget on which most components are built on. add doc comments in sooner
/// rather than later
#[allow(missing_debug_implementations)]
pub struct Cell<'a, O, Message: 'static, Renderer: self::Renderer>
where
    O: CellOption<Message = Message>,
{
    menu: &'a mut menu::State,
    menu_open: &'a mut bool,
    menu_point: &'a mut Point,
    hovered_option: &'a mut bool,
    selected: &'a mut bool,
    menu_hovered_option: &'a mut Option<usize>,
    menu_last_selection: &'a mut Option<O>,
    last_click: &'a mut Option<mouse::Click>,
    on_click: Option<Box<dyn Fn() -> Message>>,
    on_double_click: Option<Box<dyn Fn() -> Message>>,
    overriden_selected: Option<bool>,
    item: String,
    options: PhantomData<O>,
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
    selected: bool,
    menu_hovered_option: Option<usize>,
    last_click: Option<mouse::Click>,
    menu_last_selection: Option<O>,
}

impl<O> State<O> {
    pub fn set_selected(&mut self, select_val: bool) {
        self.selected = select_val;
    }
}

impl<O> Default for State<O> {
    fn default() -> Self {
        Self {
            menu: menu::State::default(),
            menu_open: bool::default(),
            menu_point: Point::default(),
            hovered_option: bool::default(),
            selected: bool::default(),
            last_click: Option::default(),
            menu_hovered_option: Option::default(),
            menu_last_selection: Option::default(),
        }
    }
}

impl<'a, O: 'a, Message, Renderer: self::Renderer> Cell<'a, O, Message, Renderer>
where
    O: CellOption<Message = Message>,
{
    /// Creates a new [`Cell`] with the given [`State`], a list of options,
    /// the current selected value(s), and the message to produce when option(s) is / are
    /// selected.
    ///
    /// [`Cell`]: struct.Cell.html
    /// [`State`]: struct.State.html
    pub fn new(state: &'a mut State<O>, item: String) -> Self {
        let State {
            menu,
            menu_open,
            menu_point,
            hovered_option,
            selected,
            menu_hovered_option,
            menu_last_selection,
            last_click,
        } = state;

        Self {
            menu,
            menu_open,
            menu_point,
            hovered_option,
            selected,
            item,
            options: PhantomData::default(),
            menu_hovered_option,
            menu_last_selection,
            width: Length::Shrink,
            last_click,
            on_click: None,
            on_double_click: None,
            text_size: None,
            overriden_selected: None,
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

    /// Optionally sets the padding of the [`Cell`].
    ///
    /// [`Cell`]: struct.Cell.html
    pub fn padding(mut self, padding: Option<u16>) -> Self {
        self.padding = padding.unwrap_or(self.padding);
        self
    }

    /// Optionally sets the text size of the [`Cell`].
    ///
    /// [`Cell`]: struct.Cell.html
    pub fn text_size(mut self, size: Option<u16>) -> Self {
        self.text_size = size;
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
    pub fn style(mut self, style: impl Into<<Renderer as self::Renderer>::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Switch to allow the select logic to be overriden at the application level
    ///
    /// Useful when you have a collection of Cells, where select logic should be mutually exclusive
    ///
    /// [`Cell`]: struct.Cell.html
    pub fn override_selected(mut self, override_select: bool) -> Self {
        self.overriden_selected = Some(override_select);
        self
    }

    /// Closure to generate the message when the Cell is left clicked
    ///
    /// [`Cell`]: struct.Cell.html
    pub fn on_click(mut self, on_click: Box<dyn Fn() -> Message + 'static>) -> Self {
        self.on_click = Some(on_click);
        self
    }

    /// Closure to generate the message when the Cell is left clicked
    ///
    /// [`Cell`]: struct.Cell.html
    pub fn on_double_click(mut self, dbl_click: Box<dyn Fn() -> Message + 'static>) -> Self {
        self.on_double_click = Some(dbl_click);
        self
    }
}

impl<'a, O: 'a, Message, Renderer> Widget<Message, Renderer> for Cell<'a, O, Message, Renderer>
where
    O: CellOption<Message = Message>,
    Message: 'static,
    Renderer: self::Renderer + text::Renderer + menu::Renderer + 'a,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        use std::f32;

        let limits = limits.width(Length::Fill).height(Length::Shrink);
        let text_size = self.text_size.unwrap_or(renderer.default_size());

        let size = {
            let intrinsic = Size::new(0.0, f32::from(text_size + self.padding * 2));

            limits.resolve(intrinsic)
        };

        layout::Node::new(size)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash as _;

        match self.width {
            Length::Shrink => {
                self.item.hash(state);
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
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if *self.menu_open {
                    if let Some(selection) = self.menu_last_selection {
                        messages.push(selection.to_message())
                    }
                    *self.menu_open = false;
                    *self.menu_last_selection = None;
                    return event::Status::Captured;
                } else if bounds.contains(cursor_position) {
                    let click = mouse::Click::new(cursor_position, *self.last_click);
                    match click.kind() {
                        mouse::click::Kind::Single => {
                            info!(
                                "Single click event, padding is: {}, text_size is: {}",
                                self.padding,
                                self.text_size.clone().unwrap_or(12)
                            );
                            if let Some(ref click_generator) = self.on_click {
                                messages.push(click_generator());
                            }
                        }
                        mouse::click::Kind::Double | mouse::click::Kind::Triple => {
                            info!("Double+ click event");

                            if let Some(ref dbl_click_gen) = self.on_double_click {
                                messages.push(dbl_click_gen());
                            }
                        }
                    }

                    if *self.hovered_option {
                        *self.selected = !*self.selected;
                    } else {
                        *self.selected = false;
                    }
                    *self.last_click = Some(click);
                    return event::Status::Captured;
                }
            }

            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if bounds.contains(cursor_position) {
                    if *self.selected {
                        *self.menu_open = !*self.menu_open;
                        *self.menu_point = cursor_position;
                        info!(
                            "Opening menu at position x: {}, y: {}",
                            cursor_position.x, cursor_position.y
                        );
                        *self.menu_last_selection = None;
                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    *self.hovered_option = true;
                }
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
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        //TODO: redo this all
        self::Renderer::draw(
            renderer,
            layout.bounds(),
            cursor_position,
            self.item.as_ref(),
            self.overriden_selected.unwrap_or(*self.selected),
            self.padding,
            self.text_size.unwrap_or(renderer.default_size()),
            self.font,
            &self.style,
        )
    }

    fn overlay(&mut self, layout: Layout<'_>) -> Option<overlay::Element<'_, Message, Renderer>> {
        if *self.menu_open {
            let bounds = layout.bounds();

            let mut menu = Menu::new(
                &mut self.menu,
                O::all(),
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
    fn menu_style(style: &<Self as Renderer>::Style) -> <Self as menu::Renderer>::Style;

    /// Draws a [`Cell`].
    ///
    /// [`Cell`]: struct.Cell.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        item: &str,
        selected: bool,
        padding: u16,
        text_size: u16,
        font: Self::Font,
        style: &<Self as Renderer>::Style,
    ) -> Self::Output;
}

impl<'a, O, Message, Renderer> From<Cell<'a, O, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    O: CellOption<Message = Message> + 'a,
    Renderer: self::Renderer + 'a,
    Message: 'static,
{
    fn from(cell: Cell<'a, O, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(cell)
    }
}

//impl<'a, T, O, Message, Renderer> Into<Element<'a, Message, Renderer>>
//    for Cell<'a, T, O, Message, Renderer>
//where
//    T: Clone + AsRef<str> + 'a,
//    O: CellOption<Message = Message> + 'a,
//    Renderer: self::Renderer + 'a,
//    Message: 'static,
//{
//    fn into(self) -> Element<'a, Message, Renderer> {
//        Element::new(self)
//    }
//}
