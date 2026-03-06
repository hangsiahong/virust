use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use once_cell::sync::Lazy;
use virust_protocol::{HttpRequest, HttpResponse};
use crate::typescript::TypeScriptGenerator;
use crate::inventory_registry::{collect_routes, Route as InventoryRoute};

pub type HttpHandler = Arc<dyn Fn(HttpRequest) -> HttpResponse + Send + Sync>;
pub type WsHandler = Arc<dyn Fn(serde_json::Value) -> serde_json::Value + Send + Sync>;

/// Route type enum for route registry
#[derive(Debug, Clone, Copy)]
pub enum RouteType {
    WebSocket,
    HttpGet,
    HttpPost,
    HttpPut,
    HttpDelete,
}

/// Route entry for inventory-based route discovery
#[derive(Debug)]
pub struct RouteEntry {
    pub path: &'static str,
    pub route_type: RouteType,
}

inventory::collect!(RouteEntry);

/// Route with handler for Axum registration
#[derive(Clone)]
pub struct RegisteredRoute {
    pub path: String,
    pub method: String,
    pub handler: HttpHandler,
}

#[derive(Clone, Debug)]
pub struct TypeDefinition {
    pub name: String,
    pub input_type: String,
    pub output_type: String,
}

pub struct RouteRegistry {
    http_routes: HashMap<String, HttpHandler>,
    ws_handlers: HashMap<String, WsHandler>,
    type_definitions: HashMap<String, TypeDefinition>,
}

impl RouteRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            http_routes: HashMap::new(),
            ws_handlers: HashMap::new(),
            type_definitions: HashMap::new(),
        };

        // Collect routes from global inventory
        for route in collect_routes() {
            registry.register_route(route);
        }

        registry
    }

    pub fn register_http(&mut self, path: String, handler: HttpHandler) {
        self.http_routes.insert(path, handler);
    }

    pub fn register_ws(&mut self, path: String, handler: WsHandler) {
        self.ws_handlers.insert(path, handler);
    }

    pub fn register_type(&mut self, name: String, def: TypeDefinition) {
        self.type_definitions.insert(name, def);
    }

    pub fn register_route(&mut self, route: &'static InventoryRoute) {
        // Register the route metadata
        // Note: This just registers the route path and type information
        // Actual handlers are registered separately via register_http/register_ws
        let type_def = TypeDefinition {
            name: route.path.to_string(),
            input_type: format!("{:?}", route.route_type),
            output_type: "Unknown".to_string(),
        };
        self.type_definitions.insert(route.path.to_string(), type_def);
    }

    pub fn get_http(&self, path: &str) -> Option<HttpHandler> {
        self.http_routes.get(path).cloned()
    }

    pub fn get_ws(&self, path: &str) -> Option<WsHandler> {
        self.ws_handlers.get(path).cloned()
    }

    pub fn get_type_definitions(&self) -> &HashMap<String, TypeDefinition> {
        &self.type_definitions
    }

    pub fn generate_typescript(&self) -> String {
        let generator = TypeScriptGenerator::from(self);
        generator.generate()
    }

    pub fn get_routes(&self) -> Vec<RegisteredRoute> {
        let mut routes = Vec::new();

        for route_entry in collect_routes() {
            let method = match route_entry.route_type {
                RouteType::HttpGet => "GET",
                RouteType::HttpPost => "POST",
                RouteType::HttpPut => "PUT",
                RouteType::HttpDelete => "DELETE",
                RouteType::WebSocket => continue, // Skip WebSocket routes for now
            };

            // Get the handler from http_routes if it exists
            if let Some(handler) = self.get_http(route_entry.path) {
                routes.push(RegisteredRoute {
                    path: route_entry.path.to_string(),
                    method: method.to_string(),
                    handler,
                });
            }
        }

        routes
    }
}

/// Global registry for type metadata
static GLOBAL_TYPE_REGISTRY: Lazy<StdMutex<HashMap<String, TypeDefinition>>> =
    Lazy::new(|| StdMutex::new(HashMap::new()));

/// Register a type definition globally (called from macros)
pub fn register_type(name: String, def: TypeDefinition) {
    let mut registry = GLOBAL_TYPE_REGISTRY.lock().unwrap();
    registry.insert(name, def);
}

/// Get all registered type definitions
pub fn get_registered_types() -> HashMap<String, TypeDefinition> {
    let registry = GLOBAL_TYPE_REGISTRY.lock().unwrap();
    registry.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_collects_from_inventory() {
        // Create a test registry
        let registry = RouteRegistry::new();
        let _types = registry.get_type_definitions();

        // The registry should have been created successfully
        // Note: In a real scenario with inventory-registered routes,
        // this would contain those routes. For now, we just verify
        // the constructor works and can be extended with inventory.
        assert!(registry.http_routes.is_empty());
        assert!(registry.ws_handlers.is_empty());
    }
}
