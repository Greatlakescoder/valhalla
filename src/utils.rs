use std::{fs::File, io::BufWriter, path::Path};

use serde::Serialize;

pub fn write_to_json<T: Serialize, P: AsRef<Path>>(data: &T, path: P) -> std::io::Result<()> {
    // Create file and wrap in buffered writer
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    // Serialize and write data
    serde_json::to_writer_pretty(writer, data)?;

    Ok(())
}
