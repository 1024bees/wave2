use crate::backend::errors;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use vcd::{IdCode, Parser, ReferenceIndex, ScopeItem, Value, Var};

pub struct WaveParser<R: io::Read> {
    VCDParser: Parser<R>,
    filepath: String,
    header: Option<vcd::Header>,
}

#[derive(Default)]
pub struct IDMap(HashMap<String, IdCode>);

impl Serialize for IDMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            let id: u32 = v.0 as u32;
            map.serialize_entry(k.as_str(), &id)?;
        }
        map.end()
    }
}

pub struct IDMVisitor {}

impl<'de> serde::de::Visitor<'de> for IDMVisitor {
    type Value = IDMap;

    fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        write!(formatter, "a map from strings to uint32s")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        let mut map: HashMap<String, IdCode> =
            HashMap::with_capacity(access.size_hint().unwrap_or(0));
        while let Some((key, value)) = access.next_entry()? {
            let annotater: &str = key;
            let val_cpy: u32 = value;
            map.insert(annotater.to_string(), IdCode(value as u64));
        }
        Ok(IDMap(map))
    }
}

impl<'de> Deserialize<'de> for IDMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(IDMVisitor {})
    }
}

impl IDMap {
    pub fn signal_to_id(&self, signal: &str) -> Result<u32, errors::Waverr> {
        match self.0.get(signal) {
            Some(id_code) => Ok(id_code.0 as u32),
            None => Err(errors::Waverr::GenericErr("No signal exists".into())),
        }
    }
}

impl From<&vcd::Header> for IDMap {
    fn from(header: &vcd::Header) -> IDMap {
        fn recurse_parse(
            map: &mut HashMap<String, IdCode>,
            scope: &vcd::Scope,
            active_scope: &mut String,
        ) {
            let bl = active_scope.len();
            active_scope.push_str(scope.identifier.as_ref());
            active_scope.push('.');
            let ol = active_scope.len();
            for child in scope.children.iter() {
                match child {
                    ScopeItem::Var(variable) => {
                        active_scope.push_str(variable.reference.as_ref());
                        map.insert(active_scope.clone(), variable.code);
                        active_scope.truncate(ol);
                    }
                    ScopeItem::Scope(scope) => {
                        recurse_parse(map, scope, active_scope);
                    }
                }
            }
            active_scope.truncate(bl);
        }

        let mut map = HashMap::new();
        let mut scope_str = String::default();
        for item in header.items.iter() {
            match item {
                ScopeItem::Var(variable) => {
                    map.insert("".into(), variable.code);
                }
                ScopeItem::Scope(scope) => {
                    recurse_parse(&mut map, scope, &mut scope_str);
                }
            }
        }
        IDMap(map)
    }
}

impl WaveParser<io::BufReader<File>> {
    //TODO: move from option to waverr
    pub fn new(
        file_path: String,
    ) -> Result<WaveParser<io::BufReader<File>>, errors::Waverr> {
        if let Ok(f) = File::open(&file_path) {
            let mut rv = WaveParser {
                VCDParser: Parser::new(io::BufReader::new(f)),
                filepath: file_path,
                header: None,
            };
            rv.populate_header();
            Ok(rv)
        } else {
            Err(errors::Waverr::VCDErr("Could not open VCD!"))
        }
    }
}

impl<R: io::Read> WaveParser<R> {
    fn new_test(raw_file: R) -> WaveParser<R> {
        let mut rv = WaveParser {
            VCDParser: Parser::new(raw_file),
            filepath: String::default(),
            header: None,
        };
        rv.populate_header();
        rv
    }

    fn populate_header(&mut self) {
        if let Ok(header) = self.VCDParser.parse_header() {
            self.header = Some(header);
        }
    }

    pub fn create_idmap(&self) -> IDMap {
        if let Some(ref header) = self.header {
            IDMap::from(header)
        } else {
            IDMap::default()
        }
    }
}

impl<P: io::Read> Iterator for WaveParser<P> {
    type Item = Result<vcd::Command, io::Error>;
    fn next(&mut self) -> Option<Result<vcd::Command, io::Error>> {
        self.VCDParser.next()
    }
}

mod tests {
    use crate::backend::vcd_parser::*;
    use bincode;

    #[test]
    fn wikipedia_sample_idmap() {
        let sample = b"
        $date
        Date text.
        $end
        $version
        VCD generator text.
        $end
        $comment
        Any comment text.
        $end
        $timescale 100 ns $end
        $scope module logic $end
        $var wire 8 # data $end
        $var wire 1 $ data_valid $end
        $var wire 1 % en $end
        $var wire 1 & rx_en $end
        $var wire 1 ' tx_en $end
        $var wire 1 ( empty $end
        $var wire 1 ) underrun $end
        $upscope $end
        $enddefinitions $end
        $dumpvars
        bxxxxxxxx #
        x$
        0%
        x&
        x'
        1(
        0)
        $end
        #0
        b10000001 #
        0$
        1%
        #2211
        0'
        #2296
        b0 #
        1$
        #2302
        0$
        #2303
            ";

        let mut parser = WaveParser::new_test(&sample[..]);
        let idmap = parser.create_idmap();
        let fm_map = &idmap.0;
        let key_vec = vec![
            "logic.data",
            "logic.data_valid",
            "logic.en",
            "logic.rx_en",
            "logic.tx_en",
            "logic.empty",
            "logic.underrun",
        ];
        assert_eq!(fm_map.len(), 7);
        for key in key_vec {
            assert!(fm_map.contains_key(key));
        }
        let idmap_clone: IDMap =
            bincode::deserialize(&bincode::serialize(&idmap).unwrap()[..])
                .unwrap();
    }
}
