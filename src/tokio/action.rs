use crate::Task;

pub type RefreshFn<'a> = &'a (dyn Fn() -> bool + Sync);
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
