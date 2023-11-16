use std::{
    error::Error,
    fs::{create_dir_all, read_dir, File},
    io::{BufWriter, Write},
};

const BASE_PATH: &str = "./results";

pub fn json_write(circuit_name: String, data: serde_json::Value) -> Result<(), Box<dyn Error>> {
    let mut count = 0;

    let dir = &format!("{}/{}/", BASE_PATH, circuit_name);
    create_dir_all(dir).unwrap();

    for entry in read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            count += 1;
        }
    }

    let file = File::create(dir.to_owned() + format!("data-{}.json", count).as_str())?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &data)?;
    writer.flush()?;

    Ok(())
}
