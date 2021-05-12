use iced::{
    canvas::{self, event, Canvas, Cursor, Event, Frame, Geometry, Path, Stroke},
    mouse, Color, Element, HorizontalAlignment, Length, Point, Rectangle,
};

use super::display_wave::{generate_canvas_text, DisplayedWave, SBWaveState};
use log::info;
use wave2_custom_widgets::widget::hscroll;
use wave2_custom_widgets::widget::hscroll::HScroll;
use super::Message;

pub const BUFFER_PX: f32 = 1.5;
pub const WAVEHEIGHT: f32 = 16.0;
pub const VEC_SHIFT_WIDTH: f32 = 4.0;
pub const TS_FONT_SIZE: f32 = 12.0;

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



pub struct WaveWindow<'a> {
    signals: &'a [DisplayedWave],
    frame_state: &'a mut FrameState,
    wave_cache: &'a canvas::Cache,
    cursor_cache: &'a canvas::Cache,
}
#[derive(Default)]
pub struct WaveWindowState {
    live_waves: Vec<DisplayedWave>,
    cache: canvas::Cache,
    cursor_cache: canvas::Cache,
    frame_state: FrameState,
    scroll_state: hscroll::State,
}

#[derive(Debug, Clone, Copy)]
/// State for handling zoom state
pub struct FrameState {
    start_time: u32,
    end_time: u32,
    ns_per_unit: f32,
    pub cursor_location: u32,
    offset: f32,
}

impl Default for FrameState {
    fn default() -> FrameState {
        FrameState {
            start_time: 0,
            end_time: 1000,
            ns_per_unit: 1.0,
            cursor_location: 0,
            offset: 0.0,
        }
    }
}

impl WaveWindowState {
    pub fn view(&mut self) -> Element<Message> {
        let val = HScroll::new(&mut self.scroll_state).scrollbar_width(10);
        val.push(
            Canvas::new(WaveWindow {
                signals: &self.live_waves[..],
                frame_state: &mut self.frame_state,
                wave_cache: &self.cache,
                cursor_cache: &self.cursor_cache,
            })
            .width(Length::Units(u16::MAX))
            .height(Length::Fill),
        )
        .width(Length::Shrink)
        .height(Length::Fill)
        .padding(10)
        .into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::UpdateCursor(cursor_location) => {
                self.frame_state.cursor_location = cursor_location;
                self.redraw_cursor();
            }
            Message::UpdateOffset(offset) => {
                self.frame_state.offset = offset;
            }
            Message::UpdateBounds((start, end)) => {
                self.frame_state.start_time = start;
                self.frame_state.end_time = end;
            },
            Message::AddWave(imw) => {
                match imw {
                    Ok(wave) => { 
                        self.live_waves.push(DisplayedWave::from(wave));
                        self.request_redraw();
                    }
                    Err(err) => log::info!("Failed to add wave with err {:?}",err),
                }
            },
            _ => { log::info!("Not covered"); },
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
    fn start_time(&self) -> u32 {
        self.frame_state.start_time
    }

    fn end_time(&self) -> u32 {
        self.frame_state.end_time
    }

    fn offset(&self) -> f32 {
        self.frame_state.offset
    }

    fn get_timestamp(&self, xcoord: f32) -> u32 {
        let offset = self.offset();

        ((offset + xcoord) * self.frame_state.ns_per_unit).round() as u32
    }

    fn end_window_time(&self) -> u32 {
        return self.frame_state.end_time;
    }

    fn x_abs(&self, ts: u32) -> f32 {
        ts as f32 / (self.frame_state.ns_per_unit)
    }

    fn x_abs_cursor(&self) -> f32 {
        self.x_abs(self.frame_state.cursor_location)
    }

    /// Util for finding the x offset in the wave window where a wave should change values
    /// Used in the context of streaming through a container of "changed value" instances
    fn xdelt_from_prev(&self, ts: u32, prev_ts: u32, _bounds: &Rectangle) -> f32 {
        (ts - prev_ts) as f32 * self.frame_state.ns_per_unit
    }

    fn draw_header(&self, frame: &mut Frame, bounds: Rectangle) {
        //FIXME: need to think of way to generate uniform timestamp delimiters
        //       probably something probably something like 1,2,5
        let ts_width: u32 = (200.0 * self.frame_state.ns_per_unit) as u32;

        let mut prev_ts = self.start_time();
        let mut xpos: f32 = 0.0;

        let hdr_line = Point {
            x: 0.0,
            y: TS_FONT_SIZE, //+ bounds.y ,
        };

        let right_side = [hdr_line.x + bounds.width, hdr_line.y].into();

        let boundary_line = Path::new(|p| {
            p.move_to(hdr_line);
            p.line_to(right_side);
        });
        //TODO: make this const or global in some capacity?
        let bg_stroke = Stroke::default().with_width(1.0).with_color(BLUE);
        frame.stroke(&boundary_line, bg_stroke);

        for ts in (self.start_time()..self.end_time()).step_by(ts_width as usize) {
            xpos += self.xdelt_from_prev(ts, prev_ts, &bounds);
            if xpos > TS_CLIP_RANGE {
                frame.fill_text(canvas::Text {
                    content: format!("{}ns", ts),
                    position: Point {
                        x: xpos,
                        y: 0.0, //bounds.y,
                    },
                    color: Color::WHITE,
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

    fn out_of_range(&self, time: u32) -> bool {
        time > self.end_window_time()
    }

    fn draw_cursor(&self, frame: &mut Frame, bounds: Rectangle) {
        let cur_pos: Point = [self.x_abs_cursor(), TS_FONT_SIZE].into();
        let cursor_line = Path::new(|p| {
            p.move_to(cur_pos);
            p.line_to(Point {
                y: bounds.height,
                ..cur_pos
            });
        });
        frame.stroke(
            &cursor_line,
            Stroke::default().with_width(2.0).with_color(ORANGE),
        );
    }

    //TODO: only redraw "dirty" signals
    fn draw_all(&self, frame: &mut Frame, bounds: Rectangle) {
        let mut leftmost_pt = Point::default();
        leftmost_pt.y += WAVEHEIGHT + 2.0 * BUFFER_PX + TS_FONT_SIZE;
        let background = Path::rectangle(Point::default(), bounds.size());
        frame.fill(&background, Color::BLACK);
        let wave_list: Vec<Path> = self
            .signals
            .iter()
            .map(|display| {
                Path::new(|p| {
                    let wave = display.get_wave();
                    let mut working_pt = leftmost_pt.clone();
                    let display_options = display.display_conf.unwrap_or_default();
                    p.move_to(leftmost_pt);
                    let mut prev_xcoord = self.start_time();
                    let width = wave.get_width();
                    match width {
                        1 => {
                            let mut sb_state = SBWaveState::Beginning;

                            for (time, sig_payload) in
                                wave.data_in_range(self.start_time(), self.end_time())
                            {
                                if self.out_of_range(time.clone()) {
                                    break;
                                }

                                working_pt.x += self.xdelt_from_prev(time, prev_xcoord, &bounds);
                                p.line_to(working_pt);
                                p.move_to(working_pt);
                                //TODO: handle z/x case
                                match (&mut sb_state, sig_payload[0] & 0x1) {
                                    (SBWaveState::Beginning, 0) => {
                                        sb_state = SBWaveState::Low;
                                    }
                                    (SBWaveState::Beginning, 1) => {
                                        working_pt.y -= WAVEHEIGHT;
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
                                        panic!("Impliment me");
                                    }
                                }
                                prev_xcoord = time;
                                p.line_to(working_pt);
                                p.move_to(working_pt);
                            }
                            let fin_x_delt =
                                self.xdelt_from_prev(self.end_window_time(), prev_xcoord, &bounds);
                            working_pt.x += fin_x_delt;
                            p.line_to(working_pt);
                        }
                        _ => {
                            let working_pt_top = Point {
                                y: working_pt.y - WAVEHEIGHT,
                                ..working_pt
                            };
                            let mut working_pts = [working_pt_top, working_pt];
                            for (time, sig_payload) in
                                wave.droplets_in_range(self.start_time(), self.end_time())
                            {
                                if self.out_of_range(time) {
                                    break;
                                }

                                let x_delt = self.xdelt_from_prev(time, prev_xcoord, &bounds)
                                    - VEC_SHIFT_WIDTH / 2.0;

                                let text = generate_canvas_text(
                                    sig_payload,
                                    display_options,
                                    width,
                                    x_delt,
                                );

                                for (point, direction) in
                                    working_pts.iter_mut().zip([1.0, -1.0].iter())
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
                                prev_xcoord = time
                            }
                            let fin_x_delt =
                                self.xdelt_from_prev(self.end_window_time(), prev_xcoord, &bounds);
                            for point in working_pts.iter_mut() {
                                p.move_to(*point);
                                point.x += fin_x_delt;
                                p.line_to(*point);
                            }
                        }
                    }
                    leftmost_pt.y += WAVEHEIGHT + 2.0 * BUFFER_PX;
                })
            })
            .collect();

        //TODO: cache wavelist in the case of append only?

        for waves in wave_list {
            frame.stroke(&waves, Stroke::default().with_width(1.0).with_color(GREEN));
        }
    }
}


impl<'a> canvas::Program<Message> for WaveWindow<'a> {
    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Message>) {
        // TODO: Is there a more idiomatic way to do this?
        let cursor_position = if let Some(pos) = cursor.position_in(&bounds) {
            pos
        } else {
            return (event::Status::Ignored, None);
        };

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    self.frame_state.cursor_location = self.get_timestamp(cursor_position.x);
                    info!("click location is {}", self.frame_state.cursor_location,);
                    (
                        event::Status::Captured,
                        Some(Message::UpdateCursor(self.frame_state.cursor_location)),
                    )
                }

                _ => (event::Status::Captured, None),
            },
            _ => (event::Status::Ignored, None),
        }
    }

    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let content = self.wave_cache.draw(bounds.size(), |frame: &mut Frame| {
            self.draw_all(frame, bounds);
        });

        let cursors = self.cursor_cache.draw(bounds.size(), |frame: &mut Frame| {
            self.draw_header(frame, bounds);
            self.draw_cursor(frame, bounds);
        });

        vec![content, cursors]
    }
}
