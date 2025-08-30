mod base;
mod traits;
mod utilities;

mod local;
mod remote;

pub use traits::*;

pub mod state_manager {
    pub use crate::base::*;
    pub use crate::local::DefaultLocalResourceReader as Local;
    pub use crate::remote::DefaultRemoteResourceReader as Remote;
}
