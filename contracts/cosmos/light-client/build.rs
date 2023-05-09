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
    let path = format!("/{}/nimcache/light_client.json", option_env!("NIMCACHE_PARENT").unwrap_or("code").to_string());
    let mut file = File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let json: serde_json::Value =
    serde_json::from_str(&data).expect("JSON was not well-formatted");
    let links = json["link"].as_array();
    for link in links.unwrap() {
        let path_as_string = rem_first_and_last(link.to_string().as_str()).to_string();
        let path = Path::new(&path_as_string);
        let file_name = path.file_name().unwrap();
        println!("cargo:rustc-link-arg=/{}/nimcache/{}", option_env!("NIMCACHE_PARENT").unwrap_or("code").to_string(), (file_name.to_str().unwrap()));
    }
}
