use std::sync::Arc;

use serde::{de::DeserializeOwned, Serialize};

use crate::base::ResourceState;

#[derive(Debug, Clone)]
pub enum ResourceFileType {
    Json,
    Yaml,
    Toml,
    Text,
}

impl std::fmt::Display for ResourceFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceFileType::Json => write!(f, "JSON"),
            ResourceFileType::Yaml => write!(f, "YAML"),
            ResourceFileType::Toml => write!(f, "TOML"),
            ResourceFileType::Text => write!(f, "Text"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("Failed to acquire cache lock")]
    CacheLock,

    #[error("Serialization error: {0}")]
    Serialization(&'static str),

    #[error("Deserialization error: {0}")]
    Deserialization(&'static str),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported file type: {0:?}")]
    UnsupportedFileType(ResourceFileType),

    #[error("Data not found")]
    FreshingData,

    #[error("No fresh or stale data found")]
    StaleInternalNone,

    #[error("Timeout exceeded")]
    Timeout,
}

pub enum DataResult<T> {
    Fresh(T),
    Stale(T),
}

#[async_trait::async_trait]
pub trait ResourceReader<T>
where
    T: Send + Sync + DeserializeOwned + Serialize + Default,
{
    fn get_state(&self) -> &ResourceState<T>;

    fn mark_as_stale(&self) -> Result<(), ResourceError> {
        self.get_state().mark_as_stale();
        Ok(())
    }

    fn is_marked_stale(&self) -> Result<bool, ResourceError> {
        self.get_state().is_marked_stale()
    }

    fn is_fresh(&self) -> Result<bool, ResourceError> {
        Ok(!self.is_marked_stale()?
            || self.get_state().is_internal_data_fresh()?
            || self.get_state().is_disk_cached_data_fresh()?)
    }

    async fn get_data_or_error(
        &self,
        allow_stale: bool,
    ) -> Result<DataResult<Arc<T>>, ResourceError>;

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
            },
            Err(_) => None,
        }
    }
}
