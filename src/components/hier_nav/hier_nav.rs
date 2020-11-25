use crate::components::hier_nav::hier_node::{HierNode, HierRoot};
use iced::{
    button, scrollable, text_input, Align, Column, Container, Element, Length,
    Row, Scrollable, TextInput,
};
use log::error;
use std::cell::Cell;
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
    pub const ALL: [HierOptions; 1] = [HierOptions::Expand];
}

#[derive(Default)]
pub struct HierNav {
    live_module: Cell<usize>,
    scroll_x: scrollable::State,
    hier_root: HierRoot,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetHier(Arc<HierMap>),
    Toggle(usize),
    SendModule(usize),
    Placeholder,
}

impl HierNav {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetHier(payload) => {
                self.hier_root = HierRoot::from(payload.as_ref());
            }
            Message::Toggle(module_idx) => {
                self.hier_root.update_expander(module_idx);
            }
            _ => {
                error!("Not implimented yet!");
            }
        }
    }
    pub fn view(&mut self) -> Element<Message> {
        let HierNav {
            live_module,
            scroll_x,
            hier_root,
        } = self;

        let content = Container::new(hier_root.view())
            .padding(20)
            .max_height(400)
            .max_width(200)
            .center_x();

        Scrollable::new(scroll_x)
            .push(content)
            .max_height(400)
            .max_width(200)
            .into()
    }
}
