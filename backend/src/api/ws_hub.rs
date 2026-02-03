use std::collections::HashMap;

use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::api::dto::ws::WsServerEvent;

/// Zentraler Event-Hub für WebSocket-Events.
/// Pro Match-ID gibt es einen Broadcast-Kanal.
/// WS-Clients subscriben; HTTP-Routen publishen.
pub struct WsHub {
    rooms: RwLock<HashMap<Uuid, broadcast::Sender<WsServerEvent>>>,
    capacity: usize,
}

impl WsHub {
    /// `capacity` = wie viele Events pro Room gepuffert werden, bevor Receiver laggen.
    pub fn new(capacity: usize) -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
            capacity,
        }
    }

    /// Liefert (und erstellt ggf.) den Sender für ein Match.
    pub async fn room_sender(&self, match_id: Uuid) -> broadcast::Sender<WsServerEvent> {
        let mut rooms = self.rooms.write().await;

        rooms
            .entry(match_id)
            .or_insert_with(|| broadcast::channel::<WsServerEvent>(self.capacity).0)
            .clone()
    }

    /// Subscribe (Receiver) für ein Match.
    pub async fn subscribe(&self, match_id: Uuid) -> broadcast::Receiver<WsServerEvent> {
        self.room_sender(match_id).await.subscribe()
    }

    /// Publish Event an alle Subscriber des Matches.
    pub async fn publish(&self, match_id: Uuid, event: WsServerEvent) {
        let tx = self.room_sender(match_id).await;
        let _ = tx.send(event); // ignorieren, wenn niemand connected ist
    }

    /// Optional: Room entfernen, wenn keiner mehr subscribed ist.
    /// (MVP: kannst du ignorieren. Später: memory cleanup.)
    pub async fn cleanup_room_if_unused(&self, match_id: Uuid) {
        let mut rooms = self.rooms.write().await;

        if let Some(sender) = rooms.get(&match_id) {
            // receiver_count() ist verfügbar bei tokio::sync::broadcast::Sender
            if sender.receiver_count() == 0 {
                rooms.remove(&match_id);
            }
        }
    }
}
