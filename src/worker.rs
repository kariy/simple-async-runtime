use parking_lot::{Condvar, Mutex};
use std::{
    collections::VecDeque,
    sync::Arc,
    task::{Context, Wake, Waker},
    thread::{self},
};

use crate::task::Task;

#[derive(Default, Clone)]
pub struct Worker {
    pub(crate) inner: Arc<WorkerInner>,
}

impl Worker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&self, id: usize) {
        let worker = self.clone();
        thread::Builder::new()
            .name(format!("worker-thread-{id}"))
            .spawn(move || worker.run())
            .expect("failed to spawn worker thread");
    }

    pub fn insert_task(&self, task: Task) {
        self.inner.tasks.lock().push_back(task);
        self.inner.condvar.notify_one();
    }

    fn run(&self) {
        loop {
            let mut tasks = self.inner.tasks.lock();
            if let Some(mut task) = tasks.pop_front() {
                let task_waker = Arc::new(TaskWaker {
                    worker: self.clone(),
                    task: Default::default(),
                });

                let waker = Waker::from(task_waker.clone());
                let mut cx = Context::from_waker(&waker);

                if task.fut.as_mut().poll(&mut cx).is_pending() {
                    task_waker.task.lock().replace(task);
                }
            } else {
                self.inner.condvar.wait(&mut tasks);
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct WorkerInner {
    /// queue of pending tasks
    pub(crate) tasks: Mutex<VecDeque<Task>>,
    pub(crate) condvar: Condvar,
}

pub struct TaskWaker {
    worker: Worker,
    task: Mutex<Option<Task>>,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if let Some(task) = self.task.lock().take() {
            self.worker.insert_task(task);
        }
    }
}
