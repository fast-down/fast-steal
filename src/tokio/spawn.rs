use super::action::{Action, CurrentTask, RefreshFn};
use super::executor::Executor;
use crate::{split_task::SplitTask, task::Task, task_list::TaskList};
use alloc::{sync::Arc, vec::Vec};
use core::mem::ManuallyDrop;
use tokio::task::JoinHandle;

pub trait Spawn {
    fn spawn<S, T, R>(self, threads: usize, spawn: S, action: Action<T, R>) -> Vec<JoinHandle<()>>
    where
        S: Fn(Executor<T, R>) -> JoinHandle<()>,
        T: FnOnce(usize, CurrentTask, RefreshFn) -> R + Send + Clone + 'static,
        R: Future<Output = ()> + Send;
}

impl Spawn for Arc<TaskList> {
    fn spawn<S, T, R>(self, threads: usize, spawn: S, action: Action<T, R>) -> Vec<JoinHandle<()>>
    where
        S: Fn(Executor<T, R>) -> JoinHandle<()>,
        T: FnOnce(usize, CurrentTask, RefreshFn) -> R + Send + Clone + 'static,
        R: Future<Output = ()> + Send,
    {
        let bump = Arc::pin(bumpalo::Bump::with_capacity(
            threads * size_of::<Task>() + 10,
        ));
        let task_ptrs: Arc<[*const Task]> = Arc::from(
            Task::from(&*self)
                .split_task(threads)
                .map(|t| bump.alloc(t) as *const Task)
                .collect::<Vec<*const Task>>(),
        );
        let mutex = Arc::new(tokio::sync::Mutex::new(()));
        let mut handles = Vec::with_capacity(threads);
        for id in 0..threads {
            let task_ptrs = task_ptrs.clone();
            let action = action.clone();
            let mutex = mutex.clone();
            let handle = spawn(Executor {
                id,
                action,
                mutex,
                task_ptrs: ManuallyDrop::new(task_ptrs),
            });
            handles.push(handle);
        }
        handles
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::tokio::action::{self, RefreshFn};
    use std::{
        collections::{HashMap, hash_map::Entry},
        dbg, vec,
    };
    use tokio::sync::mpsc;

    fn fib(n: usize) -> usize {
        match n {
            0 => 0,
            1 => 1,
            _ => fib(n - 1) + fib(n - 2),
        }
    }

    fn fib_fast(n: usize) -> usize {
        let mut a = 0;
        let mut b = 1;
        for _ in 0..n {
            (a, b) = (b, a + b);
        }
        a
    }

    #[tokio::test]
    async fn test_spawn() {
        let tasks: Arc<TaskList> = Arc::new(vec![0..48].into());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let tasks_clone = tasks.clone();
        let f = Action::new(async move |_, task: &Task, refresh: RefreshFn| {
            loop {
                while task.start() < task.end() {
                    let i = tasks_clone.get(task.start());
                    task.fetch_add_start(1);
                    let _ = tx.send((i, fib(i)));
                }
                if !refresh() {
                    break;
                }
            }
        });
        let handles = tasks.clone().spawn(
            8,
            |executor| tokio::spawn(async move { executor.run().await }),
            f,
        );
        // 汇总任务结果
        let mut data = HashMap::new();
        while let Some((i, res)) = rx.recv().await {
            // 如果重复计算就报错
            match data.entry(i) {
                Entry::Occupied(_) => {
                    panic!("数字 {i}，值为 {res} 重复计算")
                }
                Entry::Vacant(entry) => {
                    entry.insert(res);
                }
            }
            // 删除这行，因为在上面的match块中已经插入了数据
        }
        // 等待任务结束
        for handle in handles {
            handle.await.unwrap();
        }
        // 验证结果
        dbg!(&data);
        for i in 0..tasks.len {
            let index = tasks.get(i);
            assert_eq!((index, data.get(&index)), (index, Some(&fib_fast(index))));
        }
    }

    // #[tokio::test]
    // async fn test_spawn2() {
    //     let tasks: Arc<TaskList> = Arc::new(vec![0..48].into());
    //     let (tx, mut rx) = mpsc::unbounded_channel();
    //     let tasks_clone = tasks.clone();
    //     let handles = tasks.clone().spawn(
    //         8,
    //         |executor| tokio::spawn(async move { executor.run().await }),
    //         action::from_fn(async move |_, task, get_task| {
    //             loop {
    //                 while task.start() < task.end() {
    //                     let i = tasks_clone.get(task.start());
    //                     task.fetch_add_start(2);
    //                     let _ = tx.send((i, fib(i)));
    //                     if i + 1 < task.end() {
    //                         let _ = tx.send((i + 1, fib(i + 1)));
    //                     } else {
    //                         task.fetch_sub_start(1);
    //                     }
    //                 }
    //                 if !get_task() {
    //                     break;
    //                 }
    //             }
    //         }),
    //     );
    //     // 汇总任务结果
    //     let mut data = HashMap::new();
    //     while let Some((i, res)) = rx.recv().await {
    //         // 如果重复计算就报错
    //         match data.entry(i) {
    //             Entry::Occupied(_) => {
    //                 panic!("数字 {i}，值为 {res} 重复计算")
    //             }
    //             Entry::Vacant(entry) => {
    //                 entry.insert(res);
    //             }
    //         }
    //         // 删除这行，因为在上面的match块中已经插入了数据
    //     }
    //     // 等待任务结束
    //     for handle in handles {
    //         handle.await.unwrap();
    //     }
    //     // 验证结果
    //     dbg!(&data);
    //     for i in 0..tasks.len {
    //         let index = tasks.get(i);
    //         assert_eq!((index, data.get(&index)), (index, Some(&fib_fast(index))));
    //     }
    // }

    // #[tokio::test]
    // async fn test_spawn3() {
    //     let tasks: Arc<TaskList> = Arc::new(vec![0..48].into());
    //     let (tx, mut rx) = mpsc::channel(100);
    //     let tasks_clone = tasks.clone();
    //     let handles = tasks.clone().spawn(
    //         8,
    //         |executor| tokio::spawn(async move { executor.run().await }),
    //         action::from_fn(async move |_, task, get_task| {
    //             loop {
    //                 while task.start() < task.end() {
    //                     let i = tasks_clone.get(task.start());
    //                     task.fetch_add_start(3);
    //                     tx.send((i, fib(i))).await.unwrap();
    //                     if i + 2 < task.end() {
    //                         tx.send((i + 1, fib(i + 1))).await.unwrap();
    //                         tx.send((i + 2, fib(i + 2))).await.unwrap();
    //                     } else if i + 1 < task.end() {
    //                         tx.send((i + 1, fib(i + 1))).await.unwrap();
    //                         task.fetch_sub_start(1);
    //                     } else {
    //                         task.fetch_sub_start(2);
    //                     }
    //                 }
    //                 if !get_task() {
    //                     break;
    //                 }
    //             }
    //         }),
    //     );
    //     // 汇总任务结果
    //     let mut data = HashMap::new();
    //     while let Some((i, res)) = rx.recv().await {
    //         // 如果重复计算就报错
    //         match data.entry(i) {
    //             Entry::Occupied(_) => {
    //                 panic!("数字 {i}，值为 {res} 重复计算")
    //             }
    //             Entry::Vacant(entry) => {
    //                 entry.insert(res);
    //             }
    //         }
    //     }
    //     // 等待任务结束
    //     for handle in handles {
    //         handle.await.unwrap();
    //     }
    //     // 验证结果
    //     dbg!(&data);
    //     for i in 0..tasks.len {
    //         let index = tasks.get(i);
    //         assert_eq!((index, data.get(&index)), (index, Some(&fib_fast(index))));
    //     }
    // }

    // #[tokio::test]
    // async fn test_spawn4() {
    //     let tasks: Arc<TaskList> = Arc::new(vec![0..48].into());
    //     let (tx, mut rx) = mpsc::channel(100);
    //     let tasks_clone = tasks.clone();
    //     let handles = tasks.clone().spawn(
    //         8,
    //         |executor| tokio::spawn(async move { executor.run().await }),
    //         action::from_fn(async move |_, task, get_task| {
    //             loop {
    //                 while task.start() < task.end() {
    //                     let i = tasks_clone.get(task.start());
    //                     task.fetch_add_start(4);
    //                     tx.send((i, fib(i))).await.unwrap();
    //                     if i + 3 < task.end() {
    //                         tx.send((i + 1, fib(i + 1))).await.unwrap();
    //                         tx.send((i + 2, fib(i + 2))).await.unwrap();
    //                         tx.send((i + 3, fib(i + 3))).await.unwrap();
    //                     } else if i + 2 < task.end() {
    //                         tx.send((i + 1, fib(i + 1))).await.unwrap();
    //                         tx.send((i + 2, fib(i + 2))).await.unwrap();
    //                         task.fetch_sub_start(1);
    //                     } else if i + 1 < task.end() {
    //                         tx.send((i + 1, fib(i + 1))).await.unwrap();
    //                         task.fetch_sub_start(2);
    //                     } else {
    //                         task.fetch_sub_start(3);
    //                     }
    //                 }
    //                 if !get_task() {
    //                     break;
    //                 }
    //             }
    //         }),
    //     );
    //     // 汇总任务结果
    //     let mut data = HashMap::new();
    //     while let Some((i, res)) = rx.recv().await {
    //         // 如果重复计算就报错
    //         match data.entry(i) {
    //             Entry::Occupied(_) => {
    //                 panic!("数字 {i}，值为 {res} 重复计算")
    //             }
    //             Entry::Vacant(entry) => {
    //                 entry.insert(res);
    //             }
    //         }
    //     }
    //     // 等待任务结束
    //     for handle in handles {
    //         handle.await.unwrap();
    //     }
    //     // 验证结果
    //     dbg!(&data);
    //     for i in 0..tasks.len {
    //         let index = tasks.get(i);
    //         assert_eq!((index, data.get(&index)), (index, Some(&fib_fast(index))));
    //     }
    // }
}
