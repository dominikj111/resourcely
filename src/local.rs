use std::sync::Arc;

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    base::ResourceState,
    traits::{DataResult, ResourceError, ResourceReader},
    utilities::{get_files_starts_with, parse_file},
};

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
    fn get_state(&self) -> &ResourceState<T> {
        &self.state
    }

    async fn get_data_or_error(
        &self,
        allow_stale: bool,
    ) -> Result<DataResult<Arc<T>>, ResourceError> {
        let mut stale_internal_data: Option<Arc<T>> = None;

        if !self.get_state().is_marked_stale()? {
            ///////////////////////////////////////////
            // 1. Check current internal state first //
            ///////////////////////////////////////////

            if let Some((data, fresh, _)) = self.get_state().get_internal_data()? {
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
            self.get_state().get_file_name(),
            self.get_state().get_storage_directory(),
        )
        .first()
        {
            Some(file_path) => match parse_file::<T>(file_path, self.get_state().get_file_type()) {
                Ok(data) => Some(Arc::new(data)),
                Err(_) => None,
            },
            None => None,
        };

        if fresh_data_from_drive.is_none() && allow_stale {
            if stale_internal_data.is_some() {
                return Ok(DataResult::Stale(
                    stale_internal_data.unwrap(), // safe to unwrap since checked above
                ));
            }

            return Err(ResourceError::StaleInternalNone);
        }

        let fresh_data = fresh_data_from_drive.ok_or(ResourceError::FreshingData)?;

        self.get_state().set_internal_cache(fresh_data.clone())?;

        Ok(DataResult::Fresh(fresh_data))
    }
}
