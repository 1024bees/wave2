
use vcd::{IdCode,ScopeItem};
use std::cell::Cell;
use crate::errors::Waverr;

pub struct HierMap(Vec<ModuleItem>);


impl From<&vcd::Header> for HierMap {
    fn from(header: &vcd::Header) -> HierMap {


        let mut HierMapVec : Vec<ModuleItem> = Vec::new();
        let mut parentmod : Option<usize>  = None;
        let mut livemod_ref : usize = 0;

        fn recurse_parse(
            map: &mut Vec<ModuleItem>,
            scope: &vcd::Scope,
            parent_mod : Option<usize>
        ) {

        }









        for item in header.items.iter() {
            match item {
                ScopeItem::Var(variable) => {
                    HierMapVec[livemod_ref].add_sig(SignalItem::from(variable));
                }
                ScopeItem::Scope(scope) => {
                    if let Some(parent_idx)  = parentmod {
                        HierMapVec[parent_idx].add_
                    }
                    parentmod = Some(livemod_ref);
                    HierMapVec.push(ModuleItem::new(scope.identifier.clone(),parentmod));
                    livemod_ref = HierMapVec.len() - 1;
                    
                }
            }
        }

        HierMap(HierMapVec)

    }
}
#[derive(Default)]
pub struct ModuleItem {
    name : String,
    submodules : Vec<usize>,
    signals : Vec<SignalItem>,
    parent : Option<usize>
    
}


impl From<&vcd::Var> for SignalItem {
    fn from(var : &vcd::Var) -> SignalItem {
        SignalItem(var.reference, var.code)
    }
}

impl ModuleItem {
    fn new(name: String, parent : Option<usize> ) -> Self {
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

