use crate::components::hier_nav::hier_nav::{HierOptions, Message};
use iced::{button, Button, Column, Element, Length, Row, Text};
use log::warn;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use wave2_custom_widgets::widget::cell;
use wave2_custom_widgets::widget::cell::Cell as VizCell;
use wave2_wavedb::hier_map::{HierMap, ModuleItem};



const PADDING : u16 = 1;

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
impl AsRef<str> for ModuleWrapper {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
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
    shared_state: SharedNodeState,
    payload: ModuleWrapper,
}

#[derive(Default, Debug, Clone)]
struct SharedNodeState {
    pub expanded: Rc<Cell<bool>>,
    pub selected: Rc<Cell<bool>>,
}

#[derive(Debug, Default)]
pub struct HierRoot {
    root_vec: Vec<HierNode>,
    flat_expander_map: HashMap<usize, SharedNodeState>,
}

impl HierRoot {
    pub fn update_expander(&mut self, module_idx: usize) {
        let exp = self.flat_expander_map.get(&module_idx);
        if let Some(real_expander) = exp {
            let state = real_expander.expanded.get();
            real_expander.expanded.set(!state);
        } else {
            warn!(
                "Trying to expand {}; this index should not have children",
                module_idx
            );
        }
    }

    pub fn toggle_selected(&mut self, module_idx: usize) {
        let sel = self.flat_expander_map.get(&module_idx);
        if let Some(real_selector) = sel {
            let state = real_selector.selected.get();
            real_selector.selected.set(!state);
        } else {
            warn!(
                "Trying to select {}; this index should not exist",
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
        let mut flat_expander_map: HashMap<usize, SharedNodeState> =
            HashMap::new();
        let rootlist: Vec<HierNode> = map
            .get_roots()
            .iter()
            .cloned()
            .map(|x| HierNode::from_hmap(x, map, &mut flat_expander_map))
            .collect();
        HierRoot {
            root_vec: rootlist,
            flat_expander_map,
        }
    }
}

impl HierNode {
    fn from_hmap(
        live_idx: usize,
        map: &HierMap,
        flat_expander_map: &mut HashMap<usize, SharedNodeState>,
    ) -> HierNode {
        let module = &map.module_list[live_idx];
        let payload = ModuleWrapper::from(module);
        let shared_state = SharedNodeState::default();
        let rv = if !module.submodules.is_empty() {
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
                shared_state,
                ..HierNode::default()
            }
        } else {
            HierNode {
                payload,
                shared_state,
                ..HierNode::default()
            }
        };
        flat_expander_map.insert(rv.payload.hier_idx, rv.shared_state.clone());
        rv
    }

    pub fn view(&mut self) -> Element<Message> {
        let HierNode {
            children,
            ui_state,
            expanded_button,
            shared_state,
            payload,
        } = self;

        let expanded_val = shared_state.expanded.get();

        let expander = Button::new(
            expanded_button,
            Text::new(if expanded_val { "-" } else { "+" }),
        )
        .padding(PADDING)
        .on_press(Message::Toggle(payload.hier_idx));

        //TODO: fixme, placeholder message closure
        let idx = payload.hier_idx.clone();
        let root_cell = VizCell::new(ui_state, self.payload.as_ref())
            .on_click(Box::new(move || Message::SendModule(idx)))
            .override_selected(shared_state.selected.get());

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
