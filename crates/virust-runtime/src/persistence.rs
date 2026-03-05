//! Persistence utilities and helpers for the runtime
//!
//! This module provides convenience functions and extensions for working
//! with persistence implementations.

use virust_protocol::{Persistence, PersistenceError, InMemoryPersistence};
use serde_json::Value;
use std::sync::Arc;

/// Create a new in-memory persistence instance
pub fn create_in_memory_persistence() -> Arc<InMemoryPersistence> {
    Arc::new(InMemoryPersistence::new())
}

/// Helper trait for common persistence patterns
#[async_trait::async_trait]
pub trait PersistenceHelpers: Persistence {
    /// Get an item or return a default value
    async fn get_or_default(&self, collection: &str, id: &str, default: Value) -> Result<Value, PersistenceError> {
        match self.get(collection, id).await? {
            Some(item) => Ok(item),
            None => Ok(default),
        }
    }

    /// Ensure an item exists, creating it if it doesn't
    async fn ensure_exists(&self, collection: &str, id: &str, item: Value) -> Result<(bool, String), PersistenceError> {
        match self.get(collection, id).await? {
            Some(_) => Ok((false, id.to_string())),
            None => {
                let new_id = self.create(collection, item).await?;
                Ok((true, new_id))
            }
        }
    }

    /// Update an item if it exists, otherwise create it
    async fn upsert(&self, collection: &str, id: &str, item: Value) -> Result<bool, PersistenceError> {
        match self.get(collection, id).await? {
            Some(_) => {
                self.update(collection, id, item).await?;
                Ok(false) // Was updated, not created
            }
            None => {
                self.create(collection, item).await?;
                Ok(true) // Was created
            }
        }
    }

    /// Check if an item exists
    async fn exists(&self, collection: &str, id: &str) -> Result<bool, PersistenceError> {
        Ok(self.get(collection, id).await?.is_some())
    }

    /// Count items in a collection
    async fn count(&self, collection: &str) -> Result<usize, PersistenceError> {
        let items = self.list(collection).await?;
        Ok(items.len())
    }

    /// Clear all items in a collection
    async fn clear_collection(&self, collection: &str) -> Result<usize, PersistenceError> {
        let items = self.list(collection).await?;
        let mut count = 0;

        for item in items {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                self.delete(collection, id).await?;
                count += 1;
            }
        }

        Ok(count)
    }
}

// Implement PersistenceHelpers for all Persistence implementations
#[async_trait::async_trait]
impl<T: Persistence + ?Sized> PersistenceHelpers for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_or_default() {
        let p = create_in_memory_persistence();

        let result = p.get_or_default("test", "missing", json!({"default": true})).await.unwrap();
        assert_eq!(result, json!({"default": true}));
    }

    #[tokio::test]
    async fn test_ensure_exists_creates() {
        let p = create_in_memory_persistence();
        let item = json!({"name": "test"});

        let (created, id) = p.ensure_exists("test", "new-id", item).await.unwrap();
        assert!(created);
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn test_exists() {
        let p = create_in_memory_persistence();
        let item = json!({"name": "test"});

        let id = p.create("test", item).await.unwrap();
        assert!(p.exists("test", &id).await.unwrap());
        assert!(!p.exists("test", "nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_count() {
        let p = create_in_memory_persistence();

        assert_eq!(p.count("test").await.unwrap(), 0);

        p.create("test", json!({"name": "1"})).await.unwrap();
        p.create("test", json!({"name": "2"})).await.unwrap();

        assert_eq!(p.count("test").await.unwrap(), 2);
    }
}
