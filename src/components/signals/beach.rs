use super::sigwindow;
use super::wavewindow;
use super::Message;
use iced::{pane_grid, Command, Element, PaneGrid};
use wave2_wavedb::storage::display_wave::DisplayedWave;

pub enum BeachContent {
    Waves(wavewindow::WaveWindowState),
    Signals(sigwindow::SigViewer),
}

impl BeachContent {
    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            BeachContent::Waves(waves) => waves.update(message),
            BeachContent::Signals(signals) => signals.update(message),
        }
    }
    fn view(&mut self, beach: &mut Beach) -> Element<Message> {
        match self {
            BeachContent::Waves(waves) => waves.view(),
            BeachContent::Signals(signals) => signals.view(),
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

impl Beach {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            //Messages that are only handled by the sigwindow
            Message::UpdateBounds(_) | Message::UpdateCursor(_) => {
                self.beach_panes
                    .get_mut(&self.waves_pane)
                    .unwrap()
                    .update(message);
            }
            Message::AddWave(imw_res) => match imw_res {
                Ok(imw) => {
                    //self.waves_state.push();
                    self.waves.push(DisplayedWave::from(imw));
                    //self.wavewindow.request_redraw();
                }
                Err(err) => {
                    log::info!("Cannot create InMemWave, err is {:#?}", err);
                }
            },
            Message::ClearWaves => self.waves.clear(),
            Message::CellListPlaceholder | Message::RemoveSelected | Message::SelectedWave(_) => {
                self.beach_panes
                    .get_mut(&self.waves_pane)
                    .unwrap()
                    .update(message);
            }
            _ => {
                log::info!("Not covered");
            }
        }
        Command::none()
    }
}
