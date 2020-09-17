//! Display a list of selectable values, optionally mutable .
use iced_native::{
    keyboard, layout, mouse, overlay,
    overlay::menu::{self, Menu},
    scrollable, text, Clipboard, Element, Event, Hasher, Layout, Length, Point,
    Rectangle, Size, Widget,
};

/// A widget for selecting a single value from a list of options.
#[allow(missing_debug_implementations)]
pub struct CellList<'a, T, Message, Renderer: self::Renderer>
where
    [T]: ToOwned<Owned = Vec<T>>,
    T: ToString,
{
    menu: &'a mut menu::State,
    bulk_select: &'a mut bool,
    ctrl_select: &'a mut bool,
    cursor_held: &'a mut bool,
    setting_options : &'a mut bool,
    can_select_multiple: &'a mut bool,
    hovered_option: &'a mut Option<usize>,
    last_selection: &'a mut Vec<usize>,
    //on_right_click: Box<dyn Fn(&'a [T]) -> Message>,
    on_selected: Box<dyn Fn(T) -> Message>,
    items: &'a [T],
    width: Length,
    padding: u16,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: <Renderer as self::Renderer>::Style,
}

/// The local state of a [`CellList`].
///
/// [`CellList`]: struct.CellList.html
#[derive(Debug, Clone)]
pub struct State {
    menu: menu::State,
    //TODO: put control flags into struct 
    can_select_multiple: bool,
    bulk_select: bool,
    ctrl_select: bool,
    cursor_held: bool,
    setting_options: bool,
    hovered_option: Option<usize>,
    last_selection: Vec<usize>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            menu: menu::State::default(),
            can_select_multiple: bool::default(),
            bulk_select: bool::default(),
            ctrl_select: bool::default(),
            cursor_held: bool::default(),
            setting_options : bool::default(),
            hovered_option: Option::default(),
            last_selection: Vec::new(),
        }
    }
}

impl<'a, T: 'a, Message, Renderer: self::Renderer>
    CellList<'a, T, Message, Renderer>
where
    T: ToString,
    [T]: ToOwned<Owned = Vec<T>>,
{
    /// Creates a new [`CellList`] with the given [`State`], a list of options,
    /// the current selected value(s), and the message to produce when option(s) is / are
    /// selected.
    ///
    /// [`CellList`]: struct.CellList.html
    /// [`State`]: struct.State.html
    pub fn new(
        state: &'a mut State,
        items: &'a [T],
        on_selected: impl Fn(T) -> Message + 'static,
    ) -> Self {
        let State {
            menu,
            can_select_multiple,
            bulk_select,
            ctrl_select,
            cursor_held,
            setting_options,
            hovered_option,
            last_selection,
        } = state;

        Self {
            menu,
            bulk_select,
            ctrl_select,
            cursor_held,
            setting_options,
            can_select_multiple,
            hovered_option,
            last_selection,
            items: items,
            on_selected: Box::new(on_selected),
            width: Length::Shrink,
            text_size: None,
            padding: Renderer::DEFAULT_PADDING,
            font: Default::default(),
            style: Default::default(),
        }
    }

    /// Sets the width of the [`CellList`].
    ///
    /// [`CellList`]: struct.CellList.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the padding of the [`CellList`].
    ///
    /// [`CellList`]: struct.CellList.html
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the text size of the [`CellList`].
    ///
    /// [`CellList`]: struct.CellList.html
    pub fn text_size(mut self, size: u16) -> Self {
        self.text_size = Some(size);
        self
    }

    /// Sets the font of the [`CellList`].
    ///
    /// [`CellList`]: struct.CellList.html
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`CellList`].
    ///
    /// [`CellList`]: struct.CellList.html
    pub fn style(
        mut self,
        style: impl Into<<Renderer as self::Renderer>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, T: 'a, Message, Renderer> Widget<Message, Renderer>
    for CellList<'a, T, Message, Renderer>
where
    T: Clone + ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
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
            let intrinsic = Size::new(
                0.0,
                f32::from(text_size + self.padding * 2)
                    * self.items.len() as f32,
            );

            limits.resolve(intrinsic)
        };

        layout::Node::new(size)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash as _;

        match self.width {
            Length::Shrink => {
                self.items
                    .iter()
                    .map(ToString::to_string)
                    .for_each(|label| label.hash(state));
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
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let bounds = layout.bounds();
                if bounds.contains(cursor_position) {
                    *self.cursor_held = true;
                    if let Some(index) = *self.hovered_option {
                        if let Some(option) = self.items.get(index) {
                            match (*self.ctrl_select, *self.bulk_select) {
                                (true, _) => {
                                    if self.last_selection.contains(&index) {
                                        self.last_selection
                                            .retain(|x| *x != index);
                                    } else {
                                        self.last_selection.push(index);
                                    }
                                }
                                (false, true) => {
                                    let starting_val = *self
                                        .last_selection
                                        .first()
                                        .unwrap_or(&0);
                                    self.last_selection.clear();
                                    if starting_val > index {
                                        self.last_selection
                                            .extend(index..starting_val);
                                    } else {
                                        self.last_selection
                                            .extend(starting_val..index);
                                    }
                                }
                                (false, false) => {
                                    self.last_selection.clear();
                                    self.last_selection.push(index);
                                }
                            }
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                *self.cursor_held = false;
            },
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if *self.cursor_held == false && !self.last_selection.is_empty() {
                    *self.setting_options = true; 
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(mod_state)) => {
                *self.ctrl_select = mod_state.control;
                *self.bulk_select = mod_state.shift;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let bounds = layout.bounds();
                let text_size =
                    self.text_size.unwrap_or(renderer.default_size());

                if bounds.contains(cursor_position) {
                    *self.hovered_option = Some(
                        ((cursor_position.y - bounds.y)
                            / f32::from(text_size + self.padding * 2))
                            as usize,
                    );
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
            *self.cursor_held,
            self.items,
            Some(&self.last_selection[..]),
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

       None
        
    }
}

/// The renderer of a [`CellList`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`CellList`] in your user interface.
///
/// [`CellList`]: struct.CellList.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: text::Renderer + menu::Renderer {
    /// The default padding of a [`CellList`].
    ///
    /// [`CellList`]: struct.CellList.html
    const DEFAULT_PADDING: u16;

    /// The [`CellList`] style supported by this renderer.
    ///
    /// [`CellList`]: struct.CellList.html
    type Style: Default;

    /// Returns the style of the [`Menu`] of the [`CellList`].
    ///
    /// [`Menu`]: ../../overlay/menu/struct.Menu.html
    /// [`CellList`]: struct.CellList.html
    fn menu_style(
        style: &<Self as Renderer>::Style,
    ) -> <Self as menu::Renderer>::Style;

    /// Draws a [`CellList`].
    ///
    /// [`CellList`]: struct.CellList.html
    fn draw<T: ToString>(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        cursor_held: bool,
        items: &[T],
        selected: Option<&[usize]>,
        padding: u16,
        text_size: u16,
        font: Self::Font,
        style: &<Self as Renderer>::Style,
    ) -> Self::Output;
}

impl<'a, T: 'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for CellList<'a, T, Message, Renderer>
where
    T: Clone + ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: self::Renderer + 'a,
    Message: 'static,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
