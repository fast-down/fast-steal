use std::{
    ops::Range,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::task_list::TaskList;
#[derive(Debug)]
pub struct Task {
    start: AtomicUsize,
    end: AtomicUsize,
}

impl Task {
    pub fn remain(&self) -> usize {
        self.end() - self.start()
    }

    pub fn start(&self) -> usize {
        self.start.load(Ordering::Acquire)
    }
    pub fn set_start(&self, start: usize) {
        self.start.store(start, Ordering::Release);
    }
    pub fn end(&self) -> usize {
        self.end.load(Ordering::Acquire)
    }
    pub fn set_end(&self, end: usize) {
        self.end.store(end, Ordering::Release);
    }

    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: AtomicUsize::new(start),
            end: AtomicUsize::new(end),
        }
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.start() == other.start() && self.end() == other.end()
    }
}

impl From<&(usize, usize)> for Task {
    fn from(value: &(usize, usize)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<&Range<usize>> for Task {
    fn from(value: &Range<usize>) -> Self {
        Self::new(value.start, value.end)
    }
}

impl From<&TaskList> for Task {
    fn from(value: &TaskList) -> Self {
        Self::new(0, value.len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_new_task() {
        let task = Task::new(10, 20);
        assert_eq!(task.start(), 10);
        assert_eq!(task.end(), 20);
        assert_eq!(task.remain(), 10);
    }

    #[test]
    fn test_from_tuple() {
        let task: Task = (&(5, 15)).into();
        assert_eq!(task.start(), 5);
        assert_eq!(task.end(), 15);
    }

    #[test]
    fn test_from_range() {
        let range = 3..8;
        let task: Task = (&range).into();
        assert_eq!(task.start(), range.start);
        assert_eq!(task.end(), range.end);
    }

    #[test]
    fn test_setters() {
        let task = Task::new(0, 0);
        task.set_start(7);
        task.set_end(14);
        assert_eq!(task.start(), 7);
        assert_eq!(task.end(), 14);
    }

    #[test]
    fn test_remain() {
        let task = Task::new(10, 25);
        assert_eq!(task.remain(), 15);

        task.set_start(20);
        assert_eq!(task.remain(), 5);
    }

    #[test]
    fn test_partial_eq() {
        let task1 = Task::new(1, 10);
        let task2 = Task::new(1, 10);
        let task3 = Task::new(2, 10);
        let task4 = Task::new(1, 11);

        assert_eq!(task1, task2);
        assert_ne!(task1, task3);
        assert_ne!(task1, task4);
    }

    #[test]
    fn test_thread_safety() {
        let task = Arc::new(Task::new(0, 100));

        let task_clone = Arc::clone(&task);
        let handle1 = thread::spawn(move || {
            task_clone.set_start(10);
        });

        let task_clone = Arc::clone(&task);
        let handle2 = thread::spawn(move || {
            task_clone.set_end(90);
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        assert_eq!(task.start(), 10);
        assert_eq!(task.end(), 90);
    }

    #[test]
    fn test_from_task_list() {
        let task_list = TaskList::from(vec![10..42, 80..84]);
        let task: Task = (&task_list).into();
        assert_eq!(task.start(), 0);
        assert_eq!(task.end(), 36);
    }
}
