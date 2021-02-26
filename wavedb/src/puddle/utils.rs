use vcd::Command;
use crate::errors::Waverr;

pub fn get_id(command : &Command) -> Result<u32,Waverr> {
    match command {
        Command::ChangeScalar(id,.. ) | Command::ChangeVector(id,..) | Command::ChangeReal(id,..) | Command::ChangeString(id,..) => {
            Ok(id.0 as u32)
        },
        _ => Err(Waverr::VcdCommandErr(command.clone()))
    }
}
