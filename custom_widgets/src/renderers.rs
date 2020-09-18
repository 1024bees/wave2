mod cell_list {
    //! Display a dropdown list of selectable values.
    use crate::cell_list;
    use iced_graphics::backend::{self, Backend};
    use iced_graphics::{Primitive, Renderer};
    use iced_native::{
        mouse, Color, Font, HorizontalAlignment, Point, Rectangle,
        VerticalAlignment,
    };
    use iced_style::menu::Style;

    pub use iced_native::pick_list::State;
    pub use iced_style::pick_list::StyleSheet;

    /// A widget allowing the selection of a single value from a list of options.
    pub type CellList<'a, T, O, Message, Backend> =
        cell_list::CellList<'a, T, O, Message, Renderer<Backend>>;

    impl<B> cell_list::Renderer for Renderer<B>
    where
        B: Backend + backend::Text,
    {
        type Style = Box<dyn StyleSheet>;

        const DEFAULT_PADDING: u16 = 5;

        fn menu_style(style: &Box<dyn StyleSheet>) -> Style {
            style.menu()
        }

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
            style: &Box<dyn StyleSheet>,
        ) -> Self::Output {
            let is_mouse_over = bounds.contains(cursor_position);

            let mut primitives = Vec::new();
            let style = if is_mouse_over {
                style.hovered()
            } else {
                style.active()
            };

            let selected = selected.unwrap_or_default();
            for (i, item) in items.iter().enumerate() {
                let is_selected = selected.contains(&i);
                let bounds = Rectangle {
                    x: bounds.x,
                    y: bounds.y
                        + ((text_size as usize + padding as usize * 2) * i)
                            as f32,
                    width: bounds.width,
                    height: f32::from(text_size + padding * 2),
                };

                //TODO: fix later, this is .. very bad

                match is_selected {
                    true => {
                        primitives.push(Primitive::Quad {
                            bounds,
                            background: Style::default().selected_background,
                            border_color: Color::TRANSPARENT,
                            border_width: 0,
                            border_radius: 0,
                        });
                    }
                    false => {
                        primitives.push(Primitive::Quad {
                            bounds,
                            background: Style::default().background,
                            border_color: Color::TRANSPARENT,
                            border_width: 0,
                            border_radius: 0,
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
                        Style::default().selected_text_color
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
}
