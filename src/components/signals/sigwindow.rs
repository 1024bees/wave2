use super::display_wave::DisplayedWave;
use crate::components::shared::cell_list::{CellList, ListNodeState};
use iced::{Column, Container, Element, Row};
use strum_macros;
use wave2_custom_widgets::traits::CellOption;
use super::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display)]
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
            WaveOptions::Delete => Message::RemoveSelected,
        }
    }
}


pub struct SigViewer {
    waves_state: CellList<DisplayedWave, WaveOptions>,
    selected: Option<Vec<usize>>,
}

impl Default for SigViewer {
    fn default() -> Self {
        SigViewer {
            waves_state: CellList::default().set_cell_padding(4).set_text_size(11),
            //.set_spacing(wavewindow::BUFFER_PX as u16),
            selected: Option::default(),
        }
    }
}

impl SigViewer {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::AddWave(imw_res) => match imw_res {
                Ok(imw) => {
                    self.waves_state
                        .push(DisplayedWave::from(imw));
                    //self.wavewindow.request_redraw();
                }
                Err(err) => {
                    log::info!("Cannot create InMemWave, err is {:#?}", err);
                }
            },
            Message::ClearWaves => {
                self.waves_state.clear()
            }
            Message::RemoveSelected => {
                if let Some(selected) = self.selected.as_ref() {
                    for idx in selected.into_iter().rev().cloned() {
                        self.waves_state.remove(idx);
                    }
                    //self.wavewindow.request_redraw();
                }
                self.selected = None;
            }
            Message::CellListPlaceholder => {
                println!("Cell list interaction, impl me");
            }
            Message::SelectedWave(offset) => {
                if self.selected.is_some() {
                    for prev_offset in self.selected.as_ref().unwrap().iter().cloned() {
                        self.waves_state.toggle_selected(prev_offset, false);
                    }
                }

                self.waves_state.toggle_selected(offset, true);

                self.selected = Some(vec![offset]);
            }
            _ => {
                log::info!("Not yet impl'd");
            }
        }
    }
    pub fn view(&mut self) -> Element<Message> {
        let SigViewer {
            waves_state,
            ..
            //wavewindow,
            //live_waves,
        } = self;

        //TODO: move message logic out of wavewindow
        //let ww = wavewindow
        //    .view(&live_waves[..])
        //    .map(move |message| Message::WWMessage(message));

        fn click_func(
            node_state: ListNodeState,
        ) -> Box<dyn Fn(&DisplayedWave) -> Message + 'static> {
            return Box::new(move |_| Message::SelectedWave(node_state.offset));
        }

        fn double_click(
            _node_state: ListNodeState,
        ) -> Box<dyn Fn(&DisplayedWave) -> Message + 'static> {
            return Box::new(move |_| Message::CellListPlaceholder);
        }

        let cl = waves_state.view(click_func, double_click);

        let pick_list = Column::new()
            //.push(
            //    Text::new("Active signals")
            //        .height(iced::Length::Units(
            //            (wavewindow::TS_FONT_SIZE + wavewindow::BUFFER_PX) as u16,
            //        ))
            //        .size(wavewindow::TS_FONT_SIZE as u16),
            //)
            .push(cl)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .max_width(400)
            .padding(20);
        //.spacing(20);

        Container::new(Row::new().push(pick_list).height(iced::Length::Fill)).into()
    }
}
