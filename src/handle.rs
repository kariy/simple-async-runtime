use std::{
    borrow::BorrowMut,
    future::Future,
    pin::{pin, Pin},
    task::Poll,
};

use tokio::sync::oneshot::Receiver;

pub struct JoinHandle<T>(Receiver<T>);

impl<T> JoinHandle<T> {
    pub(crate) fn new(rx: Receiver<T>) -> Self {
        Self(rx)
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = T;
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match pin!(self.get_mut().0.borrow_mut()).poll(cx) {
            Poll::Ready(Ok(value)) => Poll::Ready(value),
            Poll::Ready(Err(error)) => panic!("error in task: {error}"),
            Poll::Pending => Poll::Pending,
        }
    }
}
