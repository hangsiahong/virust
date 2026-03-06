use crate::registry::RouteEntry;

/// Collect all routes registered via inventory
///
/// This function iterates over the global inventory of RouteEntry
/// items that were registered by the route macros (#[get], #[post], etc.)
pub fn collect_routes() -> Vec<&'static RouteEntry> {
    inventory::iter::<RouteEntry>.into_iter().collect()
}

/// Re-export Route type for convenience
pub use crate::registry::RouteEntry as Route;
