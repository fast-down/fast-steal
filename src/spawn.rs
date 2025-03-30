use crate::{
    get_remain::GetRemain,
    split_task::SplitTask,
    task::{TaskGroup, Tasks},
    total::Total,
    worker::Worker,
};
use std::{
    fmt::Debug,
    iter::Sum,
    ops::{Add, Div, Mul, Sub},
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
        + Debug
        + 'static,
    F: FnOnce(crossbeam_channel::Receiver<Tasks<Idx>>, &dyn Fn(Idx)) + Send + Clone + 'static,
{
    thread::spawn(move || {
        thread::scope(|s| {
            let mut workers = Vec::with_capacity(task_group.len());
            let (tx_progress, rx_progress) = crossbeam_channel::unbounded();
            for (id, tasks) in task_group.into_iter().enumerate() {
                let (tx_task, rx_task) = crossbeam_channel::unbounded();
                let tx_progress = tx_progress.clone();
                let action = action.clone();
                s.spawn(move || {
                    action(rx_task, &|reduce| {
                        tx_progress.send((id, reduce)).unwrap();
                    });
                });
                tx_task.send(tasks.clone()).unwrap();
                workers.push(Worker {
                    tx_task,
                    remain: tasks.total(),
                    tasks,
                });
            }
            s.spawn(move || {
                for (id, reduce) in &rx_progress {
                    if workers[id].remain > reduce {
                        workers[id].remain = workers[id].remain - reduce;
                        continue;
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
                        continue;
                    }
                    let max_remain_tasks = workers[max_pos].tasks.get_remain(max_remain);
                    let split = max_remain_tasks.split_task(two);
                    let prev = split[0].clone();
                    let next = split[1].clone();
                    workers[max_pos].remain = prev.total();
                    workers[id].remain = next.total();
                    workers[max_pos].tasks = prev;
                    workers[id].tasks = next;
                    println!(
                        "线程 {} 从线程 {} 窃取任务 {:?}",
                        id, max_pos, workers[id].tasks
                    );
                    workers[id].tx_task.send(workers[id].tasks.clone()).unwrap();
                }
                for _ in rx_progress {}
            });
        });
    })
}
