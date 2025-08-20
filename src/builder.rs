// src/builder.rs
use std::path::PathBuf;
use std::time::Duration;

use crate::traits::{LocalResource, RemoteResource};
use crate::ResourceFileType;

/// Builder for creating resource instances with a fluent interface
pub struct ResourceBuilder<T> {
    file_name: Option<String>,
    url: Option<String>,
    cache_directory: Option<PathBuf>,
    timeout: Option<Duration>,
    file_type: Option<ResourceFileType>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for ResourceBuilder<T> {
    fn default() -> Self {
        Self {
            file_name: None,
            url: None,
            cache_directory: None,
            timeout: None,
            file_type: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> ResourceBuilder<T>
where
    T: Send + Sync + 'static,
{
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the file name for the resource
    pub fn file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = Some(file_name.into());
        self
    }

    /// Set the URL for remote resources
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set the cache directory
    pub fn cache_directory(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cache_directory = Some(dir.into());
        self
    }

    /// Set the cache timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the file type
    pub fn file_type(mut self, file_type: ResourceFileType) -> Self {
        self.file_type = Some(file_type);
        self
    }

    /// Build a remote resource
    pub fn build_remote(self) -> Result<impl RemoteResource<T>, String> {
        let file_name = self.file_name.ok_or("File name is required")?;
        let url = self.url.ok_or("URL is required for remote resources")?;
        let cache_dir = self.cache_directory.unwrap_or_else(|| PathBuf::from("."));

        // Create and return your Remote<T> here
        // Example: Remote::new(file_name, url, cache_dir, self.timeout)
        todo!("Implement remote resource creation")
    }

    /// Build a local resource
    pub fn build_local(self) -> Result<impl LocalResource<T>, String> {
        let file_name = self.file_name.ok_or("File name is required")?;
        let cache_dir = self.cache_directory.unwrap_or_else(|| PathBuf::from("."));

        // Create and return your Local<T> here
        // Example: Local::new(file_name, cache_dir, self.file_type)
        todo!("Implement local resource creation")
    }
}