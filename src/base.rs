use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::time::{Duration, SystemTime};

use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::{ResourceError, ResourceFileType};
use crate::utilities::{get_files_starts_with, parse_file_with_timestamp_by_path};

pub struct Cache<T> {
    data: Option<Arc<T>>,
    is_stale: bool,
    timestamp: SystemTime,
}

pub struct ResourceProps<T> {
    file_name: String,
    file_type: ResourceFileType,
    url: Url,
    storage_directory: PathBuf,
    internal_cache: RwLock<Cache<T>>,
    timeout: Option<Duration>,
}

pub struct ResourceState<T> {
    props: ResourceProps<T>,
}

impl<T: Serialize + DeserializeOwned> ResourceState<T> {
    pub fn new(props: ResourceProps<T>) -> Self {
        Self { props }
    }

    fn get_internal_cache_guard(&self) -> Result<RwLockReadGuard<Cache<T>>, ResourceError> {
        match self.props.internal_cache.read() {
            Ok(guard) => Ok(guard),
            Err(_) => Err(ResourceError::CacheLock),
        }
    }

    pub fn is_marked_stale(&self) -> Result<bool, ResourceError> {
        let cache = self.get_internal_cache_guard()?;
        Ok(cache.is_stale)
    }

    pub fn mark_as_stale(&self) {
        if let Ok(mut internal_cache) = self.props.internal_cache.write() {
            internal_cache.is_stale = true;
        }
    }

    pub fn get_file_type(&self) -> &ResourceFileType {
        &self.props.file_type
    }

    pub fn get_file_path(&self) -> PathBuf {
        self.props.storage_directory.join(&self.props.file_name)
    }

    pub fn get_file_name(&self) -> &str {
        &self.props.file_name
    }

    pub fn get_storage_directory(&self) -> &Path {
        self.props.storage_directory.as_path()
    }

    pub fn get_url(&self) -> &Url {
        &self.props.url
    }

    pub fn is_internal_data_fresh(&self) -> Result<bool, ResourceError> {
        let cache = self.get_internal_cache_guard()?;

        let is_fresh = cache
            .timestamp
            .elapsed()
            .map(|elapsed| match self.props.timeout {
                Some(timeout) => elapsed < timeout,
                None => true,
            })
            .unwrap_or(false); // treat clock rollback as stale

        Ok(is_fresh)
    }

    pub fn is_disk_cached_data_fresh(&self) -> Result<bool, ResourceError> {
        // TODO: improvement required - this causes the drive reading and content parsing;
        match self.get_disk_cached_data()? {
            Some((_, fresh, _)) => Ok(fresh),
            None => Ok(false),
        }
    }

    pub fn get_internal_data(&self) -> Result<Option<(Arc<T>, bool, SystemTime)>, ResourceError> {
        let cache = self.get_internal_cache_guard()?;

        if cache.data.is_none() {
            return Ok(None);
        }

        let data = Arc::clone(cache.data.as_ref().unwrap()); // safe to unwrap since checked above

        let is_fresh = cache
            .timestamp
            .elapsed()
            .map(|elapsed| match self.props.timeout {
                Some(timeout) => elapsed < timeout,
                None => true,
            })
            .unwrap_or(false); // treat clock rollback as stale

        Ok(Some((data, is_fresh, cache.timestamp)))
    }

    pub fn set_internal_cache<D>(&self, data: D) -> Result<(), ResourceError>
    where
        D: Into<Arc<T>>,
    {
        let mut cache_write = self
            .props
            .internal_cache
            .write()
            .map_err(|_| ResourceError::CacheLock)?;

        *cache_write = Cache {
            data: Some(data.into()), // auto converts T → Arc<T> or Arc<T> → Arc<T>
            is_stale: false,
            timestamp: SystemTime::now(),
        };

        Ok(())
    }

    pub fn get_disk_cached_data(
        &self,
    ) -> Result<Option<(Arc<T>, bool, SystemTime)>, ResourceError> {
        let disk_files =
            get_files_starts_with(&self.props.file_name, &self.props.storage_directory);

        for file_path in disk_files {
            if let Ok((data, timestamp)) =
                parse_file_with_timestamp_by_path::<T>(&file_path, &self.props.file_type)
            {
                let arc_data = std::sync::Arc::new(data);

                let is_fresh = timestamp
                    .elapsed()
                    .map(|elapsed| match self.props.timeout {
                        Some(timeout) => elapsed < timeout,
                        None => true,
                    })
                    .unwrap_or(false); // treat clock rollback as stale

                return Ok(Some((arc_data, is_fresh, timestamp)));
            }
        }

        Ok(None)
    }
}
