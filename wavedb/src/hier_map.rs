use crate::errors::Waverr;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use vcd::{IdCode, Scope, ScopeItem};

#[derive(Debug)]
pub struct HierMap {
    module_list: Vec<ModuleItem>,
    top_indices: Vec<usize>,
    live_module: Cell<usize>,
}

impl HierMap {
    fn get_starting_idx(&self, mod_name: &str) -> Result<usize, Waverr> {
        for idx in self.top_indices.iter().cloned() {
            if mod_name == self.module_list[idx].name {
                return Ok(idx);
            }
        }
        return Err(Waverr::HierMapError(
            "Incorrect path; top level module is not in top_indices",
        ));
    }

    pub fn set_path_abs<S: Into<String>>(
        &self,
        in_path: S,
    ) -> Result<usize, Waverr> {
        let path = in_path.into();
        let module_list: Vec<&str> = path.split(".").collect();

        let mut idx = if let Some(top_module) = module_list.first() {
            self.get_starting_idx(*top_module)?
        } else {
            return Err(Waverr::HierMapError(
                "Malformed path; no dotted references found",
            ));
        };

        for mod_name in module_list[1..].iter() {
            let cm: &ModuleItem = &self.module_list[idx];
            for child in cm.submodules.iter().cloned() {
                if self.module_list[child].name == *mod_name {
                    idx = child;
                }
                continue;
            }
            return Err(Waverr::HierMapError("Cannot find module in abs path"));
        }
        self.live_module.set(idx);
        Ok(idx)
    }

    pub fn set_path_relative<S: Into<String>>(
        &self,
        in_path: S,
    ) -> Result<usize, Waverr> {
        let rel_path: String = in_path.into();
        let mut idx = self.live_module.get();
        let module_list: Vec<&str> = rel_path.split(".").collect();

        for mod_name in module_list.iter() {
            let cm: &ModuleItem = &self.module_list[idx];
            for child in cm.submodules.iter().cloned() {
                if self.module_list[child].name == *mod_name {
                    idx = child;
                }
                continue;
            }
        }
        if idx == self.live_module.get() {
            Err(Waverr::HierMapError("Cannot find module in abs path"))
        } else {
            self.live_module.set(idx);
            Ok(idx)
        }
    }

    /// Get the submodules of the "live" module. This is exposed to wave2 app
    /// for filling in the module navigator
    pub fn get_module_children(&self) -> Vec<&ModuleItem> {
        self.module_list[self.live_module.get()]
            .submodules
            .iter()
            .cloned()
            .map(|x| &self.module_list[x])
            .collect()
    }

    /// Get the signals of the "live" module. This is exposed to wave2 app
    /// for filling in the signal navigator
    pub fn get_module_signals(&self) -> &[SignalItem] {
        self.module_list[self.live_module.get()].signals.as_slice()
    }

    /// Return string of "current path"
    pub fn get_current_path(&self) -> String {
        let mut idx = self.live_module.get();
        let mut path = self.module_list[idx].name.clone();
        loop {
            if let Some(pidx) = self.module_list[idx].parent {
                path = format!(
                    "{}.{}",
                    self.module_list[pidx].name.as_str(),
                    path
                );
                idx = pidx;
            } else {
                break;
            }
        }
        path
    }
}

impl From<vcd::Header> for HierMap {
    fn from(header: vcd::Header) -> HierMap {
        let mut HierMapVec: Vec<ModuleItem> = Vec::new();
        let mut TopMods: Vec<usize> = Vec::new();
        let mut parentmod: Option<usize> = None;
        let mut livemod_ref: usize = 0;

        fn recurse_parse(
            map: &mut Vec<ModuleItem>,
            TopMods: &mut Vec<usize>,
            items: Vec<ScopeItem>,
            livemod_ref: usize,
            parent_mod: Option<usize>,
        ) {
            for item in items.into_iter() {
                match item {
                    ScopeItem::Var(variable) => {
                        debug_assert_ne!(
                            None, parent_mod,
                            "Scopeless variables are forbidden"
                        );
                        map[livemod_ref].add_sig(SignalItem::from(variable));
                    }
                    ScopeItem::Scope(scope) => {
                        map.push(ModuleItem::new(
                            scope.identifier.clone(),
                            parent_mod,
                        ));
                        let new_idx = map.len() - 1;

                        if parent_mod.is_none() {
                            TopMods.push(new_idx);
                        } else {
                            map[livemod_ref].add_child(new_idx);
                        }

                        recurse_parse(
                            map,
                            TopMods,
                            scope.children,
                            map.len() - 1,
                            Some(livemod_ref),
                        )
                    }
                }
            }
        }

        recurse_parse(
            &mut HierMapVec,
            &mut TopMods,
            header.items,
            livemod_ref,
            None,
        );

        HierMap {
            module_list: HierMapVec,
            top_indices: TopMods,
            live_module: Cell::default(),
        }
    }
}
#[derive(Default, Debug)]
pub struct ModuleItem {
    name: String,
    submodules: Vec<usize>,
    signals: Vec<SignalItem>,
    parent: Option<usize>,
}

impl From<vcd::Var> for SignalItem {
    fn from(var: vcd::Var) -> SignalItem {
        SignalItem(var.reference, var.code.0 as u32)
    }
}

impl ModuleItem {
    fn new(name: String, parent: Option<usize>) -> Self {
        ModuleItem {
            name,
            parent,
            ..ModuleItem::default()
        }
    }
    fn add_sig(&mut self, sig_item: SignalItem) {
        self.signals.push(sig_item);
    }

    fn add_child(&mut self, child_idx: usize) {
        self.submodules.push(child_idx);
    }
}
//TODO: move to &str if possible
//

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct SignalItem(String, u32);

impl SignalItem {
    fn name(&self) -> &str {
        self.0.as_str()
    }
}

impl ToString for SignalItem {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

mod tests {
    use crate::*;
    use std::collections::HashSet;
    use std::fs::*;
    use std::io::*;
    use std::path::*;

    macro_rules! set {
        ( $( $x:expr ),* ) => {  // Match zero or more comma delimited items
            {
                let mut temp_set = HashSet::new();  // Create a mutable HashSet
                $(
                    temp_set.insert($x); // Insert each item matched into the HashSet
                )*
                temp_set // Return the populated HashSet
            }
        };
    }

    fn vcd_test_path(path: &str) -> String {
        let mut path_to_vcd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_to_vcd.push(path);
        path_to_vcd.into_os_string().into_string().unwrap()
    }

    #[test]
    //first sanity test,simple vcd from wikipedia
    fn wikipedia_hier_map() {
        let pb = vcd_test_path("test_vcds/wikipedia.vcd");
        let mut wp = vcd_parser::WaveParser::new(pb).unwrap();

        let hm = wp.create_hiermap().unwrap();
        let offset = hm.set_path_abs("logic").unwrap();
        assert_eq!(offset, 0);
        let signals = hm.get_module_signals();
        let ref_set = set![
            "logic",
            "data",
            "data_valid",
            "en",
            "rx_en",
            "tx_en",
            "empty",
            "underrun"
        ];

        for signal in signals {
            assert!(ref_set.contains(signal.name()))
        }

        let submodules = hm.get_module_children();
        assert!(submodules.is_empty());

        let path = hm.get_current_path();
        assert_eq!(path, "logic");
    }

    #[test]
    fn vga_hier_map() {
        let pb = vcd_test_path("test_vcds/vga.vcd");
        let mut wp = vcd_parser::WaveParser::new(pb).unwrap();
        let hm = wp.create_hiermap().unwrap();
        hm.set_path_abs("TOP").unwrap();
        let submodules = hm.get_module_children();
        assert!(!submodules.is_empty());

        let fail = hm.set_path_relative("does not exist");
        assert!(fail.is_err(), "Path exists!");

        let success = hm.set_path_relative("vga");
        assert!(success.is_ok(), "Path exists!");
        let num_children = hm.get_module_signals().len();
        assert_eq!(num_children, 30);
    }
}
