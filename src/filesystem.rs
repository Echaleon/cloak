use std::{fs, path::Path};

use anyhow::{Context, Result};
use clap::ValueEnum;

// Enum of types of objects to hide
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ObjectType {
    File,
    Folder,
    Symlink,
    Unknown,
}

// --- public functions --- //

// Returns true if the path matches one of the given types.
pub fn matches_type(path: &Path, types: &[ObjectType]) -> Result<bool> {
    // Get the type of the object at the path
    let object_type = object_type(path)?;

    // Check if the object type matches one of the given types
    Ok(types.iter().any(|t| *t == object_type))
}

// Windows only function to hide a file or folder
#[cfg(target_family = "windows")]
pub fn hide(path: &Path) -> Result<()> {
    use std::{
        io::Error,
        os::windows::{ffi::OsStrExt, fs::MetadataExt},
    };

    use winapi::{
        shared::minwindef::FALSE,
        um::{fileapi::SetFileAttributesW, winnt::FILE_ATTRIBUTE_HIDDEN},
    };

    // Get the current file attributes
    let attributes = fs::metadata(path)
        .with_context(|| format!("Failed to get file attributes for {}", path.display()))?
        .file_attributes();

    // Convert the path to a wide string for the Windows API
    let wide_path = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();

    // Check if the file is already hidden. Otherwise, hide it.
    if attributes & FILE_ATTRIBUTE_HIDDEN == FILE_ATTRIBUTE_HIDDEN {
        Ok(())
    } else {
        let result =
            unsafe { SetFileAttributesW(wide_path.as_ptr(), attributes | FILE_ATTRIBUTE_HIDDEN) };
        if result == FALSE {
            Err::<(), anyhow::Error>(Error::last_os_error().into())
                .with_context(|| format!("Failed to hide path {}", path.display()))
        } else {
            Ok(())
        }
    }
}

// Unix only function to hide a file or folder. Just prepends a dot to the file name.
#[cfg(target_family = "unix")]
pub fn hide(path: &Path) -> Result<()> {
    // Get the file name from the path
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("Failed to get file name from path {}", path.display()))?;

    // Change the file name to a string
    let file_name = file_name.to_str().ok_or_else(|| {
        anyhow!(
            "Failed to convert file name to string from path {}",
            path.display()
        )
    })?;

    // Check if the file is already hidden. Otherwise, hide it.
    if file_name.starts_with('.') {
        OK(())
    } else {
        // Get the parent directory
        let parent = path.parent().with_context(|| {
            format!("Failed to get parent directory of path {}", path.display())
        })?;

        // Get the new file name
        let new_file_name = format!(".{}", file_name);

        // Rename the file
        fs::rename(path, parent.join(new_file_name))
            .with_context(|| format!("Failed to rename path {}", path.display()))?;

        Ok(())
    }
}

// --- private functions --- //

// Returns the type of object at a path.
fn object_type(path: &Path) -> Result<ObjectType> {
    // Get the metadata for the path
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to get metadata for path {}", path.display()))?;

    // Check if the path is a file
    if metadata.is_file() {
        Ok(ObjectType::File)
        // Check if the path is a directory
    } else if metadata.is_dir() {
        Ok(ObjectType::Folder)
        // Check if the path is a symbolic link
    } else if metadata.is_symlink() {
        Ok(ObjectType::Symlink)
        // Otherwise, return an error
    } else {
        Ok(ObjectType::Unknown)
    }
}
