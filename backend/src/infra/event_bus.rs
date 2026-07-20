use tokio::sync::broadcast;

use crate::domain::events::{DomainEvent, EventBus};

#[derive(Clone)]
pub struct InMemoryEventBus {
    tx: broadcast::Sender<DomainEvent>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx }
    }
}

impl EventBus for InMemoryEventBus {
    fn publish(&self, event: DomainEvent) {
        let _ = self.tx.send(event);
    }

    fn subscribe(&self, handler: Box<dyn Fn(DomainEvent) + Send + Sync>) {
        let mut rx = self.tx.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                handler(event);
            }
        });
    }
}
