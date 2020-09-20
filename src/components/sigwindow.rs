use wave2_wavedb::{InMemWave};
use crate::components::display_wave::WaveDisplayOptions;
use crate::components::wavewindow;
use wave2_custom_widgets::{cell_list};
use wave2_custom_widgets::cell_list::CellList;
use iced::{button, scrollable, text_input, Align, Column,Row, TextInput, Element, Container};
use std::sync::Arc;


struct DisplayedWave {
    wave_content : Arc<InMemWave>,
    display_conf : Option<WaveDisplayOptions>
}



impl From<Arc<InMemWave>> for DisplayedWave {
    fn from(imw : Arc<InMemWave>) -> Self {
        DisplayedWave {
            wave_content : imw,
            display_conf : Option::default(),
        }
    }
}

enum WaveOptions {
    Delete
}

impl WaveOptions {
   const ALL : [WaveOptions; 1] = [
        WaveOptions::Delete,
   ];

}
impl std::fmt::Display for WaveOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WaveOptions::Delete => "Delete",
            }
        )
    }
}




#[derive(Debug,Clone)]
pub enum Message {
    AddWave(Arc<InMemWave>),
    RemoveWave(usize),
    SetOpts(u32,WaveDisplayOptions),
    UpdateCursor(wavewindow::Message),
    ClearWaves,
    CellListPlaceholder(Arc<InMemWave>),

}

pub struct SigViewer {
    waves_state : cell_list::State<WaveOptions>,
    wavewindow : wavewindow::WaveWindowState,
    live_waves : Vec<DisplayedWave>,
    temp_hack : Vec<Arc<InMemWave>>, //don't want to touch wave window logic yet
    cursor : wavewindow::CursorState,
    scroll_x: scrollable::State,
    scroll_y: scrollable::State,
}


impl SigViewer {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::AddWave(imw) => {
                self.live_waves.push(DisplayedWave::from(imw));
                self.wavewindow.request_redraw();
            }
            Message::ClearWaves => {
                self.live_waves.clear();
                self.wavewindow = wavewindow::WaveWindowState::default();
            }
            Message::RemoveWave(idx) => {
                self.live_waves.remove(idx);
                self.wavewindow.request_redraw();
            }
            Message::CellListPlaceholder(_) => {
                println!("Cell list interaction, impl me");
            }
            _ => {
                println!("Not yet impl'd");
            }
        }
    }
    pub fn view(&mut self) -> Element<Message> {
        let SigViewer {
            waves_state,
            wavewindow,
            live_waves,
            temp_hack,
            cursor,
            scroll_x,
            scroll_y,
            } = self;

        *temp_hack = live_waves.iter()
            .map(|x| x.wave_content.clone())
            .collect();
        let ww = wavewindow.view(
                    &temp_hack[..],
                    *cursor)
            .map(move |message| Message::UpdateCursor(message)); //TODO: fix this


        let wave_view = Column::new()
            .padding(20)
            .spacing(20)
            .max_height(800)
            .push(ww);

        //let pick_list = Column::new()
        //    .push(
        //        CellList::new(
        //        waves_state,
        //        &temp_vec[..],
        //        &WaveOptions::ALL,
        //        Message::CellListPlaceholder,
        //        )
        //    )
        //    .max_height(800)
        //    .padding(20)
        //    .spacing(20);

        Container::new(
            Row::new()
            //.push(pick_list)
            .push(wave_view)
            .height(iced::Length::Shrink)
        ).into()

            




        }



    


}

