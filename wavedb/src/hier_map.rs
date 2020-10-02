
use vcd::{IdCode,ScopeItem};
use std::cell::Cell;
use crate::errors::Waverr;

pub struct HierMap<'a>(Vec<ModuleItem<'a>>);

type ModRef<'a> = Option<&'a mut ModuleItem<'a>>;

impl<'a> From<&vcd::Header> for HierMap<'a> {
    fn from(header: &vcd::Header) -> HierMap<'a> {


        let mut HierMapVec = Vec::new();
        let mut parentmod : ModRef<'a> = None;
        let mut livemod_ref : ModRef<'a> = None;
        for item in header.items.iter() {
            match item {
                ScopeItem::Var(variable) => {
                    livemod_ref.unwrap().add_sig(SignalItem::from(variable));
                }
                ScopeItem::Scope(scope) => {
                    parentmod = livemod_ref;
                    HierMapVec.push(ModuleItem::new(scope.identifier.clone(),parentmod));
                    livemod_ref = HierMapVec.last_mut();

                }
            }
        }

        HierMap(HierMapVec)

    }
}
#[derive(Default)]
pub struct ModuleItem<'a> {
    name : String,
    submodules : Vec<&'a ModuleItem<'a>>,
    signals : Vec<SignalItem>,
    parent : Option<&'a ModuleItem<'a>>
    
}


impl From<&vcd::Var> for SignalItem {
    fn from(var : &vcd::Var) -> SignalItem {
        SignalItem(var.reference, var.code)
    }
}

impl<'a> ModuleItem<'a> {
    fn new(name: String, parent : ModRef<'a> ) -> Self {
        ModuleItem {
            name,
            parent,
            ..ModuleItem::default()
        }
    }
    fn add_sig(&mut self, sig_item : SignalItem) {
        self.signals.push(sig_item);
    }

}
//TODO: move to &str if possible
pub struct SignalItem(String,IdCode);

