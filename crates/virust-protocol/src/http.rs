use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Option<serde_json::Value>,
}

impl HttpResponse {
    pub fn ok() -> Self {
        Self {
            status: 200,
            body: None,
        }
    }

    pub fn json<T: Serialize>(body: T) -> Self {
        Self {
            status: 200,
            body: Some(serde_json::to_value(body).unwrap()),
        }
    }
}
