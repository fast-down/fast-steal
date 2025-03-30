use crate::{
    task::{Task, TaskGroup, Tasks},
    total::Total,
};
use std::{
    iter::Sum,
    ops::{Add, Div, Mul, Sub},
};

pub trait SplitTask<Idx> {
    fn split_task(&self, n: Idx) -> TaskGroup<Idx>;
}

impl<Idx> SplitTask<Idx> for Tasks<Idx>
where
    Idx: Copy
        + PartialOrd
        + Add<Output = Idx>
        + Sub<Output = Idx>
        + Mul<Output = Idx>
        + Div<Output = Idx>
        + Sum<Idx>,
{
    fn split_task(&self, n: Idx) -> TaskGroup<Idx> {
        let one = n / n;
        let zero = n - n;

        let total = self.total();
        let per_group = total / n;
        let remainder = total - per_group * n;

        let mut task_group = vec![];
        let mut iter = self.iter();

        let mut i = zero;
        let mut prev_remain_task: Option<Task<Idx>> = None;
        while i < n {
            let mut tasks = vec![];
            let mut remain = if i < remainder {
                per_group + one
            } else {
                per_group
            };
            if let Some(task) = prev_remain_task.take() {
                let size = task.total();
                if size > remain {
                    let (prev, curr) = task.split_task(task.start + remain);
                    prev_remain_task = Some(curr);
                    tasks.push(prev);
                    task_group.push(tasks);
                    i = i + one;
                    continue;
                } else {
                    remain = remain - size;
                    tasks.push(task.clone());
                }
            }
            loop {
                let task = iter.next();
                match task {
                    None => break,
                    Some(task) => {
                        let size = task.total();
                        if size > remain {
                            let (prev, curr) = task.split_task(task.start + remain);
                            prev_remain_task = Some(curr);
                            if !prev.is_empty() {
                                tasks.push(prev);
                            }
                            break;
                        } else {
                            remain = remain - size;
                            tasks.push(task.clone());
                        }
                    }
                }
            }
            task_group.push(tasks);
            i = i + one;
        }

        task_group
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_evenly_divisible() {
        let tasks = vec![
            Task { start: 0, end: 2 }, // total 2
            Task { start: 2, end: 6 }, // total 4
        ];
        let groups = tasks.split_task(3); // total 6, 6/3=2 per group

        assert_eq!(
            groups,
            vec![
                vec![Task { start: 0, end: 2 }],
                vec![Task { start: 2, end: 4 }],
                vec![Task { start: 4, end: 6 }],
            ]
        );
    }

    #[test]
    fn split_with_remainder() {
        let tasks = vec![
            Task { start: 0, end: 5 }, // total 5
        ];
        let groups = tasks.split_task(3); // total 5, 5/3=1 per group + rem 2

        assert_eq!(
            groups,
            vec![
                vec![Task { start: 0, end: 2 }],
                vec![Task { start: 2, end: 4 }],
                vec![Task { start: 4, end: 5 }]
            ]
        );
    }

    #[test]
    fn split_single_large_task() {
        let tasks = vec![Task { start: 0, end: 10 }];
        let groups = tasks.split_task(3); // total 10, 10/3=3 rem 1

        assert_eq!(
            groups,
            vec![
                vec![Task { start: 0, end: 4 }],
                vec![Task { start: 4, end: 7 }],
                vec![Task { start: 7, end: 10 }]
            ]
        );
    }

    #[test]
    fn split_empty_tasks() {
        let tasks: Tasks<i32> = vec![];
        let groups = tasks.split_task(5);
        assert_eq!(groups, vec![vec![], vec![], vec![], vec![], vec![]]);
    }

    #[test]
    fn split_into_one_group() {
        let tasks = vec![Task { start: 0, end: 5 }, Task { start: 5, end: 8 }];
        let groups = tasks.split_task(1); // total 8

        assert_eq!(
            groups,
            vec![vec![Task { start: 0, end: 5 }, Task { start: 5, end: 8 }]]
        );
    }

    #[test]
    fn split_with_multiple_remainders() {
        let tasks = vec![
            Task { start: 0, end: 3 }, // total 3
            Task { start: 3, end: 7 }, // total 4
        ];
        let groups = tasks.split_task(4); // total 7, 7/4=1 rem 3

        assert_eq!(
            groups,
            vec![
                vec![Task { start: 0, end: 2 }],
                vec![Task { start: 2, end: 3 }, Task { start: 3, end: 4 }],
                vec![Task { start: 4, end: 6 }],
                vec![Task { start: 6, end: 7 }],
            ]
        )
    }
}
