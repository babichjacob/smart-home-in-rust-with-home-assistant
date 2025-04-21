use ext_trait::extension;
use tokio::{select, task::JoinHandle};

use super::emitter::{Capacity, Emitter, NextError};

#[extension(pub trait EmitterExt)]
impl<T> Emitter<T> {
    fn map<M, F>(self, mut func: F, capacity: Capacity) -> (Emitter<M>, JoinHandle<()>)
    where
        T: Send + 'static + Clone,
        M: Send + 'static + Clone,
        F: Send + 'static + FnMut(T) -> M,
    {
        Emitter::new(
            |mut publisher_stream| async move {
                while let Some(publisher) = publisher_stream.wait().await {
                    let Ok(mut subscription) = self.listen() else {
                        return;
                    };

                    loop {
                        select! {
                            biased;
                            _ = publisher.all_unsubscribed() => {
                                break;
                            }
                            event_res = subscription.next() => {
                                match event_res {
                                    Ok(event) => publisher.publish(func(event)),
                                    Err(NextError::Lagged { .. }) => {},
                                    Err(NextError::ProducerExited(_)) => return,
                                }
                            }
                        }
                    }
                }
            },
            capacity,
        )
    }

    fn filter<F>(self, mut func: F, capacity: Capacity) -> (Emitter<T>, JoinHandle<()>)
    where
        T: Send + 'static + Clone,
        F: Send + 'static + FnMut(&T) -> bool,
    {
        Emitter::new(
            |mut publisher_stream| async move {
                while let Some(publisher) = publisher_stream.wait().await {
                    let Ok(mut subscription) = self.listen() else {
                        return;
                    };

                    loop {
                        select! {
                            biased;
                            _ = publisher.all_unsubscribed() => {
                                break;
                            }
                            event_res = subscription.next() => {
                                match event_res {
                                    Ok(event) => if func(&event) {
                                        publisher.publish(event)
                                    },
                                    Err(NextError::Lagged { .. }) => {},
                                    Err(NextError::ProducerExited(_)) => return,
                                }
                            }
                        }
                    }
                }
            },
            capacity,
        )
    }

    fn filter_mut<F>(self, mut func: F, capacity: Capacity) -> (Emitter<T>, JoinHandle<()>)
    where
        T: Send + 'static + Clone,
        F: Send + 'static + FnMut(&mut T) -> bool,
    {
        Emitter::new(
            |mut publisher_stream| async move {
                while let Some(publisher) = publisher_stream.wait().await {
                    let Ok(mut subscription) = self.listen() else {
                        return;
                    };

                    loop {
                        select! {
                            biased;
                            _ = publisher.all_unsubscribed() => {
                                break;
                            }
                            event_res = subscription.next() => {
                                match event_res {
                                    Ok(mut event) => if func(&mut event) {
                                        publisher.publish(event)
                                    },
                                    Err(NextError::Lagged { .. }) => {},
                                    Err(NextError::ProducerExited(_)) => return,
                                }
                            }
                        }
                    }
                }
            },
            capacity,
        )
    }

    fn filter_map<M, F>(self, mut func: F, capacity: Capacity) -> (Emitter<M>, JoinHandle<()>)
    where
        T: Send + 'static + Clone,
        M: Send + 'static + Clone,
        F: Send + 'static + FnMut(T) -> Option<M>,
    {
        Emitter::new(
            |mut publisher_stream| async move {
                while let Some(publisher) = publisher_stream.wait().await {
                    let Ok(mut subscription) = self.listen() else {
                        return;
                    };

                    loop {
                        select! {
                            biased;
                            _ = publisher.all_unsubscribed() => {
                                break;
                            }
                            event_res = subscription.next() => {
                                match event_res {
                                    Ok(event) => if let Some(mapped) = func(event) {
                                        publisher.publish(mapped)
                                    },
                                    Err(NextError::Lagged { .. }) => {},
                                    Err(NextError::ProducerExited(_)) => return,
                                }
                            }
                        }
                    }
                }
            },
            capacity,
        )
    }
}
