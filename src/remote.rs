use crate::{
    base::ResourceState,
    traits::{DataResult, ResourceError, ResourceFileType, ResourceReader},
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
    async fn get_data_or_error(
        &self,
        allow_stale: bool,
    ) -> Result<DataResult<Arc<T>>, ResourceError> {
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

        let fresh_data_from_server: Option<Arc<T>> =
            match reqwest::get(self.state.get_url().to_owned()).await {
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
                        stale_disk_cached_data.unwrap(), // safe to unwrap since checked above
                    ));
                }
                return Ok(DataResult::Stale(
                    stale_internal_data.unwrap(), // safe to unwrap since checked above
                ));
            } else if stale_internal_data.is_some() {
                return Ok(DataResult::Stale(
                    stale_internal_data.unwrap(), // safe to unwrap since checked above
                ));
            } else if stale_disk_cached_data.is_some() {
                return Ok(DataResult::Stale(
                    stale_disk_cached_data.unwrap(), // safe to unwrap since checked above
                ));
            }

            return Err(ResourceError::StaleInternalNone);
        }

        let fresh_data = fresh_data_from_server.ok_or(ResourceError::FreshingData)?;

        save_to_disk_override(
            &*fresh_data,
            self.state.get_file_path().as_ref(),
            self.state.get_file_type(),
        )?;

        self.state.set_internal_cache(fresh_data.clone())?;

        Ok(DataResult::Fresh(fresh_data))
    }
}
