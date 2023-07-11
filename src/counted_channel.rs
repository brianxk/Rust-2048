use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender, error::SendError};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use gloo_console::log;

pub struct CountedChannel;

impl CountedChannel {
    pub fn new() -> (CountedSender, CountedReceiver) {
        let tx_counter = Arc::new(AtomicU16::new(0));
        let rx_counter = tx_counter.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        let counted_sender = CountedSender {
            sender: tx,
            counter: tx_counter,
        };

        let counted_receiver = CountedReceiver {
            receiver: rx,
            counter: rx_counter,
        };

        (counted_sender, counted_receiver)
    }
}

pub struct CountedSender {
    sender: UnboundedSender<String>,
    counter: Arc<AtomicU16>,
}

impl CountedSender {
    pub fn send(&self, msg: String) -> Result<(), SendError<String>> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        self.sender.send(msg)?;

        Ok(())
    }

    pub fn get_count(&self) -> u16 {
        self.counter.load(Ordering::SeqCst)
    }
}

pub struct CountedReceiver {
    receiver: UnboundedReceiver<String>,
    counter: Arc<AtomicU16>,
}

impl CountedReceiver {
    /// Consumes all remaining messages in the channel.
    pub async fn recv_all(&mut self, pattern: Option<String>) {
        while self.counter.load(Ordering::SeqCst) > 0 {
            if let Some(msg) = self.receiver.recv().await {
                if pattern.as_ref().is_some_and(|pattern| *pattern == msg) || pattern.is_none() {
                    self.counter.fetch_sub(1, Ordering::SeqCst);
                } 
            }
        }
    }

    pub async fn recv_qty(&mut self, mut quantity: u16) {
        while quantity > 0 && matches!(self.receiver.recv().await, Some(_)) {
            quantity -= 1;
            self.counter.fetch_sub(1, Ordering::SeqCst);
        }
    }

    pub async fn recv(&mut self, pattern: Option<String>) {
        if let Some(msg) = self.receiver.recv().await {
            if pattern.as_ref().is_some_and(|pattern| *pattern == msg) || pattern.is_none() {
                self.counter.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }

    pub fn get_count(&self) -> u16 {
        self.counter.load(Ordering::SeqCst)
    }
}

