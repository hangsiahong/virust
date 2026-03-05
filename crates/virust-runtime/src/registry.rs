use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use once_cell::sync::Lazy;
use virust_protocol::{HttpRequest, HttpResponse};
use crate::typescript::TypeScriptGenerator;

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
        Self {
            http_routes: HashMap::new(),
            ws_handlers: HashMap::new(),
            type_definitions: HashMap::new(),
        }
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
