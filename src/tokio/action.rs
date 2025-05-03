use crate::Task;
use alloc::boxed::Box;
use core::pin::Pin;

pub type RefreshFn<'a> = &'a (dyn Fn() -> Pin<Box<dyn Future<Output = bool>>> + Sync);
pub type CurrentTask<'a> = &'a Task;

pub trait Action: Send + Clone + 'static {
    fn execute(&self, id: usize, task: CurrentTask, refresh: RefreshFn) -> impl Future + Send;
}

impl<T, R> Action for T
where
    T: FnOnce(usize, CurrentTask, RefreshFn) -> R + Send + Clone + 'static,
    R: Future + Send,
{
    #[inline(always)]
    fn execute(&self, id: usize, task: CurrentTask, refresh: RefreshFn) -> impl Future + Send {
        self.clone()(id, task, refresh)
    }
}

// provide type inference
/// Create `Action` implementation from `FnOnce`
#[inline]
pub fn from_fn<T, R>(f: T) -> impl Action
where
    T: FnOnce(usize, CurrentTask, RefreshFn) -> R + Send + Clone + 'static,
    R: Future + Send,
{
    f
}
