use super::action::{Action, CurrentTask, RefreshFn};
use crate::{SplitTask, Task};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::mem::ManuallyDrop;
use tokio::sync::Mutex;

pub struct Executor<T, R>
where
    T: FnOnce(usize, CurrentTask, RefreshFn) -> R + Send + Sync + Clone + 'static,
    R: Future<Output = ()> + Send + Sync,
{
    pub(crate) action: Action<T, R>,
    pub(crate) task_ptrs: Arc<Vec<Arc<Task>>>,
    pub(crate) id: usize,
    pub(crate) mutex: Arc<Mutex<()>>,
}

impl<T, R> Executor<T, R>
where
    T: FnOnce(usize, CurrentTask, RefreshFn) -> R + Send + Sync + Clone + 'static,
    R: Future<Output = ()> + Send + Sync,
{
    #[inline]
    pub async fn run(&self) {
        let task = self.task_ptrs[self.id].clone();
        self.action
            .execute(
                self.id,
                &task,
                Box::new(|| {
                    let _guard = self.mutex.lock();
                    let (max_pos, max_remain) = self
                        .task_ptrs
                        .iter()
                        .enumerate()
                        .map(|(i, w)| (i, unsafe { &**w }.remain()))
                        .max_by_key(|(_, remain)| *remain)
                        .unwrap();
                    if max_remain < 2 {
                        return false;
                    }
                    let (start, end) = unsafe { &*self.task_ptrs[max_pos] }.split_two();
                    task.set_end(end);
                    task.set_start(start);
                    true
                }),
            )
            .await;
    }
}
