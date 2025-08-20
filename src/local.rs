use std::marker::PhantomData;

pub struct Local<T> {
    _marker: PhantomData<T>,
}


/*

use std::path::PathBuf;
use std::sync::RwLock;
use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::traits::RemoteResource;
use crate::utilities::now;

pub struct Remote<T> {
    file_name: String,
    url: String,
    cache_directory: PathBuf,
    timeout: Option<Duration>,
    cached_data: RwLock<Option<(T, u64)>>,
    is_stale: RwLock<bool>,
}

impl<T> Remote<T>
where
    T: Send + Sync + Clone + DeserializeOwned + Serialize,
{
    pub fn new(
        file_name: String,
        url: String,
        cache_directory: PathBuf,
        timeout: Option<Duration>,
    ) -> Self {
        Self {
            url,
            cache_directory,
            file_name,
            timeout,
            cached_data: RwLock::new(None),
            is_stale: RwLock::new(false),
        }
    }

    fn marked_stale(&self) -> Result<bool, String> {
        let is_stale = self
            .is_stale
            .read()
            .map_err(|e| format!("Failed to read stale flag: {e}"))?;

        Ok(*is_stale)
    }

    /// Check current internal state; return tuple (data: T, is_fresh: bool)
    fn check_current(&self, since_in_secs: u64) -> Result<Option<(T, bool)>, String> {
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
                        return Ok(Some((data.clone(), true)));
                    }
                    // Using stale data from memory cache
                    return Ok(Some((data.clone(), false)));
                }
                None => {
                    // Using data from memory cache (no expiry)
                    return Ok(Some((data.clone(), true)));
                }
            }
        }

        Ok(None)
    }

    /// Check disk cache for fresh data; return tuple (data: T, is_fresh: bool)
    fn check_disk_cache(&self, since_in_secs: u64) -> Result<Option<(T, bool)>, String> {
        let disk_files = fs_utils::get_files_starts_with(&self.file_name, &self.cache_directory);

        // Process the most recent first; if invalid, process older files
        for file_path in disk_files {
            if let Ok(parsed_file) = parse_with_timestap_by_path::<T>(&file_path) {
                let (data, timestamp_duration) = parsed_file;

                match self.timeout {
                    Some(timeout) => {
                        if since_in_secs - timestamp_duration.as_secs() < timeout.as_secs() {
                            // Using fresh data from disk cache
                            return Ok(Some((data, true)));
                        }
                        // Using stale data from disk cache
                        return Ok(Some((data, false)));
                    }
                    None => {
                        // Using data from disk cache (no expiry)
                        return Ok(Some((data, true)));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Update memory cache
    fn update_current_member(
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
}

impl<T> RemoteResource<T> for Remote<T>
where
    T: Send + Sync + Clone + DeserializeOwned + Serialize + Default,
{
    async fn get_data(&self) -> Result<T, String> {
        let now_duration_in_secs = now()
            .map_err(|e| format!("Failed to get current time: {e}"))?
            .as_secs();

        if !self.marked_stale()? {
            ///////////////////////////////////////////
            // 1. Check current internal state first //
            ///////////////////////////////////////////

            if let Some((data, fresh)) = self.check_current(now_duration_in_secs)? {
                if fresh {
                    return Ok(data);
                }
            }

            ///////////////////////////////////
            // 2. Check on disk cached state //
            ///////////////////////////////////

            if let Some((data, fresh)) = self.check_disk_cache(now_duration_in_secs)? {
                if fresh {
                    return Ok(data);
                }
            }
        }

        /////////////////////////////////////////////////////////////////
        // 3. Data member is either stale or not available; refreshing //
        /////////////////////////////////////////////////////////////////

        // both url is empty and timeout is none has to be satisfy to work locally only
        let local_refreshing_only = self.url.trim().is_empty() && self.timeout.is_none();
        let file_path = self.cache_directory.join(&self.file_name);

        if local_refreshing_only && file_path.exists() {
            return Err(format!(
                "Override local {} is not allowed ",
                file_path.to_string_lossy()
            ));
        }

        let fresh_data: T = if local_refreshing_only {
            T::default()
        } else {
            utils::fetch(&self.url)
                .await
                .map_err(|e| format!("Failed to fetch data from remote URL: {e}"))?
        };

        save_to_disk_override(&fresh_data, &file_path)?;

        self.update_current_member(fresh_data.clone(), now_duration_in_secs)?;

        Ok(fresh_data)
    }

    fn mark_stale(&self) {
        if let Ok(mut is_stale) = self.is_stale.write() {
            *is_stale = true;
        }
    }
}



*/
