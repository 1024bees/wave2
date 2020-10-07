
use vcd::{IdCode,ScopeItem, Scope};
use std::cell::Cell;
use crate::errors::Waverr;
use serde::{Deserialize, Serialize};

pub struct HierMap(Vec<ModuleItem>);


impl From<&vcd::Header> for HierMap {
    fn from(header: &vcd::Header) -> HierMap {


        let mut HierMapVec : Vec<ModuleItem> = Vec::new();
        let mut parentmod : Option<usize>  = None;
        let mut livemod_ref : usize = 0;

        fn recurse_parse(
            map: &mut Vec<ModuleItem>,
            items: Vec<ScopeItem>,
            livemod_ref : usize,
            parent_mod : Option<usize>
        ) {
            for item in items.into_iter() {
                match item {
                    ScopeItem::Var(variable) => {
                        map[livemod_ref].add_sig(SignalItem::from(variable));
                    }
                    ScopeItem::Scope(scope) => {
                        if let Some(parent_idx)  = parent_mod {
                            map[parent_idx].add_child(livemod_ref)
                        }
                        map.push(ModuleItem::new(scope.identifier.clone(),parent_mod));
                        recurse_parse(map, 
                            scope.children,
                            map.len() -1,
                            Some(livemod_ref)
                        )
                        
                    }
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


impl From<vcd::Var> for SignalItem {
    fn from(var : vcd::Var) -> SignalItem {
        SignalItem(var.reference, var.code.0 as u32)
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

    fn add_child(&mut self, child_idx : usize){
        self.submodules.push(child_idx);
    }

}
//TODO: move to &str if possible
//

#[derive (Deserialize,Serialize)]
pub struct SignalItem(String,u32);


mod tests {
    use crate::*;
    use std::path::*;
    use std::fs::*;
    use std::io::*;

    fn vcd_test_path(path : &str ) -> String {
        let mut path_to_vcd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_to_vcd.push(path);
        path_to_vcd.into_os_string().into_string().unwrap()
    }



    #[test]
    fn wikipedia_hier_map() {
        let pb = vcd_test_path("test_vcds/wikipedia.vcd");
        let wp = vcd_parser::WaveParser::new(pb).unwrap();

        let hm = wp.create_hiermap().unwrap();


    }


}

