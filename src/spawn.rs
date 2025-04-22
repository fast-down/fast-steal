extern crate alloc;
use core::mem::ManuallyDrop;

use crate::{split_task::SplitTask, task::Task, task_list::TaskList};
use alloc::{sync::Arc, vec::Vec};
use crate::action::Action;
use crate::executor::Executor;

pub trait Spawn {
    fn spawn<S, R, F>(self, threads: usize, spawn: S, action: F) -> Vec<R>
    where
        S: Fn(Executor<F>) -> R,
        F: Action;
}

impl Spawn for Arc<TaskList> {
    fn spawn<S, R, F>(self, threads: usize, spawn: S, action: F) -> Vec<R>
    where
        S: Fn(Executor<F>) -> R,
        F: Action,
    {
        let bump = Arc::pin(bumpalo::Bump::with_capacity(threads * size_of::<Task>() + 10));
        let task_ptrs: Arc<[*const Task]> = Arc::from(
            Task::from(&*self)
                .split_task(threads)
                .map(|t| bump.alloc(t) as *const Task)
                .collect::<Vec<*const Task>>(),
        );
        let mutex = Arc::new(spin::Mutex::new(()));
        let mut handles = Vec::with_capacity(threads);
        for id in 0..threads {
            let task_ptrs = task_ptrs.clone();
            let action = action.clone();
            let mutex = mutex.clone();
            let bump = bump.clone();
            let handle = spawn(Executor {
                bump,
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
    use std::{
        collections::{HashMap, hash_map::Entry},
        dbg,
        sync::mpsc,
        thread, vec,
    };
    use crate::action;

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

    #[test]
    fn test_spawn() {
        let tasks: Arc<TaskList> = Arc::new(vec![0..48].into());
        let (tx, rx) = mpsc::channel();
        let tasks_clone = tasks.clone();
        let handles = tasks.clone().spawn(
            8,
            |executor| thread::spawn(move || executor.run()),
            action::from_fn(move |_, task, refresh| {
                loop {
                    while task.start() < task.end() {
                        let i = tasks_clone.get(task.start());
                        task.fetch_add_start(1);
                        tx.send((i, fib(i))).unwrap();
                    }
                    if !refresh() {
                        break;
                    }
                }
            }),
        );
        // 汇总任务结果
        let mut data = HashMap::new();
        for (i, res) in rx {
            // 如果重复计算就报错
            match data.entry(i) {
                Entry::Occupied(_) => {
                    panic!("数字 {i}，值为 {res} 重复计算")
                }
                Entry::Vacant(entry) => {
                    entry.insert(res);
                }
            }
            data.insert(i, res);
        }
        // 等待任务结束
        for handle in handles {
            handle.join().unwrap();
        }
        // 验证结果
        dbg!(&data);
        for i in 0..tasks.len {
            let index = tasks.get(i);
            assert_eq!((index, data.get(&index)), (index, Some(&fib_fast(index))));
        }
    }

    #[test]
    fn test_spawn2() {
        let tasks: Arc<TaskList> = Arc::new(vec![0..48].into());
        let (tx, rx) = mpsc::channel();
        let tasks_clone = tasks.clone();
        let handles = tasks.clone().spawn(
            8,
            |executor| thread::spawn(move || executor.run()),
            action::from_fn(move |_, task, get_task| {
                loop {
                    while task.start() < task.end() {
                        let i = tasks_clone.get(task.start());
                        task.fetch_add_start(2);
                        tx.send((i, fib(i))).unwrap();
                        if i + 1 < task.end() {
                            tx.send((i + 1, fib(i + 1))).unwrap();
                        } else {
                            task.fetch_sub_start(1);
                        }
                    }
                    if !get_task() {
                        break;
                    }
                }
            }),
        );
        // 汇总任务结果
        let mut data = HashMap::new();
        for (i, res) in rx {
            // 如果重复计算就报错
            match data.entry(i) {
                Entry::Occupied(_) => {
                    panic!("数字 {i}，值为 {res} 重复计算")
                }
                Entry::Vacant(entry) => {
                    entry.insert(res);
                }
            }
            data.insert(i, res);
        }
        // 等待任务结束
        for handle in handles {
            handle.join().unwrap();
        }
        // 验证结果
        dbg!(&data);
        for i in 0..tasks.len {
            let index = tasks.get(i);
            assert_eq!((index, data.get(&index)), (index, Some(&fib_fast(index))));
        }
    }

    #[test]
    fn test_spwan3() {
        let tasks: Arc<TaskList> = Arc::new(vec![0..48].into());
        let (tx, rx) = mpsc::channel();
        let tasks_clone = tasks.clone();
        let handles = tasks.clone().spawn(
            8,
            |executor| thread::spawn(move || executor.run()),
            action::from_fn(move |_, task, get_task| {
                loop {
                    while task.start() < task.end() {
                        let i = tasks_clone.get(task.start());
                        task.fetch_add_start(3);
                        tx.send((i, fib(i))).unwrap();
                        if i + 2 < task.end() {
                            tx.send((i + 1, fib(i + 1))).unwrap();
                            tx.send((i + 2, fib(i + 2))).unwrap();
                        } else if i + 1 < task.end() {
                            tx.send((i + 1, fib(i + 1))).unwrap();
                            task.fetch_sub_start(1);
                        } else {
                            task.fetch_sub_start(2);
                        }
                    }
                    if !get_task() {
                        break;
                    }
                }
            }),
        );
        // 汇总任务结果
        let mut data = HashMap::new();
        for (i, res) in rx {
            // 如果重复计算就报错
            match data.entry(i) {
                Entry::Occupied(_) => {
                    panic!("数字 {i}，值为 {res} 重复计算")
                }
                Entry::Vacant(entry) => {
                    entry.insert(res);
                }
            }
            data.insert(i, res);
        }
        // 等待任务结束
        for handle in handles {
            handle.join().unwrap();
        }
        // 验证结果
        dbg!(&data);
        for i in 0..tasks.len {
            let index = tasks.get(i);
            assert_eq!((index, data.get(&index)), (index, Some(&fib_fast(index))));
        }
    }

    #[test]
    fn test_spwan4() {
        let tasks: Arc<TaskList> = Arc::new(vec![0..48].into());
        let (tx, rx) = mpsc::channel();
        let tasks_clone = tasks.clone();
        let handles = tasks.clone().spawn(
            8,
            |executor| thread::spawn(move || executor.run()),
            action::from_fn(move |_, task, get_task| {
                loop {
                    while task.start() < task.end() {
                        let i = tasks_clone.get(task.start());
                        task.fetch_add_start(4);
                        tx.send((i, fib(i))).unwrap();
                        if i + 3 < task.end() {
                            tx.send((i + 1, fib(i + 1))).unwrap();
                            tx.send((i + 2, fib(i + 2))).unwrap();
                            tx.send((i + 3, fib(i + 3))).unwrap();
                        } else if i + 2 < task.end() {
                            tx.send((i + 1, fib(i + 1))).unwrap();
                            tx.send((i + 2, fib(i + 2))).unwrap();
                            task.fetch_sub_start(1);
                        } else if i + 1 < task.end() {
                            tx.send((i + 1, fib(i + 1))).unwrap();
                            task.fetch_sub_start(2);
                        } else {
                            task.fetch_sub_start(3);
                        }
                    }
                    if !get_task() {
                        break;
                    }
                }
            }),
        );
        // 汇总任务结果
        let mut data = HashMap::new();
        for (i, res) in rx {
            // 如果重复计算就报错
            match data.entry(i) {
                Entry::Occupied(_) => {
                    panic!("数字 {i}，值为 {res} 重复计算")
                }
                Entry::Vacant(entry) => {
                    entry.insert(res);
                }
            }
            data.insert(i, res);
        }
        // 等待任务结束
        for handle in handles {
            handle.join().unwrap();
        }
        // 验证结果
        dbg!(&data);
        for i in 0..tasks.len {
            let index = tasks.get(i);
            assert_eq!((index, data.get(&index)), (index, Some(&fib_fast(index))));
        }
    }
}
