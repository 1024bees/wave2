use iced::{
    canvas::{self, Canvas, Cursor, Event, Frame, Geometry, Path, Stroke},
    mouse, Element, Length, Point, Rectangle,Color
};

const BUFFER_PX : f32 = 4.0;
const WAVEHEIGHT : f32 = 19.0;


pub struct WaveWindow<'a>{
    signals: &'a[Wave],
    state : &'a State,
    cur_state: CursorState,
}



#[derive(Default)]
pub struct State {
    cache: canvas::Cache,
}

#[derive(Debug, Clone, Copy)]
pub struct CursorState {
    pub cursor_location : u32,
    ViewRange : (u32,u32)
}


impl Default for CursorState {
    fn default() -> Self {
        CursorState { 
            cursor_location: 10,
            ViewRange: (0,600)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SigType{
    Bit,
    Vec(u32),
}


#[derive(Debug, Clone)]
pub struct Wave {
    name : String,
    signal_content : Vec<(u32,u32)>,
    sig_type: SigType
}



impl State {
    pub fn view<'a>(
        &'a mut self,
        signals: &'a[Wave],
        cur_state : CursorState,
    ) -> Element<'a, CursorState> {
        Canvas::new(WaveWindow {
            signals: signals,
            state: self,
            cur_state : cur_state
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear()
    }
}


impl<'a> WaveWindow<'a> {
    
        
    fn get_timestamp(&self, bounds : Rectangle, xcoord : f32) -> u32 {
        let ts_width = (self.cur_state.ViewRange.1 - self.cur_state.ViewRange.0) as f32;
        self.cur_state.ViewRange.0 + (ts_width * ((bounds.x - xcoord) / bounds.width)) as u32 
    }

    //TODO: maybe add bounds.x here 
    fn ts_to_xcoord(&self, ts : u32, prev_ts: u32, bounds : &Rectangle) -> f32 {
        let ts_width = (self.cur_state.ViewRange.1 - self.cur_state.ViewRange.0) as f32;
        ((ts - prev_ts) as f32 / ts_width) * bounds.width
    }

    fn draw_all(&self,frame: &mut Frame, bounds:  Rectangle ) {
        let mut leftmost_pt = Point { x: bounds.x, y:bounds.y };
        let waves = Path::new(|p| {
            let mut working_pt = leftmost_pt.clone();
            let mut working_pt_bot = Point { y: working_pt.y + WAVEHEIGHT, ..working_pt };
            for wave in self.signals {
                p.move_to(leftmost_pt);
                let mut prev_xcoord = self.cur_state.ViewRange.0;
                for signal_change in wave.signal_content.iter() {
                    working_pt.x += self.ts_to_xcoord(signal_change.0,prev_xcoord,&bounds);
                    println!("Working pt x: {}", working_pt.x);
                    p.line_to(working_pt);
                    p.move_to(working_pt);
                    match signal_change.1 {
                        0 =>  working_pt.y -= WAVEHEIGHT,
                        _ => working_pt.y += WAVEHEIGHT
                    }
                    p.line_to(working_pt);
                }
                leftmost_pt.y += WAVEHEIGHT + BUFFER_PX;
                working_pt = leftmost_pt;

            }
        });

        frame.stroke(&waves, Stroke::default().with_width(1.0));
    }
}


//TODO: move to backend, make inmemory wave
impl Default for Wave {
    fn default() -> Self {
        Wave {
            name : String::from("PlaceholderWave"),
            signal_content : vec![(0,0),(1,1),(2,0),(3,1),(50, 0), (500,1)],
            sig_type : SigType::Bit,
        }
    }
}


impl Wave { }
        
impl<'a> canvas::Program<CursorState> for WaveWindow<'a> {
    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<CursorState> {

        let cursor_position = cursor.position_in(&bounds)?;
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    self.cur_state.cursor_location = self.get_timestamp(bounds,cursor_position.x);
                    println!("cursor pos is {}", cursor_position.x);
                    Some(self.cur_state.clone())
                },
                _ => None
            },
            _ => None
        }

    }



    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let content = self.state.cache.draw(bounds.size(), |frame: &mut Frame| {
            self.draw_all(frame, bounds);

            frame.stroke(
                    &Path::rectangle(Point::ORIGIN, frame.size()),
                    Stroke::default()
                );
        });

        vec![content]
    }


}
