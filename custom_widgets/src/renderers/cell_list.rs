//! Display a dropdown list of selectable values.
use crate::widget::cell_list;
use iced_graphics::backend::{self, Backend};
use iced_graphics::{Primitive, Renderer};
use iced_native::{
    mouse, Color, HorizontalAlignment, Point, Rectangle, VerticalAlignment,
};

use iced_style::menu::Style as MenuStyle;

use crate::styles::cell_list::StyleSheet;

/// A widget allowing the selection of a single value from a list of options.
//pub type CellList<'a, T, O, Message, Backend> =
//    cell_list::CellList<'a, T, O, Message, Renderer<Backend>>;

impl<B> cell_list::Renderer for Renderer<B>
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
        cursor_held: bool,
        heading: Option<String>,
        items: &[T],
        selected: Option<&[usize]>,
        padding: u16,
        text_size: u16,
        heading_size: u16,
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

        let mut header_offset: usize = 0;
        if let Some(head_str) = heading {
            header_offset = (heading_size + padding * 2) as usize;
            let header_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: f32::from(heading_size + padding * 2),
            };

            primitives.push(Primitive::Quad {
                bounds: header_bounds,
                background: style.heading_background,
                border_color: Color::BLACK,
                border_width: 1.0,
                border_radius: 1.0,
            });

            primitives.push(Primitive::Text {
                content: head_str.into(),
                bounds: Rectangle {
                    x: header_bounds.x + f32::from(padding),
                    y: header_bounds.center_y(),
                    width: f32::INFINITY,
                    ..header_bounds
                },
                size: f32::from(text_size),
                font,
                color: style.text_color,
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Center,
            });
        }

        let selected = selected.unwrap_or_default();
        for (i, item) in items.iter().enumerate() {
            let is_selected = selected.contains(&i);
            let bounds = Rectangle {
                x: bounds.x,
                y: bounds.y
                    + ((text_size as usize + padding as usize * 2) * (i)
                        + header_offset) as f32,
                width: bounds.width,
                height: f32::from(text_size + padding * 2),
            };

            //TODO: fix later, this is .. very bad

            match is_selected {
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
                color: if is_selected {
                    style.selected_text_color
                } else {
                    style.text_color
                },
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Center,
            });
        }
        (
            Primitive::Group { primitives },
            if is_mouse_over {
                if cursor_held {
                    mouse::Interaction::Grabbing
                } else {
                    mouse::Interaction::Pointer
                }
            } else {
                mouse::Interaction::default()
            },
        )
    }
}
