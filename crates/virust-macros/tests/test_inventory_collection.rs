// Test that inventory collection works
use virust_macros::{get, ws};

#[get]
async fn test_get_route() -> &'static str {
    "test"
}

#[ws]
async fn test_ws_route(msg: String) -> String {
    msg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_collects_routes() {
        // This test verifies that routes are being registered with inventory
        // In a real scenario, you would iterate over inventory::iter::<RouteEntry>
        let routes: Vec<_> = inventory::iter::<virust_runtime::RouteEntry>.into_iter().collect();

        // We should have at least 2 routes registered
        assert!(routes.len() >= 2, "Expected at least 2 routes, got {}", routes.len());

        // Print the routes for debugging
        for route in routes {
            println!("Route: {:?} -> {:?}", route.path, route.route_type);
        }
    }
}
