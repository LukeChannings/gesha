use std::collections::VecDeque;
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

pub fn get_unix_timestamp(time: SystemTime) -> Result<i64, SystemTimeError> {
    return Ok(time.duration_since(UNIX_EPOCH)?.as_millis() as i64);
}

pub struct FixedCapacityQueue<T> {
    deque: VecDeque<T>,
    capacity: usize,
    pub sum: i32,
}

impl<T> FixedCapacityQueue<T>
where
    T: std::ops::Add<Output = T> + std::ops::AddAssign + std::ops::SubAssign + Into<i32> + Copy + Default,
{
    pub fn new(capacity: usize) -> Self {
        FixedCapacityQueue {
            deque: VecDeque::with_capacity(capacity),
            capacity,
            sum: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        self.sum += value.into();

        if self.deque.len() == self.capacity {
            if let Some(pop_n) = self.deque.pop_front() {
                self.sum -= pop_n.into();
            }
        }

        self.deque.push_back(value);
    }
}
