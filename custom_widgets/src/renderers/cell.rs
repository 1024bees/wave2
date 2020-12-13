//! Display a single cell
use crate::widget::cell;
use iced_graphics::backend::{self, Backend};
use iced_graphics::{Primitive, Renderer};
use iced_native::{
    mouse, Color, HorizontalAlignment, Point, Rectangle, VerticalAlignment,
};

use iced_style::menu::Style as MenuStyle;

use crate::styles::cell_list::StyleSheet;

/// A widget allowing the selection of a single value from a list of options.
pub type Cell<'a, T, O, Message, Backend> =
    cell::Cell<'a, T, O, Message, Renderer<Backend>>;

impl<B> cell::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Style = Box<dyn StyleSheet>;

    const DEFAULT_PADDING: u16 = 5;

    fn menu_style(style: &Box<dyn StyleSheet>) -> MenuStyle {
        style.options()
    }

    fn draw<T: ToString>(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        item: &T,
        selected: bool,
        padding: u16,
        text_size: u16,
        font: Self::Font,
        style: &Box<dyn StyleSheet>,
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        let style = if is_mouse_over {
            style.hovered()
        } else {
            style.active()
        };

        let bg = Primitive::Quad {
            bounds,
            background: style.background,
            border_color: Color::BLACK,
            border_width: 1.0,
            border_radius: 1.0,
        };

        let mut primitives = vec![bg];

        let bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: f32::from(text_size + padding * 2),
        };

        //TODO: fix later, this is .. very bad

        match selected {
            true => {
                primitives.push(Primitive::Quad {
                    bounds,
                    background: style.selected_background,
                    border_color: Color::BLACK,
                    border_width: 1.0,
                    border_radius: 1.0,
                });
            }
            false => {
                primitives.push(Primitive::Quad {
                    bounds,
                    background: style.background,
                    border_color: Color::TRANSPARENT,
                    border_width: 1.0,
                    border_radius: 1.0,
                });
            }
        }

        primitives.push(Primitive::Text {
            content: item.to_string(),
            bounds: Rectangle {
                x: bounds.x + f32::from(padding),
                y: bounds.center_y(),
                width: f32::INFINITY,
                ..bounds
            },
            size: f32::from(text_size),
            font,
            color: if selected {
                style.selected_text_color
            } else {
                style.text_color
            },
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Center,
        });

        (
            Primitive::Group { primitives },
            if is_mouse_over {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            },
        )
    }
}
