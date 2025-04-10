use crate::{
    split_task::{self, SplitTask},
    task::Task,
    task_list::TaskList,
};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

pub trait Spawn {
    fn spawn<F>(self, threads: usize, action: F) -> JoinHandle<()>
    where
        F: FnOnce(Arc<Task>) + Send + Clone + 'static;
}

impl Spawn for TaskList {
    fn spawn<F>(self, threads: usize, action: F) -> JoinHandle<()>
    where
        F: FnOnce(Arc<Task>) + Send + Clone + 'static,
    {
        thread::spawn(move || {
            thread::scope(|s| {
                let tasks: Arc<Vec<Arc<Task>>> = Arc::new(
                    Task::from(&self)
                        .split_task(threads)
                        .map(|t| Arc::new(t))
                        .collect(),
                );
                for id in 0..threads {
                    let tasks = tasks.clone();
                    let action = action.clone();
                    s.spawn(move || action(tasks[id].clone()));
                }
            });
        })
    }
}
// action(
//     tasks
// rx_task,
// id,
// &|reduce| {
//     let mut workers = workers.lock().unwrap();
//     workers[id].occupy = workers[id].occupy + reduce;
//     if workers[id].occupy > workers[id].remain {
//         workers[id].occupy = workers[id].remain;
//     }
// },
// &|reduce| {
//     let mut workers = workers.lock().unwrap();
//     workers[id].occupy = zero;
//     if workers[id].remain > reduce {
//         workers[id].remain = workers[id].remain - reduce;
//         return;
//     }
//     workers[id].remain = zero;
//     // 找出最大的剩余任务数
//     let (max_pos, max_remain_without_occupy) = workers
//         .iter()
//         .enumerate()
//         .map(|(i, w)| (i, w.remain - w.occupy))
//         .max_by_key(|(_, remain)| *remain)
//         .unwrap();
//     if max_remain_without_occupy < two {
//         workers[id].tx_task.send(vec![]).unwrap();
//         return;
//     }
//     let split = workers[max_pos]
//         .tasks
//         .get_remain(max_remain_without_occupy)
//         .split_task(two);
//     let next = split[1].clone();
//     workers[id].remain = next.total();
//     workers[id].tasks = next;
//     workers[max_pos].tasks = workers[max_pos]
//         .tasks
//         .get_remain_range(workers[max_pos].remain, workers[id].remain);
//     workers[max_pos].remain =
//         workers[max_pos].remain - workers[id].remain;
//     workers[id].tx_task.send(workers[id].tasks.clone()).unwrap();
// },
// );

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::collections::{HashMap, hash_map::Entry};

//     fn fib(n: u128) -> u128 {
//         match n {
//             0 => 0,
//             1 => 1,
//             _ => fib(n - 1) + fib(n - 2),
//         }
//     }

//     fn fib_fast(n: u128) -> u128 {
//         let mut a = 0;
//         let mut b = 1;
//         for _ in 0..n {
//             (a, b) = (b, a + b);
//         }
//         a
//     }

//     #[test]
//     fn test_spawn() {
//         let tasks = vec![(0..44).into()];
//         let task_group = tasks.split_task(8);
//         let (tx, rx) = crossbeam_channel::unbounded();
//         let handle = task_group.spawn(move |rx_task, id, occupy, finish| {
//             println!("线程 {id} 启动");
//             'task: for tasks in &rx_task {
//                 if tasks.is_empty() {
//                     break;
//                 }
//                 for task in tasks {
//                     for i in task.start..task.end {
//                         if !rx_task.is_empty() {
//                             continue 'task;
//                         }
//                         occupy(1);
//                         let res = fib(i);
//                         finish(1);
//                         tx.send((i, res)).unwrap();
//                     }
//                 }
//             }
//         });
//         let mut data = HashMap::new();
//         for (i, res) in rx {
//             match data.entry(i) {
//                 Entry::Occupied(_) => panic!("数字 {i}，值为 {res} 重复计算"),
//                 Entry::Vacant(entry) => {
//                     entry.insert(res);
//                 }
//             }
//         }
//         handle.join().unwrap();
//         dbg!(&data);
//         for i in tasks[0].start..tasks.last().unwrap().end {
//             assert_eq!((i, data.get(&i)), (i, Some(&fib_fast(i))));
//         }
//     }
// }
