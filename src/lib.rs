//! This crate provides an ability to send future for execution on the runtime that may be in a
//! different thread. Typical use case is heavily threaded application where there are synchronous
//! callbacks, but some asynchronous tasks also need to be executed.
//!
//! Typical usage with Tokio runtime:
//! ```rust
//! use tokio::task;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (sender, receiver) = side_futures::create();
//!     task::spawn(receiver.run_receiver(task::spawn));
//!     sender
//!         .send_future(async {
//!             // Do stuff
//!         })
//!         .unwrap();
//! }
//! ```
//! Typical usage with Actix runtime:
//! ```rust
//! #[actix_rt::main]
//! async fn main() {
//!     let (sender, receiver) = side_futures::create();
//!     actix_rt::spawn(receiver.run_receiver(actix_rt::spawn));
//!     sender
//!         .send_future(async {
//!             // Do stuff
//!         })
//!         .unwrap();
//! }
//! ```

use futures::channel::mpsc;
use futures::channel::mpsc::TrySendError;
use futures::StreamExt;
use std::future::Future;
use std::ops::Deref;
use std::ops::DerefMut;
use std::pin::Pin;

type PinnedBoxedFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Sender channel for futures to be received and spawned as executor tasks by receiver. Can be used
/// to send futures manually or (most likely) using `FuturesSender::send_future()` method
#[derive(Clone)]
pub struct FuturesSender(mpsc::UnboundedSender<PinnedBoxedFuture>);

impl FuturesSender {
    /// Sending future for execution on a runtime
    pub fn send_future(
        &self,
        future: impl Future<Output = ()> + Send + 'static,
    ) -> Result<(), TrySendError<impl Future<Output = ()> + Send>> {
        self.unbounded_send(Box::pin(future))
    }
}

impl Deref for FuturesSender {
    type Target = mpsc::UnboundedSender<PinnedBoxedFuture>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FuturesSender {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Receiver channel for futures sent with sender, can be used to run received futures manually or
/// (most likely) using `FuturesReceiver::run_receiver()` method
pub struct FuturesReceiver(mpsc::UnboundedReceiver<PinnedBoxedFuture>);

impl FuturesReceiver {
    /// Run futures sent by sender using runtime-specific tasks spawner
    pub async fn run_receiver<S, R>(mut self, spawner: S)
    where
        S: Fn(PinnedBoxedFuture) -> R,
    {
        while let Some(future) = self.0.next().await {
            spawner(future);
        }
    }
}

impl Deref for FuturesReceiver {
    type Target = mpsc::UnboundedReceiver<PinnedBoxedFuture>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FuturesReceiver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn create() -> (FuturesSender, FuturesReceiver) {
    let (sender, receiver) = mpsc::unbounded::<Pin<Box<dyn Future<Output = ()> + Send>>>();

    (FuturesSender(sender), FuturesReceiver(receiver))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[tokio::test]
    async fn tokio_test() {
        use tokio::task;
        use tokio::time;

        let (sender, receiver) = super::create();

        task::spawn(receiver.run_receiver(task::spawn));

        let vec: Arc<Mutex<Vec<usize>>> = Default::default();

        thread::spawn({
            let vec = Arc::clone(&vec);
            move || {
                let vec = Arc::clone(&vec);
                sender
                    .send_future(async move {
                        vec.lock().unwrap().push(1);
                    })
                    .unwrap();
            }
        });

        time::delay_for(Duration::from_millis(100)).await;

        assert_eq!(vec![1], vec.lock().unwrap().clone());
    }

    #[actix_rt::test]
    async fn actix_rt_test() {
        use actix_rt::time;

        let (sender, receiver) = super::create();

        actix_rt::spawn(receiver.run_receiver(actix_rt::spawn));

        let vec: Arc<Mutex<Vec<usize>>> = Default::default();

        thread::spawn({
            let vec = Arc::clone(&vec);
            move || {
                let vec = Arc::clone(&vec);
                sender
                    .send_future(async move {
                        vec.lock().unwrap().push(1);
                    })
                    .unwrap();
            }
        });

        time::delay_for(Duration::from_millis(100)).await;

        assert_eq!(vec![1], vec.lock().unwrap().clone());
    }
}
