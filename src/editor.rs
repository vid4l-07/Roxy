use std::{env, fs, io::{self, Write}, process::Command};

use tempfile::NamedTempFile;

pub fn edit(initial: &str) -> io::Result<String> {
    let mut file = NamedTempFile::new()?;
    file.write_all(initial.as_bytes())?;
    file.flush()?;

    let editor = env::var("EDITOR")
        .map_err(|_| io::Error::new(
                io::ErrorKind::NotFound,
                "Environment variable EDITOR not found",
        ))?;

    let status = Command::new(&editor)
        .arg(file.path())
        .status()
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to launch editor '{editor}': {e}"),
            )
        })?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Editor exited with an error",
        ));
    }

    let edited = fs::read_to_string(file.path())?;
    Ok(edited)
}

