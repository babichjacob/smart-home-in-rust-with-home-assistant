use std::future::Future;

use tokio::{
    sync::{mpsc, watch},
    task::{JoinError, JoinHandle},
};

#[derive(Debug)]
pub struct PublisherStream<T> {
    receiver: mpsc::Receiver<Publisher<T>>,
}

impl<T> PublisherStream<T> {
    pub async fn wait(&mut self) -> Option<Publisher<T>> {
        self.receiver.recv().await
    }
}

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
}

#[derive(Debug)]
pub struct Store<T> {
    sender: watch::Sender<T>,
    publisher_sender: mpsc::Sender<Publisher<T>>,
    producer_join_handle: JoinHandle<()>,
}

impl<T> Store<T> {
    pub fn new<Fut: Future<Output = ()> + Send + 'static>(
        initial: T,
        producer: impl FnOnce(PublisherStream<T>) -> Fut,
    ) -> Self {
        let (sender, _) = watch::channel(initial);
        let (publisher_sender, publisher_receiver) = mpsc::channel(1);

        let subscribers_stream = PublisherStream {
            receiver: publisher_receiver,
        };

        let producer_join_handle = tokio::spawn(producer(subscribers_stream));

        Self {
            publisher_sender,
            sender,
            producer_join_handle,
        }
    }

    pub fn subscribe(&self) -> Result<Subscription<T>, ProducerExited> {
        let receiver = self.sender.subscribe();

        if self.sender.receiver_count() == 1 {
            if let Err(e) = self.publisher_sender.try_send(Publisher {
                sender: self.sender.clone(),
            }) {
                match e {
                    mpsc::error::TrySendError::Full(_) => unreachable!(),
                    mpsc::error::TrySendError::Closed(_) => return Err(ProducerExited),
                }
            }
        }

        Ok(Subscription { receiver })
    }

    /// Signify that no one can ever subscribe again,
    /// and wait for the producer task to complete.
    pub fn run(self) -> impl Future<Output = Result<(), JoinError>> {
        self.producer_join_handle
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
