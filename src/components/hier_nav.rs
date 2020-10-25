use iced::{
    button, scrollable, text_input, Align, Column, Container, Element, Length,
    Row, Scrollable, TextInput,
};
use log::error;
use std::sync::Arc;
use strum::IntoEnumIterator;
use strum_macros;
use wave2_custom_widgets::widget::cell_list;
use wave2_custom_widgets::widget::cell_list::CellList;
use wave2_wavedb::hier_map::{HierMap, SignalItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display)]
pub enum HierOptions {
    Expand,
}

impl HierOptions {
    //TODO: create ALL macro
    const ALL: [HierOptions; 1] = [HierOptions::Expand];
}

///This is for navigating signals within a module
pub struct HierNavigator {
    live_hier: Option<Arc<HierMap>>,
    scroll_x: scrollable::State,
    hier_state: cell_list::State<HierOptions>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetHier(Arc<HierMap>),
    ExpandHier(usize),
    UpdateModNav(Arc<Vec<SignalItem>>),
}

impl HierNavigator {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetHier(payload) => self.live_hier = Some(payload),
            _ => {
                error!("Not implimented yet!");
            }
        }
    }
}
