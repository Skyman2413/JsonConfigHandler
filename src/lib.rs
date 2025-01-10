pub mod json;

use crate::json::{JsonError, JsonValue};
use json::Json;
use std::fs::File;
use std::io;
use std::io::{Read, Write};

pub struct Config(Json);
impl Config {
    pub fn load_from_file(path: &str) -> io::Result<Config> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Config(Json::from(contents)))
    }

    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let mut file = File::create(path)?;
        let json_string = format!("{}", self.0);
        file.write_all(json_string.as_bytes())?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<JsonValue, JsonError> {
        self.0.get(key)
    }

    pub fn set(&mut self, key: &str, value: JsonValue) -> Result<(), JsonError> {
        self.0.set(key, value)
    }
}
