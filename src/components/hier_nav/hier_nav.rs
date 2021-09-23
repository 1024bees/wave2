use crate::components::hier_nav::hier_node::HierRoot;
use iced::{scrollable, Command, Container, Element, Scrollable};
use log::info;
use std::sync::Arc;
use strum_macros;
use wave2_custom_widgets::traits::CellOption;
use wave2_wavedb::hier_map::HierMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display)]
pub enum HierOptions {
    Expand,
}

impl CellOption for HierOptions {
    type Message = Message;

    fn all() -> &'static [Self] {
        &HierOptions::ALL
    }

    fn to_message(&self) -> Self::Message {
        match self {
            HierOptions::Expand => Message::ExpandLive,
        }
    }
}

impl HierOptions {
    //TODO: create ALL macro
    const ALL: [HierOptions; 1] = [HierOptions::Expand];
}

#[derive(Default)]
pub struct HierNav {
    live_module: Option<usize>,
    scroll_x: scrollable::State,
    hier_root: HierRoot,
}

#[derive(Debug, Clone)]
pub enum Message {
    ///Message that is sent when initializing wave2; sets the hierarchy state
    SetHier(Arc<HierMap>),
    ///Toggles if a module's hierarchy is expanded or not
    Toggle(usize),
    /// Toggles if a module is "selected" or not.
    ///
    /// This messsage is stateful. If the module index wrapped by this message does
    /// NOT equal HierNav.live_module, then we send this module's signals to the
    /// ModuleNav pane, which will display its signals. This happens when an end user selects
    /// a new module to inspect
    ///
    /// If the module index wrapped by this message equals HierNav.live_module, we clear out the
    /// ModuleNav pane. This happens when a user toggles an already selected module
    SendModule(usize),
    /// Attempts to toggle the current seleted module. If it the module does not contain a
    /// submodule, we silently do nothing
    ExpandLive,
}

impl HierNav {
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SetHier(payload) => {
                self.hier_root = HierRoot::from(payload.as_ref());
            }
            Message::Toggle(module_idx) => {
                self.hier_root.update_expander(module_idx);
            }
            Message::SendModule(module_idx) => {
                let old_mod = self.live_module;
                self.live_module = if let Some(index) = self.live_module {
                    if index == module_idx {
                        None
                    } else {
                        Some(module_idx)
                    }
                } else {
                    Some(module_idx)
                };
                if let Some(old_val) = old_mod {
                    // toggle the old value off, if it exists
                    self.hier_root.toggle_selected(old_val);
                }
                if self.live_module.is_some() {
                    self.hier_root.toggle_selected(module_idx);
                }
            }
            Message::ExpandLive => {
                if let Some(live_module_idx) = self.live_module.as_ref() {
                    self.hier_root.update_expander(live_module_idx.clone())
                } else {
                    info!("Trying to expand when no module is live")
                }
            }
        }
        Command::none()
    }
    pub fn view(&mut self) -> Element<Message> {
        let HierNav {
            scroll_x,
            hier_root,
            ..
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
