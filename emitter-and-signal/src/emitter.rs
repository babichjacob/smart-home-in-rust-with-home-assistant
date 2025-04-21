use std::{future::Future, num::NonZero};

use deranged::RangedUsize;
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};

use super::ProducerExited;

#[derive(Debug)]
pub struct Publisher<T> {
    sender: broadcast::Sender<T>,
}

impl<T> Publisher<T> {
    pub async fn all_unsubscribed(&self) {
        self.sender.closed().await
    }

    pub fn publish(&self, event: T) {
        let _ = self.sender.send(event);
    }
}

#[derive(Debug)]
pub struct PublisherStream<T> {
    receiver: mpsc::Receiver<Publisher<T>>,
}

impl<T> PublisherStream<T> {
    /// Returns `None` when no more subscriptions can ever be made
    pub async fn wait(&mut self) -> Option<Publisher<T>> {
        self.receiver.recv().await
    }
}

#[derive(Debug, Clone)]
pub struct Emitter<T> {
    sender: broadcast::Sender<T>,
    publisher_sender: mpsc::Sender<Publisher<T>>,
}

pub type Capacity = RangedUsize<1, { usize::MAX / 2 }>;

impl<T> Emitter<T> {
    pub fn new<R, Fut>(
        producer: impl FnOnce(PublisherStream<T>) -> Fut,
        capacity: Capacity,
    ) -> (Self, JoinHandle<R>)
    where
        Fut: Future<Output = R> + Send + 'static,
        T: Clone,
        R: Send + 'static,
    {
        let (sender, _) = broadcast::channel(capacity.get());

        let (publisher_sender, publisher_receiver) = mpsc::channel(1);

        let publisher_stream = PublisherStream {
            receiver: publisher_receiver,
        };

        let producer_join_handle = tokio::spawn(producer(publisher_stream));

        (
            Self {
                publisher_sender,
                sender,
            },
            producer_join_handle,
        )
    }

    pub fn listen(&self) -> Result<Subscription<T>, ProducerExited> {
        let receiver = self.sender.subscribe();

        if self.sender.receiver_count() == 1 {
            if let Err(mpsc::error::TrySendError::Closed(_)) =
                self.publisher_sender.try_send(Publisher {
                    sender: self.sender.clone(),
                })
            {
                return Err(ProducerExited);
            }
        }

        Ok(Subscription { receiver })
    }
}

pub struct Subscription<T> {
    receiver: broadcast::Receiver<T>,
}

pub enum NextError {
    ProducerExited(ProducerExited),
    Lagged { skipped_events: NonZero<u64> },
}

impl<T> Subscription<T> {
    pub async fn next(&mut self) -> Result<T, NextError>
    where
        T: Clone,
    {
        self.receiver.recv().await.map_err(|err| match err {
            broadcast::error::RecvError::Closed => NextError::ProducerExited(ProducerExited),
            broadcast::error::RecvError::Lagged(skipped_events) => NextError::Lagged {
                skipped_events: skipped_events
                    .try_into()
                    .expect("lagging 0 events should be impossible"),
            },
        })
    }
}
