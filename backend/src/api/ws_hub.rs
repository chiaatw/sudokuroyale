use std::collections::HashMap;

use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::api::dto::ws::WsServerEvent;

/// Zentraler Event-Hub für WebSocket-Events
/// Pro Match-ID ein Broadcast-Kanal
pub struct WsHub {
    rooms: RwLock<HashMap<Uuid, broadcast::Sender<WsServerEvent>>>,
    capacity: usize,
}

impl WsHub {
    pub fn new(capacity: usize) -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
            capacity,
        }
    }

    /// Liefert den Sender für ein Match
    pub async fn room_sender(&self, match_id: Uuid) -> broadcast::Sender<WsServerEvent> {
        let mut rooms = self.rooms.write().await;

        rooms
            .entry(match_id)
            .or_insert_with(|| broadcast::channel::<WsServerEvent>(self.capacity).0)
            .clone()
    }

    /// Subscribe für ein Match.
    pub async fn subscribe(&self, match_id: Uuid) -> broadcast::Receiver<WsServerEvent> {
        self.room_sender(match_id).await.subscribe()
    }

    /// Publish Event alle Subscriber
    pub async fn publish(&self, match_id: Uuid, event: WsServerEvent) {
        let tx = self.room_sender(match_id).await;
        let _ = tx.send(event); 
    }

    pub async fn cleanup_room_if_unused(&self, match_id: Uuid) {
        let mut rooms = self.rooms.write().await;

        if let Some(sender) = rooms.get(&match_id) {
            if sender.receiver_count() == 0 {
                rooms.remove(&match_id);
            }
        }
    }
}
