use iced::{scrollable, Container, Element, Length, Scrollable};
use log::error;
use std::sync::Arc;
use strum_macros;
use wave2_custom_widgets::traits::CellOption;

use crate::components::shared::cell_list::{CellList, ListNodeState};
use wave2_wavedb::hier_map::SignalItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display)]
pub enum SigOptions {
    Add,
}

impl SigOptions {
    const ALL: [SigOptions; 1] = [SigOptions::Add];
}

impl CellOption for SigOptions {
    type Message = Message;

    fn all() -> &'static [Self] {
        &SigOptions::ALL
    }

    fn to_message(&self) -> Self::Message {
        match self {
            SigOptions::Add => Message::AddSelected,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SignalUpdate(Arc<Vec<SignalItem>>),
    AddSig(SignalItem),
    ClickedItem(usize),
    //Messages from SigOptions
    AddSelected,
}

///Responsible for navigating signals within a module
#[derive(Default)]
pub struct ModNavigator {
    signals: CellList<SignalItem, SigOptions>,
    selected_offset: Option<usize>,
    scroll_x: scrollable::State,
}

impl ModNavigator {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SignalUpdate(payload) => {
                self.signals = Arc::try_unwrap(payload)
                    .map_or_else(|e| CellList::new(e.as_ref().clone()), |o| CellList::new(o));

                self.selected_offset = None;
            }

            Message::ClickedItem(offset) => {
                if let Some(prev_offset) = self.selected_offset {
                    self.signals.toggle_selected(prev_offset, false);
                }
                self.signals.toggle_selected(offset, true);
                self.selected_offset = Some(offset);
            }
            Message::AddSelected => {}
            _ => {
                error!("Not implimented yet!");
            }
        }
    }
    pub fn view(&mut self) -> Element<Message> {
        let ModNavigator {
            signals, scroll_x, ..
        } = self;

        fn click_func(node_state: ListNodeState) -> Box<dyn Fn(&SignalItem) -> Message + 'static> {
            return Box::new(move |_| Message::ClickedItem(node_state.offset));
        }

        fn double_click(
            _node_state: ListNodeState,
        ) -> Box<dyn Fn(&SignalItem) -> Message + 'static> {
            return Box::new(|sig_item| Message::AddSig(sig_item.clone()));
        }

        let viewed_signals = signals.view(click_func, double_click);

        let scrollable = Scrollable::new(scroll_x).push(
            Container::new(viewed_signals)
                .height(Length::Shrink)
                .width(Length::Shrink)
                .center_x(),
        );

        Container::new(scrollable)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_y()
            .into()
    }
}
