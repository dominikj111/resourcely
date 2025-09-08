use std::sync::Arc;

use error_kit::CommonError;
use serde::{de::DeserializeOwned, Serialize};

use crate::base::ResourceState;

#[derive(Debug, Clone)]
pub enum ResourceFileType {
    Json,
    Yaml,
    Toml,
    Text,
}

impl ResourceFileType {
    /// Convert to &str representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceFileType::Json => "json",
            ResourceFileType::Yaml => "yaml",
            ResourceFileType::Toml => "toml",
            ResourceFileType::Text => "text",
        }
    }
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

impl AsRef<str> for ResourceFileType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
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

    fn mark_as_stale(&self) -> Result<(), CommonError> {
        self.get_state().mark_as_stale();
        Ok(())
    }

    fn is_marked_stale(&self) -> Result<bool, CommonError> {
        self.get_state().is_marked_stale()
    }

    fn is_fresh(&self) -> Result<bool, CommonError> {
        Ok(!self.is_marked_stale()?
            || self.get_state().is_internal_data_fresh()?
            || self.get_state().is_disk_cached_data_fresh()?)
    }

    async fn get_data_or_error(
        &self,
        allow_stale: bool,
    ) -> Result<DataResult<Arc<T>>, CommonError>;

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
