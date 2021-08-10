//! Navigate an endless amount of content with a scrollbar.
use iced_graphics::backend::{self, Backend};
use iced_graphics::{Primitive, Renderer};

use crate::core::signal_window::{render_header, render_wave, translate_wave, DisplayedWave};
use crate::widget::signal_window;
use iced_native::mouse;
use iced_native::{Background, Color, Rectangle};

pub use crate::widget::signal_window::State;

use crate::styles::signal_window::StyleSheet;

impl<B> signal_window::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    const DEFAULT_PADDING: u16 = 1;

    type Style = Box<dyn StyleSheet>;
    ///We are transposed here; scrollbar width, it actually refers to its thickness in
    ///the y dimension
    fn wave_scrollbar(
        &self,
        bounds: Rectangle,
        state: &signal_window::State,
        scrollbar_width: u16,
        scrollbar_margin: u16,
        scroller_width: u16,
    ) -> Option<signal_window::Scrollbar> {
        let ratio = bounds.width / ((state.end_time - state.start_time) as f32 * state.ns_per_unit); //content_bounds.width;

        if ratio >= 1.0 {
            let outer_width = scrollbar_width.max(scroller_width) + 2 * scrollbar_margin;

            let outer_bounds = Rectangle {
                x: bounds.x,
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

            let scroller_width_offset = bounds.width * ratio;
            let x_offset = state.offset as f32 * ratio;

            let scroller_bounds = Rectangle {
                x: bounds.x + x_offset,
                y: bounds.y + bounds.height - f32::from(outer_width / 2 + scroller_width / 2),
                width: scroller_width_offset,
                height: scroller_width as f32,
            };

            Some(signal_window::Scrollbar {
                outer_bounds,
                bounds: scrollbar_bounds,
                margin: scrollbar_margin,
                scroller: signal_window::Scroller {
                    bounds: scroller_bounds,
                },
            })
        } else {
            None
        }
    }

    fn draw(
        &mut self,
        bounds: Rectangle,
        waves: &[DisplayedWave],
        state: &State,
        scrollbar: Option<signal_window::Scrollbar>,
        _padding: u16,
        text_size: u16,
        font: Self::Font,
    ) -> Self::Output {
        let bg = Primitive::Quad {
            bounds,
            background: Background::from(Color::BLACK),
            border_color: Color::BLACK,
            border_width: 1.0,
            border_radius: 1.0,
        };
        let mut primitives = vec![bg];

        primitives.push(render_header(state, bounds, font));

        for (idx, wave) in waves.iter().enumerate() {
            primitives.push(Primitive::Translate {
                content: Box::new(render_wave(wave, state, bounds, text_size, font)),
                translation: translate_wave(idx, bounds),
            })
        }

        if let Some(scrollbar) = scrollbar {
            let scroller = Primitive::Quad {
                bounds: scrollbar.scroller.bounds,
                background: Background::Color(Color::BLACK),
                border_radius: 1.0,         //style.scroller.border_radius,
                border_width: 1.0,          //,style.scroller.border_width,
                border_color: Color::BLACK, //style.scroller.border_color,
            };

            let scrollbar_quad = Primitive::Quad {
                bounds: scrollbar.bounds,
                background: Background::Color(Color::TRANSPARENT),
                border_radius: 1.0,
                border_width: 1.0,
                border_color: Color::TRANSPARENT, //style.border_color,
            };

            primitives.push(scrollbar_quad);
            primitives.push(scroller);
        }

        (Primitive::Group { primitives }, mouse::Interaction::Idle)
    }
}
