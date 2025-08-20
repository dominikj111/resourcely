use std::marker::PhantomData;

pub enum DataResult<T> {
    Fresh(T),
    Stale(T),
    None,
}

pub trait RemoteResource<T>: Send + Sync {
    async fn get_data_or_error(&self, allow_stale: bool) -> Result<T, String>;

    async fn get_data_or_default(&self, allow_stale: bool) -> T;

    async fn get_data_or_none(&self) -> DataResult<T>;

    async fn create_data(&self, data: T) -> Result<T, String>;

    async fn update_data(&self, data: T) -> Result<T, String>;

    async fn delete_data(&self) -> Result<(), String>;

    fn mark_stale(&self);
}

pub trait LocalResource<T>: Send + Sync {
    fn get_data(&self) -> Result<T, String>;
    fn set_data(&self, data: T) -> Result<(), String>;
}

#[derive(Debug)]
pub enum ResourceFileType {
    Json,
    Yaml,
    Toml,
    Text,
}

pub enum Resource<T, R, L>
where
    R: RemoteResource<T>,
    L: LocalResource<T>,
{
    Remote(R, PhantomData<T>),
    Local(L, PhantomData<T>),
}
