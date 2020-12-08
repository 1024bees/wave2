use iced::{
    canvas::{self, Canvas, Cursor, Event, Frame, Geometry, Path, Stroke},
    mouse, Button, Color, Element, HorizontalAlignment, Length, Point,
    Rectangle, Text,
};

use super::display_wave::{DisplayedWave, WaveDisplayOptions};
use iced::{button, scrollable, text_input, Align, Column, TextInput};
use log::info;


use wave2_wavedb::{SigType};

pub const BUFFER_PX: f32 = 4.0;
pub const WAVEHEIGHT: f32 = 19.0;
pub const VEC_SHIFT_WIDTH: f32 = 4.0;
pub const MAX_NUM_TEXT_HEADERS: u32 = 30;
pub const TEXT_OFFSET: f32 = WAVEHEIGHT / 2.0;
pub const TS_FONT_SIZE: f32 = 8.0;

/// If we try to put a timestamp too close to the start of the wave window
/// it clips the black bounding box of the wave window and looks bad
const TS_CLIP_RANGE: f32 = 5.0;

const BLUE: Color = Color::from_rgba(
    0x1b as f32 / 255.0,
    0x0a as f32 / 255.0,
    0x73 as f32 / 255.0,
    0.25,
);
const GREEN: Color = Color::from_rgb(0.0, 1.0, 0.0);
const ORANGE: Color = Color::from_rgba(
    0xf5 as f32 / 255.0,
    0xc1 as f32 / 255.0,
    0x87 as f32 / 255.0,
    0.4,
);

#[derive(Debug, Clone)]
pub enum Message {
    UpdateCursor(CursorState),
}

pub struct WaveWindow<'a> {
    signals: &'a [DisplayedWave],
    state: &'a WaveWindowState,
    cur_state: CursorState,
}

pub struct WaveWindowState {
    cache: canvas::Cache,
    cursor_cache: canvas::Cache,
    cursor_state: CursorState,
    end_sim_time: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CursorState {
    pub cursor_location: u32,
    view_range: (u32, u32),
}

impl Default for CursorState {
    fn default() -> Self {
        CursorState {
            cursor_location: 10,
            view_range: (0, 800),
        }
    }
}

impl WaveWindowState {
    pub fn view<'a>(
        &'a mut self,
        signals: &'a [DisplayedWave],
    ) -> Element<'a, Message> {
        Canvas::new(WaveWindow {
            signals: signals,
            state: self,
            // FIXME: This is Disgusting and needs to be refactored 
            cur_state: self.cursor_state.clone(),
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn update(&mut self, message : Message) {
        match message {
            Message::UpdateCursor(cursor_state) => {
                self.cursor_state = cursor_state;
                info!("received click location is {}",self.cursor_state.cursor_location);
                self.redraw_cursor();
                info!("Drawing cursor");

            }
        }
    }


    pub fn request_redraw(&mut self) {
        self.cache.clear()
    }

    pub fn redraw_cursor(&mut self) {
        self.cursor_cache.clear()
    }
}

impl<'a> WaveWindow<'a> {
    fn get_timestamp(&self, bounds: Rectangle, xcoord: f32) -> u32 {
        let ts_width =
            (self.cur_state.view_range.1 - self.cur_state.view_range.0) as f32;
        info!("xcoord location is {}",xcoord);
        self.cur_state.view_range.0
            + (ts_width * ((xcoord) / bounds.width)) as u32
    }

    fn end_window_time(&self) -> u32 {
        if self.cur_state.view_range.1 < self.state.end_sim_time {
            return self.cur_state.view_range.1;
        }
        self.state.end_sim_time
    }

    fn x_abs(&self, ts: u32, bounds: &Rectangle) -> f32 {
        let ts_width =
            (self.cur_state.view_range.1 - self.cur_state.view_range.0) as f32;
        ((ts) as f32 / ts_width) * bounds.width
    }

    fn x_abs_cursor(&self, bounds: &Rectangle) -> f32 {

        self.x_abs(self.cur_state.cursor_location, bounds)
    }

    /// Util for finding the x offset in the wave window where a wave should change values
    /// Used in the context of streaming through a container of "changed value" instances
    fn xdelt_from_prev(
        &self,
        ts: u32,
        prev_ts: u32,
        bounds: &Rectangle,
    ) -> f32 {
        let ts_width =
            (self.cur_state.view_range.1 - self.cur_state.view_range.0) as f32;
        ((ts - prev_ts) as f32 / ts_width) * bounds.width
    }

    fn draw_header(&self, frame: &mut Frame, bounds: &Rectangle) {
        let mut window_width =
            self.cur_state.view_range.1 - self.cur_state.view_range.0;
        let mut ts_width: u32 = 1;
        while window_width >= MAX_NUM_TEXT_HEADERS {
            window_width /= 10;
            ts_width *= 10;
        }
        let starting_ts: u32 = self.cur_state.view_range.0
            + self.cur_state.view_range.0 % ts_width;
        let mut prev_ts = self.cur_state.view_range.0;
        let mut xpos: f32 = 0.0;
        let hdr_line = Point {
            x: bounds.x,
            y: TS_FONT_SIZE,//+ bounds.y ,
        };
        let boundary_line = Path::new(|p| {
            p.move_to(hdr_line);
            p.line_to([hdr_line.x + bounds.width, hdr_line.y].into());
        });
        //TODO: make this const or global in some capacity?
        let bg_stroke = Stroke::default().with_width(1.0).with_color(BLUE);
        frame.stroke(&boundary_line, bg_stroke);

        for ts in (starting_ts..self.cur_state.view_range.1)
            .step_by(ts_width as usize)
        {
            xpos += self.xdelt_from_prev(ts, prev_ts, bounds);
            if xpos > TS_CLIP_RANGE {
                frame.fill_text(canvas::Text {
                    content: format!("{}ns", ts),
                    position: Point {
                        x: xpos,
                        y: 0.0,//bounds.y,
                    },
                    color: GREEN,
                    size: TS_FONT_SIZE,
                    horizontal_alignment: HorizontalAlignment::Right,
                    ..canvas::Text::default()
                });
            }
            let vert_path = Path::new(|p| {
                p.move_to([xpos, TS_FONT_SIZE].into());
                p.line_to([xpos, bounds.y + bounds.height].into());
            });
            frame.stroke(&vert_path, bg_stroke);
            prev_ts = ts;
        }
    }

    fn draw_cursor(&self, frame: &mut Frame, bounds: Rectangle) {
        let cur_pos: Point =
            [self.x_abs_cursor(&bounds), TS_FONT_SIZE].into();
        let cursor_line = Path::new(|p| {
            p.move_to(cur_pos);
            p.line_to(Point {
                y: bounds.height,
                ..cur_pos
            });
        });
        info!("actually drawing cursor a x : {}", cur_pos.x);
        frame.stroke(
            &cursor_line,
            Stroke::default().with_width(2.0).with_color(ORANGE),
        );
    }

    //TODO: only redraw "dirty" signals
    fn draw_all(&self, frame: &mut Frame, bounds: Rectangle) {
        let mut leftmost_pt = Point::default();
        leftmost_pt.y += 1.5 * WAVEHEIGHT + BUFFER_PX;
        let background = Path::rectangle(Point::default(), bounds.size());
        frame.fill(&background, Color::BLACK);
        self.draw_header(frame, &bounds);
        let wave_list: Vec<Path> = self
            .signals
            .iter()
            .map(|display| {
                Path::new(|p| {
                    let wave = display.get_wave();
                    let mut working_pt = leftmost_pt.clone();
                    p.move_to(leftmost_pt);
                    let mut prev_xcoord = self.cur_state.view_range.0;
                    match wave.sig_type {
                        SigType::Bit => {
                            for (time, sig_payload) in wave.changes() {
                                working_pt.x += self.xdelt_from_prev(
                                    *time,
                                    prev_xcoord,
                                    &bounds,
                                );
                                p.line_to(working_pt);
                                p.move_to(working_pt);
                                //TODO: handle z/x case
                                match sig_payload.get_bv() {
                                    Some(false) => working_pt.y += WAVEHEIGHT,
                                    _ => working_pt.y -= WAVEHEIGHT,
                                }
                                prev_xcoord = *time;
                                p.line_to(working_pt);
                                p.move_to(working_pt);
                            }
                            let fin_x_delt = self.xdelt_from_prev(
                                self.end_window_time(),
                                prev_xcoord,
                                &bounds,
                            );
                            working_pt.x += fin_x_delt;
                            p.line_to(working_pt);
                        }
                        SigType::Vector(width) => {
                            let working_pt_top = Point {
                                y: working_pt.y - WAVEHEIGHT,
                                ..working_pt
                            };
                            let mut working_pts = [working_pt_top, working_pt];
                            for (time, sig_payload) in wave.changes() {
                                let x_delt = self.xdelt_from_prev(
                                    *time,
                                    prev_xcoord,
                                    &bounds,
                                ) - VEC_SHIFT_WIDTH / 2.0;

                                for (point, direction) in working_pts
                                    .iter_mut()
                                    .zip([1.0, -1.0].iter())
                                {
                                    p.move_to(*point);
                                    point.x += x_delt;
                                    p.line_to(*point);
                                    point.y += WAVEHEIGHT * direction;
                                    //TODO: logic for when really zoomed out, so we dont move past the next
                                    //delta
                                    point.x += VEC_SHIFT_WIDTH / 2.0;

                                    p.line_to(*point);
                                    point.y -= WAVEHEIGHT * direction;
                                }
                                prev_xcoord = *time
                            }
                            let fin_x_delt = self.xdelt_from_prev(
                                self.end_window_time(),
                                prev_xcoord,
                                &bounds,
                            );
                            for point in working_pts.iter_mut() {
                                p.move_to(*point);
                                point.x += fin_x_delt;
                                p.line_to(*point);
                            }
                        }
                    }
                    leftmost_pt.y += WAVEHEIGHT + BUFFER_PX;
                })
            })
            .collect();

        //TODO: cache wavelist in the case of append only?

        for waves in wave_list {
            frame.stroke(
                &waves,
                Stroke::default().with_width(1.0).with_color(GREEN),
            );
        }
    }
}

impl Default for WaveWindowState {
    fn default() -> Self {
        WaveWindowState {
            cache: canvas::Cache::default(),
            cursor_cache: canvas::Cache::default(),
            cursor_state: CursorState::default(),
            end_sim_time: 600,
        }
    }
}

impl<'a> canvas::Program<Message> for WaveWindow<'a> {
    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<Message> {
        let cursor_position = cursor.position_in(&bounds)?;

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    self.cur_state.cursor_location =
                        self.get_timestamp(bounds, cursor_position.x);
                    info!("click location is {}",self.cur_state.cursor_location);
                    Some(Message::UpdateCursor(self.cur_state.clone()))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let content =
            self.state.cache.draw(bounds.size(), |frame: &mut Frame| {
                self.draw_all(frame, bounds);
            });

        let cursors =
            self.state
                .cursor_cache
                .draw(bounds.size(), |frame: &mut Frame| {
                    self.draw_cursor(frame, bounds);
                });

        vec![content, cursors]
    }
}
