use alloc::boxed::Box;

use crate::Task;
use core::future::Future;

pub type RefreshFn<'a> = Box<dyn Fn() -> bool + Send>;
pub type CurrentTask<'a> = &'a Task;

pub struct Action<T, R>
where
    T: FnOnce(usize, CurrentTask, RefreshFn) -> R + Send + Sync + Clone + 'static,
    R: Future<Output = ()> + Send + Sync,
{
    f: T,
}

impl<T, R> Action<T, R>
where
    T: FnOnce(usize, CurrentTask, RefreshFn) -> R + Send + Sync + Clone + 'static,
    R: Future<Output = ()> + Send + Sync,
{
    pub fn new(f: T) -> Action<T, R> {
        Action { f }
    }

    #[inline(always)]
    pub fn execute(
        &self,
        id: usize,
        task: CurrentTask,
        refresh: RefreshFn,
    ) -> impl Future<Output = ()> + Send {
        self.f.clone()(id, task, refresh)
    }
}

impl<T, R> Clone for Action<T, R>
where
    T: FnOnce(usize, CurrentTask, RefreshFn) -> R + Send + Sync + Clone + 'static,
    R: Future<Output = ()> + Send + Sync,
{
    fn clone(&self) -> Self {
        Self { f: self.f.clone() }
    }
}
