use std::{
    fs,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use error_kit::CommonError;
use serde::{Deserialize, Serialize};

use crate::traits::ResourceFileType;

fn parse_by_json_content<T: for<'a> Deserialize<'a>>(
    file_content: &str,
) -> Result<T, CommonError> {
    match serde_json::from_str(file_content) {
        Ok(disk_manifest) => Ok(disk_manifest),
        Err(_) => Err(CommonError::Deserialization("JSON")),
    }
}

fn parse_by_yaml_content<T: for<'a> Deserialize<'a>>(
    file_content: &str,
) -> Result<T, CommonError> {
    match serde_yaml::from_str(file_content) {
        Ok(disk_manifest) => Ok(disk_manifest),
        Err(_) => Err(CommonError::Deserialization("YAML")),
    }
}

pub fn parse_file<T: for<'a> Deserialize<'a>>(
    file_path: &Path,
    file_type: &ResourceFileType,
) -> Result<T, CommonError> {
    let get_file_content = || -> Result<String, CommonError> {
        match fs::read_to_string(file_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(CommonError::Io(e)),
        }
    };

    match file_type {
        ResourceFileType::Json => Ok(parse_by_json_content::<T>(&get_file_content()?)?),
        ResourceFileType::Yaml => Ok(parse_by_yaml_content::<T>(&get_file_content()?)?),
        _ => Err(CommonError::UnsupportedFileType(file_type.as_str())),
    }
}

/// Parse a manifest file with a filename containing a timestamp "filename-[timestamp].json"
/// and return the deserialized manifest and the timestamp in secs as a u64.
pub fn parse_file_with_timestamp_by_path<T: for<'a> Deserialize<'a>>(
    file_path: &Path,
    file_type: &ResourceFileType,
) -> Result<(T, SystemTime), CommonError> {
    let filename = file_path
        .file_name()
        .ok_or_else(|| {
            CommonError::Io(Error::new(
                ErrorKind::Other,
                "Failed to get filename from path",
            ))
        })?
        .to_str()
        .ok_or_else(|| {
            CommonError::Io(Error::new(ErrorKind::Other, "Invalid filename encoding"))
        })?;

    let disk_manifest_timestamp_duration = Duration::from_secs(
        filename
            .split('-')
            .next_back()
            .ok_or_else(|| {
                CommonError::Io(Error::new(
                    ErrorKind::Other,
                    "Invalid filename format: missing timestamp separator",
                ))
            })?
            .split('.')
            .next()
            .ok_or_else(|| {
                CommonError::Io(Error::new(
                    ErrorKind::Other,
                    "Invalid filename format: missing extension",
                ))
            })?
            .parse::<u64>()
            .map_err(|e| {
                CommonError::Io(Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse timestamp: {e}"),
                ))
            })?,
    );

    let disk_manifest_timestamp = SystemTime::UNIX_EPOCH + disk_manifest_timestamp_duration;

    Ok((parse_file(file_path, file_type)?, disk_manifest_timestamp))
}

/// Get files in a directory that start with a specific prefix.
pub fn get_files_starts_with(file_name_prefix: &str, dir: &Path) -> Vec<PathBuf> {
    let mut result_files = Vec::new();

    match fs::read_dir(dir) {
        Ok(dir_entries) => {
            for entry_result in dir_entries {
                match entry_result {
                    Ok(entry) => {
                        let file_path = entry.path();

                        if file_path.is_file() {
                            if let Some(filename) = file_path.file_name() {
                                match filename.to_str() {
                                    Some(name) => {
                                        if name.starts_with(file_name_prefix) {
                                            result_files.push(file_path);
                                        }
                                    }
                                    None => {
                                        // warn!(
                                        //     "Failed to convert filename to string: {:?}",
                                        //     filename
                                        // );
                                    }
                                }
                            }
                        }
                    }
                    Err(_e) => {
                        // warn!(
                        //     "Failed to read directory entry in {}: {} (kind: {:?})",
                        //     dir.display(),
                        //     _e,
                        //     _e.kind()
                        // );
                    }
                }
            }
        }
        Err(_e) => {
            // warn!(
            //     "Failed to open directory {}: {} (kind: {:?})",
            //     dir.display(),
            //     _e,
            //     _e.kind()
            // );
        }
    }

    result_files
}

pub fn save_to_disk_override<T>(
    data: &T,
    file_path: &Path,
    file_type: &ResourceFileType,
) -> Result<(), CommonError>
where
    T: Serialize,
{
    let stringified_data = match file_type {
        ResourceFileType::Json => {
            serde_json::to_string(data).map_err(|_| CommonError::Serialization("JSON"))
        }
        ResourceFileType::Yaml => {
            serde_yaml::to_string(data).map_err(|_| CommonError::Serialization("YAML"))
        }
        _ => {
            return Err(CommonError::UnsupportedFileType(file_type.as_str()));
        }
    }?;

    fs::write(file_path, stringified_data).map_err(CommonError::Io)?;

    Ok(())
}
