use crate::handle::JoinHandle;
use std::{future::Future, pin::Pin};
use tokio::sync::oneshot;

pub struct Task {
    pub(crate) fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
}

impl Task {
    pub fn new<F>(fut: F) -> (Self, JoinHandle<F::Output>)
    where
        F: Future + Send + 'static,
        F::Output: Send,
    {
        let (tx, rx) = oneshot::channel::<F::Output>();

        let fut = Box::pin(async move {
            let result = fut.await;
            let _ = tx.send(result);
        });

        (Self { fut }, JoinHandle::new(rx))
    }
}
