use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::traits::{DataResult, ResourceFileType};
use crate::utilities::{
    get_files_starts_with, parse_file, parse_file_with_timestamp_by_path, save_to_disk_override,
};

#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("Failed to acquire cache lock")]
    CacheLock,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error), // adapt to serde_yaml / toml too

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported file type")]
    UnsupportedFileType,

    #[error("Data not found")]
    NotFound,

    #[error("Data is stale")]
    StaleData,

    #[error("Timeout exceeded")]
    Timeout,
}

pub(crate) struct Cache<T> {
    data: Option<Arc<T>>,
    is_stale: bool,
    timestamp: SystemTime,
}

pub(crate) struct ResourceProps<T> {
    file_name: String,
    file_type: ResourceFileType,
    url: Url,
    storage_directory: PathBuf,
    internal_cache: RwLock<Cache<T>>,
    timeout: Option<Duration>,
}

pub(crate) struct ResourceState<T> {
    props: ResourceProps<T>,
}

impl<T: Serialize + DeserializeOwned> ResourceState<T> {
    fn get_file_type(&self) -> &ResourceFileType {
        &self.props.file_type
    }

    fn is_marked_stale(&self) -> Result<bool, String> {
        let cache = self
            .props
            .internal_cache
            .read()
            .map_err(|e| format!("Failed to read stale flag: {e}"))?;

        Ok(cache.is_stale)
    }

    fn mark_stale(&self) {
        if let Ok(mut internal_cache) = self.props.internal_cache.write() {
            internal_cache.is_stale = true;
        }
    }

    fn get_internal_data(&self) -> Result<Option<(Arc<T>, bool, SystemTime)>, String> {
        let internal_cache_guard = self
            .props
            .internal_cache
            .read()
            .map_err(|e| format!("Failed to read cache: {e}"))?;

        if internal_cache_guard.data.is_none() {
            return Ok(None);
        }

        let data = Arc::clone(internal_cache_guard.data.as_ref().unwrap());

        let is_fresh = internal_cache_guard
            .timestamp
            .elapsed()
            .map(|elapsed| match self.props.timeout {
                Some(timeout) => elapsed < timeout,
                None => true,
            })
            .unwrap_or(false); // treat clock rollback as stale

        Ok(Some((data, is_fresh, internal_cache_guard.timestamp)))
    }

    fn set_internal_cache<D>(&self, data: D) -> Result<(), String>
    where
        D: Into<Arc<T>>,
    {
        let mut internal_cache = self
            .props
            .internal_cache
            .write()
            .map_err(|e| format!("Failed to write to cache: {e}"))?;

        let internal_cache_guard = self
            .props
            .internal_cache
            .read()
            .map_err(|e| format!("Failed to read cache: {e}"))?;

        *internal_cache = Cache {
            data: Some(data.into()), // auto converts T → Arc<T> or Arc<T> → Arc<T>
            is_stale: false,
            timestamp: internal_cache_guard.timestamp,
        };

        Ok(())
    }

    fn get_disk_cached_data(&self) -> Result<Option<(Arc<T>, bool, SystemTime)>, String> {
        let disk_files =
            get_files_starts_with(&self.props.file_name, &self.props.storage_directory);

        let internal_cache_guard = self
            .props
            .internal_cache
            .read()
            .map_err(|e| format!("Failed to read cache: {e}"))?;

        for file_path in disk_files {
            if let Ok((data, timestamp)) =
                parse_file_with_timestamp_by_path::<T>(&file_path, &self.props.file_type)
            {
                let arc_data = std::sync::Arc::new(data);

                let is_fresh = internal_cache_guard
                    .timestamp
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

    fn get_file_path(&self) -> PathBuf {
        self.props.storage_directory.join(&self.props.file_name)
    }

    fn get_file_name(&self) -> String {
        self.props.file_name.clone()
    }

    fn get_storage_directory(&self) -> PathBuf {
        self.props.storage_directory.clone()
    }

    fn get_url(&self) -> Url {
        self.props.url.clone()
    }
}

#[async_trait::async_trait]
pub trait ResourceReader<T>
where
    T: Send + Sync + DeserializeOwned + Serialize + Default,
{
    async fn get_data_or_error(&self, allow_stale: bool) -> Result<DataResult<Arc<T>>, String>;

    async fn get_data_or_default(&self, allow_stale: bool) -> Arc<T> {
        match self.get_data_or_error(allow_stale).await {
            Ok(data) => match data {
                DataResult::Fresh(data) => data,
                DataResult::Stale(data) => {
                    if allow_stale {
                        data
                    } else {
                        T::default().into()
                    }
                }
                DataResult::None => T::default().into(),
            },
            Err(_) => T::default().into(),
        }
    }

    async fn get_data_or_none(&self, allow_stale: bool) -> Option<Arc<T>> {
        match self.get_data_or_error(allow_stale).await {
            Ok(data) => match data {
                DataResult::Fresh(data) => Some(data),
                DataResult::Stale(data) => {
                    if allow_stale {
                        Some(data)
                    } else {
                        None
                    }
                }
                DataResult::None => None,
            },
            Err(_) => None,
        }
    }
}

pub struct DefaultLocalResourceReader<T> {
    state: ResourceState<T>,
}

impl<T> DefaultLocalResourceReader<T> {
    pub fn new(state: ResourceState<T>) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait]
impl<T> ResourceReader<T> for DefaultLocalResourceReader<T>
where
    T: Send + Sync + DeserializeOwned + Serialize + Default,
{
    async fn get_data_or_error(&self, allow_stale: bool) -> Result<DataResult<Arc<T>>, String> {
        let mut stale_internal_data: Option<Arc<T>> = None;

        if !self.state.is_marked_stale()? {
            ///////////////////////////////////////////
            // 1. Check current internal state first //
            ///////////////////////////////////////////

            if let Some((data, fresh, _)) = self.state.get_internal_data()? {
                if fresh {
                    // timestamp based
                    return Ok(DataResult::Fresh(data));
                }
                stale_internal_data = Some(data);
            }
        }

        /////////////////////////////////////////////////////////////////
        // 2. Data member is either stale or not available; refreshing //
        /////////////////////////////////////////////////////////////////

        let fresh_data_from_drive = match get_files_starts_with(
            &self.state.get_file_name(),
            &self.state.get_storage_directory(),
        )
        .get(0)
        {
            Some(file_path) => match parse_file::<T>(file_path, &self.state.get_file_type()) {
                Ok(data) => Some(Arc::new(data)),
                Err(_) => None,
            },
            None => None,
        };

        if fresh_data_from_drive.is_none() && allow_stale {
            if stale_internal_data.is_some() {
                return Ok(DataResult::Stale(
                    stale_internal_data.expect("logic error: stale data should exist"),
                ));
            }

            return Err(format!(
                "No fresh or stale data available for resource: {}",
                self.state.get_file_name()
            ));
        }

        let fresh_data = fresh_data_from_drive.ok_or("Failed to fetch data from remote URL")?;

        self.state.set_internal_cache(fresh_data.clone())?;

        Ok(DataResult::Fresh(fresh_data))
    }
}

pub struct DefaultRemoteResourceReader<T> {
    state: ResourceState<T>,
}

impl<T> DefaultRemoteResourceReader<T> {
    pub fn new(state: ResourceState<T>) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait]
impl<T> ResourceReader<T> for DefaultRemoteResourceReader<T>
where
    T: Send + Sync + DeserializeOwned + Serialize + Default,
{
    async fn get_data_or_error(&self, allow_stale: bool) -> Result<DataResult<Arc<T>>, String> {
        let mut stale_internal_data: Option<Arc<T>> = None;
        let mut stale_internal_data_timestamp: Option<SystemTime> = None;
        let mut stale_disk_cached_data: Option<Arc<T>> = None;
        let mut stale_disk_cached_data_timestamp: Option<SystemTime> = None;

        if !self.state.is_marked_stale()? {
            ///////////////////////////////////////////
            // 1. Check current internal state first //
            ///////////////////////////////////////////

            if let Some((data, fresh, timestamp)) = self.state.get_internal_data()? {
                if fresh {
                    // timestamp based
                    return Ok(DataResult::Fresh(data));
                }
                stale_internal_data = Some(data);
                stale_internal_data_timestamp = Some(timestamp);
            }

            ///////////////////////////////////
            // 2. Check on disk cached state //
            ///////////////////////////////////

            if let Some((data, fresh, timestamp)) = self.state.get_disk_cached_data()? {
                if fresh {
                    // timestamp based
                    return Ok(DataResult::Fresh(data));
                }
                stale_disk_cached_data = Some(data);
                stale_disk_cached_data_timestamp = Some(timestamp);
            }
        }

        /////////////////////////////////////////////////////////////////
        // 3. Data member is either stale or not available; refreshing //
        /////////////////////////////////////////////////////////////////

        let fresh_data_from_server: Option<Arc<T>> = match reqwest::get(self.state.get_url()).await
        {
            Ok(resp) => {
                match resp.text().await {
                    Ok(body) => {
                        // Try to parse as JSON or YAML depending on file_type
                        match &self.state.get_file_type() {
                            ResourceFileType::Json => {
                                serde_json::from_str(&body).ok().map(Arc::new)
                            }
                            ResourceFileType::Yaml => {
                                serde_yaml::from_str(&body).ok().map(Arc::new)
                            }
                            _ => None,
                        }
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        };

        if fresh_data_from_server.is_none() && allow_stale {
            if stale_internal_data.is_some() && stale_disk_cached_data.is_some() {
                if stale_disk_cached_data_timestamp > stale_internal_data_timestamp {
                    return Ok(DataResult::Stale(
                        stale_disk_cached_data.expect("logic error: stale data should exist"),
                    ));
                }
                return Ok(DataResult::Stale(
                    stale_internal_data.expect("logic error: stale data should exist"),
                ));
            } else if stale_internal_data.is_some() {
                return Ok(DataResult::Stale(
                    stale_internal_data.expect("logic error: stale data should exist"),
                ));
            } else if stale_disk_cached_data.is_some() {
                return Ok(DataResult::Stale(
                    stale_disk_cached_data.expect("logic error: stale data should exist"),
                ));
            }

            return Err(format!(
                "No fresh or stale data available for resource: {}",
                self.state.get_file_name()
            ));
        }

        let fresh_data = fresh_data_from_server.ok_or("Failed to fetch data from remote URL")?;

        save_to_disk_override(
            &*fresh_data,
            self.state.get_file_path().as_ref(),
            &self.state.get_file_type(),
        )?;

        self.state.set_internal_cache(fresh_data.clone())?;

        Ok(DataResult::Fresh(fresh_data))
    }
}
