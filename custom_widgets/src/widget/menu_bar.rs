use iced_native::{
    layout, mouse, overlay, event,
    scrollable, text, Clipboard, Element, Event, Hasher, Layout, Length, Point,
    Rectangle, Size, Widget,
};

use iced_native::overlay::menu::{self as iced_menu};

use super::menu::{self,Menu};


pub trait MenuOption : std::fmt::Display + 'static {
    type Message;
    fn to_message(&self) -> Self::Message;

    fn all(&self) -> &'static [&dyn MenuOption<Message=Self::Message>];
}

pub trait MenuBarOption: std::fmt::Display + Clone + 'static {
    type Message;
    fn all() -> &'static [Self];

    fn get_children(&self) -> &'static [&dyn MenuOption<Message=Self::Message>];
}


/// A widget to represent a singular "MenuBar"
///
/// This is the core widget on which most components are built on. add doc comments in sooner
/// rather than later
#[allow(missing_debug_implementations)]
pub struct MenuBar<'a,O,Message: 'static, Renderer: self::Renderer>
where
    O: MenuBarOption<Message=Message>
{
    menu: &'a mut menu::State,
    menu_selected_option: &'a mut Option<usize>,
    options: &'a [O],
    width: Length,
    padding: u16,
    text_size: Option<u16>,
    text_bounds : Vec<f32>,
    font: Renderer::Font,
    style: <Renderer as self::Renderer>::Style,
}

/// The local state of a [`MenuBar`].
///
/// [`MenuBar`]: struct.MenuBar.html
#[derive(Debug, Clone,Default)]
pub struct State {
    menu: menu::State,
    menu_selected_option: Option<usize>,
}


impl<'a, O: 'a, Message, Renderer: self::Renderer>
    MenuBar<'a, O, Message, Renderer>
where
    O: MenuBarOption<Message=Message>,
{
    /// Creates a new [`MenuBar`] with the given [`State`], a list of options,
    /// the current selected value(s), and the message to produce when option(s) is / are
    /// selected.
    ///
    /// [`MenuBar`]: struct.MenuBar.html
    /// [`State`]: struct.State.html
    pub fn new(
        state: &'a mut State,
        options: &'a [O]
    ) -> Self {
        let State {
            menu,
            menu_selected_option
        } = state;

        Self {
            menu,
            menu_selected_option,
            options: options,
            width: Length::Shrink,
            padding: Renderer::DEFAULT_PADDING,
            text_size: None,
            text_bounds: Vec::default(),
            font: Default::default(),
            style: Default::default(),
        }
    }

    /// Sets the width of the [`MenuBar`].
    ///
    /// [`MenuBar`]: struct.MenuBar.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the padding of the [`MenuBar`].
    ///
    /// [`MenuBar`]: struct.MenuBar.html
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the text size of the [`MenuBar`].
    ///
    /// [`MenuBar`]: struct.MenuBar.html
    pub fn text_size(mut self, size: u16) -> Self {
        self.text_size = Some(size);
        self
    }

    /// Sets the font of the [`MenuBar`].
    ///
    /// [`MenuBar`]: struct.MenuBar.html
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`MenuBar`].
    ///
    /// [`MenuBar`]: struct.MenuBar.html
    pub fn style(
        mut self,
        style: impl Into<<Renderer as self::Renderer>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

}

impl<'a, O, Message, Renderer> Widget<Message, Renderer>
    for MenuBar<'a,O,Message,Renderer>
where
    O: MenuBarOption<Message=Message>,
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
        let font = self.font;
        let bounds = limits.max();


        let width = self.options.iter().fold(0.0, |acc,option| {
            let (width, _) = renderer.measure(option.to_string().as_str(),text_size,font,bounds);
            acc + width + f32::from(self.padding * 2)
        });

        let (_, height) = 
            renderer.measure(self.options[0].to_string().as_str(),text_size,font,bounds);


        let size = limits.resolve(Size::new(width, height));

        
        layout::Node::new(size)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash as _;

        match self.width {
            Length::Shrink => {
                self.options.iter()
                    .for_each(|item| item.to_string().hash(state));
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
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _messages: &mut Vec<Message>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let text_size = self.text_size.unwrap_or(renderer.default_size());


        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let bounds = layout.bounds();
                if bounds.contains(cursor_position) {
                    if self.text_bounds.is_empty() {
                        let font = self.font;
                        let padding = f32::from(self.padding* 2);
                        let mut starting_bounds = 0.0;
                        self.text_bounds = O::all()
                            .iter()
                            .map(|option| {
                                starting_bounds += renderer.measure(option.to_string().as_str(),text_size,font,Size::from([bounds.width,bounds.height])).0 + padding;
                                starting_bounds})
                            .collect();
                    }
                    if let Some(selection) = self.menu_selected_option {
                        *selection = self.text_bounds.binary_search_by(|probe| probe.partial_cmp(&cursor_position.x).unwrap()).unwrap_or_else(|e| e).min(O::all().len()-1);
                    }
                }
            }



            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if bounds.contains(cursor_position) {
                    if self.menu_selected_option.is_none() {
                        let selection = self.text_bounds.binary_search_by(|probe| probe.partial_cmp(&cursor_position.x).unwrap()).unwrap_or_else(|e| e);
                        if selection < O::all().len() {
                            *self.menu_selected_option = Some(selection);
                        }
                    }
                } else {
                    *self.menu_selected_option = None;
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
        _viewport : &Rectangle,
    ) -> Renderer::Output {
        //TODO: redo this all
        self::Renderer::draw(
            renderer,
            layout.bounds(),
            cursor_position,
            O::all(),
            self.menu_selected_option.clone(),
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
        if let Some(selected) = self.menu_selected_option {
            let bounds = layout.bounds();
            let selected = selected.clone();

            let mut menu : Menu<'_, Message, Renderer> = Menu::new(
                self.menu,
                O::all().get(selected.clone()).unwrap().get_children()
            )
            .width(bounds.width.round() as u16)
            .padding(self.padding)
            .font(self.font)
            .style(Renderer::menu_style(&self.style));

            if let Some(text_size) = self.text_size {
                menu = menu.text_size(text_size);
            }

            let x_dim = if selected == 0 {
                0.0
            } else {
                self.text_bounds.get(selected - 1).unwrap().clone()
            };

            let menu_pt : Point = [x_dim, bounds.height].into();


            Some(menu.overlay(menu_pt, 0.0)) //(self.options.len() * ( 2 * self.padding + text_size ) as usize) as f32))
        } else {
            None
        }


    }
}

/// The renderer of a [`MenuBar`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`MenuBar`] in your user interface.
///
/// [`MenuBar`]: struct.MenuBar.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: text::Renderer + iced_menu::Renderer {
    /// The default padding of a [`MenuBar`].
    ///
    /// [`MenuBar`]: struct.MenuBar.html
    const DEFAULT_PADDING: u16;

    /// The [`MenuBar`] style supported by this renderer.
    ///
    /// [`MenuBar`]: struct.MenuBar.html
    type Style: Default;

    /// Returns the style of the [`Menu`] of the [`MenuBar`].
    ///
    /// [`Menu`]: ../../overlay/menu/struct.Menu.html
    /// [`MenuBar`]: struct.MenuBar.html
    fn menu_style(
        style: &<Self as Renderer>::Style,
    ) -> <Self as iced_menu::Renderer>::Style;

    /// Draws a [`MenuBar`].
    ///
    /// [`MenuBar`]: struct.MenuBar.html
    fn draw<T: ToString>(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        item: &'static[T],
        selected: Option<usize>,
        padding: u16,
        text_size: u16,
        font: Self::Font,
        style: &<Self as Renderer>::Style,
    ) -> Self::Output;
}

impl<'a, O, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for MenuBar<'a, O, Message, Renderer>
where
    O: MenuBarOption<Message=Message>,
    Renderer: self::Renderer + 'a,
    Message: 'static,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
