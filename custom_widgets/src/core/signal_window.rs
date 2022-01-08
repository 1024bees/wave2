use crate::widget::signal_window::State;
use iced::Color;
use wave2_wavedb::formatting::format_payload;
pub use wave2_wavedb::storage::display_wave::{
    DisplayedWave, SBWaveState, WaveColors, WaveDisplayOptions,
};

use iced_graphics::{triangle, Primitive};

use iced_native::{Rectangle, Vector};
use wave2_wavedb::puddle::Droplet;

use lyon::math::point as lpoint;

use lyon::path::Path;
use lyon::tessellation::*;

pub(crate) const BUFFER_PX: f32 = 1.5;
pub(crate) const WAVEHEIGHT: f32 = 16.0;
pub(crate) const VEC_SHIFT_WIDTH: f32 = 6.0;
pub(crate) const TS_FONT_SIZE: f32 = 10.0;

pub(crate) const TEXT_PADDING: f32 = TEXT_SIZE / 2.0;
/// Mininum x_delta between two "value" changes that must occur before we consider writing the
/// wave's value on the line
//const TEXT_THRESHOLD: f32 = 12.0;

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

const Z_COLOR: Color = ORANGE;
const X_COLOR: Color = ORANGE;

pub const fn to_color(opts: &WaveDisplayOptions) -> Color {
    match opts.color {
        WaveColors::Green => Color::from_rgba(0.0, 1.0, 0.0, 1.0),
        WaveColors::Red => Color::from_rgba(1.0, 0.0, 0.0, 1.0),
        WaveColors::Blue => Color::from_rgba(0.0, 0.0, 1.0, 1.0),
    }
}

impl StrokeVertex {
    fn new(color: [f32; 4]) -> Self {
        Self {
            primary: color.clone(),
            working_color: color,
            changes: vec![],
            working_idx: 0,
        }
    }
}

struct StrokeVertex {
    primary: [f32; 4],
    working_color: [f32; 4],
    changes: Vec<(SBWaveState, lyon::math::Point)>,
    working_idx: usize,
}

impl StrokeVertex {
    fn maybe_push_change(&mut self, wave_state: SBWaveState, point: lyon::math::Point) {
        let (state, _) = self
            .changes
            .first()
            .cloned()
            .unwrap_or_else(|| (SBWaveState::Beginning, point.clone()));
        if state != wave_state {
            self.changes.push((wave_state, point));
        }
    }
}

impl lyon::tessellation::StrokeVertexConstructor<triangle::Vertex2D> for StrokeVertex {
    fn new_vertex(
        &mut self,
        position: lyon::math::Point,
        _attributes: lyon::tessellation::StrokeAttributes<'_, '_>,
    ) -> triangle::Vertex2D {
        if let Some((state_change, point)) = self.changes.get(self.working_idx) {
            if position.x >= point.x {
                self.working_idx += 1;
                match state_change {
                    SBWaveState::High | SBWaveState::Low => {
                        self.working_color = self.primary.clone()
                    }
                    SBWaveState::X => self.working_color = X_COLOR.into_linear(),
                    SBWaveState::Z => self.working_color = Z_COLOR.into_linear(),
                    _ => {}
                }
            }
        }

        triangle::Vertex2D {
            position: [position.x, position.y],
            color: self.working_color,
        }
    }
}

fn xdelt_from_prev(state: &State, ts: u32, prev_ts: u32) -> f32 {
    (ts - prev_ts) as f32 / state.ns_per_pixel
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
        (self.offset + bounds.width * self.ns_per_pixel).ceil() as u32
    }

    fn cursor_in_range(&self, bounds: Rectangle) -> bool {
        self.cursor_location >= self.start_time(bounds)
            && self.cursor_location <= self.end_time(bounds)
    }
}

pub fn render_cursor(state: &State, bounds: Rectangle) -> Option<Primitive> {
    if state.cursor_in_range(bounds) {
        let xpos =
            bounds.x + xdelt_from_prev(state, state.cursor_location, state.start_time(bounds));
        let top_pt = lpoint(xpos, TS_FONT_SIZE + bounds.y);

        let bottom_pt = [xpos, bounds.y + bounds.height].into();

        let mut p = Path::builder();

        p.move_to(top_pt);
        p.line_to(bottom_pt);

        let mut geometry: VertexBuffers<triangle::Vertex2D, u32> = VertexBuffers::new();

        let top_line = p.build();
        let mut tessellator = StrokeTessellator::new();

        tessellator
            .tessellate_path(
                &top_line,
                &StrokeOptions::default(),
                &mut BuffersBuilder::new(&mut geometry, StrokeVertex::new(ORANGE.into_linear())),
            )
            .expect("Tesselator failed");

        Some(Primitive::Mesh2D {
            buffers: triangle::Mesh2D {
                vertices: geometry.vertices,
                indices: geometry.indices,
            },
            size: iced::Size::new(bounds.x + bounds.width, bounds.height + bounds.x),
        })
    } else {
        None
    }
}

pub fn render_header(state: &State, bounds: Rectangle, font: iced::Font) -> Primitive {
    //FIXME: need to think of way to generate uniform timestamp delimiters
    //       probably something probably something like 1,2,5
    let ts_width: u32 = (state.ns_per_frame) as u32;

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
            &mut BuffersBuilder::new(&mut geometry, StrokeVertex::new(ORANGE.into_linear())),
        )
        .expect("Tesselator failed");

    let next_bounds = state.start_time(bounds)
        + (state.ns_per_frame as u32 - (state.start_time(bounds) % state.ns_per_frame as u32));
    //log::info!("next bounds is {}, last bounds is {}, ppf is {}, ns_per_pixel {}", next_bounds, state.end_time(bounds), state.ppf, state.ns_per_pixel);
    for ts in (next_bounds..state.end_time(bounds)).step_by(ts_width as usize) {
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
                &mut BuffersBuilder::new(&mut geometry, StrokeVertex::new(BLUE.into_linear())),
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
    let display_options = dwave.display_conf;
    let mut prim_vec = Vec::new();

    let mut stroke_tracker = StrokeVertex::new(GREEN.into_linear());
    let (start_time, end_time) = (state.start_time(bounds), state.end_time(bounds));
    match width {
        1 => {
            let mut sb_state = SBWaveState::Beginning;

            for (time, sig_payload) in wave.data_in_range(start_time, end_time) {
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
            let mut wave_iter = wave.droplets_in_range(start_time, end_time).peekable();

            log::info!(
                "begin is : {}, end is : {}",
                state.start_time(bounds),
                state.end_time(bounds)
            );
            let beautify_text = |working_pts: [lyon::math::Point; 2], value| {
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
            };

            if wave_iter
                .peek()
                .map_or_else(|| true, |(time, _)| *time != state.offset.ceil() as u32)
            {
                let next_time = wave_iter.peek().map_or_else(
                    || (state.offset + bounds.width * state.ns_per_pixel) as u32,
                    |(time, _)| time.clone(),
                );

                if let Some((_time, sig_payload)) =
                    wave.get_prev_droplet(start_time as u32)
                {
                    let text_space = xdelt_from_prev(state, next_time, prev_xcoord);
                    let wave_state = if sig_payload.is_zx() {
                        SBWaveState::X
                    } else {
                        SBWaveState::High
                    };
                    stroke_tracker
                        .maybe_push_change(wave_state, working_pts.first().unwrap().clone());

                    let data2 =
                        format_payload(sig_payload.clone(), display_options.format, width, 40);

                    log::info!("First item is {} at {}", data2,start_time);

                    let value_text =
                        generate_canvas_text(sig_payload, display_options, width, text_space)
                            .map(|text| beautify_text(working_pts, text));

                    if let Some(text) = value_text {
                        prim_vec.push(text);
                    }
                }
            }

            while let Some((time, sig_payload)) = wave_iter.next() {
                let x_delt = xdelt_from_prev(state, time, prev_xcoord);
                let data2 = format_payload(sig_payload.clone(), display_options.format, width, 40);

                log::info!("nth item is {} at time {}", data2, time);

                if out_of_range(time, state, bounds) {
                    break;
                }

                let next_time = wave_iter.peek().map_or_else(
                    || (state.offset + bounds.width * state.ns_per_pixel) as u32,
                    |(time, _)| time.clone(),
                );

                let text_space = xdelt_from_prev(state, next_time, prev_xcoord);
                let wave_state = if sig_payload.is_zx() {
                    SBWaveState::X
                } else {
                    SBWaveState::High
                };
                stroke_tracker.maybe_push_change(wave_state, working_pts.first().unwrap().clone());
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

                let value_text =
                    generate_canvas_text(sig_payload, display_options, width, text_space)
                        .map(|text| beautify_text(working_pts, text));

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
            &mut BuffersBuilder::new(&mut geometry, stroke_tracker),
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
    let visible_chars = ((space - TEXT_PADDING) / TEXT_SIZE).floor() as usize;

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
