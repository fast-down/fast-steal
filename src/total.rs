use crate::task::{Task, TaskGroup, Tasks};
use std::{iter::Sum, ops::Sub};

pub trait Total<Idx> {
    fn total(&self) -> Idx;
}

impl<Idx> Total<Idx> for Task<Idx>
where
    Idx: Clone + Sub<Output = Idx>,
{
    fn total(&self) -> Idx {
        self.end.clone() - self.start.clone()
    }
}

impl<Idx> Total<Idx> for Tasks<Idx>
where
    Idx: Clone + Sub<Output = Idx> + Sum<Idx>,
{
    fn total(&self) -> Idx {
        self.iter().map(|task| task.total()).sum()
    }
}

impl<Idx> Total<Idx> for TaskGroup<Idx>
where
    Idx: Clone + Sub<Output = Idx> + Sum<Idx>,
{
    fn total(&self) -> Idx {
        self.iter().map(|task| task.total()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_total() {
        let task = Task { start: 3, end: 5 };
        assert_eq!(task.total(), 2);

        let single_day_task = Task { start: 4, end: 5 };
        assert_eq!(single_day_task.total(), 1);

        let single_day_task = Task { start: 4, end: 4 };
        assert_eq!(single_day_task.total(), 0);
    }

    #[test]
    fn tasks_total() {
        let tasks = vec![Task { start: 3, end: 5 }, Task { start: 2, end: 6 }];
        assert_eq!(tasks.total(), 2 + 4);

        let empty_tasks: Tasks<i32> = vec![];
        assert_eq!(empty_tasks.total(), 0);
    }

    #[test]
    fn task_group_total() {
        let task_group = vec![
            vec![Task { start: 3, end: 5 }, Task { start: 2, end: 6 }],
            vec![Task { start: 1, end: 4 }],
        ];
        assert_eq!(task_group.total(), 2 + 4 + 3);

        let empty_task_group: TaskGroup<i32> = vec![];
        assert_eq!(empty_task_group.total(), 0);
    }
}
