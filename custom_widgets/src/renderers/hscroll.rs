//! Navigate an endless amount of content with a scrollbar.
use iced_graphics::{Backend, Primitive, Renderer};
use iced_native::mouse;
use crate::widget::hscroll;
use iced_native::{Background, Color, Rectangle, Vector};

pub use crate::widget::hscroll::State;
pub use iced_style::scrollable::{Scrollbar, Scroller, StyleSheet};
/// A widget that can vertically display an infinite amount of content
/// with a scrollbar.
///
/// This is an alias of an `iced_native` hscroll with a default
/// `Renderer`.
pub type Scrollable<'a, Message, Backend> =
    iced_native::Scrollable<'a, Message, Renderer<Backend>>;

impl<B> hscroll::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn iced_style::scrollable::StyleSheet>;
    ///We are transposed here; scrollbar width, it actually refers to its thickness in
    ///the y dimension
    fn scrollbar(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        offset: u32,
        scrollbar_width: u16,
        scrollbar_margin: u16,
        scroller_width: u16,
    ) -> Option<hscroll::Scrollbar> {
        if content_bounds.width > bounds.width {
            let outer_width =
                scrollbar_width.max(scroller_width) + 2 * scrollbar_margin;

            let outer_bounds = Rectangle {
                x: bounds.x ,
                y: bounds.y + bounds.height - outer_width as f32,
                width: bounds.width,
                height: outer_width as f32,

            };

            let scrollbar_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height - f32::from(outer_width / 2 + scrollbar_width / 2),
                width: bounds.width,
                height: scrollbar_width as f32,
            };

            let ratio = bounds.width / content_bounds.width;
            let scroller_width_offset = bounds.width * ratio;
            let x_offset = offset as f32 * ratio;

            let scroller_bounds = Rectangle {
                x: bounds.x + x_offset,
                y: bounds.y + bounds.height - f32::from(outer_width / 2 + scroller_width / 2),
                width: scroller_width_offset,
                height:  scroller_width as f32,
            };

            Some(hscroll::Scrollbar {
                outer_bounds,
                bounds: scrollbar_bounds,
                margin: scrollbar_margin,
                scroller: hscroll::Scroller {
                    bounds: scroller_bounds,
                },
            })
        } else {
            None
        }
    }

    fn draw(
        &mut self,
        state: &hscroll::State,
        bounds: Rectangle,
        _content_bounds: Rectangle,
        is_mouse_over: bool,
        is_mouse_over_scrollbar: bool,
        scrollbar: Option<hscroll::Scrollbar>,
        offset: u32,
        style_sheet: &Self::Style,
        (content, mouse_interaction): Self::Output,
    ) -> Self::Output {
        (
            if let Some(scrollbar) = scrollbar {
                let clip = Primitive::Clip {
                    bounds,
                    offset: Vector::new(offset,0),
                    content: Box::new(content),
                };

                let style = if state.is_scroller_grabbed() {
                    style_sheet.dragging()
                } else if is_mouse_over_scrollbar {
                    style_sheet.hovered()
                } else {
                    style_sheet.active()
                };

                let is_scrollbar_visible =
                    style.background.is_some() || style.border_width > 0.0;

                let scroller = if is_mouse_over
                    || state.is_scroller_grabbed()
                    || is_scrollbar_visible
                {
                    Primitive::Quad {
                        bounds: scrollbar.scroller.bounds,
                        background: Background::Color(style.scroller.color),
                        border_radius: style.scroller.border_radius,
                        border_width: style.scroller.border_width,
                        border_color: style.scroller.border_color,
                    }
                } else {
                    Primitive::None
                };

                let scrollbar = if is_scrollbar_visible {
                    Primitive::Quad {
                        bounds: scrollbar.bounds,
                        background: style
                            .background
                            .unwrap_or(Background::Color(Color::TRANSPARENT)),
                        border_radius: style.border_radius,
                        border_width: style.border_width,
                        border_color: style.border_color,
                    }
                } else {
                    Primitive::None
                };

                Primitive::Group {
                    primitives: vec![clip, scrollbar, scroller],
                }
            } else {
                content
            },
            if is_mouse_over_scrollbar || state.is_scroller_grabbed() {
                mouse::Interaction::Idle
            } else {
                mouse_interaction
            },
        )
    }
}
