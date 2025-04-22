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
    pub fn position(&self, index: usize) -> usize {
        self.start_point.partition_point(|&x| x <= index) - 1
    }

    pub fn get(&self, index: usize) -> usize {
        let point = self.position(index);
        self.tasks[point].start + index - self.start_point[point]
    }

    pub fn get_range(&self, range: Range<usize>) -> Vec<Range<usize>> {
        if range.is_empty() {
            return Vec::new();
        }

        let start_seg = self.position(range.start);
        let end_seg = self.position(range.end - 1);
        let tasks_len = self.tasks.len();
        let mut result = Vec::with_capacity(end_seg + 1 - start_seg);

        for seg in start_seg..=end_seg {
            // 获取当前段的全局索引范围
            let seg_start = self.start_point[seg];
            let seg_end = if seg + 1 < tasks_len {
                self.start_point[seg + 1]
            } else {
                self.len
            };

            // 计算当前段在请求范围内的实际截取部分
            let curr_start = seg_start.max(range.start);
            let curr_end = seg_end.min(range.end);

            if curr_start < curr_end {
                // 转换为实际数值范围
                let actual_start = self.tasks[seg].start + (curr_start - seg_start);
                let actual_end = self.tasks[seg].start + (curr_end - seg_start);
                result.push(actual_start..actual_end);
            }
        }

        result
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

    #[test]
    fn test_get_range() {
        let tasks = TaskList::from(vec![10..15, 20..25]);

        // 单任务段范围
        assert_eq!(tasks.get_range(0..5), vec![10..15]);
        // 跨任务段范围
        assert_eq!(tasks.get_range(3..7), vec![13..15, 20..22]);
        // 空范围
        assert_eq!(tasks.get_range(5..5), vec![]);
        // 边界测试
        assert_eq!(tasks.get_range(4..5), vec![14..15]);
        assert_eq!(tasks.get_range(4..6), vec![14..15, 20..21]);
        assert_eq!(tasks.get_range(5..7), vec![20..22]);
        // 完整覆盖多个段
        assert_eq!(tasks.get_range(0..10), vec![10..15, 20..25]);
    }

    #[test]
    fn test_get_range2() {
        let tasks = TaskList::from(vec![20..25, 10..15]);

        // 单任务段范围
        assert_eq!(tasks.get_range(0..5), vec![20..25]);
        // 跨任务段范围
        assert_eq!(tasks.get_range(3..7), vec![23..25, 10..12]);
        // 空范围
        assert_eq!(tasks.get_range(5..5), vec![]);
        // 边界测试
        assert_eq!(tasks.get_range(4..5), vec![24..25]);
        assert_eq!(tasks.get_range(4..6), vec![24..25, 10..11]);
        assert_eq!(tasks.get_range(5..7), vec![10..12]);
        // 完整覆盖多个段
        assert_eq!(tasks.get_range(0..10), vec![20..25, 10..15]);
    }
}
