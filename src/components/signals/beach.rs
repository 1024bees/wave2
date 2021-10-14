use super::sigwindow;
use super::wavewindow;
use super::Message;
use iced::{pane_grid, Command, Element, Length, PaneGrid};
use wave2_wavedb::formatting::format_payload;
use wave2_wavedb::formatting::WaveFormat;
use wave2_wavedb::storage::display_wave::DisplayedWave;
use wave2_wavedb::storage::in_memory::InMemWave;

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
    pub fn mut_cursor_location(&mut self) -> &mut u32 {
        &mut self.get_wavewindow().widget_state.cursor_location
    }

    pub fn curr_cursor_location(&mut self) -> u32 {
        self.get_wavewindow().widget_state.cursor_location
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

        fn get_data(
            imw: std::sync::Arc<InMemWave>,
            time: u32,
            format: WaveFormat,
        ) -> Option<(u32, String)> {
            let drop = imw.get_droplet_at(time);
            if let Some(droplet) = drop {
                //TODO: fixme, this 400 is sinful! we should have a more thoughtful way
                //of setting max characters visible, or make this parameter optional to
                //show that there is no max, since we eventually want sigview to be in
                //an hscroll s.t. you can see the entire value that the cursor is
                //hovering over
                let outstr = format_payload(droplet, format, imw.get_width(), 400);
                Some((imw.signal_id, outstr))
            } else {
                None
            }
        }

        match message {
            Message::UpdateWaveValues(Some((id, value))) => {
                self.waves
                    .iter_mut()
                    .filter(|wave| wave.get_wave().signal_id == id)
                    .for_each(|wave| wave.val_under_cursor = Some(value.clone()));
            }

            //Messages that are only handled by the sigwindow
            Message::UpdateBounds(_) | Message::UpdateCursor(_) => {
                log::info!("Updating the cursor, yipee!");
                update_wavewindow(self, message.clone());
                if let Message::UpdateCursor(time) = message {
                    let cv: Vec<Command<Message>> = self
                        .waves
                        .iter()
                        .map(|wave| {
                            let format = wave.display_conf.unwrap_or_default().format;
                            let imw = wave.get_wave().clone();
                            Command::perform(
                                async move { get_data(imw, time, format) },
                                Message::UpdateWaveValues,
                            )
                        })
                        .collect();
                    return Command::batch(cv);
                }
            }

            Message::Next | Message::Prev => {
                if let Some(ref selected_wave) = self.get_sigwindow().selected {
                    if selected_wave.len() == 1 {
                        let wave_idx = selected_wave.get(0).unwrap().clone();
                        let working_wave = self.waves.get(wave_idx).expect("Some waves should be here, we have one selected, if this fires then our clearing logic for sigwindow is messed up").get_wave().clone();
                        let current_cursor_loc = self.get_wavewindow().widget_state.cursor_location;
                        if let Message::Next = message {
                            return Command::perform(
                                async move {
                                    working_wave
                                        .get_next_time(current_cursor_loc)
                                        .map(|(time, _)| time)
                                        .unwrap_or(current_cursor_loc)
                                },
                                Message::UpdateCursor,
                            );
                        } else {
                            return Command::perform(
                                async move {
                                    working_wave
                                        .get_prev_time(current_cursor_loc)
                                        .map(|(time, _)| time)
                                        .unwrap_or(current_cursor_loc)
                                },
                                Message::UpdateCursor,
                            );
                        }
                    } else {
                        return Command::none();
                    }
                } else {
                    return Command::none();
                }
            }

            Message::AddWave(imw_res) => match imw_res {
                Ok(imw) => {
                    //self.waves_state.push();
                    let holder = imw.clone();
                    let ct = self.get_wavewindow().widget_state.cursor_location;
                    let mut dw = DisplayedWave::from(imw);

                    self.get_sigwindow().add_wave();
                    //TODO: get waveformat from some global state
                    if let Some((_, val)) = get_data(holder, ct, WaveFormat::default()) {
                        dw.val_under_cursor = Some(val);
                    }
                    self.waves.push(dw);
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
                if let Message::RemoveSelected = message {
                    // look this really pisses me off i need a way to have fine grained mutable
                    // borrows
                    let vec = if let Some(ref selected_vec) = self.get_sigwindow().selected {
                        let mut rv = selected_vec.clone();
                        rv.sort();
                        rv
                    } else {
                        vec![]
                    };
                    vec.into_iter().for_each(|i| {
                        self.waves.remove(i);
                    });
                }

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
