#![allow(dead_code)]

use std::{collections::HashMap, sync::Arc};

use east_core::{context::Context, message::Msg};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, Context<Msg>>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub async fn add(&self, id: String, ctx: Context<Msg>) {
        self.connections
            .write()
            .await
            .insert(id, ctx);
    }

    pub async fn get(&self, id: String) -> Option<Context<Msg>> {
        self.connections.read().await.get(&id).cloned()
    }

    pub async fn close(&self, id: String) {
        self.connections.write().await.remove(&id);
    }
}


