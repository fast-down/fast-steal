#![feature(test)] // Enable the test feature (required for benchmarking)

#[cfg(test)]
mod benches {
    extern crate test; // Import the test crate

    use std::hint::black_box;

    use num::Complex;
    use test::Bencher;

    fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
        let mut z = Complex { re: 0.0, im: 0.0 };
        for i in 0..limit {
            if z.norm_sqr() > 4.0 {
                return Some(i);
            }
            z = z * z + c;
        }

        None
    }

    const UL: Complex<f64> = Complex {
        re: -1f64,
        im: 0f64,
    };
    const LR: Complex<f64> = Complex {
        re: 1f64,
        im: 0.20f64,
    };
    const BOUNDS: (usize, usize) = (800, 600);

    fn pixel_to_point(
        bounds: (usize, usize),
        pixel: (usize, usize),
        upper_left: Complex<f64>,
        lower_right: Complex<f64>,
    ) -> Complex<f64> {
        let (width, height) = (
            lower_right.re - upper_left.re,
            upper_left.im - lower_right.im,
        );
        Complex {
            re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
            im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64, // Why subtraction here? pixel.1 increases as we go down,
                                                                           // but the imaginary component increases as we go up.
        }
    }

    // A benchmark test function.  The #[bench] attribute marks it as a benchmark.
    #[bench]
    fn bench_mandelbrot(b: &mut Bencher) {
        // Use the Bencher's iter method to run your benchmarked code repeatedly.
        b.iter(|| {
            for x in 0..BOUNDS.0 {
                for y in 0..BOUNDS.1 {
                    black_box(escape_time(
                        pixel_to_point(
                            BOUNDS,
                            (x, y),
                            UL,
                            LR,
                        ),
                        100,
                    ));
                }
            }
        });
    }

    #[bench]
    fn bench_mandelbrot_rayon(b: &mut Bencher) {
        use rayon::prelude::*;

        b.iter(|| {
            let mut pixels = vec![0; BOUNDS.0 * BOUNDS.1];

            fn render(pixels: &mut [u8],
                      bounds: (usize, usize),
                      upper_left: Complex<f64>,
                      lower_right: Complex<f64>)
            {
                assert!(pixels.len() == bounds.0 * bounds.1);

                for row in 0..bounds.1 {
                    for column in 0..bounds.0 {
                        let point = pixel_to_point(bounds, (column, row),
                                                   upper_left, lower_right);
                        pixels[row * bounds.0 + column] =
                            match escape_time(point, 255) {
                                None => 0,
                                Some(count) => 255 - count as u8
                            };
                    }
                }
            }

            {
                let bands: Vec<(usize, &mut [u8])> = pixels
                    .chunks_mut(BOUNDS.0)
                    .enumerate()
                    .collect();

                bands.into_par_iter()
                    .for_each(|(i, band)| {
                        let top = i;
                        let band_bounds = (BOUNDS.0, 1);
                        let band_upper_left = pixel_to_point(BOUNDS, (0, top),
                                                             UL, LR);
                        let band_lower_right = pixel_to_point(BOUNDS, (BOUNDS.0, top + 1),
                                                              UL, LR);
                        black_box(render(band, band_bounds, band_upper_left, band_lower_right));
                    });
            }
            black_box(pixels);
        });
    }

    #[bench]
    fn bench_mandelbrot_steal(b: &mut Bencher) {
        b.iter(|| {
            use fast_steal::{Spawn, TaskList, action};
            use std::{
                collections::{HashMap, hash_map::Entry},
                sync::{Arc, mpsc},
                thread,
            };

            let tasks: Arc<TaskList> = Arc::new(vec![0..BOUNDS.0].into());
            let (tx, rx) = mpsc::channel();
            let tasks_clone = tasks.clone();
            let handles = tasks.clone().spawn(
                24,
                |executor| thread::spawn(move || executor.run()),
                action::from_fn(move |id, task, refresh| {
                    // use `action::from_fn` for type inference
                    loop {
                        let mut results = Vec::with_capacity(36);
                        // 必须在每次循环开始判断 task.start() < task.end()，因为其他线程可能会修改 task
                        while task.start() < task.end() {
                            let i = tasks_clone.get(task.start());
                            // 提前更新进度，防止其他线程重复计算
                            task.fetch_add_start(1);
                            for y in 0..BOUNDS.1 {
                                let e = escape_time(
                                    pixel_to_point(
                                        BOUNDS,
                                        (i, y),
                                        UL,
                                        LR,
                                    ),
                                    100,
                                );
                                results.push((i, y, e));
                            }
                            for d in results.drain(..) {
                                tx.send(d).unwrap();
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
            for (x, y, res) in rx {
                // 如果重复计算就报错
                match data.entry((x, y)) {
                    Entry::Occupied(_) => {
                        panic!("数字 {:?}，值为 {res:?} 重复计算", (x, y))
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(res);
                    }
                }
                data.insert((x, y), res);
            }
            // 等待任务结束
            for handle in handles {
                handle.join().unwrap();
            }
        });
    }
}
