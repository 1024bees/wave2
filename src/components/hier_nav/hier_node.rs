use iced::{
    button, scrollable, text_input, Align, Button, Column, Container, Element,
    Length, Row, Scrollable, Text, TextInput,
};
use log::error;
use std::sync::{Arc, Mutex};
use wave2_custom_widgets::widget::cell;
use wave2_custom_widgets::widget::cell::Cell as VizCell;
use std::cell::Cell;
use crate::components::hier_nav::hier_nav::{Message,HierOptions};
use wave2_wavedb::hier_map::{HierMap, ModuleItem, SignalItem};

#[derive(Debug,Clone,Default)]
struct ModuleWrapper{
    hier_idx : usize,
    name : String,
}


impl ToString for ModuleWrapper {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl From<&ModuleItem> for ModuleWrapper {
    fn from(module: &ModuleItem) -> ModuleWrapper {
        ModuleWrapper {
            hier_idx : module.self_idx,
            name : module.name.clone()
        }
    }

}



#[derive(Debug,Clone,Default)]
pub struct HierNode {
    children: Vec<HierNode>,
    ui_state: cell::State<HierOptions>,
    expanded_button: button::State,
    expanded: Cell<bool>,
    payload: ModuleWrapper,
}




#[derive(Debug,Default)]
pub struct HierRoot (Vec<HierNode>);

impl HierRoot {
    pub fn expand_module<S: Into<String>>(&self, in_path : S) -> Result<(), &'static str>{
        let path = in_path.into();
        let scope_list: Vec<&str> = path.split(".").collect();
        let mut hierarchy_ptr : &Vec<HierNode> = &self.0;
        let mut mutator : Option<&HierNode> = None;
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
    pub fn view(&mut self) -> Element<Message> {
        let elements = self.0.
            iter_mut()
            .map(|x| x.view())
            .collect();

        //elements.extend(child_refs.iter())
        Column::with_children(elements).into()
    }


}

impl From<&HierMap> for HierRoot {
    fn from(map: &HierMap) -> HierRoot {
        let rootlist : Vec<HierNode> = map.get_roots().iter()
            .cloned()
            .map(|x| HierNode::from_hmap(x,map))
            .collect();
        HierRoot(rootlist)
    }
}



impl HierNode {
    fn from_hmap(live_idx: usize, map : &HierMap) -> HierNode {
        let module = &map.module_list[live_idx];
        let payload = ModuleWrapper::from(module);
        if !module.submodules.is_empty() {
            HierNode {
                payload,
                // Look. I get it. It's ugly. You hate this
                // But this is what peak peformance looks like.
                // ... Right?
                // Unsure how expensive a move of a recursive DS like this is. I'd like to avoid it
                // if possible
                children : module.submodules.iter().cloned()
                            .map(|x| { HierNode::from_hmap(x,map ) })
                            .collect(),
                ..HierNode::default()
            }
        } else {
            HierNode {
                payload,
                ..HierNode::default()
            }

        }
        

    }
    
    pub fn view(&mut self) -> Element<Message>
    {
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
            Text::new(if expanded_val{ "↓" } else { "←" }),
        ).on_press(Message::Toggle(0));




        //TODO: fixme, placeholder message closure
        let root_cell =
            VizCell::new(ui_state, payload, &HierOptions::ALL, |module| Message::SendModule(module.hier_idx));

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

