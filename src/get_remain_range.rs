use crate::{
    task::{Task, Tasks},
    total::Total,
};
use std::{iter, ops::Sub};

pub trait GetRemainRange<Idx> {
    fn get_remain_range(&self, remain: Idx, skip: Idx) -> Tasks<Idx>;
}

impl<Idx> GetRemainRange<Idx> for Tasks<Idx>
where
    Idx: Copy + Sub<Output = Idx> + PartialOrd,
{
    fn get_remain_range(&self, mut remain: Idx, mut skip: Idx) -> Tasks<Idx> {
        let mut res = vec![];
        if skip >= remain {
            return res;
        }
        remain = remain - skip;
        let mut remain_task = None;
        let mut it = self.iter().rev();
        for task in &mut it {
            let total = task.total();
            if total <= skip {
                if total == skip {
                    break;
                }
                skip = skip - total;
            } else {
                remain_task = Some(Task {
                    start: task.start,
                    end: task.end - skip,
                });
                break;
            }
        }
        match remain_task {
            Some(task) => {
                for task in iter::once(&task).chain(it) {
                    let total = task.total();
                    if total <= remain {
                        res.push(task.clone());
                        if total == remain {
                            break;
                        }
                        remain = remain - total;
                    } else {
                        res.push(Task {
                            start: task.end - remain,
                            end: task.end,
                        });
                        break;
                    }
                }
            }
            None => {
                for task in it {
                    let total = task.total();
                    if total <= remain {
                        res.push(task.clone());
                        if total == remain {
                            break;
                        }
                        remain = remain - total;
                    } else {
                        res.push(Task {
                            start: task.end - remain,
                            end: task.end,
                        });
                        break;
                    }
                }
            }
        };
        res.reverse();
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_remain_empty() {
        let tasks = vec![];
        assert_eq!(tasks.get_remain_range(8, 1), vec![]);
        assert_eq!(tasks.get_remain_range(8, 0), vec![]);
    }

    #[test]
    fn test_get_remain() {
        let tasks = vec![
            Task { start: 0, end: 2 },
            Task { start: 2, end: 4 },
            Task { start: 4, end: 6 },
            Task { start: 6, end: 8 },
        ];
        assert_eq!(
            tasks.get_remain_range(5, 1),
            vec![
                Task { start: 3, end: 4 }, // 1
                Task { start: 4, end: 6 }, // 2
                Task { start: 6, end: 7 }, // 1
            ]
        );
        assert_eq!(
            tasks.get_remain_range(5, 0),
            vec![
                Task { start: 3, end: 4 }, // 1
                Task { start: 4, end: 6 }, // 2
                Task { start: 6, end: 8 }, // 2
            ]
        );
        assert_eq!(
            tasks.get_remain_range(5, 2),
            vec![
                Task { start: 3, end: 4 }, // 1
                Task { start: 4, end: 6 }, // 2
            ]
        );
        assert_eq!(
            tasks.get_remain_range(5, 3),
            vec![
                Task { start: 3, end: 4 }, // 1
                Task { start: 4, end: 5 }, // 1
            ]
        );
        assert_eq!(
            tasks.get_remain_range(5, 4),
            vec![
                Task { start: 3, end: 4 }, // 1
            ]
        );
        assert_eq!(tasks.get_remain_range(5, 5), vec![]);
        assert_eq!(tasks.get_remain_range(5, 6), vec![]);
        assert_eq!(
            tasks.get_remain_range(4, 2),
            vec![Task { start: 4, end: 6 }] // 2
        );
        assert_eq!(
            tasks.get_remain_range(4, 3),
            vec![Task { start: 4, end: 5 }] // 1
        );
    }
}
