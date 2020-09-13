use vcd::{ReferenceIndex, Value, Var,Parser,ScopeItem, IdCode};
use std::io;
use std::fs::File;
use std::collections::HashMap;
use crate::backend::errors;



pub struct WaveParser<R: io::Read> {
    VCDParser : Parser<R>,
    filepath : String,
    header : Option<vcd::Header>
}




#[derive(Default)]
struct FlatMap(HashMap<String,IdCode>);

impl From<&vcd::Header> for FlatMap{
    fn from(header: &vcd::Header) -> FlatMap{

        fn recurse_parse(map :&mut HashMap<String,IdCode>, scope : &vcd::Scope, active_scope:  &mut String) {
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
                        recurse_parse(map,scope,active_scope);
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
                    map.insert("".into(),variable.code);
                },
                ScopeItem::Scope(scope) => {
                    recurse_parse(&mut map,scope,&mut scope_str);
                }

            }
        }
        FlatMap(map)
    }
}


impl WaveParser<io::BufReader<File>>{
//TODO: move from option to waverr
    pub fn new(file_path : String) -> Result<WaveParser<io::BufReader<File>>,errors::Waverr> { 
        if let Ok(f) = File::open(&file_path) {
            let mut rv = WaveParser {
                VCDParser : Parser::new(io::BufReader::new(f)),
                filepath : file_path,
                header: None
            };
            rv.populate_header();
            Ok(rv)
        } else {
            Err(errors::Waverr::VCDErr("Could not open VCD!"))
        }
    }

}


impl<R: io::Read> WaveParser<R> {
    
    fn new_test(raw_file : R) -> WaveParser<R> {
        let mut rv = WaveParser {
            VCDParser : Parser::new(raw_file),
            filepath : String::default(),
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

    pub fn create_flatmap(&self) -> FlatMap{
        if let Some(ref header) = self.header {
            FlatMap::from(header)
        } else {
            FlatMap::default()
        }
    }

}




impl<P: io::Read> Iterator for WaveParser<P> {
    type Item = Result<vcd::Command, io::Error>;
    fn next(&mut self) -> Option<Result<vcd::Command,io::Error>> {
        self.VCDParser.next()
    }
}


mod tests {
    use crate::backend::vcd_parser::*;

    #[test]
    fn wikipedia_sample() {
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
        let fm_map = parser.create_flatmap().0;
        let key_vec = vec![ "logic.data", "logic.data_valid", "logic.en", "logic.rx_en", "logic.tx_en", "logic.empty", "logic.underrun"];
        assert_eq!(fm_map.len(),7);
        for key in key_vec {
            assert!(fm_map.contains_key(key));
        }




    }
}

