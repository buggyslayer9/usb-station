use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum DomainEvent {
    UsbInserted(Uuid),
    UsbRemoved(Uuid),
    FlashJobCreated(Uuid),
    FlashJobStarted(Uuid),
    FlashJobProgress(Uuid, f64),
    FlashJobCompleted(Uuid),
    FlashJobFailed(Uuid, String),
    FlashJobCancelled(Uuid),
    BatchCreated(Uuid),
    BatchStarted(Uuid),
    BatchJobCompleted(Uuid, Uuid),
    BatchJobFailed(Uuid, Uuid, String),
    BatchCompleted(Uuid),
    IsoDiscovered(Uuid),
    IsoDeleted(Uuid),
}

pub trait EventBus: Send + Sync {
    fn publish(&self, event: DomainEvent);
    fn subscribe(&self, handler: Box<dyn Fn(DomainEvent) + Send + Sync>);
}
