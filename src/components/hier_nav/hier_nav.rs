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
use crate::components::hier_nav::hier_node::{HierNode, HierRoot};


#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display)]
pub enum HierOptions {
    Expand,
}

impl HierOptions {
    //TODO: create ALL macro
    pub const ALL: [HierOptions; 1] = [HierOptions::Expand];
}

pub struct HierNav {
    live_hier: Arc<HierMap>,
    scroll_x: scrollable::State,
    hier_root: HierRoot,
}


#[derive(Debug, Clone)]
pub enum Message {
    SetHier(Arc<HierMap>),
    UpdateModNav(Arc<Vec<SignalItem>>),
    Toggle(usize),
    SendModule(usize),
    Placeholder,
}

impl HierNav {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetHier(payload) => {
                self.live_hier = payload;
                self.hier_root = HierRoot::from(self.live_hier.as_ref());
            },
            Message::Toggle(idx) => {
                let map_ref = self.live_hier.as_ref();
                self.hier_root.expand_module(map_ref.idx_to_path(idx));
            },

            _ => {
                error!("Not implimented yet!");
            }
        }
    }
    pub fn view(&mut self) -> Element<Message> {

        Container::new(self.hier_root.view())
            .padding(20)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .max_height(800)
            .into()

    }



}
