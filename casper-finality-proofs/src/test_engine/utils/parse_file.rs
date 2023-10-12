use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[allow(dead_code)] // We allow dead_code since this is used in tests
pub fn read_fixture<T: for<'a> Deserialize<'a>>(filename: &str) -> T {
    let mut file = File::open(filename).unwrap();
    let mut context = String::new();
    file.read_to_string(&mut context).unwrap();

    let context: T = serde_json::from_str(context.as_str()).unwrap();
    context
}
