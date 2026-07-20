use tokio::sync::broadcast;
use tracing::info;

use crate::domain::usb::UsbEvent;

/// Monitors USB hotplug events via udev (or polling fallback).
#[derive(Clone)]
pub struct UsbMonitor {
    tx: broadcast::Sender<UsbEvent>,
}

impl UsbMonitor {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(64);

        let _tx_clone = tx.clone();
        tokio::spawn(async move {
            // TODO: implement udev monitor or inotify on /sys/block
            // For now, poll every 5 seconds as a fallback
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                // Poll /sys/block for changes
            }
        });

        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<UsbEvent> {
        self.tx.subscribe()
    }
}
