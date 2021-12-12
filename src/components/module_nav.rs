use iced::{scrollable, Command, Container, Element, Length, Scrollable};
use log::error;
use std::sync::Arc;



use crate::components::shared::cell_list::{CellList, ListNodeState};
use wave2_wavedb::hier_map::SignalItem;

//#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display)]
//pub enum SigOptions {
//    Add,
//}
//
//impl SigOptions {
//    const ALL: [SigOptions; 1] = [SigOptions::Add];
//}
//
//impl CellOption for SigOptions {
//    type Message = Message;
//
//    fn all() -> &'static [Self] {
//        &SigOptions::ALL
//    }
//
//    fn to_message(&self) -> Self::Message {
//        match self {
//            SigOptions::Add => Message::AddSelected,
//        }
//    }
//}

#[derive(Debug, Clone)]
pub enum Message {
    SignalUpdate(Arc<Vec<SignalItem>>),
    AddSigOffset(usize),
    AddSig(SignalItem),
    ClickedItem(usize),
    //Messages from SigOptions
    AddSelected,
}

///Responsible for navigating signals within a module
#[derive(Default)]
pub struct ModNavigator {
    signal_vec: Vec<SignalItem>,
    signals: CellList<Message>,
    selected_offset: Option<usize>,
    scroll_x: scrollable::State,
}

impl ModNavigator {
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SignalUpdate(payload) => {
                self.signal_vec = Arc::try_unwrap(payload)
                    .map_or_else(|arcvec| arcvec.as_ref().clone(), |vec| vec);

                self.signals = CellList::new(self.signal_vec.len());
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
            Message::AddSigOffset(offset) => {
                let sig_item = self.signal_vec[offset].clone();
                return Command::perform(async move { sig_item }, Message::AddSig);
            }
            _ => {
                error!("Not implimented yet!");
            }
        }
        Command::none()
    }
    pub fn view(&mut self) -> Element<Message> {
        let ModNavigator {
            signal_vec,
            signals,
            scroll_x,
            ..
        } = self;

        fn click_func(node_state: ListNodeState) -> Box<dyn Fn() -> Message + 'static> {
            return Box::new(move || Message::ClickedItem(node_state.offset));
        }

        fn double_click(node_state: ListNodeState) -> Box<dyn Fn() -> Message + 'static> {
            return Box::new(move || Message::AddSigOffset(node_state.offset));
        }

        let viewed_signals = signals.view(signal_vec.iter(), click_func, double_click);

        let scrollable = Scrollable::new(scroll_x).push(
            Container::new(viewed_signals)
                .height(Length::Shrink)
                .width(Length::Fill)
                .center_x(),
        );

        Container::new(scrollable)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_y()
            .into()
    }
}
