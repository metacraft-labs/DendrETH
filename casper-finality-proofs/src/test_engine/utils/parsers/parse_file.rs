use serde::Deserialize;
use std::fs::File;
use std::io::Read;

pub fn read_fixture<T: for<'a> Deserialize<'a>>(path: &str) -> T {
    let mut context = String::new();

    let mut file = File::open(path).unwrap();
    file.read_to_string(&mut context).unwrap();

    match serde_json::from_str(&context.as_str()) {
        Ok(context) => return context,
        Err(_) => return serde_yaml::from_str(&context).unwrap(),
    }
}
