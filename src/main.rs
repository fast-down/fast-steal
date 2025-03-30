use fast_steal::{spawn, split_task::SplitTask, task::Task};

fn main() {
    let tasks = vec![Task {
        start: 0u128,
        end: 46u128,
    }];
    let task_group = tasks.split_task(7);
    let (tx, rx) = crossbeam_channel::unbounded();
    let handle = spawn::spawn(task_group, move |rx_task, progress| {
        'task: for tasks in &rx_task {
            if tasks.is_empty() {
                break;
            }
            for task in tasks {
                for i in task.start..task.end {
                    if !rx_task.is_empty() {
                        continue 'task;
                    }
                    let res = fib(i);
                    if !rx_task.is_empty() {
                        continue 'task;
                    }
                    progress(1);
                    tx.send((i, res)).unwrap();
                }
            }
        }
    });
    for (i, res) in rx {
        println!("{}: {}", i, res);
    }
    handle.join().unwrap();
}

fn fib(n: u128) -> u128 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}
