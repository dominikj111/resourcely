use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

use crate::traits::{ResourceError, ResourceFileType};

fn parse_by_json_content<T: for<'a> Deserialize<'a>>(file_content: &str) -> Result<T, String> {
    match serde_json::from_str(file_content) {
        Ok(disk_manifest) => Ok(disk_manifest),
        Err(e) => Err(format!("Failed to parse json content: {e}")),
    }
}

fn parse_by_yaml_content<T: for<'a> Deserialize<'a>>(file_content: &str) -> Result<T, String> {
    match serde_yaml::from_str(file_content) {
        Ok(disk_manifest) => Ok(disk_manifest),
        Err(e) => Err(format!("Failed to parse yaml content: {e}")),
    }
}

pub fn parse_file<T: for<'a> Deserialize<'a>>(
    file_path: &Path,
    file_type: &ResourceFileType,
) -> Result<T, String> {
    let get_file_content = || -> Result<String, String> {
        match fs::read_to_string(file_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(format!("Failed to read the file: {e}")),
        }
    };

    match file_type {
        ResourceFileType::Json => Ok(parse_by_json_content::<T>(&get_file_content()?)?),
        ResourceFileType::Yaml => Ok(parse_by_yaml_content::<T>(&get_file_content()?)?),
        _ => Err(format!(
            "Unsupported file type: {file_type:?}; use json or yaml only"
        )),
    }
}

/// Parse a manifest file with a filename containing a timestamp "filename-[timestamp].json"
/// and return the deserialized manifest and the timestamp in secs as a u64.
pub fn parse_file_with_timestamp_by_path<T: for<'a> Deserialize<'a>>(
    file_path: &Path,
    file_type: &ResourceFileType,
) -> Result<(T, SystemTime), String> {
    let filename = file_path
        .file_name()
        .ok_or_else(|| format!("Failed to get filename from path: {}", file_path.display()))?
        .to_str()
        .ok_or_else(|| "Invalid filename encoding".to_string())?;

    let disk_manifest_timestamp_duration = Duration::from_secs(
        filename
            .split('-')
            .next_back()
            .ok_or_else(|| "Invalid filename format: missing timestamp separator".to_string())?
            .split('.')
            .next()
            .ok_or_else(|| "Invalid filename format: missing extension".to_string())?
            .parse::<u64>()
            .map_err(|e| format!("Failed to parse timestamp: {e}").to_string())?,
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
) -> Result<(), ResourceError>
where
    T: Serialize,
{
    let stringified_data = match file_type {
        ResourceFileType::Json => {
            serde_json::to_string(data).map_err(|_| ResourceError::Serialization("JSON"))
        }
        ResourceFileType::Yaml => {
            serde_yaml::to_string(data).map_err(|_| ResourceError::Serialization("YAML"))
        }
        _ => {
            return Err(ResourceError::UnsupportedFileType(file_type.clone()));
        }
    }?;

    fs::write(file_path, stringified_data).map_err(ResourceError::Io)?;

    Ok(())
}
