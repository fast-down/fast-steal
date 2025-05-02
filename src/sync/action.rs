use crate::Task;

type RefreshFn<'a> = &'a dyn Fn() -> bool;
type CurrentTask<'a> = &'a Task;

pub trait Action: Send + Clone + 'static {
    fn execute(&self, id: usize, task: CurrentTask, refresh: RefreshFn);
}

impl<T> Action for T
where
    T: FnOnce(usize, CurrentTask, RefreshFn) + Send + Clone + 'static,
{
    #[inline(always)]
    fn execute(&self, id: usize, task: CurrentTask, refresh: RefreshFn) {
        self.clone()(id, task, refresh);
    }
}

// provide type inference
/// Create `Action` implementation from `FnOnce`
#[inline]
pub fn from_fn(
    f: impl FnOnce(usize, CurrentTask, RefreshFn) + Send + Clone + 'static,
) -> impl Action {
    f
}
