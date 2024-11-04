use std::{
    future::Future,
    pin::pin,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread::{self, available_parallelism, Thread},
};

pub mod handle;
pub mod task;
pub mod worker;

use handle::JoinHandle;
use task::Task;
use worker::Worker;

pub struct Runtime {
    worker: Worker,
}

impl Runtime {
    pub fn new() -> Self {
        let worker = Worker::new();
        let threads_count = available_parallelism().map(|c| c.get()).unwrap_or(1);
        for i in 0..threads_count {
            worker.spawn(i);
        }
        Self { worker }
    }

    pub fn spawn<F>(&self, fut: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + Sync + 'static,
        F::Output: Send,
    {
        let (task, handle) = Task::new(fut);
        self.worker.insert_task(task);
        handle
    }

    pub fn block_on<F: Future>(&self, fut: F) -> F::Output {
        // poll the future on the current thread

        let mut fut = pin!(fut);

        let waker = Waker::from(Arc::new(MainWaker {
            thread: thread::current(),
        }));
        let mut cx = Context::from_waker(&waker);

        loop {
            match fut.as_mut().poll(&mut cx) {
                Poll::Pending => thread::park(),
                Poll::Ready(value) => break value,
            }
        }
    }
}

pub struct MainWaker {
    thread: Thread,
}

impl Wake for MainWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.thread.unpark();
    }
}

#[cfg(test)]
mod tests {
    use crate::Runtime;
    // use tokio::runtime::Builder;

    #[test]
    fn test_runtime() {
        let rt = Runtime::new();
        rt.block_on(async { println!("Hello, world!") });
    }
}
