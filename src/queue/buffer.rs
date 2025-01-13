use crate::core::QueueError::QueueEmptyWhenRead;
use crate::prelude::{Error, Header};
use std::collections::HashMap;
use std::sync::Arc;
use log::debug;
use tokio::sync::{Mutex, Notify};
use tokio_util::codec::{BytesCodec, FramedRead};

struct ReadQueue {
    // Stores the `TransactionId`: Data(Bytes)
    messages: HashMap<u16, Vec<u8>>,
}

#[derive(Debug)]
struct Subscriber {
    notify: Arc<Notify>,
}

impl Subscriber {
    fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
        }
    }

    async fn wait_for_event(&self) {
        self.notify.notified().await;
        debug!("Subscriber awoken");
    }

    fn wake(&self) {
        self.notify.notify_one();
    }
}

#[derive(Debug)]
pub struct Topic {
    data: Mutex<HashMap<u16, (Header, Vec<u8>)>>,
    observers: Mutex<HashMap<u16, Arc<Subscriber>>>,
}

impl Topic {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            data: Mutex::new(HashMap::new()),
            observers: Mutex::new(HashMap::new()),
        })
    }

    pub async fn wait_on(&self, id: u16) -> Result<(Header, Vec<u8>), Error> {
        debug!("Registered subscriber to TcpTopic on TxnID={id}");

        let observer = self.add_observer(id).await;
        observer.wait_for_event().await;

        let data = self
            .data
            .lock()
            .await;
        let response = data
            .get(&id)
            .ok_or(Error::Queue(QueueEmptyWhenRead))?;

        debug!("Wait-Signal triggered on response TxnID={}", response.0.transaction_id);

        self.remove_observer(id).await;
        Ok((*response).clone())
    }

    pub(crate) async fn publish(&self, header: Header, packet: Vec<u8>) {
        let identifier = header.transaction_id;

        // Add data into the queue
        self.data.lock().await.insert(identifier, (header, packet));

        // Wake the relevant subscriber
        if let Some(observer) = self.observers.lock().await.get(&identifier) {
            debug!("Found observer to notify of packet TxnID={:?}", identifier);
            observer.wake();
        }
    }

    async fn add_observer(&self, id: u16) -> Arc<Subscriber> {
        let observer = Arc::new(Subscriber::new());

        self.observers
            .lock()
            .await
            .insert(id, Arc::clone(&observer));

        observer
    }

    async fn remove_observer(&self, id: u16) {
        self.observers.lock().await.remove(&id);
    }
}
