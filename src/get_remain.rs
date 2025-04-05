use crate::{
    task::{Task, Tasks},
    total::Total,
};
use std::ops::Sub;

pub trait GetRemain<Idx> {
    fn get_remain(&self, remain: Idx) -> Tasks<Idx>;
}

impl<Idx> GetRemain<Idx> for Tasks<Idx>
where
    Idx: Copy + Sub<Output = Idx> + PartialOrd,
{
    fn get_remain(&self, mut remain: Idx) -> Tasks<Idx> {
        let mut res = vec![];
        for task in self.iter().rev() {
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
        res.reverse();
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_remain_empty() {
        let tasks = Tasks::new();
        assert_eq!(tasks.get_remain(0), Tasks::new());
        assert_eq!(tasks.get_remain(10), Tasks::new());
    }

    #[test]
    fn test_get_remain_single_task() {
        let mut tasks = Tasks::new();
        tasks.push(Task { start: 0, end: 5 });

        let result = tasks.get_remain(5);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Task { start: 0, end: 5 });

        let result = tasks.get_remain(10);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Task { start: 0, end: 5 });
    }

    #[test]
    fn test_get_remain_multiple_tasks() {
        let mut tasks = Tasks::new();
        tasks.push(Task { start: 0, end: 5 });
        tasks.push(Task { start: 5, end: 10 });
        tasks.push(Task { start: 10, end: 15 });

        let result = tasks.get_remain(5);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Task { start: 10, end: 15 });

        let result = tasks.get_remain(10);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Task { start: 5, end: 10 });
        assert_eq!(result[1], Task { start: 10, end: 15 });

        let result = tasks.get_remain(15);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Task { start: 0, end: 5 });
        assert_eq!(result[1], Task { start: 5, end: 10 });
        assert_eq!(result[2], Task { start: 10, end: 15 });

        let result = tasks.get_remain(20);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Task { start: 0, end: 5 });
        assert_eq!(result[1], Task { start: 5, end: 10 });
        assert_eq!(result[2], Task { start: 10, end: 15 });
    }

    #[test]
    fn test_get_remain_partial_task() {
        let mut tasks = Tasks::new();
        tasks.push(Task { start: 0, end: 5 });
        tasks.push(Task { start: 5, end: 10 });
        tasks.push(Task { start: 10, end: 15 });

        let result = tasks.get_remain(8);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Task { start: 7, end: 10 });
        assert_eq!(result[1], Task { start: 10, end: 15 });

        let result = tasks.get_remain(12);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Task { start: 3, end: 5 });
        assert_eq!(result[1], Task { start: 5, end: 10 });
        assert_eq!(result[2], Task { start: 10, end: 15 });
    }
}
