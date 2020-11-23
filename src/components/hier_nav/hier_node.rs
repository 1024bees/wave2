use crate::components::hier_nav::hier_nav::{HierOptions, Message};
use iced::{
    button, scrollable, text_input, Align, Button, Column, Container, Element,
    Length, Row, Scrollable, Text, TextInput,
};
use log::{error, warn};
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
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

impl From<&ModuleItem> for ModuleWrapper {
    fn from(module: &ModuleItem) -> ModuleWrapper {
        ModuleWrapper {
            hier_idx: module.self_idx,
            name: module.name.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HierNode {
    children: Vec<HierNode>,
    ui_state: cell::State<HierOptions>,
    expanded_button: button::State,
    expanded: Rc<Cell<bool>>,
    payload: ModuleWrapper,
}

#[derive(Debug, Default)]
pub struct HierRoot {
    root_vec: Vec<HierNode>,
    flat_expander_map: HashMap<usize, Rc<Cell<bool>>>,
}

impl HierRoot {
    pub fn expand_module<S: Into<String>>(
        &self,
        in_path: S,
    ) -> Result<(), &'static str> {
        let path = in_path.into();
        let scope_list: Vec<&str> = path.split(".").collect();
        let mut hierarchy_ptr: &Vec<HierNode> = &self.root_vec;
        let mut mutator: Option<&HierNode> = None;
        for scope in scope_list {
            mutator = None;
            for node in hierarchy_ptr {
                if scope == node.payload.name {
                    hierarchy_ptr = &node.children;
                    mutator = Some(node);
                    break;
                }
            }
        }
        if let Some(expandee) = mutator {
            expandee.expanded.set(!expandee.expanded.get());
            Ok(())
        } else {
            Err("Trying to expand nonexistent path; TODO: refactor this error")
        }
    }
    pub fn update_expander(&mut self, module_idx: usize) {
        let exp = self.flat_expander_map.get(&module_idx);
        if let Some(real_expander) = exp {
            let state = real_expander.get();
            real_expander.set(!state);
        } else {
            warn!(
                "Trying to expand {}; this index should not have children",
                module_idx
            );
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let elements = self.root_vec.iter_mut().map(|x| x.view()).collect();

        //elements.extend(child_refs.iter())
        Column::with_children(elements).into()
    }
}

impl From<&HierMap> for HierRoot {
    fn from(map: &HierMap) -> HierRoot {
        let mut flat_expander_map: HashMap<usize, Rc<Cell<bool>>> =
            HashMap::new();
        let rootlist: Vec<HierNode> = map
            .get_roots()
            .iter()
            .cloned()
            .map(|x| HierNode::from_hmap(x, map, &mut flat_expander_map))
            .collect();
        HierRoot {
            root_vec: rootlist,
            flat_expander_map: flat_expander_map,
        }
    }
}

impl HierNode {
    fn from_hmap(
        live_idx: usize,
        map: &HierMap,
        flat_expander_map: &mut HashMap<usize, Rc<Cell<bool>>>,
    ) -> HierNode {
        let module = &map.module_list[live_idx];
        let payload = ModuleWrapper::from(module);
        if !module.submodules.is_empty() {
            let expanded = Rc::new(Cell::new(false));
            flat_expander_map.insert(payload.hier_idx, expanded.clone());
            HierNode {
                payload,
                // Look. I get it. It's ugly. You hate this
                // But this is what peak peformance looks like.
                // ... Right?
                // Unsure how expensive a move of a recursive DS like this is. I'd like to avoid it
                // if possible
                children: module
                    .submodules
                    .iter()
                    .cloned()
                    .map(|x| HierNode::from_hmap(x, map, flat_expander_map))
                    .collect(),
                expanded: expanded.clone(),
                ..HierNode::default()
            }
        } else {
            HierNode {
                payload,
                expanded: Rc::new(Cell::new(false)),
                ..HierNode::default()
            }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let HierNode {
            children,
            ui_state,
            expanded_button,
            expanded,
            payload,
        } = self;

        let expanded_val = expanded.get();

        let expander = Button::new(
            expanded_button,
            Text::new(if expanded_val { "↓" } else { "←" }),
        )
        .on_press(Message::Toggle(payload.hier_idx));

        //TODO: fixme, placeholder message closure
        let root_cell = VizCell::new(ui_state, payload, &HierOptions::ALL);
        //|module| {
        //    Message::SendModule(module.hier_idx)
        //});

        let top_row = if !children.is_empty() {
            Row::new()
                .push(expander)
                .push(root_cell)
                .width(Length::Fill)
                .height(Length::Shrink)
        } else {
            Row::new()
                .push(root_cell)
                .width(Length::Fill)
                .height(Length::Shrink)
        };

        if expanded_val {
            let mut elements = vec![top_row.into()];
            elements.extend(children.iter_mut().map(|x| x.view()));

            //elements.extend(child_refs.iter())
            Column::with_children(elements).into()
        } else {
            top_row.into()
        }
    }
}
