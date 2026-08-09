use std::{env, fs, io::{self, Write}, process::Command};

use tempfile::NamedTempFile;

pub fn edit(initial: &str) -> io::Result<String> {
    let mut file = NamedTempFile::new()?;
    file.write_all(initial.as_bytes())?;
    file.flush()?;

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    let status = Command::new(&editor)
        .arg(file.path())
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Editor exited with an error",
        ));
    }

    let edited = fs::read_to_string(file.path())?;
    Ok(edited)
}
