extern crate alloc;
use crate::{split_task::SplitTask, task::Task, task_list::TaskList};
use alloc::{boxed::Box, sync::Arc, vec::Vec};

pub trait Spawn {
    fn spawn<S, R, F>(self, threads: usize, spawn: S, action: F) -> Vec<R>
    where
        S: Fn(Box<dyn FnOnce() + Send + 'static>) -> R,
        F: FnOnce(Arc<Task>, &dyn Fn() -> bool) + Send + Clone + 'static;
}

impl Spawn for Arc<TaskList> {
    fn spawn<S, R, F>(self, threads: usize, spawn: S, action: F) -> Vec<R>
    where
        S: Fn(Box<dyn FnOnce() + Send + 'static>) -> R,
        F: FnOnce(Arc<Task>, &dyn Fn() -> bool) + Send + Clone + 'static,
    {
        let tasks: Arc<Vec<Arc<Task>>> = Arc::new(
            Task::from(self.as_ref())
                .split_task(threads)
                .map(|t| Arc::new(t))
                .collect(),
        );
        let mut handles = Vec::with_capacity(threads);
        for id in 0..threads {
            let tasks = tasks.clone();
            let action = action.clone();
            let handle = spawn(Box::new(move || {
                action(tasks[id].clone(), &|| {
                    let (max_pos, max_remain) = tasks
                        .iter()
                        .enumerate()
                        .map(|(i, w)| (i, w.remain()))
                        .max_by_key(|(_, remain)| *remain)
                        .unwrap();
                    if max_remain < 2 {
                        return false;
                    }
                    let (start, end) = tasks[max_pos].split_two();
                    tasks[id].set_start(start);
                    tasks[id].set_end(end);
                    true
                })
            }));
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
            |closure| thread::spawn(move || closure()),
            move |task, get_task| {
                loop {
                    while task.start() < task.end() {
                        let i = tasks_clone.get(task.start());
                        task.fetch_add_start(1);
                        tx.send((i, fib(i))).unwrap();
                    }
                    if !get_task() {
                        break;
                    }
                }
            },
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
            |closure| thread::spawn(move || closure()),
            move |task, get_task| {
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
            },
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
