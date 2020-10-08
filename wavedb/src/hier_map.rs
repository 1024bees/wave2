use vcd::{IdCode,ScopeItem, Scope};
use std::cell::Cell;
use crate::errors::Waverr;
use serde::{Deserialize, Serialize};

pub struct HierMap{ 
    module_list : Vec<ModuleItem>,
    top_indices : Vec<usize>,
    live_module : usize, 
} 


impl HierMap {
    

    fn get_starting_idx(&self, mod_name : &str) -> Result<usize,Waverr>{
        for idx in self.top_indices.iter().cloned() {
            if mod_name == self.module_list[idx].name {
                return Ok(idx);
            }
        }
        return Err(Waverr::HierMapError("Incorrect path; top level module is not in top_indices"))
    }


    fn set_path_from_base(&mut self, path: String) -> Result<usize, Waverr> {
        let module_list : Vec<&str> = path.split(".").collect();

        let mut idx = if let Some(top_module) = module_list.first() {
            self.get_starting_idx(*top_module)?
        } else {
            return Err(Waverr::HierMapError("Malformed path; no dotted references found"));
        };
        

        for mod_name in module_list[1..].iter() {
            let cm : &ModuleItem = &self.module_list[idx];
            for child in cm.submodules.iter().cloned() {
                if self.module_list[child].name == *mod_name {
                    idx = child;
                }
                continue;
            }
            return Err(Waverr::HierMapError("Cannot find module in abs path"));
        }
        self.live_module = idx;
        Ok(idx)
    }

    fn set_path_relative(&mut self, rel_path: String) -> Result<usize, Waverr> {
        let mut idx = self.live_module;
        let module_list : Vec<&str> = rel_path.split(".").collect();

        for mod_name in module_list[1..].iter() {
            let cm : &ModuleItem = &self.module_list[idx];
            for child in cm.submodules.iter().cloned() {
                if self.module_list[child].name == *mod_name {
                    idx = child;
                }
                continue;
            }
            return Err(Waverr::HierMapError("Cannot find module in abs path"));
        }
        self.live_module = idx;
        Ok(idx)

    }



}



impl From<vcd::Header> for HierMap {
    fn from(header: vcd::Header) -> HierMap {
        let mut HierMapVec : Vec<ModuleItem> = Vec::new();
        let mut TopMods : Vec<usize> = Vec::new();
        let mut parentmod : Option<usize>  = None;
        let mut livemod_ref : usize = 0;

        fn recurse_parse(
            map: &mut Vec<ModuleItem>,
            TopMods: &mut Vec<usize>,
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
                        if parent_mod.is_none() {
                            TopMods.push(livemod_ref);
                        }
                        recurse_parse(map, 
                            TopMods,
                            scope.children,
                            map.len() -1,
                            Some(livemod_ref)
                        )
                        
                    }
                }
            }
        }

        recurse_parse(&mut HierMapVec, &mut TopMods, header.items, livemod_ref, None);


        HierMap {
            module_list : HierMapVec,
            top_indices : TopMods,
            live_module : 0
        }

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
        let mut wp = vcd_parser::WaveParser::new(pb).unwrap();

        let hm = wp.create_hiermap().unwrap();

    }


}

