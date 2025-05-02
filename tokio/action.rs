use crate::Task;

type RefreshFn<'a> = &'a (dyn Fn() -> bool + Send + Sync);
type CurrentTask<'a> = &'a Task;

#[cfg(not(feature = "tokio"))]
pub mod sync {
    use super::*;

    pub trait Action: Send + Clone + 'static {
        fn execute(&self, id: usize, task: CurrentTask, refresh: RefreshFn);
    }

    impl<T: FnOnce(usize, CurrentTask, RefreshFn) + Send + Clone + 'static> Action for T {
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
}

#[cfg(feature = "tokio")]
pub mod r#async {
    use super::*;
    use alloc::boxed::Box;
    use core::future::Future;
    use core::pin::Pin;

    pub trait Action: Send + Clone + 'static {
        fn execute<'a>(
            &'a self,
            id: usize,
            task: CurrentTask<'a>,
            refresh: RefreshFn<'a>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
    }

    impl<F, Fut> Action for F
    where
        F: FnOnce(usize, CurrentTask, RefreshFn) -> Fut + Send + Clone + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        #[inline(always)]
        fn execute<'a>(
            &'a self,
            id: usize,
            task: CurrentTask<'a>,
            refresh: RefreshFn<'a>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
            let f = self.clone();
            Box::pin(async move { f(id, task, refresh).await })
        }
    }

    // provide type inference
    /// Create async `Action` implementation from an async function
    #[inline]
    pub fn from_fn<F, Fut>(f: F) -> impl Action
    where
        F: FnOnce(usize, CurrentTask, RefreshFn) -> Fut + Send + Clone + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        f
    }
}

// Re-export based on feature flag
#[cfg(not(feature = "tokio"))]
pub use sync::*;

#[cfg(feature = "tokio")]
pub use r#async::*;
