use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read};
use std::ffi;

use image::{
    error::ImageResult,
    DynamicImage,
};


#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    FileContainsNil,
    FailedToGetExePath,
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::Io(other)
    }
}
pub struct Resources {
    root_path: PathBuf,
}

impl Resources {
    /// Creates a path from resources folder
    pub fn from_relative_path(rel_path: &Path) -> Result<Resources, Error> {
        let exe_file_name = std::env::current_exe()
            .map_err(|_| Error::FailedToGetExePath)?;

        let exe_path = exe_file_name.parent()
            .ok_or(Error::FailedToGetExePath)?;

        Ok(Resources {
            root_path: exe_path.join(rel_path)
        })
    }

    pub fn to_abs_path(&self, rel_path: &str) -> PathBuf {
        resource_name_to_path(&self.root_path, rel_path)
    }

    pub fn load_buffer(&self, resource_name: &str) -> Result<Vec<u8>, Error> {
        let mut file = fs::File::open(
            resource_name_to_path(&self.root_path, resource_name)
        )?;

        // allocate buffer of the same size as file
        let mut buffer: Vec<u8> = Vec::with_capacity(
            file.metadata()?.len() as usize + 1
        );
        file.read_to_end(&mut buffer)?;

        Ok(buffer)
    }

    pub fn load_cstring(&self, resource_name: &str) -> Result<ffi::CString, Error> {
        let buffer = self.load_buffer(resource_name)?;

        // check for nul byte
        if buffer.iter().find(|i| **i == 0).is_some() {
            return Err(Error::FileContainsNil);
        }

        Ok(unsafe { ffi::CString::from_vec_unchecked(buffer) })
    }

    #[allow(dead_code)]
    pub fn load_image(&self, resource_name: &str) -> ImageResult<DynamicImage> {
        // TODO: validate extension name to be supported format

        image::open(
            resource_name_to_path(&self.root_path, resource_name)
        )
    }
}

/// converts relative resource names to absolute paths
fn resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    for part in location.split("/") {
        path = path.join(part);
    }

    path
}