use nfd::Response;
use std::io;
use std::path::{Path, PathBuf};

//Taken almost verbatim from Tolstack. Thank you :-) 
pub async fn open() -> Result<PathBuf, io::Error> {
    let result: nfd::Response =
        match async { nfd::open_file_dialog(Some("vcd"), None) }.await {
            Ok(result) => result,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unable to unwrap data from new file dialog",
                ))
            }
        };

    let file_string: String = match result {
        Response::Okay(file_path) => file_path,
        Response::OkayMultiple(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Multiple files returned when one was expected",
            ))
        }
        Response::Cancel => {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "User cancelled file open",
            ))
        }
    };

    let mut result: PathBuf = PathBuf::new();
    result.push(Path::new(&file_string));

    if result.exists() {
        Ok(result)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "File does not exist",
        ))
    }
}
