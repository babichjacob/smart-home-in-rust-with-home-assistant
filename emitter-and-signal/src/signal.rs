use std::future::Future;

use tokio::sync::{mpsc, watch};
pub use tokio::task::JoinError;

#[derive(Debug)]
pub struct Publisher<T> {
    sender: watch::Sender<T>,
}

impl<T> Publisher<T> {
    pub async fn all_unsubscribed(&self) {
        self.sender.closed().await
    }

    pub fn publish(&self, value: T) {
        self.sender.send_replace(value);
    }

    pub fn publish_with<F: FnOnce(&mut T) -> bool>(&self, maybe_modify: F) {
        self.sender.send_if_modified(maybe_modify);
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

#[derive(Debug)]
pub struct Signal<T> {
    sender: watch::Sender<T>,
    publisher_sender: mpsc::Sender<Publisher<T>>,
}

impl<T> Signal<T> {
    pub fn new<R, Fut: Future<Output = R> + Send + 'static>(
        initial: T,
        producer: impl FnOnce(PublisherStream<T>) -> Fut,
    ) -> (Self, impl Future<Output = Result<R, JoinError>>)
    where
        R: Send + 'static,
    {
        let (sender, _) = watch::channel(initial);
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

    pub fn subscribe(&self) -> Result<Subscription<T>, ProducerExited> {
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
    receiver: watch::Receiver<T>,
}

#[derive(Debug, Clone, Copy)]
pub struct ProducerExited;

impl<T> Subscription<T> {
    pub async fn changed(&mut self) -> Result<(), ProducerExited> {
        self.receiver.changed().await.map_err(|_| ProducerExited)
    }

    pub fn get(&mut self) -> T::Owned
    where
        T: ToOwned,
    {
        self.receiver.borrow_and_update().to_owned()
    }

    pub async fn for_each<Fut: Future<Output = ()>>(mut self, mut func: impl FnMut(T::Owned) -> Fut)
    where
        T: ToOwned,
    {
        loop {
            func(self.get()).await;
            if self.changed().await.is_err() {
                return;
            }
        }
    }
}
