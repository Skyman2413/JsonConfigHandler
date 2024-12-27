pub mod json;

use std::fs::File;
use std::io::Read;
use json::Json;

pub struct Config {
    data: Json,
}

impl Config {
    pub fn load_from_file(path: &str) -> Config {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let data = Json::from(contents);
        Config { data }
    }
}
