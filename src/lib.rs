//! # fast-steal 神偷
//!
//! ![GitHub last commit (branch)](https://img.shields.io/github/last-commit/share121/fast-steal/master)
//! [![Rust](https://github.com/share121/fast-steal/workflows/Test/badge.svg)](https://github.com/share121/fast-steal/actions)
//! [![Latest version](https://img.shields.io/crates/v/fast-steal.svg)](https://crates.io/crates/fast-steal)
//! [![Documentation](https://docs.rs/fast-steal/badge.svg)](https://docs.rs/fast-steal)
//! ![License](https://img.shields.io/crates/l/fast-steal.svg)
//!
//! `fast-steal` 是一个特别快的多线程库，支持超细颗粒度的任务窃取。
//!
//! ## 优势
//!
//! 1. 零依赖
//! 2. 无锁
//! 3. 零拷贝
//! 4. 安全的 Rust 代码
//! 5. no_std 支持，不依赖于标准库
//! 6. 超细颗粒度任务窃取，速度非常快
//!
//! ```rust
//! use fast_steal::{spawn::Spawn, task_list::TaskList};
//! use std::{
//!     collections::{HashMap, hash_map::Entry},
//!     sync::{Arc, mpsc},
//!     thread,
//! };
//!
//! fn fib(n: usize) -> usize {
//!     match n {
//!         0 => 0,
//!         1 => 1,
//!         _ => fib(n - 1) + fib(n - 2),
//!     }
//! }
//!
//! fn fib_fast(n: usize) -> usize {
//!     let mut a = 0;
//!     let mut b = 1;
//!     for _ in 0..n {
//!         (a, b) = (b, a + b);
//!     }
//!     a
//! }
//!
//! fn main() {
//!     let tasks: Arc<TaskList> = Arc::new(vec![0..48].into());
//!     let (tx, rx) = mpsc::channel();
//!     let tasks_clone = tasks.clone();
//!     let handles = tasks.clone().spawn(
//!         8,
//!         |closure| thread::spawn(move || closure()),
//!         move |task, get_task| {
//!             loop {
//!                 while task.start() < task.end() {
//!                     let i = tasks_clone.get(task.start());
//!                     tx.send((i, fib(i))).unwrap();
//!                     task.fetch_start(1);
//!                 }
//!                 if !get_task() {
//!                     break;
//!                 }
//!             }
//!         },
//!     );
//!     // 汇总任务结果
//!     let mut data = HashMap::new();
//!     for (i, res) in rx {
//!         // 如果重复计算就报错
//!         match data.entry(i) {
//!             Entry::Occupied(_) => {
//!                 panic!("数字 {i}，值为 {res} 重复计算")
//!             }
//!             Entry::Vacant(entry) => {
//!                 entry.insert(res);
//!             }
//!         }
//!         data.insert(i, res);
//!     }
//!     // 等待任务结束
//!     for handle in handles {
//!         handle.join().unwrap();
//!     }
//!     // 验证结果
//!     dbg!(&data);
//!     for i in 0..tasks.len {
//!         let index = tasks.get(i);
//!         assert_eq!((index, data.get(&index)), (index, Some(&fib_fast(index))));
//!     }
//! }
//! ```

#![no_std]
pub mod spawn;
pub mod split_task;
pub mod task;
pub mod task_list;
