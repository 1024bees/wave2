use super::sigwindow;
use super::wavewindow;
use super::Message;
use iced::{pane_grid, Command, Element, Length, PaneGrid};
use wave2_wavedb::storage::display_wave::DisplayedWave;

pub enum BeachContent {
    Waves(wavewindow::WaveWindowState),
    Signals(sigwindow::SigViewer),
}

#[derive(Debug, Clone)]
pub enum BeachPane {
    Resize(pane_grid::ResizeEvent),
}

impl BeachContent {
    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            BeachContent::Waves(waves) => waves.update(message),
            BeachContent::Signals(signals) => signals.update(message),
        }
    }
    fn view<'a>(&'a mut self, waves: &'a [DisplayedWave]) -> Element<Message> {
        match self {
            BeachContent::Waves(ww) => ww.view2(waves),
            BeachContent::Signals(signals) => signals.view2(waves),
        }
    }
}

pub struct Beach {
    pub cursor_location: u32,
    pub waves: Vec<DisplayedWave>,
    beach_panes: pane_grid::State<BeachContent>,
    waves_pane: pane_grid::Pane,
    sig_pane: pane_grid::Pane,
}

impl Default for Beach {
    fn default() -> Self {
        let waves = BeachContent::Waves(wavewindow::WaveWindowState::default());
        let signals = BeachContent::Signals(sigwindow::SigViewer::default());
        let (mut beach_panes, sig_pane) = pane_grid::State::new(signals);

        let (waves_pane, split) = beach_panes
            .split(pane_grid::Axis::Vertical, &sig_pane, waves)
            .unwrap();
        //beach_panes.swap(&waves_pane, &sig_pane);
        beach_panes.resize(&split, 0.3);
        Beach {
            cursor_location: 0,
            waves: vec![],
            beach_panes,
            waves_pane,
            sig_pane,
        }
    }
}

impl Beach {
    fn get_sigwindow(&mut self) -> &mut sigwindow::SigViewer {
        let pane_val = self.beach_panes.get_mut(&self.sig_pane).unwrap();
        match pane_val {
            BeachContent::Signals(sigs) => sigs,
            _ => unreachable!("Should never get to this point!"),
        }
    }

    fn get_wavewindow(&mut self) -> &mut wavewindow::WaveWindowState {
        let pane_val = self.beach_panes.get_mut(&self.waves_pane).unwrap();
        match pane_val {
            BeachContent::Waves(waves) => waves,
            _ => unreachable!("Should never get to this point!"),
        }
}


    pub fn update(&mut self, message: Message) -> Command<Message> {
        fn update_sigwindow(beach: &mut Beach, message: Message) -> Command<Message> {
            beach
                .beach_panes
                .get_mut(&beach.sig_pane)
                .unwrap()
                .update(message)
        }

        fn update_wavewindow(beach: &mut Beach, message: Message) -> Command<Message> {
            beach
                .beach_panes
                .get_mut(&beach.waves_pane)
                .unwrap()
                .update(message)
        }

        match message {
            //Messages that are only handled by the sigwindow
            Message::UpdateBounds(_) | Message::UpdateCursor(_) => {
                update_wavewindow(self, message);
            }
            Message::AddWave(imw_res) => match imw_res {
                Ok(imw) => {
                    //self.waves_state.push();
                    self.waves.push(DisplayedWave::from(imw));
                    self.get_sigwindow().add_wave();
                    //self.wavewindow.request_redraw();
                }
                Err(err) => {
                    log::info!("Cannot create InMemWave, err is {:#?}", err);
                }
            },
            Message::ClearWaves => {
                self.waves.clear();
                update_sigwindow(self, message);
            }
            Message::CellListPlaceholder | Message::RemoveSelected | Message::SelectedWave(_) => {
                update_sigwindow(self, message);
            }
            _ => {
                log::info!("Not covered");
            }
        }
        Command::none()
    }

    pub fn view(&mut self) -> Element<Message> {
        let waves = &self.waves[..];
        PaneGrid::new(&mut self.beach_panes, |_, content| {
            //let title_bar = pane_grid::TitleBar::new(Text::new(format!("Focused pane"))).padding(3);
            pane_grid::Content::new(content.view(waves))
            //.title_bar(title_bar)
        })
        .width(Length::Fill)
        .height(Length::Fill)
        //FIXME: causes int overflow in the glow backend
        //.on_drag(|pane_data| Message::PaneMessage(PaneMessage::Dragged(pane_data)))
        .on_resize(10, |resize_data| {
            Message::BeachPane(BeachPane::Resize(resize_data))
        })
        .spacing(3)
        .into()
    }
}
