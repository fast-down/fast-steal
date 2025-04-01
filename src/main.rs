use fast_steal::{spawn::spawn, split_task::SplitTask, task::Task};
use std::collections::{HashMap, hash_map::Entry};

fn fib(n: u128) -> u128 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}

fn main() {
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
                    println!("开始计算 {}", i);
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
