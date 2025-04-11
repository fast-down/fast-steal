use core::ops::Range;
extern crate alloc;
use alloc::vec::Vec;

pub struct TaskList {
    tasks: Vec<Range<usize>>,
    start_point: Vec<usize>,
    pub len: usize,
}

impl From<Vec<Range<usize>>> for TaskList {
    fn from(tasks: Vec<Range<usize>>) -> Self {
        let mut len = 0;
        let mut start_point = Vec::with_capacity(tasks.len());
        for i in 0..tasks.len() {
            start_point.push(len);
            len += tasks[i].len()
        }
        Self {
            tasks,
            start_point,
            len,
        }
    }
}

impl TaskList {
    pub fn get(&self, index: usize) -> usize {
        let point = self.start_point.partition_point(|&x| x <= index) - 1;
        self.tasks[point].start + index - self.start_point[point]
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::vec;

    #[test]
    fn test_empty_list() {
        let tasks = TaskList::from(vec![]);
        assert_eq!(tasks.len, 0);
    }

    #[test]
    fn test_single_range() {
        let tasks = TaskList::from(vec![10..15]);
        assert_eq!(tasks.len, 5);

        assert_eq!(tasks.get(0), 10);
        assert_eq!(tasks.get(1), 11);
        assert_eq!(tasks.get(4), 14);
    }

    #[test]
    fn test_multiple_ranges() {
        let tasks = TaskList::from(vec![10..12, 20..25, 30..35]);
        assert_eq!(tasks.len, (12 - 10) + (25 - 20) + (35 - 30));

        // First range
        assert_eq!(tasks.get(0), 10);
        assert_eq!(tasks.get(1), 11);

        // Second range
        assert_eq!(tasks.get(2), 20);
        assert_eq!(tasks.get(3), 21);
        assert_eq!(tasks.get(6), 24);

        // Third range
        assert_eq!(tasks.get(7), 30);
        assert_eq!(tasks.get(11), 34);
    }

    #[test]
    fn test_consecutive_ranges() {
        let tasks = TaskList::from(vec![10..15, 15..20]);
        assert_eq!(tasks.len, 10);

        assert_eq!(tasks.get(0), 10);
        assert_eq!(tasks.get(4), 14);
        assert_eq!(tasks.get(5), 15);
        assert_eq!(tasks.get(9), 19);
    }

    #[test]
    fn test_non_contiguous_ranges() {
        let tasks = TaskList::from(vec![100..101, 200..203, 300..305]);
        assert_eq!(tasks.len, 1 + 3 + 5);

        assert_eq!(tasks.get(0), 100);
        assert_eq!(tasks.get(1), 200);
        assert_eq!(tasks.get(2), 201);
        assert_eq!(tasks.get(4), 300);
        assert_eq!(tasks.get(8), 304);
    }

    #[test]
    fn test_zero_length_ranges() {
        let tasks = TaskList::from(vec![10..10, 20..20, 30..35]);
        assert_eq!(tasks.len, 5);

        assert_eq!(tasks.get(0), 30);
        assert_eq!(tasks.get(4), 34);
    }
}
