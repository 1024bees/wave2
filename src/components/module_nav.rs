

pub struct ModNavigator {
    module_path : String,
    module_members : Vec<String>,
    
}

#[derive(Debug,Clone)]
pub enum Message {
    ChangePath(String),
}


impl ModN
