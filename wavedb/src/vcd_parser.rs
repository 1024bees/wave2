use crate::errors;
use crate::hier_map::HierMap;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use vcd::Parser;

pub struct WaveParser<R: io::Read> {
    vcd_parser: Parser<R>,
    header: Option<vcd::Header>,
}

impl WaveParser<io::BufReader<File>> {
    //TODO: move from option to waverr
    pub fn new(
        file_path: PathBuf,
    ) -> Result<WaveParser<io::BufReader<File>>, errors::Waverr> {
        if let Ok(f) = File::open(&file_path) {
            let mut rv = WaveParser {
                vcd_parser: Parser::new(io::BufReader::new(f)),
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
    fn populate_header(&mut self) {
        if let Ok(header) = self.vcd_parser.parse_header() {
            self.header = Some(header);
        }
    }

    pub fn create_hiermap(&mut self) -> Result<HierMap, errors::Waverr> {
        if let Some(header) = self.header.take() {
            Ok(HierMap::from(header))
        } else {
            Err(errors::Waverr::VCDErr("Header is not found from vcd!"))
        }
    }
}

impl<P: io::Read> Iterator for WaveParser<P> {
    type Item = Result<vcd::Command, io::Error>;
    fn next(&mut self) -> Option<Result<vcd::Command, io::Error>> {
        self.vcd_parser.next()
    }
}

mod tests {}
