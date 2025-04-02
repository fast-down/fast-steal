//! # fast-steal
//!
//! `fast-steal` 是一个特别快的多线程库，支持超细颗粒度的任务窃取。
//!
//! ## 优势
//!
//! 1. 只有一个依赖 `crossbeam-channel`
//! 2. 使用无锁消息队列，速度非常快
//! 3. 支持各种数字类型，以及各种实现了 `Send` + `Copy` + `Add<Output = Idx>` + `Sub<Output = Idx>` + `Mul<Output = Idx>` + `Div<Output = Idx>` + `Sum<Idx>` + `Ord` + `PartialEq` 的类型
//!
//! ## 使用方法
//!
//! ```rust
//! use fast_steal::{spawn::spawn, split_task::SplitTask, task::Task};
//! use std::collections::{HashMap, hash_map::Entry};
//!
//! fn fib(n: u128) -> u128 {
//!     match n {
//!         0 => 0,
//!         1 => 1,
//!         _ => fib(n - 1) + fib(n - 2),
//!     }
//! }
//!
//! fn main() {
//!     // 设定任务
//!     let tasks = vec![Task {
//!         start: 0u128,
//!         end: 44u128,
//!     }];
//!     // 切分任务
//!     let task_group = tasks.split_task(8);
//!     // 接受任务结果
//!     let (tx, rx) = crossbeam_channel::unbounded();
//!     let handle = spawn(task_group, move |rx_task, progress| {
//!         // 监听任务
//!         'task: for tasks in &rx_task {
//!             // 退出条件
//!             if tasks.is_empty() {
//!                 break;
//!             }
//!             // 业务逻辑
//!             for task in tasks {
//!                 for i in task.start..task.end {
//!                     // 任务窃取过程，必须放在任务前
//!                     if !rx_task.is_empty() {
//!                         continue 'task;
//!                     }
//!                     // 返回任务执行进度，必须放在任务前
//!                     progress(1);
//!                     println!("开始计算 {}", i);
//!                     // 任务执行
//!                     let res = fib(i);
//!                     // 任务结果发送
//!                     tx.send((i, res)).unwrap();
//!                 }
//!             }
//!         }
//!     });
//!     // 汇总任务结果
//!     let mut data = HashMap::new();
//!     for (i, res) in rx {
//!         // 如何重复计算就报错
//!         match data.entry(i) {
//!             Entry::Occupied(_) => panic!("数字 {i}，值为 {res} 重复计算"),
//!             Entry::Vacant(entry) => {
//!                 entry.insert(res);
//!             }
//!         }
//!     }
//!     // 等待任务结束
//!     handle.join().unwrap();
//!     // 验证结果
//!     dbg!(&data);
//!     for i in tasks[0].start..tasks.last().unwrap().end {
//!         assert_eq!((i, data.get(&i)), (i, Some(&fib(i))));
//!     }
//! }
//! ```
//!

mod get_remain;
pub mod spawn;
pub mod split_task;
pub mod task;
pub mod total;
mod worker;
