use crate::errors::Waverr;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use vcd::{IdCode, Scope, ScopeItem};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HierMap {
    pub module_list: Vec<ModuleItem>,
    top_indices: Vec<usize>,
}

impl HierMap {
    pub fn get_roots(&self) -> &[usize] {
        &self.top_indices[..]
    }

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
        Ok(idx)
    }

    pub fn set_path_relative<S: Into<String>>(
        &self,
        in_path: S,
        starting_idx: usize,
    ) -> Result<usize, Waverr> {
        let rel_path: String = in_path.into();
        let mut idx = starting_idx;
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
        if idx == starting_idx {
            Err(Waverr::HierMapError("Cannot find module in abs path"))
        } else {
            Ok(idx)
        }
    }

    /// Get the submodules of the "live" module. This is exposed to wave2 app
    /// for filling in the module navigator
    pub fn get_module_children(&self, live_module: usize) -> Vec<&ModuleItem> {
        self.module_list[live_module]
            .submodules
            .iter()
            .cloned()
            .map(|x| &self.module_list[x])
            .collect()
    }

    fn get_module_signals(&self, live_module: usize) -> &[SignalItem] {
        self.module_list[live_module].signals.as_slice()
    }

    /// Get the signals of the "live" module. This is exposed to wave2 app
    /// for filling in the signal navigator
    pub fn get_module_signals_vec(&self, live_module: usize) -> Vec<SignalItem> {
        self.module_list[live_module].signals.clone()
    }


    /// Map absolute path -> signal id
    /// This is to support the older API of an ID map, where raw paths can map directly
    /// to signal ids
    pub fn path_to_id(&self, abs_path: &str) -> Result<u32, Waverr> {
        if let Some(base_path_idx) = abs_path.rfind('.') {
            let module_idx = self.set_path_abs(&abs_path[..base_path_idx])?;
            let sig_name = &abs_path[base_path_idx + 1..];
            let rv = self
                .get_module_signals(module_idx)
                .iter()
                .find(|signal| signal.name() == sig_name)
                .map_or(
                    Err(Waverr::HierMapError("Malformed path passed in")),
                    |signal| Ok(signal.id()),
                );

            rv
        } else {
            Err(Waverr::HierMapError("Malformed path passed in"))
        }
    }

    pub fn idx_to_path(&self, in_idx: usize) -> String {
        let mut idx = in_idx;
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
                            map.len(),
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
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ModuleItem {
    pub name: String,
    pub submodules: Vec<usize>,
    pub signals: Vec<SignalItem>,
    pub self_idx: usize,
    pub parent: Option<usize>,
}

impl From<vcd::Var> for SignalItem {
    fn from(var: vcd::Var) -> SignalItem {
        SignalItem(var.reference, var.code.0 as u32)
    }
}

impl ModuleItem {
    fn new(name: String, parent: Option<usize>, self_idx: usize) -> Self {
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
    pub fn name(&self) -> &str {
        self.0.as_str()
    }
    pub fn id(&self) -> u32 {
        self.1
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

    fn vcd_test_path(path: &str) -> PathBuf {
        let mut path_to_vcd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_to_vcd.push(path);
        path_to_vcd
    }

    #[test]
    //first sanity test,simple vcd from wikipedia
    fn wikipedia_hier_map() {
        let pb = vcd_test_path("test_vcds/wikipedia.vcd");
        let mut wp = vcd_parser::WaveParser::new(pb).unwrap();

        let hm = wp.create_hiermap().unwrap();
        let live_module = hm.set_path_abs("logic").unwrap();
        assert_eq!(live_module, 0);
        let signals = hm.get_module_signals(live_module);
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

        let submodules = hm.get_module_children(live_module);
        assert!(submodules.is_empty());

        let path = hm.idx_to_path(live_module);
        assert_eq!(path, "logic");
    }

    #[test]
    fn vga_hier_map() {
        let pb = vcd_test_path("test_vcds/vga.vcd");
        let mut wp = vcd_parser::WaveParser::new(pb).unwrap();
        let hm = wp.create_hiermap().unwrap();
        let live_module = hm.set_path_abs("TOP").unwrap();
        let submodules = hm.get_module_children(live_module);
        assert!(!submodules.is_empty());

        let fail = hm.set_path_relative("does not exist", live_module);
        assert!(fail.is_err(), "Path exists!");

        let new_live_module = hm.set_path_relative("vga", live_module);
        assert!(new_live_module.is_ok(), "Path exists!");

        let num_children =
            hm.get_module_signals(new_live_module.unwrap()).len();
        assert_eq!(num_children, 30);
    }
}
