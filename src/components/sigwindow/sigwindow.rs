use super::display_wave::{DisplayedWave, WaveDisplayOptions};
use super::wavewindow;
use iced::{Column, Container, Element, Row};
use log::info;
use std::sync::Arc;
use crate::components::shared::cell_list::{CellList, ListNodeState};
use strum_macros;
use wave2_custom_widgets::traits::CellOption;
use wave2_wavedb::errors::Waverr;
use wave2_wavedb::storage::in_memory::InMemWave;

#[derive(Debug, Clone, Copy, PartialEq, Eq,strum_macros::Display)]
//TODO: add options, move to its own module?
pub enum WaveOptions {
    Delete,
}

impl WaveOptions {
    const ALL: [WaveOptions; 1] = [WaveOptions::Delete];
}

impl CellOption for WaveOptions {
    type Message = Message;

    fn all() -> &'static [Self] {
        &WaveOptions::ALL
    }

    fn to_message(&self) -> Self::Message {
        match self {
            WaveOptions::Delete => Message::RemoveSelected
        }
    }
}


#[derive(Debug, Clone)]
pub enum Message {
    AddWave(Result<Arc<InMemWave>, Arc<Waverr>>),
    SetOpts(u32, WaveDisplayOptions),
    InitializeWW((u32,u32)),
    WWMessage(wavewindow::Message),
    ClearWaves,
    CellListPlaceholder,
    SelectedWave(usize),
    //Option Messages
    RemoveSelected


}

#[derive(Default)]
pub struct SigViewer {
    waves_state: CellList<DisplayedWave, WaveOptions>,
    wavewindow: wavewindow::WaveWindowState,
    live_waves: Vec<DisplayedWave>,
    selected: Option<Vec<usize>>
}




impl SigViewer {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::AddWave(imw_res) => {
                match imw_res {
                    Ok(imw) => {
                        self.live_waves.push(DisplayedWave::from(imw));
                        self.waves_state.push(self.live_waves.last().unwrap().clone());
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
            Message::RemoveSelected => {
                if let Some(selected) = self.selected.as_ref() {
                    for idx in selected.into_iter().rev().cloned() {
                        self.live_waves.remove(idx);
                        self.waves_state.remove(idx);
                    }
                    self.wavewindow.request_redraw();
                }
                self.selected = None;
            }
            Message::CellListPlaceholder => {
                println!("Cell list interaction, impl me");
            }
            Message::InitializeWW(bounds) => {
                self.wavewindow.update(wavewindow::Message::UpdateBounds(bounds));
            }
            Message::WWMessage(ww_message) => {
                self.wavewindow.update(ww_message);
            }
            Message::SelectedWave(offset) => {
                if self.selected.is_some() {
                    for prev_offset in self.selected.as_ref().unwrap().iter().cloned() {
                        self.waves_state.toggle_selected(prev_offset,false);
                    }
                }

                self.waves_state.toggle_selected(offset,true);

                self.selected = Some(vec![offset]);

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
            ..
        } = self;

        //TODO: move message logic out of wavewindow
        let ww = wavewindow
            .view(&live_waves[..])
            .map(move |message| Message::WWMessage(message));

        let wave_view = Container::new(ww)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(20);



        fn click_func(node_state: ListNodeState) -> Box<dyn Fn(&DisplayedWave) -> Message + 'static > {
            return Box::new(move |_| Message::SelectedWave(node_state.offset))
        }

        fn double_click(_node_state: ListNodeState) -> Box<dyn Fn(&DisplayedWave) -> Message + 'static > {
            return Box::new(move |_| Message::CellListPlaceholder)
        }


        let cl = waves_state.view(click_func,double_click);

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
