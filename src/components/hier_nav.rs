use wave2_wavedb::hier_map::{HierMap,SignalItem};
use std::sync::Arc;
use wave2_custom_widgets::widget::{cell_list};
use wave2_custom_widgets::widget::cell_list::CellList;
use iced::{button, scrollable, text_input, Align, Column,Row, TextInput, Element, Container, Scrollable,Length};
use strum::IntoEnumIterator;
use strum_macros;
use log::{error};






#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display)]
pub enum HierOptions {
    Expand,
}


impl HierOptions {
    //TODO: create ALL macro
   const ALL: [HierOptions; 1] =  [HierOptions::Expand];
}

///This is for navigating signals within a module
pub struct HierNavigator {
    live_hier : Option<Arc<HierMap>>,
    scroll_x: scrollable::State,
    hier_state : cell_list::State<HierOptions>,
}

#[derive(Debug,Clone)]
pub enum Message {
    SetHier(Arc<HierMap>),
    ExpandHier(usize),
    UpdateModNav(Arc<Vec<SignalItem>>),
}


impl HierNavigator {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetHier(payload) => {
                self.live_hier = Some(payload)
            },
            _ => {
                error!("Not implimented yet!");
            }
        }
    }
}




