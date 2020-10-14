use wave2_wavedb::hier_map::{HierMap,SignalItem};
use std::sync::Arc;
use wave2_custom_widgets::{cell_list};
use wave2_custom_widgets::cell_list::CellList;
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
#[derive(Default)]
pub struct HierNavigator {
    live_hier : Option<HierMap>,
    scroll_x: scrollable::State,
    hier_state : cell_list::State<HierOptions>,
}


pub enum Message {
    SetHier(Arc<HierMap>),
    ExpandHier(usize),
    UpdateModNav(Arc<Vec<SignalItem>>),
}



