use std::path::PathBuf;
use std::sync::RwLock;
use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::traits::{DataResult, RemoteResource, ResourceFileType};
use crate::utilities::{
    get_files_starts_with, now, parse_file_with_timestamp_by_path, save_to_disk_override,
};

pub struct Remote<T> {
    file_name: String,
    file_type: ResourceFileType,
    url: String,
    cache_directory: PathBuf,
    timeout: Option<Duration>,
    cached_data: RwLock<Option<(T, u64)>>,
    is_stale: RwLock<bool>,
    http_client: reqwest::Client,
}

impl<T> Remote<T>
where
    T: Send + Sync + Clone + DeserializeOwned + Serialize,
{
    pub fn new(
        file_name: String,
        file_type: ResourceFileType,
        url: String,
        cache_directory: PathBuf,
        timeout: Option<Duration>,
    ) -> Self {
        Self {
            file_name,
            file_type,
            url,
            cache_directory,
            timeout,
            cached_data: RwLock::new(None),
            is_stale: RwLock::new(false),
            http_client: reqwest::Client::new(),
        }
    }

    fn is_marked_stale(&self) -> Result<bool, String> {
        let is_stale = self
            .is_stale
            .read()
            .map_err(|e| format!("Failed to read stale flag: {e}"))?;

        Ok(*is_stale)
    }

    /// Check current internal state; return tuple (data: T, is_fresh: bool)
    fn get_internal_data(&self, since_in_secs: u64) -> Result<Option<(T, bool, u64)>, String> {
        let cached_data = self
            .cached_data
            .read()
            .map_err(|e| format!("Failed to read cache: {e}"))?;

        if let Some((data, timestamp_in_secs)) = cached_data.as_ref() {
            match self.timeout {
                Some(timeout) => {
                    let timeout_in_secs = timeout.as_secs();
                    if since_in_secs - timestamp_in_secs < timeout_in_secs {
                        // Using fresh data from memory cache
                        return Ok(Some((data.clone(), true, *timestamp_in_secs)));
                    }
                    // Using stale data from memory cache
                    return Ok(Some((data.clone(), false, *timestamp_in_secs)));
                }
                None => {
                    // Using data from memory cache (no expiry)
                    return Ok(Some((data.clone(), true, *timestamp_in_secs)));
                }
            }
        }

        Ok(None)
    }

    /// Update memory cache
    fn set_current_member(
        &self,
        data: T,
        now_since_epoch_start_in_secs: u64,
    ) -> Result<(), String> {
        let mut cached_data = self
            .cached_data
            .write()
            .map_err(|e| format!("Failed to write to cache: {e}"))?;

        let mut is_stale = self
            .is_stale
            .write()
            .map_err(|e| format!("Failed to write stale flag: {e}"))?;

        *cached_data = Some((data, now_since_epoch_start_in_secs));
        *is_stale = false;

        Ok(())
    }

    /// Check disk cache for fresh data; return tuple (data: T, is_fresh: bool)
    fn get_disk_cached_data(&self, since_in_secs: u64) -> Result<Option<(T, bool, u64)>, String> {
        let disk_files = get_files_starts_with(&self.file_name, &self.cache_directory);

        // Process the most recent first; if invalid, process older files
        for file_path in disk_files {
            if let Ok(parsed_file) =
                parse_file_with_timestamp_by_path::<T>(&file_path, &self.file_type)
            {
                let (data, timestamp_duration) = parsed_file;

                match self.timeout {
                    Some(timeout) => {
                        if since_in_secs - timestamp_duration.as_secs() < timeout.as_secs() {
                            // Using fresh data from disk cache
                            return Ok(Some((data, true, timestamp_duration.as_secs())));
                        }
                        // Using stale data from disk cache
                        return Ok(Some((data, false, timestamp_duration.as_secs())));
                    }
                    None => {
                        // Using data from disk cache (no expiry)
                        return Ok(Some((data, true, timestamp_duration.as_secs())));
                    }
                }
            }
        }

        Ok(None)
    }

    fn refresh_staled_data(&self, fresh_data: &T, since_in_secs: u64) -> Result<(), String> {
        let file_path = self.cache_directory.join(&self.file_name);
        save_to_disk_override(&fresh_data, &file_path, &self.file_type)?;
        self.set_current_member(fresh_data.clone(), since_in_secs)?;
        Ok(())
    }
}

impl<T> RemoteResource<T> for Remote<T>
where
    T: Send + Sync + Clone + DeserializeOwned + Serialize,
{
    async fn get_data_or_error(&self, allow_stale: bool) -> Result<T, String> {
        let now_duration_in_secs = now()
            .map_err(|e| format!("Failed to get current time: {e}"))?
            .as_secs();

        let mut stale_internal_data: Option<T> = None;
        let mut stale_internal_data_timestamp: u64 = 0;
        let mut stale_disk_cached_data: Option<T> = None;
        let mut stale_disk_cached_data_timestamp: u64 = 0;

        if !self.is_marked_stale()? {
            ///////////////////////////////////////////
            // 1. Check current internal state first //
            ///////////////////////////////////////////

            if let Some((data, fresh, timestamp)) = self.get_internal_data(now_duration_in_secs)? {
                if fresh {
                    // timestamp based
                    return Ok(data);
                }
                stale_internal_data = Some(data);
                stale_internal_data_timestamp = timestamp;
            }

            ///////////////////////////////////
            // 2. Check on disk cached state //
            ///////////////////////////////////

            if let Some((data, fresh, timestamp)) =
                self.get_disk_cached_data(now_duration_in_secs)?
            {
                if fresh {
                    // timestamp based
                    return Ok(data);
                }
                stale_disk_cached_data = Some(data);
                stale_disk_cached_data_timestamp = timestamp;
            }
        }

        /////////////////////////////////////////////////////////////////
        // 3. Data member is either stale or not available; refreshing //
        /////////////////////////////////////////////////////////////////

        let fresh_data_from_server: Option<T> = match reqwest::get(&self.url).await {
            Ok(resp) => {
                match resp.text().await {
                    Ok(body) => {
                        // Try to parse as JSON or YAML depending on file_type
                        match self.file_type {
                            ResourceFileType::Json => serde_json::from_str(&body).ok(),
                            ResourceFileType::Yaml => serde_yaml::from_str(&body).ok(),
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
                    return Ok(
                        stale_disk_cached_data.expect("logic error: stale data should exist")
                    );
                }
                return Ok(stale_internal_data.expect("logic error: stale data should exist"));
            } else if stale_internal_data.is_some() {
                return Ok(stale_internal_data.expect("logic error: stale data should exist"));
            } else if stale_disk_cached_data.is_some() {
                return Ok(stale_disk_cached_data.expect("logic error: stale data should exist"));
            }

            return Err(format!(
                "No fresh or stale data available for resource: {}",
                self.file_name
            ));
        }

        let fresh_data = fresh_data_from_server.ok_or("Failed to fetch data from remote URL")?;

        self.refresh_staled_data(&fresh_data, now_duration_in_secs)?;

        Ok(fresh_data)
    }

    async fn get_data_or_default(&self, allow_stale: bool) -> T {}

    async fn get_data_or_none(&self) -> DataResult<T> {}

    async fn create_data(&self, data: T) -> Result<T, String> {
        let now_duration_in_secs = crate::utilities::now()
            .map_err(|e| format!("Failed to get current time: {e}"))?
            .as_secs();

        /*

           0. confirm the get_data may return NONE in case the server end with no data or error

           1. because the struct is lazy-loaded for data, we can initialise and call create_data (this may end with error, but we will post; we need single marker that get was called at least once)

           2. if data are not stale, we can't post
               - by marking stale (we marking only runtime data)
               - hard-drive data are recognised as stale by timestamp (we may ignore timestap in this case)

           3. before we post (data are stale), call get_data to confirm NONE is returned, call create after


        */

        // match (self.get_internal_data(now_duration_in_secs)?, self.get_disk_cached_data(now_duration_in_secs)?) {
        //     (Some((data, fresh)), _) => {
        //         if fresh {
        //             return Ok(data);
        //         }
        //     }
        //     (_, Some((data, fresh))) => {
        //         if fresh {
        //             return Ok(data);
        //         }
        //     }
        //     _ => {}
        // }

        let response = self
            .http_client
            .post(&self.url)
            .json(&data)
            .send()
            .await
            .map_err(|e| format!("Failed to send POST request: {e}"))?;

        if !response.status().is_success() {
            self.mark_stale();
            return Err(format!("Server returned error: {}", response.status()));
        }

        let fresh_data: T = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse server response: {e}"))?;

        self.refresh_staled_data(&fresh_data, now_duration_in_secs)?;

        Ok(fresh_data)
    }

    async fn update_data(&self, data: T) -> Result<T, String> {
        let now_duration_in_secs = now()
            .map_err(|e| format!("Failed to get current time: {e}"))?
            .as_secs();

        let response = self
            .http_client
            .put(&self.url)
            .json(&data)
            .send()
            .await
            .map_err(|e| format!("Failed to send PUT request: {e}"))?;

        if !response.status().is_success() {
            self.mark_stale(); // stale data since it failed to update, so we have to refresh
            return Err(format!("Server returned error: {}", response.status()));
        }

        let fresh_data: T = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse server response: {e}"))?;

        self.refresh_staled_data(&fresh_data, now_duration_in_secs)?;

        Ok(fresh_data)
    }

    async fn delete_data(&self) -> Result<(), String> {
        let response = self
            .http_client
            .delete(&self.url)
            .send()
            .await
            .map_err(|e| format!("Failed to send DELETE request: {e}"))?;

        if !response.status().is_success() {
            self.mark_stale(); // stale data since it failed to delete, so we have to refresh
            return Err(format!("Server returned error: {}", response.status()));
        }

        self.mark_stale();

        Ok(())
    }

    fn mark_stale(&self) {
        if let Ok(mut is_stale) = self.is_stale.write() {
            *is_stale = true;
        }
    }
}
