use crate::task::Task;

pub trait SplitTask {
    fn split_task(&self, n: usize) -> impl Iterator<Item = Task>;
    fn split_two(&self) -> (usize, usize);
}

impl SplitTask for Task {
    fn split_task(&self, n: usize) -> impl Iterator<Item = Task> {
        assert!(n > 0, "n must be greater than 0");
        let total = self.remain();
        let offset = self.start();
        let per_group = total / n;
        let remainder = total % n;
        (0..n).map(move |i| {
            let start = offset + i * per_group + i.min(remainder);
            let end = start + per_group + if i < remainder { 1 } else { 0 };
            Task::new(start, end)
        })
    }

    fn split_two(&self) -> (usize, usize) {
        let start = self.start();
        let end = self.end();
        let mid = (start + end) / 2;
        self.set_end(mid);
        (mid, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_evenly_divisible() {
        let task = Task::new(1, 6); // 1, 2, 3, 4, 5
        let groups: Vec<_> = task.split_task(3).collect(); // 5 / 3 = 1 remainder 2

        assert_eq!(
            groups,
            vec![Task::new(1, 3), Task::new(3, 5), Task::new(5, 6)]
        );
    }
}
