use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read};
use std::ffi;

#[derive(Debug)]
pub enum ResourcesError {
    Io(io::Error),
    FileContainsNil,
    FailedToGetExePath,
}

pub struct Resources {
    root_path: PathBuf,
}

impl Resources {
    pub fn from_relative_exe_path(rel_path: &Path) -> Result<Resources, ResourcesError> {
        let exe_file_name = std::env::current_exe()
            .map_err(|_| ResourcesError::FailedToGetExePath)?;

        let exe_path = exe_file_name.parent()
            .ok_or(ResourcesError::FailedToGetExePath)?;

        Ok(Resources {
            root_path: exe_path.join(rel_path)
        })
    }
}