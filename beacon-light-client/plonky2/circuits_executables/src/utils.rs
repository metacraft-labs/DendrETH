use anyhow::Result;
use std::{collections::HashMap, fs::File, io::Read};

pub fn parse_config_file(filepath: String) -> Result<HashMap<String, String>> {
    let mut content = String::new();
    let mut file = File::open(filepath)?;
    file.read_to_string(&mut content)?;
    Ok(serde_json::from_str(&content.as_str())?)
}

pub fn gindex_from_validator_index(index: u64, depth: u32) -> u64 {
    return 2u64.pow(depth) - 1 + index;
}

pub fn format_hex(str: String) -> String {
    if str.starts_with("0x") {
        return str[2..].to_string();
    }

    return str;
}
