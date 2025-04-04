use std::ops::Range;

/// 这是一个左闭右开的区间 [start, end)
#[derive(Clone, Debug, PartialEq)]
pub struct Task<Idx> {
    pub start: Idx,
    pub end: Idx,
}
pub type Tasks<Idx> = Vec<Task<Idx>>;
pub type TaskGroup<Idx> = Vec<Tasks<Idx>>;

impl<Idx: Clone> Task<Idx> {
    pub fn split_task(&self, point: Idx) -> (Self, Self) {
        (
            Task {
                start: self.start.clone(),
                end: point.clone(),
            },
            Task {
                start: point,
                end: self.end.clone(),
            },
        )
    }
}

impl<Idx: PartialOrd> Task<Idx> {
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

impl<Idx> From<Range<Idx>> for Task<Idx> {
    fn from(range: Range<Idx>) -> Self {
        Task {
            start: range.start,
            end: range.end,
        }
    }
}

#[cfg(test)]
mod tests_split_task {
    use super::*;

    #[test]
    fn test_split_task() {
        let task = Task { start: 0, end: 10 };
        let (task1, task2) = task.split_task(5);
        assert_eq!(task1, Task { start: 0, end: 5 });
        assert_eq!(task2, Task { start: 5, end: 10 });
    }

    #[test]
    fn test_split_task_out_of_bounds() {
        let task = Task { start: 0, end: 10 };
        let (task1, task2) = task.split_task(15);
        assert_eq!(task1, Task { start: 0, end: 15 });
        assert_eq!(task2, Task { start: 15, end: 10 });
    }

    #[test]
    fn test_split_task_at_start() {
        let task = Task { start: 0, end: 10 };
        let (task1, task2) = task.split_task(0);
        assert_eq!(task1, Task { start: 0, end: 0 });
        assert_eq!(task2, Task { start: 0, end: 10 });
    }

    #[test]
    fn test_split_task_at_end() {
        let task = Task { start: 0, end: 10 };
        let (task1, task2) = task.split_task(10);
        assert_eq!(task1, Task { start: 0, end: 10 });
        assert_eq!(task2, Task { start: 10, end: 10 });
    }
}

#[cfg(test)]
mod tests_is_empty {
    use super::*;

    #[test]
    fn test_is_empty() {
        // 测试空区间
        let range1 = Task { start: 5, end: 5 };
        assert_eq!(range1.is_empty(), true);

        // 测试非空区间
        let range2 = Task { start: 1, end: 5 };
        assert_eq!(range2.is_empty(), false);

        // 测试负数区间
        let range3 = Task { start: -5, end: -1 };
        assert_eq!(range3.is_empty(), false);

        // 测试负数空区间
        let range4 = Task { start: -1, end: -1 };
        assert_eq!(range4.is_empty(), true);

        // 测试反向区间
        let range5 = Task { start: 5, end: 1 };
        assert_eq!(range5.is_empty(), true);
    }
}

#[cfg(test)]
mod tests_from_range {
    use super::*;

    #[test]
    fn converts_range_to_task() {
        let range = 5..8;
        let task = Task::from(range);
        assert_eq!(task.start, 5);
        assert_eq!(task.end, 8);
    }

    #[test]
    fn converts_range_to_task_with_negative_start() {
        let range = -5..-1;
        let task = Task::from(range);
        assert_eq!(task.start, -5);
        assert_eq!(task.end, -1);
    }

    #[test]
    fn converts_empty_range_to_task() {
        let range = 5..5;
        let task = Task::from(range);
        assert_eq!(task.start, 5);
        assert_eq!(task.end, 5);
    }

    #[test]
    fn converts_range_to_task_with_negative_end() {
        let range = 5..-1;
        let task = Task::from(range);
        assert_eq!(task.start, 5);
        assert_eq!(task.end, -1);
    }
}
