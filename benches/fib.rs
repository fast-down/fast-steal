#![feature(test)] // Enable the test feature (required for benchmarking)


#[cfg(test)]
mod benches {
    extern crate test; // Import the test crate

    use std::hint::black_box;

    use test::Bencher;

    // The function you want to benchmark
    fn fibonacci(n: usize) -> usize {
        match n {
            0 => 0,
            1 => 1,
            _ => fibonacci(n - 1) + fibonacci(n - 2),
        }
    }

    // A benchmark test function.  The #[bench] attribute marks it as a benchmark.
    #[bench]
    fn bench_fibonacci_20(b: &mut Bencher) {
        // Use the Bencher's iter method to run your benchmarked code repeatedly.
        b.iter(|| {
            for i in 0..36 {
                black_box(fibonacci(i));
            }
        });
    }

    #[bench]
    fn bench_fibonacci_20_steal(b: &mut Bencher) {
        b.iter(|| {
            use fast_steal::{Spawn, TaskList, action};
            use std::{
                collections::{HashMap, hash_map::Entry},
                sync::{Arc, mpsc},
                thread,
            };
            let tasks: Arc<TaskList> = Arc::new(vec![0..36].into());
            let (tx, rx) = mpsc::channel();
            let tasks_clone = tasks.clone();
            let handles = tasks.clone().spawn(
                8,
                |executor| thread::spawn(move || executor.run()),
                action::from_fn(move |id, task, refresh| { // use `action::from_fn` for type inference
                    loop {
                        const BATCH_COUNT: usize = 6;
                        let mut results = Vec::with_capacity(BATCH_COUNT);
                        // 必须在每次循环开始判断 task.start() < task.end()，因为其他线程可能会修改 task
                        while task.start() < task.end() {
                            let i = tasks_clone.get(task.start());
                            // 提前更新进度，防止其他线程重复计算
                            task.fetch_add_start(1);
                            // 计算
                            results.push((i, fibonacci(i)));
                            if results.len() >= BATCH_COUNT {
                                for d in results.drain(..) {
                                    tx.send(d).unwrap();
                                }
                            }
                        }

                        for d in results {
                            tx.send(d).unwrap();
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
        });
    }
}
