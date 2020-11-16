use crate::components::hier_nav::hier_nav::{HierOptions, Message};
use iced::{
    button, scrollable, text_input, Align, Button, Column, Container, Element, Length, Row,
    Scrollable, Text, TextInput,
};
use log::error;
use std::slice::IterMut;
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use wave2_custom_widgets::widget::cell;
use wave2_custom_widgets::widget::cell::Cell as VizCell;
use wave2_wavedb::hier_map::{HierMap, ModuleItem, SignalItem};

#[derive(Debug, Clone, Default)]
struct ModuleWrapper {
    hier_idx: usize,
    name: String,
}

impl ToString for ModuleWrapper {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl From<&ModuleItem> for HierNode {
    fn from(module: &ModuleItem) -> HierNode {
        HierNode{
            children : module.submodules.clone(),
            payload : ModuleWrapper::from(module),
            ..HierNode::default()
        }
    }


}

impl From<&ModuleItem> for ModuleWrapper {
    fn from(module: &ModuleItem) -> ModuleWrapper {
        ModuleWrapper { hier_idx: module.self_idx, name: module.name.clone() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HierNode {
    ui_state: cell::State<HierOptions>,
    expanded_button: button::State,
    expanded: Cell<bool>,
    payload: ModuleWrapper,
    children: Vec<usize>,
}

#[derive(Debug, Default)]
pub struct HierRoot{
    viz_module_list: Vec<HierNode>,
    root_nodes: Vec<usize>,
}

impl HierRoot {
    pub fn view (&mut self) -> Element<Message> {
        let viz_mod_slice = self.viz_module_list.as_mut_slice();
        //let elements = self.root_nodes
        //    .iter()
        //    .cloned()
        //    .map(|x| viz_mod_slice[x].view(viz_mod_slice))
        //    .collect();
        let mut elements : Vec<Element<Message>> = Vec::new();
        let mut itermut2 = viz_mod_slice.iter_mut();
        let mut starting = 0;
        for val in self.root_nodes.iter().cloned() {
            elements.push(itermut2.nth(val).unwrap().view(viz_mod_slice.iter_mut()));
        }

        //elements.extend(child_refs.iter())
        Column::with_children(elements).into()
    }
}

impl From<&HierMap> for HierRoot {
    fn from(map: &HierMap) -> HierRoot {
        let viz_module_list : Vec<HierNode> = map.module_list
            .iter()
            .map(|x| HierNode::from(x))
            .collect();
        HierRoot{
            viz_module_list,
            root_nodes : map.get_roots().into()
        }
    }
}



impl HierNode {
    pub fn view<'a> (&'a mut self, itermut :  IterMut<HierNode>) -> Element<'a, Message> {
        let HierNode { children, ui_state, expanded_button, expanded, payload } = self;

        let expanded_val = expanded.get();

        let expander =
            Button::new(expanded_button, Text::new(if expanded_val { "↓" } else { "←" }))
                .on_press(Message::Toggle(payload.hier_idx));

        //TODO: fixme, placeholder message closure
        let root_cell = VizCell::new(ui_state, payload, &HierOptions::ALL, |module| {
            Message::SendModule(module.hier_idx)
        });

        let top_row = if !children.is_empty() {
            Row::new().push(expander).push(root_cell).width(Length::Fill).height(Length::Shrink)
        } else {
            Row::new().push(root_cell).width(Length::Fill).height(Length::Shrink)
        };

        if expanded_val {
            let mut elements = vec![top_row.into()];
            //elements.extend(children.iter().cloned().
             //   map(|x| (trunk_state.viz_module_list[x]).view(trunk_state)));


            //elements.extend(child_refs.iter())
            Column::with_children(elements).into()
        } else {
            top_row.into()
        }
    }
}
