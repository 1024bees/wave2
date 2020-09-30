use wave2_wavedb::{InMemWave};
use crate::components::display_wave::{WaveDisplayOptions,DisplayedWave};
use crate::components::wavewindow;
use wave2_custom_widgets::{cell_list};
use wave2_custom_widgets::cell_list::CellList;
use iced::{button, scrollable, text_input, Align, Column,Row, TextInput, Element, Container, Scrollable};
use std::sync::Arc;



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
//TODO: delete
pub enum WaveOptions {
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
    CellListPlaceholder(DisplayedWave),
}


pub struct SigViewer {
    waves_state : cell_list::State<WaveOptions>,
    wavewindow : wavewindow::WaveWindowState,
    live_waves : Vec<DisplayedWave>,
    cursor : wavewindow::CursorState,
    scroll_x: scrollable::State,
}

impl Default for SigViewer {
    fn default() -> Self {

        SigViewer {
            waves_state : cell_list::State::default(),
            wavewindow : wavewindow::WaveWindowState::default(),
            live_waves : vec![DisplayedWave::default()],
            cursor : wavewindow::CursorState::default(),
            scroll_x : scrollable::State::default(),
        }
    }
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
            cursor,
            scroll_x,
            } = self;

            //TODO: move message logic out of wavewindow
            let ww = wavewindow.view(
                    &live_waves[..],
                    *cursor)
            .map(move |message| Message::UpdateCursor(message)); 


        let wave_view = Column::new()
            .padding(20)
            .spacing(20)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .max_height(800)
            .push(ww);


        let cl = CellList::new(
                waves_state,
                &live_waves[..],
                &WaveOptions::ALL,
                Message::CellListPlaceholder,
                )
            .heading("Time".into())
            .heading_size(10);

        let pick_list = Column::new()
            .push(cl)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .max_height(800)
            .max_width(400)
            .padding(20)
            .spacing(20);

        
            Container::new(
                Row::new()
                .push(pick_list)
                .push(wave_view)
                .height(iced::Length::Fill)
                ).into()
        }

}

