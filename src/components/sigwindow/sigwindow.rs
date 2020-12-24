use super::display_wave::{DisplayedWave, WaveDisplayOptions};
use super::wavewindow;
use iced::{Column, Container, Element, Row};
use log::info;
use std::sync::Arc;
use wave2_custom_widgets::widget::cell_list;
use wave2_custom_widgets::widget::cell_list::CellList;


use wave2_wavedb::errors::Waverr;
use wave2_wavedb::InMemWave;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
//TODO: add options, move to its own module?
pub enum WaveOptions {
    Delete,
}

impl WaveOptions {
    const ALL: [WaveOptions; 1] = [WaveOptions::Delete];
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

#[derive(Debug, Clone)]
pub enum Message {
    AddWave(Result<Arc<InMemWave>, Arc<Waverr>>),
    RemoveWave(usize),
    SetOpts(u32, WaveDisplayOptions),
    InitializeWW((u32,u32)),
    WWMessage(wavewindow::Message),
    ClearWaves,
    CellListPlaceholder(DisplayedWave),
}

pub struct SigViewer {
    waves_state: cell_list::State<WaveOptions>,
    wavewindow: wavewindow::WaveWindowState,
    live_waves: Vec<DisplayedWave>,
}


impl Default for SigViewer {
    fn default() -> Self {
        SigViewer {
            waves_state: cell_list::State::default(),
            wavewindow: wavewindow::WaveWindowState::default(),
            live_waves: Vec::default(),
        }
    }
}

impl SigViewer {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::AddWave(imw_res) => {
                match imw_res {
                    Ok(imw) => {
                        self.live_waves.push(DisplayedWave::from(imw));
                        self.wavewindow.request_redraw();
                    },
                    Err(err) => {
                        info!("Cannot create InMemWave, err is {:#?}",err);
                    }
                }
            }
            Message::ClearWaves => {
                self.live_waves.clear();
            }
            Message::RemoveWave(idx) => {
                self.live_waves.remove(idx);
                self.wavewindow.request_redraw();
            }
            Message::CellListPlaceholder(_) => {
                println!("Cell list interaction, impl me");
            }
            Message::InitializeWW(bounds) => {
                self.wavewindow.update(wavewindow::Message::UpdateBounds(bounds));
            }
            Message::WWMessage(ww_message) => {
                self.wavewindow.update(ww_message);
            }
            _ => {
                info!("Not yet impl'd");
            }
        }
    }
    pub fn view(&mut self) -> Element<Message> {
        let SigViewer {
            waves_state,
            wavewindow,
            live_waves,
        } = self;

        //TODO: move message logic out of wavewindow
        let ww = wavewindow
            .view(&live_waves[..])
            .map(move |message| Message::WWMessage(message));

        let wave_view = Container::new(ww)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(20);
            
        let cl = CellList::new(waves_state, &live_waves[..], &WaveOptions::ALL)
            .text_size(12)
            .heading("Time".into())
            .heading_size(10);

        let pick_list = Column::new()
            .push(cl)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .max_width(400)
            .padding(20)
            .spacing(20);

        Container::new(
            Row::new()
                .push(pick_list)
                .push(wave_view)
                .height(iced::Length::Fill),
        )
        .into()
    }
}
