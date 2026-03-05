use std::collections::HashMap;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Error types for persistence operations
#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("Item not found: {id}")]
    NotFound { id: String },

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Error kind for categorizing persistence errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Item was not found
    NotFound,
    /// Invalid operation or parameters
    InvalidOperation,
    /// Serialization/deserialization failed
    Serialization,
    /// Underlying storage error
    Storage,
}

impl PersistenceError {
    /// Get the error kind
    pub fn kind(&self) -> ErrorKind {
        match self {
            PersistenceError::NotFound { .. } => ErrorKind::NotFound,
            PersistenceError::InvalidOperation(_) => ErrorKind::InvalidOperation,
            PersistenceError::SerializationError(_) => ErrorKind::Serialization,
            PersistenceError::StorageError(_) => ErrorKind::Storage,
        }
    }
}

/// Persistence trait for CRUD operations
#[async_trait::async_trait]
pub trait Persistence: Send + Sync {
    /// Get a single item by ID
    async fn get(&self, collection: &str, id: &str) -> Result<Option<Value>, PersistenceError>;

    /// List all items in a collection
    async fn list(&self, collection: &str) -> Result<Vec<Value>, PersistenceError>;

    /// Create a new item (returns the generated ID)
    async fn create(&self, collection: &str, item: Value) -> Result<String, PersistenceError>;

    /// Update an existing item
    async fn update(&self, collection: &str, id: &str, item: Value) -> Result<Value, PersistenceError>;

    /// Delete an item
    async fn delete(&self, collection: &str, id: &str) -> Result<(), PersistenceError>;
}

/// In-memory persistence implementation using HashMap
#[derive(Debug, Clone)]
pub struct InMemoryPersistence {
    storage: Arc<RwLock<HashMap<String, HashMap<String, Value>>>>,
}

impl InMemoryPersistence {
    /// Create a new in-memory persistence store
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a unique ID for a new item
    fn generate_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

impl Default for InMemoryPersistence {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Persistence for InMemoryPersistence {
    async fn get(&self, collection: &str, id: &str) -> Result<Option<Value>, PersistenceError> {
        let storage = self.storage.read().await;
        Ok(storage.get(collection).and_then(|col| col.get(id).cloned()))
    }

    async fn list(&self, collection: &str) -> Result<Vec<Value>, PersistenceError> {
        let storage = self.storage.read().await;
        Ok(storage
            .get(collection)
            .map(|col| col.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn create(&self, collection: &str, item: Value) -> Result<String, PersistenceError> {
        let id = self.generate_id();
        let mut storage = self.storage.write().await;

        let collection_storage = storage.entry(collection.to_string()).or_default();

        if collection_storage.contains_key(&id) {
            return Err(PersistenceError::StorageError(
                "ID collision detected".to_string(),
            ));
        }

        collection_storage.insert(id.clone(), item);
        Ok(id)
    }

    async fn update(&self, collection: &str, id: &str, item: Value) -> Result<Value, PersistenceError> {
        let mut storage = self.storage.write().await;

        let collection_storage = storage
            .get_mut(collection)
            .ok_or_else(|| PersistenceError::NotFound {
                id: format!("{}:{}", collection, id),
            })?;

        if !collection_storage.contains_key(id) {
            return Err(PersistenceError::NotFound {
                id: format!("{}:{}", collection, id),
            });
        }

        collection_storage.insert(id.to_string(), item.clone());
        Ok(item)
    }

    async fn delete(&self, collection: &str, id: &str) -> Result<(), PersistenceError> {
        let mut storage = self.storage.write().await;

        let collection_storage = storage
            .get_mut(collection)
            .ok_or_else(|| PersistenceError::NotFound {
                id: format!("{}:{}", collection, id),
            })?;

        collection_storage
            .remove(id)
            .ok_or_else(|| PersistenceError::NotFound {
                id: format!("{}:{}", collection, id),
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_create_and_get() {
        let persistence = InMemoryPersistence::new();
        let item = json!({"name": "test", "value": 42});

        let id = persistence.create("test_collection", item.clone()).await.unwrap();
        let retrieved = persistence.get("test_collection", &id).await.unwrap().unwrap();

        assert_eq!(retrieved, item);
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let persistence = InMemoryPersistence::new();
        let result = persistence.get("test_collection", "nonexistent").await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list() {
        let persistence = InMemoryPersistence::new();

        let item1 = json!({"name": "test1"});
        let item2 = json!({"name": "test2"});

        persistence.create("test_collection", item1).await.unwrap();
        persistence.create("test_collection", item2).await.unwrap();

        let items = persistence.list("test_collection").await.unwrap();
        assert_eq!(items.len(), 2);
    }

    #[tokio::test]
    async fn test_list_empty_collection() {
        let persistence = InMemoryPersistence::new();
        let items = persistence.list("nonexistent").await.unwrap();

        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn test_update() {
        let persistence = InMemoryPersistence::new();
        let item = json!({"name": "test", "value": 42});

        let id = persistence.create("test_collection", item).await.unwrap();

        let updated_item = json!({"name": "updated", "value": 100});
        let result = persistence
            .update("test_collection", &id, updated_item.clone())
            .await
            .unwrap();

        assert_eq!(result, updated_item);

        let retrieved = persistence.get("test_collection", &id).await.unwrap().unwrap();
        assert_eq!(retrieved, updated_item);
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let persistence = InMemoryPersistence::new();
        let item = json!({"name": "test"});

        let result = persistence
            .update("test_collection", "nonexistent", item)
            .await;

        assert!(matches!(result, Err(PersistenceError::NotFound { .. })));
    }

    #[tokio::test]
    async fn test_delete() {
        let persistence = InMemoryPersistence::new();
        let item = json!({"name": "test"});

        let id = persistence.create("test_collection", item).await.unwrap();
        persistence.delete("test_collection", &id).await.unwrap();

        let result = persistence.get("test_collection", &id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let persistence = InMemoryPersistence::new();
        let result = persistence
            .delete("test_collection", "nonexistent")
            .await;

        assert!(matches!(result, Err(PersistenceError::NotFound { .. })));
    }

    #[tokio::test]
    async fn test_multiple_collections() {
        let persistence = InMemoryPersistence::new();

        let item1 = json!({"name": "item1"});
        let item2 = json!({"name": "item2"});

        let id1 = persistence.create("collection1", item1).await.unwrap();
        let id2 = persistence.create("collection2", item2).await.unwrap();

        let items1 = persistence.list("collection1").await.unwrap();
        let items2 = persistence.list("collection2").await.unwrap();

        assert_eq!(items1.len(), 1);
        assert_eq!(items2.len(), 1);

        let retrieved1 = persistence.get("collection1", &id1).await.unwrap().unwrap();
        let retrieved2 = persistence.get("collection2", &id2).await.unwrap().unwrap();

        assert_eq!(retrieved1, json!({"name": "item1"}));
        assert_eq!(retrieved2, json!({"name": "item2"}));
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let persistence = Arc::new(InMemoryPersistence::new());
        let mut handles = vec![];

        for i in 0..10 {
            let p = persistence.clone();
            handles.push(tokio::spawn(async move {
                let item = json!({"index": i});
                p.create("concurrent_test", item).await
            }));
        }

        let results: Vec<Result<String, PersistenceError>> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(results.len(), 10);
        assert!(results.iter().all(|r| r.is_ok()));

        let items = persistence.list("concurrent_test").await.unwrap();
        assert_eq!(items.len(), 10);
    }
}
