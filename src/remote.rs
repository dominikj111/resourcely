use crate::{
    base::ResourceState,
    error::ResourceError,
    traits::{DataResult, ResourceFileType, ResourceReader},
    utilities::save_to_disk_override,
};

use serde::{de::DeserializeOwned, Serialize};
use std::{sync::Arc, time::SystemTime};

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
    fn get_state(&self) -> &ResourceState<T> {
        &self.state
    }

    async fn get_data_or_error(
        &self,
        allow_stale: bool,
    ) -> Result<DataResult<Arc<T>>, ResourceError> {
        let mut stale_internal_data: Option<Arc<T>> = None;
        let mut stale_internal_data_timestamp: Option<SystemTime> = None;
        let mut stale_disk_cached_data: Option<Arc<T>> = None;
        let mut stale_disk_cached_data_timestamp: Option<SystemTime> = None;

        if !self.get_state().is_marked_stale()? {
            ///////////////////////////////////////////
            // 1. Check current internal state first //
            ///////////////////////////////////////////

            if let Some((data, fresh, timestamp)) = self.get_state().get_internal_data()? {
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

            if let Some((data, fresh, timestamp)) = self.get_state().get_disk_cached_data()? {
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

        let fresh_data_from_server: Option<Arc<T>> =
            match reqwest::get(self.get_state().get_url().to_owned()).await {
                Ok(resp) => {
                    match resp.text().await {
                        Ok(body) => {
                            // Try to parse as JSON or YAML depending on file_type
                            match &self.get_state().get_file_type() {
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
            match (stale_internal_data, stale_disk_cached_data) {
                (Some(internal), Some(disk)) => {
                    // Both stale sources available, return the newer one
                    if stale_disk_cached_data_timestamp > stale_internal_data_timestamp {
                        return Ok(DataResult::Stale(disk));
                    }
                    return Ok(DataResult::Stale(internal));
                }
                (Some(internal), None) => {
                    // Only internal cache available
                    return Ok(DataResult::Stale(internal));
                }
                (None, Some(disk)) => {
                    // Only disk cache available
                    return Ok(DataResult::Stale(disk));
                }
                (None, None) => {
                    // No stale data available, continue next ti fresh data
                }
            }
        }

        let fresh_data = fresh_data_from_server.ok_or(ResourceError::UnableToFreshData)?;

        save_to_disk_override(
            &*fresh_data,
            self.get_state().get_file_path().as_ref(),
            self.get_state().get_file_type(),
        )?;

        self.get_state().set_internal_cache(fresh_data.clone())?;

        Ok(DataResult::Fresh(fresh_data))
    }
}
