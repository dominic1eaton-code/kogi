pub mod error;
pub mod model;
pub mod snapshots;
pub mod system;
pub mod ffi;

pub use error::{OfficeError, OfficeResult};
pub use model::*;
pub use snapshots::*;
pub use system::{OfficeConfig, OfficeState, OfficeSystem};
