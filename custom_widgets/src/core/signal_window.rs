use crate::widget::signal_window::State;
use iced::Color;
use std::sync::Arc;
use wave2_wavedb::formatting::{format_payload, WaveFormat};
pub use wave2_wavedb::storage::display_wave::{
    DisplayedWave, SBWaveState, WaveColors, WaveDisplayOptions,
};
use wave2_wavedb::storage::in_memory::InMemWave;

use iced_graphics::{triangle, Primitive};

use iced_native::{Rectangle, Vector};
use wave2_wavedb::puddle::Droplet;

use lyon::math::point as lpoint;

use lyon::path::Path;
use lyon::tessellation::*;

pub(crate) const BUFFER_PX: f32 = 1.5;
pub(crate) const WAVEHEIGHT: f32 = 16.0;
pub(crate) const VEC_SHIFT_WIDTH: f32 = 4.0;
pub(crate) const TS_FONT_SIZE: f32 = 10.0;

/// Mininum x_delta between two "value" changes that must occur before we consider writing the
/// wave's value on the line
const TEXT_THRESHOLD: f32 = 12.0;

const TEXT_SIZE: f32 = 12.0;

/// If we try to put a timestamp too close to the start of the wave window
/// it clips the black bounding box of the wave window and looks bad
const TS_CLIP_RANGE: f32 = 5.0;

const GREEN: Color = Color::from_rgb(0.0, 1.0, 0.0);
const BLUE: Color = Color::from_rgba(
    0x1b as f32 / 255.0,
    0x0a as f32 / 255.0,
    0x73 as f32 / 255.0,
    0.25,
);

const ORANGE: Color = Color::from_rgba(
    0xf5 as f32 / 255.0,
    0xc1 as f32 / 255.0,
    0x87 as f32 / 255.0,
    0.4,
);

pub const fn to_color(opts: &WaveDisplayOptions) -> Color {
    match opts.color {
        WaveColors::Green => Color::from_rgba(0.0, 1.0, 0.0, 1.0),
        WaveColors::Red => Color::from_rgba(1.0, 0.0, 0.0, 1.0),
        WaveColors::Blue => Color::from_rgba(0.0, 0.0, 1.0, 1.0),
    }
}

struct StrokeVertex([f32; 4]);

impl lyon::tessellation::StrokeVertexConstructor<triangle::Vertex2D> for StrokeVertex {
    fn new_vertex(
        &mut self,
        position: lyon::math::Point,
        _attributes: lyon::tessellation::StrokeAttributes<'_, '_>,
    ) -> triangle::Vertex2D {
        triangle::Vertex2D {
            position: [position.x, position.y],
            color: self.0,
        }
    }
}

fn xdelt_from_prev(state: &State, ts: u32, prev_ts: u32) -> f32 {
    (ts - prev_ts) as f32 * state.ns_per_unit
}

pub fn translate_wave(wave_num: usize, bounds: Rectangle) -> Vector {
    Vector {
        x: bounds.x,
        y: bounds.y
            + TS_FONT_SIZE
            + ((wave_num) as f32 * (WAVEHEIGHT + BUFFER_PX * 2.0))
            + BUFFER_PX,
    }
}

impl State {
    fn start_time(&self, _bounds: Rectangle) -> u32 {
        self.offset.ceil() as u32
    }
    fn end_time(&self, bounds: Rectangle) -> u32 {
        (self.offset + bounds.width * self.ns_per_unit).ceil() as u32
    }
}

pub fn render_header(state: &State, bounds: Rectangle, font: iced::Font) -> Primitive {
    //FIXME: need to think of way to generate uniform timestamp delimiters
    //       probably something probably something like 1,2,5
    let ts_width: u32 = (200.0 * state.ns_per_unit) as u32;

    let mut prev_ts = state.start_time(bounds);
    let mut xpos: f32 = bounds.x;

    let hdr_line = lpoint(bounds.x, TS_FONT_SIZE + bounds.y);

    let right_side = [bounds.x + bounds.width, hdr_line.y].into();

    let mut p = Path::builder();

    p.move_to(hdr_line);
    p.line_to(right_side);

    let mut geometry: VertexBuffers<triangle::Vertex2D, u32> = VertexBuffers::new();

    let top_line = p.build();
    let mut tessellator = StrokeTessellator::new();

    let mut prim_vec = Vec::new();

    tessellator
        .tessellate_path(
            &top_line,
            &StrokeOptions::default(),
            &mut BuffersBuilder::new(&mut geometry, StrokeVertex(ORANGE.into_linear())),
        )
        .expect("Tesselator failed");

    for ts in (state.start_time(bounds)..state.end_time(bounds)).step_by(ts_width as usize) {
        xpos += xdelt_from_prev(state, ts, prev_ts);
        if xpos > TS_CLIP_RANGE {
            prim_vec.push(Primitive::Text {
                content: format!("{}ns", ts),
                bounds: Rectangle {
                    x: xpos,
                    y: bounds.y,
                    width: f32::INFINITY,
                    height: TEXT_SIZE,
                },
                color: Color::WHITE,
                size: TS_FONT_SIZE,
                font,
                horizontal_alignment: iced::HorizontalAlignment::Left,
                vertical_alignment: iced::VerticalAlignment::Top,
            });
        }

        let mut p2 = Path::builder();

        p2.move_to([xpos, bounds.y + TS_FONT_SIZE].into());
        p2.line_to([xpos, bounds.y + bounds.height].into());
        let line = p2.build();

        tessellator
            .tessellate_path(
                &line,
                &StrokeOptions::default(),
                &mut BuffersBuilder::new(&mut geometry, StrokeVertex(BLUE.into_linear())),
            )
            .expect("Tesselator failed");

        prev_ts = ts;
    }
    prim_vec.push(Primitive::Mesh2D {
        buffers: triangle::Mesh2D {
            vertices: geometry.vertices,
            indices: geometry.indices,
        },
        size: iced::Size::new(bounds.x + bounds.width, bounds.height + bounds.x),
    });

    Primitive::Group {
        primitives: prim_vec,
    }
}

pub fn render_wave(
    dwave: &DisplayedWave,
    state: &State,
    bounds: Rectangle,
    _text_size: u16,
    _font: iced::Font,
) -> Primitive {
    fn out_of_range(time: u32, state: &State, bounds: Rectangle) -> bool {
        time > state.end_time(bounds)
    }

    let mut p = Path::builder();
    let wave = dwave.get_wave();
    let width = wave.get_width();
    let mut working_pt = lpoint(0.0, 0.0);
    let mut prev_xcoord = state.offset as u32;
    log::info!("wave offset is {}", state.offset);
    log::info!("wave name is {}", wave.get_name());
    let display_options = dwave.display_conf.unwrap_or_default();
    let mut prim_vec = Vec::new();
    match width {
        1 => {
            let mut sb_state = SBWaveState::Beginning;

            for (time, sig_payload) in
                wave.data_in_range(state.start_time(bounds), state.end_time(bounds))
            {
                if out_of_range(time, state, bounds) {
                    break;
                }
                working_pt.x += xdelt_from_prev(state, time, prev_xcoord);
                p.line_to(working_pt);
                p.move_to(working_pt);
                //TODO: handle z/x case
                match (&mut sb_state, sig_payload[0] & 0x1) {
                    (SBWaveState::Beginning, 0) => {
                        working_pt.y += WAVEHEIGHT;
                        sb_state = SBWaveState::Low;
                    }
                    (SBWaveState::Beginning, 1) => {
                        sb_state = SBWaveState::High;
                    }
                    (SBWaveState::High, 0) => {
                        working_pt.y += WAVEHEIGHT;
                        sb_state = SBWaveState::Low;
                    }
                    (SBWaveState::Low, 1) => {
                        working_pt.y -= WAVEHEIGHT;
                        sb_state = SBWaveState::High;
                    }
                    (_, _) => {
                        unreachable!("Invalid state when processing single bit signals!");
                    }
                }
                prev_xcoord = time;
                p.line_to(working_pt);
                p.move_to(working_pt);
            }
            let fin_x_delt = xdelt_from_prev(state, state.end_time, prev_xcoord);
            working_pt.x += fin_x_delt;
            p.line_to(working_pt);
        }
        _ => {
            let working_pt_bot = lpoint(working_pt.x, working_pt.y + WAVEHEIGHT);
            let mut working_pts = [working_pt, working_pt_bot];
            let mut wave_iter = wave
                .droplets_in_range(state.start_time(bounds), state.end_time(bounds))
                .peekable();

            while let Some((time, sig_payload)) = wave_iter.next() {
                log::info!("first xdelt");
                let x_delt = xdelt_from_prev(state, time, prev_xcoord);
                
                if out_of_range(time, state, bounds) {
                    break;
                }

                let next_time = wave_iter.peek().map_or_else(
                    || (state.offset + bounds.width * state.ns_per_unit) as u32,
                    |(time, _)| time.clone(),
                );
                log::info!("second xdelt");

                let text_space = xdelt_from_prev(state, next_time, prev_xcoord);

                let mut value_text =
                    generate_canvas_text(sig_payload, display_options, width, text_space);

                for (point, direction) in working_pts.iter_mut().zip([1.0, -1.0].iter()) {
                    p.move_to(*point);
                    point.x += x_delt - VEC_SHIFT_WIDTH / 2.0;
                    p.line_to(*point);
                    point.y += WAVEHEIGHT * direction;
                    //TODO: logic for when really zoomed out, so we dont move past the next
                    //delta
                    point.x += VEC_SHIFT_WIDTH / 2.0;

                    p.line_to(*point);
                    point.y -= WAVEHEIGHT * direction;
                }
                value_text = value_text.map(|value| {
                    let value = match value {
                        Primitive::Text {
                            content,
                            size,
                            font,
                            color,
                            ..
                        } => {
                            let bounds: Rectangle = Rectangle {
                                x: working_pts[0].x,
                                y: working_pts[0].y,
                                width: f32::INFINITY,
                                height: TEXT_SIZE,
                            };

                            Primitive::Text {
                                content,
                                bounds,
                                size,
                                color,
                                font,
                                horizontal_alignment: iced::HorizontalAlignment::Left,
                                vertical_alignment: iced::VerticalAlignment::Top,
                            }
                        }
                        _ => {
                            unimplemented!()
                        }
                    };

                    value
                });

                //FIXME: seems like this closure is very overloaded
                //       think of a way to pull this out
                if let Some(text) = value_text {
                    prim_vec.push(text);
                }

                prev_xcoord = time
            }
            // This draws a line towards the end of frame
            let fin_x_delt = xdelt_from_prev(state, state.end_time, prev_xcoord);
            for point in working_pts.iter_mut() {
                p.move_to(*point);
                point.x += fin_x_delt;
                p.line_to(*point);
            }
        }
    }
    let wave_path = p.build();
    let mut tessellator = StrokeTessellator::new();

    let mut geometry: VertexBuffers<triangle::Vertex2D, u32> = VertexBuffers::new();
    tessellator
        .tessellate_path(
            &wave_path,
            &StrokeOptions::default(),
            &mut BuffersBuilder::new(&mut geometry, StrokeVertex(GREEN.into_linear())),
        )
        .expect("Tesselator failed");

    prim_vec.push(Primitive::Mesh2D {
        buffers: triangle::Mesh2D {
            vertices: geometry.vertices,
            indices: geometry.indices,
        },
        size: iced::Size::new(bounds.width, bounds.height),
    });

    Primitive::Group {
        primitives: prim_vec,
    }
}

/// Utility for converting value -> canvas based text.
/// The text that we are generating exists in the margins between two "wave deltas", so we have to
/// truncate that value occasionally
pub fn generate_canvas_text(
    data: Droplet,
    display_options: WaveDisplayOptions,
    bitwidth: usize,
    space: f32,
) -> Option<Primitive> {
    let str_format = display_options.format;
    if space < TEXT_SIZE {
        return None;
    }
    let visible_chars = (space / TEXT_SIZE).ceil() as usize;

    let value = format_payload(data, str_format, bitwidth, visible_chars);
    Some(Primitive::Text {
        content: value,
        bounds: Rectangle {
            width: f32::INFINITY,
            ..Rectangle::default()
        },
        size: TEXT_SIZE,
        color: Color::WHITE,
        font: iced::Font::Default,
        horizontal_alignment: iced::HorizontalAlignment::Left,
        vertical_alignment: iced::VerticalAlignment::Bottom,
    })
}
