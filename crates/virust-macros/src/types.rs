
#[derive(Debug, Clone)]
pub enum RouteType {
    WebSocket,
    HttpGet,
    HttpPost,
    HttpPut,
    HttpDelete,
}

#[derive(Debug, Clone)]
pub struct RouteEntry {
    pub name: &'static str,
    pub route_type: RouteType,
}

inventory::collect!(RouteEntry);
