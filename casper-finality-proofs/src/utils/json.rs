use std::fs::File;
use anyhow::Error;
use serde_json::Value;
use std::io::Read;

pub fn read_json_from_file(path_to_file: &str) -> Result<Value, Error> {
    let mut file = File::open(path_to_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(serde_json::from_str(&contents)?)
}
