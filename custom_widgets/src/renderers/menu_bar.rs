//! Display a single cell
use crate::widget::menu_bar;
use iced_graphics::backend::{self, Backend};
use iced_native::text::Renderer as TextRenderer;
use iced_graphics::{Primitive, Renderer};
use log::info;
use iced_native::{
    mouse, Color, HorizontalAlignment, Point, Rectangle, Size, VerticalAlignment,
};

use iced_style::menu::Style as MenuStyle;

use crate::styles::cell_list::StyleSheet;

/// A widget allowing the selection of a single value from a list of options.
pub type MenuBar<'a, O, Message, Backend> =
    menu_bar::MenuBar<'a, O, Message, Renderer<Backend>>;

impl<B> menu_bar::Renderer for Renderer<B>
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
            items: &'static[T],
            selected: Option<usize>,
            padding: u16,
            text_size: u16,
            font: Self::Font,
            style: &<Self as menu_bar::Renderer>::Style,
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
            height: f32::from(text_size),
        };

        //TODO: fix later, this is .. very bad

        let mut x_start = 0.0;
        for (idx, option) in items.iter().enumerate() {
            let selected = selected == Some(idx);
            let tab_width = self.measure(option.to_string().as_str(),text_size,font,Size::from([bounds.width,bounds.height])).0 + 2.0 * f32::from(padding);

            let tab_bounds = Rectangle {
                x: bounds.x + x_start,
                width: tab_width,
                ..bounds
            };

            if selected {
                let highlight_bounds = Rectangle {
                    width : tab_width,
                    x: bounds.x + x_start ,
                    ..tab_bounds
                };

                primitives.push(Primitive::Quad {
                    bounds: highlight_bounds,
                    background: style.selected_background,
                    border_color: Color::TRANSPARENT,
                    border_width: 1.0,
                    border_radius: 1.0,
                });
            } 
            primitives.push(Primitive::Text {
                content: option.to_string(),
                bounds: Rectangle {
                    x: tab_bounds.x + f32::from(padding),
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

            x_start += tab_width;


        }
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
