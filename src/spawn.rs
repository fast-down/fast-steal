use crate::{
    get_remain::GetRemain,
    split_task::SplitTask,
    task::{TaskGroup, Tasks},
    total::Total,
    worker::Worker,
};
use std::{
    iter::Sum,
    ops::{Add, Div, Mul, Sub},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub fn spawn<Idx, F>(task_group: TaskGroup<Idx>, action: F) -> JoinHandle<()>
where
    Idx: Send
        + Copy
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + Mul<Output = Idx>
        + Div<Output = Idx>
        + Sum<Idx>
        + Ord
        + 'static,
    F: FnOnce(crossbeam_channel::Receiver<Tasks<Idx>>, &dyn Fn(Idx)) + Send + Clone + 'static,
{
    thread::spawn(move || {
        thread::scope(|s| {
            let workers: Arc<Mutex<Vec<Worker<Idx>>>> =
                Arc::new(Mutex::new(Vec::with_capacity(task_group.len())));
            for (id, tasks) in task_group.into_iter().enumerate() {
                let (tx_task, rx_task) = crossbeam_channel::unbounded();
                let action = action.clone();
                tx_task.send(tasks.clone()).unwrap();
                workers.lock().unwrap().push(Worker {
                    tx_task,
                    remain: tasks.total(),
                    tasks,
                });
                let workers = workers.clone();
                s.spawn(move || {
                    action(rx_task, &|reduce| {
                        let mut workers = workers.lock().unwrap();
                        if workers[id].remain > reduce {
                            workers[id].remain = workers[id].remain - reduce;
                            return;
                        }
                        let one = reduce / reduce;
                        workers[id].remain = one - one;
                        // 找出最大的剩余任务数
                        let (max_pos, max_remain) = workers
                            .iter()
                            .enumerate()
                            .map(|(i, w)| (i, w.remain))
                            .max_by_key(|(_, remain)| *remain)
                            .unwrap();
                        let two = one + one;
                        if max_remain < two {
                            workers[id].tx_task.send(vec![]).unwrap();
                            return;
                        }
                        let split = workers[max_pos]
                            .tasks
                            .get_remain(max_remain)
                            .split_task(two);
                        let prev = split[0].clone();
                        let next = split[1].clone();
                        workers[id].remain = next.total();
                        workers[id].tasks = next;
                        workers[max_pos].remain = workers[max_pos].remain - workers[id].remain;
                        workers[max_pos].tasks = prev;
                        workers[id].tx_task.send(workers[id].tasks.clone()).unwrap();
                    });
                });
            }
        });
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;
    use std::collections::{HashMap, hash_map::Entry};

    fn fib(n: u128) -> u128 {
        match n {
            0 => 0,
            1 => 1,
            _ => fib(n - 1) + fib(n - 2),
        }
    }

    #[test]
    fn test_spawn() {
        let tasks = vec![Task {
            start: 0u128,
            end: 44u128,
        }];
        let task_group = tasks.split_task(8);
        let (tx, rx) = crossbeam_channel::unbounded();
        let handle = spawn(task_group, move |rx_task, progress| {
            'task: for tasks in &rx_task {
                if tasks.is_empty() {
                    break;
                }
                for task in tasks {
                    for i in task.start..task.end {
                        if !rx_task.is_empty() {
                            continue 'task;
                        }
                        progress(1);
                        let res = fib(i);
                        tx.send((i, res)).unwrap();
                    }
                }
            }
        });
        let mut data = HashMap::new();
        for (i, res) in rx {
            match data.entry(i) {
                Entry::Occupied(_) => panic!("数字 {i}，值为 {res} 重复计算"),
                Entry::Vacant(entry) => {
                    entry.insert(res);
                }
            }
        }
        handle.join().unwrap();
        dbg!(&data);
        for i in tasks[0].start..tasks.last().unwrap().end {
            assert_eq!((i, data.get(&i)), (i, Some(&fib(i))));
        }
    }
}
