use iced::{
    button, scrollable, text_input, Align, Column, Container, Element, Length,
    Row, Scrollable, TextInput,
};
use log::{error,info};
use std::sync::Arc;
use strum::IntoEnumIterator;
use strum_macros;
use wave2_custom_widgets::widget::cell_list;
use wave2_custom_widgets::widget::cell_list::CellList;

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

///This is for navigating signals within a module
#[derive(Default)]
pub struct ModNavigator {
    signals: Vec<SignalItem>,
    scroll_x: scrollable::State,
    sig_state: cell_list::State<SigOptions>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SignalUpdate(Arc<Vec<SignalItem>>),
    AddSig(SignalItem),
}

impl ModNavigator {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SignalUpdate(payload) => {
                self.signals = Arc::try_unwrap(payload).map_or_else(
                    |e| {
                        info!("Number of ref counts is {}",Arc::strong_count(&e));
                        e.as_ref().clone()
                    },
                    |o| o);
            }
            _ => {
                error!("Not implimented yet!");
            }
        }
    }
    pub fn view(&mut self) -> Element<Message> {
        let ModNavigator {
            signals,
            scroll_x,
            sig_state,
        } = self;

        let ts = CellList::new(
            sig_state,
            &signals[..],
            &SigOptions::ALL,
            //Message::AddSig,
        )
        .text_size(12)
        .on_double_click(|sig_item| {info!("hello!"); Message::AddSig(sig_item.clone())})
        .heading("Signals".into())
        .heading_size(10);

        let scrollable = Scrollable::new(scroll_x).push(
            Container::new(ts)
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
