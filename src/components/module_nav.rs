use iced::{scrollable, Column, Container, Element, Length, Scrollable};
use log::{error,info};
use std::sync::Arc;
use strum_macros;

use wave2_custom_widgets::widget::cell;
use wave2_custom_widgets::widget::cell::Cell as VizCell;

use wave2_wavedb::hier_map::SignalItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display)]
//TODO: add options, move to its own module?
pub enum SigOptions {
    Add,
}

impl SigOptions {
    //TODO: create ALL macro
    const ALL: [SigOptions; 1] = [SigOptions::Add];
}

#[derive(Debug, Clone)]
pub enum Message {
    SignalUpdate(Arc<Vec<SignalItem>>),
    AddSig(SignalItem),
    ClickedItem(usize),
}

#[derive(Debug, Clone, Default)]
pub struct SignalNode {
    ui_state: cell::State<SigOptions>,
    payload: SignalItem,
    offset: usize,
    selected: bool,
}

impl SignalNode {
    fn new(payload: SignalItem, offset: usize) -> Self {
        SignalNode {
            payload,
            offset,
            ..SignalNode::default()
        }
    }
    fn view(&mut self) -> Element<Message> {
        let SignalNode {
            ui_state,
            payload,
            offset,
            selected,
        } = self;
        let local_offset = offset.clone();
        let sig_cell = VizCell::new(ui_state, payload, &SigOptions::ALL)
            .on_double_click(|signal| { info!("Double click!"); Message::AddSig(signal.clone())})
            .on_click(move |_| Message::ClickedItem(local_offset))
            .override_selected(selected.clone());

        sig_cell.into()
    }
}

///This is for navigating signals within a module
#[derive(Default)]
pub struct ModNavigator {
    signals: Vec<SignalNode>,
    selected_offset: Option<usize>,
    scroll_x: scrollable::State,
}

impl ModNavigator {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SignalUpdate(payload) => {
                self.signals = Arc::try_unwrap(payload).map_or_else(
                    |e| {
                        e.as_ref()
                            .iter()
                            .cloned()
                            .enumerate()
                            .map(|(idx, payload)| SignalNode::new(payload, idx))
                            .collect()
                    },
                    |o| {
                        o.into_iter()
                            .enumerate()
                            .map(|(idx, payload)| SignalNode::new(payload, idx))
                            .collect()
                    },
                );

                self.selected_offset = None;
            }

            Message::ClickedItem(offset) => {
                if let Some(prev_offset) = self.selected_offset {
                    self.signals[prev_offset].selected = false;
                }
                self.signals[offset].selected = true;
                self.selected_offset = Some(offset);
            }
            _ => {
                error!("Not implimented yet!");
            }
        }
    }
    pub fn view(&mut self) -> Element<Message> {
        let ModNavigator {
            signals, scroll_x, ..
        } = self;
        let viewed_signals = Column::with_children(
            signals.iter_mut().map(|x| x.view()).collect(),
        );

        let scrollable = Scrollable::new(scroll_x).push(
            Container::new(viewed_signals)
                .height(Length::Fill)
                .width(Length::Fill)
                .center_x(),
        );

        Container::new(scrollable)
            .height(Length::Fill)
            .center_y()
            .max_width(200)
            .max_height(400)
            .into()
    }
}
