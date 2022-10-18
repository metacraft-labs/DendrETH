use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde_json;

fn rem_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}

fn main() {
    let mut file = File::open("/code/nimbuild/light_client.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let json: serde_json::Value =
    serde_json::from_str(&data).expect("JSON was not well-formatted");
    let links = json["link"].as_array();
    for link in links.unwrap() {
        let pathString = rem_first_and_last(link.to_string().as_str()).to_string();
        let path = Path::new(&pathString);
        let fileName = path.file_name().unwrap();
        println!("cargo:rustc-link-arg=/code/nimbuild/{}", (fileName.to_str().unwrap()));
    }
}
