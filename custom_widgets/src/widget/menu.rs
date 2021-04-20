//! Build and show dropdown menus.

use iced_native::container;
use iced_native::event::{self, Event};
use iced_native::layout;
use iced_native::mouse;
use iced_native::overlay;
use iced_native::scrollable;
use iced_native::text;
use iced_native::{
    Clipboard, Container, Element, Hasher, Layout, Length, Point, Rectangle,
    Size, Vector, Widget, Scrollable
};

use super::menu_bar::MenuOption;


use iced_native::overlay::menu::{self as iced_menu, Menu as IcedMenu};

//TODO, FIXME: We are currently hardcoding menu behaviour. This is.. not good. Think of a way to
//better encode this, especially with key bindings
const MENU_WIDTH : f32 = 200.0;


/// A list of selectable options.
#[allow(missing_debug_implementations)]
pub struct Menu<'a, Message: 'static, Renderer: iced_menu::Renderer> {
    state: &'a mut State,
    options: &'static [&'static dyn MenuOption<Message=Message>],
    width: u16,
    padding: u16,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: <Renderer as iced_menu::Renderer>::Style,
}

impl<'a, Renderer,Message: 'a> Menu<'a, Message, Renderer>
where
    Renderer: iced_menu::Renderer + 'a,
{
    /// Creates a new [`Menu`] with the given [`State`], a list of options, and
    /// the message to produced when an option is selected.
    pub fn new(
        state: &'a mut State,
        options: &'static [&'static dyn MenuOption<Message=Message>],
    ) -> Self {
        Menu {
            state,
            options,
            width: 0,
            padding: 0,
            text_size: None,
            font: Default::default(),
            style: Default::default(),
        }
    }

    /// Sets the width of the [`Menu`].
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Sets the padding of the [`Menu`].
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the text size of the [`Menu`].
    pub fn text_size(mut self, text_size: u16) -> Self {
        self.text_size = Some(text_size);
        self
    }

    /// Sets the font of the [`Menu`].
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`Menu`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer as iced_menu::Renderer>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    /// Turns the [`Menu`] into an overlay [`Element`] at the given target
    /// position.
    ///
    /// The `target_height` will be used to display the menu either on top
    /// of the target or under it, depending on the screen position and the
    /// dimensions of the [`Menu`].
    pub fn overlay(
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

/// The local state of a [`Menu`].
#[derive(Debug, Clone, Default)]
pub struct State {
    hovered_option: Option<usize>,
    scrollable: scrollable::State,
}

impl State {
    /// Creates a new [`State`] for a [`Menu`].
    pub fn new() -> Self {
        Self::default()
    }
}

struct Overlay<'a, Message, Renderer: iced_menu::Renderer> {
    container: Container<'a, Message, Renderer>,
    width: u16,
    target_height: f32,
    style: <Renderer as iced_menu::Renderer>::Style,
}

impl<'a, Message, Renderer: iced_menu::Renderer> Overlay<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a,
{
    pub fn new(menu: Menu<'a, Message, Renderer>, target_height: f32) -> Self
    {
        let Menu {
            state,
            options,
            width,
            padding,
            font,
            text_size,
            style,
        } = menu;

        let container =
            Container::new(Scrollable::new(&mut state.scrollable).push(List {
                options,
                hovered_option: &mut state.hovered_option,
                font,
                text_size,
                padding,
                style: style.clone(),
            }))
            .padding(1);

        Self {
            container,
            width: width,
            target_height,
            style: style,
        }
    }
}

impl<'a, Message, Renderer> iced_native::Overlay<Message, Renderer>
    for Overlay<'a, Message, Renderer>
where
    Renderer: iced_menu::Renderer,
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
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        self.container.on_event(
            event.clone(),
            layout,
            cursor_position,
            renderer,
            clipboard,
            messages,
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

struct List<'a, Message: 'static, Renderer: iced_menu::Renderer> {
    options: &'static [&'static dyn MenuOption<Message=Message>],
    hovered_option: &'a mut Option<usize>,
    padding: u16,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: <Renderer as iced_menu::Renderer>::Style,
}

impl<'a, Message, Renderer: iced_menu::Renderer> Widget<Message, Renderer>
    for List<'a, Message, Renderer>
where
    Renderer: iced_menu::Renderer,
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

        let limits = limits.width(Length::Shrink).height(Length::Shrink);
        let text_size = self.text_size.unwrap_or(renderer.default_size());

        let size = {
            let intrinsic = Size::new(
                MENU_WIDTH,
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
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
        
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    if let Some(index) = *self.hovered_option {
                        if let Some(option) = self.options.get(index) {
                           messages.push(option.to_message())
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
        iced_menu::Renderer::draw(
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


impl<'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for List<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced_menu::Renderer,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
