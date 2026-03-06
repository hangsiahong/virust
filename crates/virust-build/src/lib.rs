mod builder;
mod discovery;
mod error;

pub use builder::{SsgBuilder, SsgStats};
pub use discovery::discover_ssg_routes;
pub use error::{BuildError, Result};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsgRoute {
    pub path: String,
    pub handler: String,
    pub revalidate: Option<u64>,
}

pub trait SsgRouteMetadata {
    const REVALIDATE: Option<u64>;
    fn route_path() -> &'static str;
}
