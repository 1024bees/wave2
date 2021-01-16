//! Build and show dropdown menus.
use iced_native::container;
use iced_native::event::{self, Event};
use iced_native::layout;
use iced::mouse;
use iced_native::overlay;
use iced_native::scrollable;
use iced_native::text;
use iced_native::{
    Clipboard, Container, Element, Hasher, Layout, Length, Point, Rectangle,
     Size, Vector, Widget,
};

use std::any::Any;

pub trait NestedMenuOptions : ToString + Clone + Any+ 'static {
    type Message: NestedMenuOptions;
    fn get_children(&self) -> Option<&dyn Any>;
    fn to_message(&self) -> Self::Message;
}

/// A list of selectable options.
#[allow(missing_debug_implementations)]
pub struct NestedMenu<'a, T: NestedMenuOptions, Renderer: self::Renderer> {
    state: &'a mut State,
    options: &'static [T],
    hovered_option: &'a mut Option<usize>,
    width: u16,
    padding: u16,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: <Renderer as self::Renderer>::Style,
}

impl<'a, T, Renderer> NestedMenu<'a, T, Renderer>
where
    T: NestedMenuOptions,
    Renderer: self::Renderer + 'a,
{
    /// Creates a new [`NestedMenu`] with the given [`State`], a list of options, and
    /// the message to produced when an option is selected.
    pub fn new(
        state: &'a mut State,
        options: &'static [T],
        hovered_option: &'a mut Option<usize>,
    ) -> Self {
        NestedMenu {
            state,
            options,
            hovered_option,
            width: 0,
            padding: 0,
            text_size: None,
            font: Default::default(),
            style: Default::default(),
        }
    }

    /// Sets the width of the [`NestedMenu`].
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Sets the padding of the [`NestedMenu`].
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the text size of the [`NestedMenu`].
    pub fn text_size(mut self, text_size: u16) -> Self {
        self.text_size = Some(text_size);
        self
    }

    /// Sets the font of the [`NestedMenu`].
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`NestedMenu`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer as self::Renderer>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    /// Turns the [`NestedMenu`] into an overlay [`Element`] at the given target
    /// position.
    ///
    /// The `target_height` will be used to display the menu either on top
    /// of the target or under it, depending on the screen position and the
    /// dimensions of the [`NestedMenu`].
    pub fn overlay<Message: 'a>(
        self,
        position: Point,
        target_height: f32,
    ) -> overlay::Element<'a, Message, Renderer> {
        overlay::Element::new(
            position,
            Box::new(Overlay::new(self, target_height)),
        )
    }
}

/// The local state of a [`NestedMenu`].
#[derive(Debug, Clone, Default)]
pub struct State {
    hovered_option: Option<usize>,
    children: Vec<State>,
}

impl State {
    /// Creates a new [`State`] for a [`NestedMenu`].
    pub fn new() -> Self {
        Self::default()
    }
}

struct Overlay<'a, Message, Renderer: self::Renderer> {
    containers: Vec<Container<'a, Message, Renderer>>,
    width: u16,
    target_height: f32,
    style: <Renderer as self::Renderer>::Style,
}

impl<'a, Message, Renderer: self::Renderer> Overlay<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a,
{
    pub fn new<T>(menu: NestedMenu<'a, T, Renderer>, target_height: f32) -> Self
    where
        T: NestedMenuOptions
    {
        let NestedMenu {
            state,
            options,
            hovered_option,
            width,
            padding,
            font,
            text_size,
            style,
        } = menu;





        let mut containers=
            vec![Container::new(NestedList {
                options,
                hovered_option,
                font,
                text_size,
                padding,
                style: style.clone(),
            })
            .padding(1)];

        let mut walking_node = state;
        while walking_node.hovered_option.is_some() {
            walking_node = walking_node.children.get(walking_node.hovered_option.clone().unwrap()).as_mut().unwrap();
            let hovered_option = &mut walking_node.hovered_option;
            containers.push(Container::new( NestedList {
                options,
                hovered_option,
                font,
                text_size,
                padding,
                style: style.clone(),
            }));
        }

        Self {
            containers,
            width: width,
            target_height,
            style: style,
        }
    }
}

impl<'a, Message, Renderer> iced_native::Overlay<Message, Renderer>
    for Overlay<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        let space_below = bounds.height - (position.y + self.target_height);
        let space_above = position.y;

        let limits = layout::Limits::new(
            Size::ZERO,
            Size::new(
                bounds.width - position.x,
                if space_below > space_above {
                    space_below
                } else {
                    space_above
                },
            ),
        )
        .width(Length::Units(self.width));

        let mut node = self.container.layout(renderer, &limits);

        node.move_to(if space_below > space_above {
            position + Vector::new(0.0, self.target_height)
        } else {
            position - Vector::new(0.0, node.size().height)
        });

        node
    }

    fn hash_layout(&self, state: &mut Hasher, position: Point) {
        use std::hash::Hash;

        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        (position.x as u32).hash(state);
        (position.y as u32).hash(state);
        self.container.hash_layout(state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) -> event::Status {
        self.container.on_event(
            event.clone(),
            layout,
            cursor_position,
            messages,
            renderer,
            clipboard,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        let primitives = self.container.draw(
            renderer,
            defaults,
            layout,
            cursor_position,
            &layout.bounds(),
        );

        renderer.decorate(
            layout.bounds(),
            cursor_position,
            &self.style,
            primitives,
        )
    }
}

struct NestedList<'a, T: 'static, Renderer: self::Renderer> {
    options: &'static [T],
    hovered_option: &'a mut Option<usize>,
    padding: u16,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: <Renderer as self::Renderer>::Style,
}

impl<'a, T, Message, Renderer: self::Renderer> Widget<Message, Renderer>
    for NestedList<'a, T, Renderer>
where
    T: NestedMenuOptions,
    Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        Length::Fill
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
                    * self.options.len() as f32,
            );

            limits.resolve(intrinsic)
        };

        layout::Node::new(size)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash as _;

        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.options.len().hash(state);
        self.text_size.hash(state);
        self.padding.hash(state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _messages: &mut Vec<Message>,
        renderer: &Renderer, _clipboard: Option<&dyn Clipboard>,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    if let Some(index) = *self.hovered_option {
                        if let Some(option) = self.options.get(index) {
                            *self.last_selection = Some(option.clone());
                        }
                    }
                }
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

        event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Renderer::Output {
        self::Renderer::draw(
            renderer,
            layout.bounds(),
            cursor_position,
            viewport,
            self.options,
            *self.hovered_option,
            self.padding,
            self.text_size.unwrap_or(renderer.default_size()),
            self.font,
            &self.style,
        )
    }
}

/// The renderer of a [`NestedMenu`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`NestedMenu`] in your user interface.
///
/// [renderer]: crate::renderer
pub trait Renderer:
     scrollable::Renderer + container::Renderer + text::Renderer
{
    /// The [`NestedMenu`] style supported by this renderer.
    type Style: Default + Clone;

    /// Decorates a the list of options of a [`NestedMenu`].
    ///
    /// This method can be used to draw a background for the [`NestedMenu`].
    fn decorate(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        style: &<Self as Renderer>::Style,
        primitive: Self::Output,
    ) -> Self::Output;

    /// Draws the list of options of a [`NestedMenu`].
    fn draw<T: ToString>(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        viewport: &Rectangle,
        options: &[T],
        hovered_option: Option<usize>,
        padding: u16,
        text_size: u16,
        font: Self::Font,
        style: &<Self as Renderer>::Style,
    ) -> Self::Output;
}

impl<'a, T, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for NestedList<'a, T, Renderer>
where
    T: NestedMenuOptions,
    Message: 'a,
    Renderer: 'a + self::Renderer,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
