# fast-steal 神偷

![GitHub last commit (branch)](https://img.shields.io/github/last-commit/share121/fast-steal/master)
[![Rust](https://github.com/share121/fast-steal/workflows/Test/badge.svg)](https://github.com/share121/fast-steal/actions)
[![Latest version](https://img.shields.io/crates/v/fast-steal.svg)](https://crates.io/crates/fast-steal)
[![Documentation](https://docs.rs/fast-steal/badge.svg)](https://docs.rs/fast-steal)
![License](https://img.shields.io/crates/l/fast-steal.svg)

`fast-steal` 是一个特别快的多线程库，支持超细颗粒度的任务窃取。

## 优势

1. no_std 支持，不依赖于标准库
2. 超细颗粒度任务窃取，速度非常快

```rust
use fast_steal::{sync::Spawn, TaskList, sync::action};
use std::{
    collections::{HashMap, hash_map::Entry},
    sync::{Arc, mpsc},
    thread,
};

fn fib(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}

fn fib_fast(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        (a, b) = (b, a + b);
    }
    a
}

fn main() {
    let tasks: Arc<TaskList> = Arc::new(vec![0..48].into());
    let (tx, rx) = mpsc::channel();
    let tasks_clone = tasks.clone();
    let handles = tasks.clone().spawn(
        8,
        |executor| thread::spawn(move || executor.run()),
        action::from_fn(move |id, task, refresh| { // use `action::from_fn` for type inference
            loop {
                // 必须在每次循环开始判断 task.start() < task.end()，因为其他线程可能会修改 task
                while task.start() < task.end() {
                    let i = tasks_clone.get(task.start());
                    // 提前更新进度，防止其他线程重复计算
                    task.fetch_add_start(1);
                    // 计算
                    tx.send((i, fib(i))).unwrap();
                }
                // 检查是否还有任务
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
```
