use iced::{
    canvas::{self, Canvas, Cursor, Event, Frame, Geometry, Path, Stroke},
    mouse, Element, Length, Point, Rectangle,Color


};

const BUFFER_PX: f32 = 4.0;
const WAVEHEIGHT: f32 = 19.0;
const VEC_SHIFT_WIDTH: f32 = 4.0;

pub struct WaveWindow<'a> {
    signals: &'a [Wave],
    state: &'a WaveWindowState,
    cur_state: CursorState,
}


pub struct WaveWindowState {
    cache: canvas::Cache,
    end_sim_time : u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CursorState {
    pub cursor_location: u32,
    view_range: (u32, u32),
}

impl Default for CursorState {
    fn default() -> Self {
        CursorState { cursor_location: 10, view_range: (0, 800) }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SigType {
    Bit,
    Vector(u32),
}

#[derive(Debug, Clone)]
pub struct Wave {
    name: String,
    signal_content: Vec<(u32, u32)>,
    sig_type: SigType,
}

impl WaveWindowState {
    pub fn view<'a>(
        &'a mut self,
        signals: &'a [Wave],
        cur_state: CursorState,
    ) -> Element<'a, CursorState> {
        Canvas::new(WaveWindow { signals: signals, state: self, cur_state: cur_state })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear()
    }
}

impl<'a> WaveWindow<'a> {
    fn get_timestamp(&self, bounds: Rectangle, xcoord: f32) -> u32 {
        let ts_width = (self.cur_state.view_range.1 - self.cur_state.view_range.0) as f32;
        println!("ts_width is {}", ts_width);
        self.cur_state.view_range.0 + (ts_width * ((xcoord - bounds.x) / bounds.width)) as u32
    }


    fn end_window_time(&self) -> u32 {
        if self.cur_state.view_range.1 < self.state.end_sim_time {
            return self.cur_state.view_range.1;
        }
        self.state.end_sim_time
    }

    /// Util for finding the x offset in the wave window where a wave should change values 
    /// Used in the context of streaming through a container of "changed value" instances 
    fn xdelt_from_prev(&self, ts: u32, prev_ts: u32, bounds: &Rectangle) -> f32 {
        let ts_width = (self.cur_state.view_range.1 - self.cur_state.view_range.0) as f32;
        ((ts - prev_ts) as f32 / ts_width) * bounds.width
    }

    //TODO: only redraw "dirty" signals
    fn draw_all(&self, frame: &mut Frame, bounds: Rectangle) {
        let mut leftmost_pt = bounds.position();
        leftmost_pt.y += WAVEHEIGHT + BUFFER_PX;
        //TODO: file issue on iced? maybe its a line width issue
        let mut bgpt = bounds.position();
        bgpt.x -= 2.0;
        let background = Path::rectangle(bgpt,bounds.size());
        let wave_list: Vec<Path> = self
            .signals
            .iter()
            .map(|wave| {
                Path::new(|p| {
                    let mut working_pt = leftmost_pt.clone();
                    p.move_to(leftmost_pt);
                    let mut prev_xcoord = self.cur_state.view_range.0;
                    match wave.sig_type {
                        SigType::Bit => {
                            println!("rendering bitwave!");
                            for signal_change in wave.signal_content.iter() {
                                //println!("xdelt is {}",self.xdelt_from_prev(signal_change.0, prev_xcoord, &bounds);)

                                working_pt.x +=
                                    self.xdelt_from_prev(signal_change.0, prev_xcoord, &bounds);
                                p.line_to(working_pt);
                                p.move_to(working_pt);
                                match signal_change.1 {
                                    0 => working_pt.y += WAVEHEIGHT,
                                    _ => working_pt.y -= WAVEHEIGHT,
                                }
                                prev_xcoord = signal_change.0;
                                p.line_to(working_pt);
                                p.move_to(working_pt);
                            }
                            let fin_x_delt = self.xdelt_from_prev(self.end_window_time(), prev_xcoord, &bounds);
                            working_pt.x += fin_x_delt;
                            p.line_to(working_pt);
                        }
                        SigType::Vector(width) => {
                            let working_pt_top =
                                    Point { y: working_pt.y - WAVEHEIGHT, ..working_pt };
                            let mut working_pts = [working_pt_top, working_pt];
                            for signal_change in wave.signal_content.iter() {
                                let x_delt = self.xdelt_from_prev(signal_change.0, prev_xcoord, &bounds) - VEC_SHIFT_WIDTH / 2.0;
                                println!("working pt top x : {}, y : {}", working_pt.x, working_pt.y);
                                                                println!("working pt top x : {}, y : {}", working_pt_top.x, working_pt_top.y);


                                for (point, direction) in
                                     working_pts.iter_mut().zip([1.0, -1.0].iter())
                                {
                                    p.move_to(*point);
                                    point.x += x_delt;
                                    p.line_to(*point);
                                    point.y += WAVEHEIGHT * direction;
                                    //TODO: logic for when really zoomed out, so we dont move past the next
                                    //delta
                                    point.x += VEC_SHIFT_WIDTH/2.0; 

                                    p.line_to(*point);
                                    point.y -= WAVEHEIGHT * direction;

                                }
                                prev_xcoord = signal_change.0;
                            }
                            let fin_x_delt = self.xdelt_from_prev(self.end_window_time(), prev_xcoord, &bounds);
                            for point in working_pts.iter_mut() {
                                p.move_to(*point);
                                point.x += fin_x_delt;
                                p.line_to(*point);
                            }

                        }
                    }
                    println!("leftmost pt x : {}, y : {}",leftmost_pt.x, leftmost_pt.y);
                    leftmost_pt.y += WAVEHEIGHT + BUFFER_PX;
                })
            })
            .collect();


        //TODO: cache wavelist in the case of append only?
        frame.fill(&background, Color::BLACK);
        for waves in wave_list {
            frame.stroke(&waves, Stroke::default().with_width(1.0).with_color(Color::from_rgb(0.0,1.0,0.0)));
        }
    }



}


impl Default for WaveWindowState {
    fn default() -> Self {
        WaveWindowState {
            cache: canvas::Cache::default(),
            end_sim_time : 600
        }
    }
}

//TODO: move to backend, make inmemory wave
impl Default for Wave {
    fn default() -> Self {
        Wave {
            name: String::from("PlaceholderWave"),
            signal_content: vec![(0, 1), (10, 0), (20, 1), (30, 0), (50, 1), (500, 0)],
            sig_type: SigType::Bit,
        }
    }
}

//TODO: move from Wave -> InMemoryWave... should there be a transform there even?
impl Wave {
    pub fn default_vec() -> Self {
        Wave { sig_type: SigType::Vector(4), ..Wave::default() }
    }
}

impl<'a> canvas::Program<CursorState> for WaveWindow<'a> {
    fn update(&mut self, event: Event, bounds: Rectangle, cursor: Cursor) -> Option<CursorState> {
        let cursor_position = cursor.position_in(&bounds)?;
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    self.cur_state.cursor_location = self.get_timestamp(bounds, cursor_position.x);
                    println!("Click!, timestamp is {}",self.get_timestamp(bounds, cursor_position.x));
                    Some(self.cur_state.clone())
                }
                _ => None,
            },
        }
    }

    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let content = self.state.cache.draw(bounds.size(), |frame: &mut Frame| {
            self.draw_all(frame, bounds);

            frame.stroke(&Path::rectangle(Point::ORIGIN, frame.size()), Stroke::default());
        });

        vec![content]
    }
}
