use std::{
    error::Error,
    fs::{create_dir_all, read_dir, File},
    io::{BufWriter, Write},
};

const PATH: &str = "./results/";

pub unsafe fn json_write(data: serde_json::Value) -> Result<(), Box<dyn Error>> {
    let mut count = 0;

    create_dir_all(PATH).unwrap();

    for entry in read_dir(PATH).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            count += 1;
        }
    }

    let file = File::create(PATH.to_owned() + format!("data-{}.json", count).as_str())?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &data)?;
    writer.flush()?;

    Ok(())
}
