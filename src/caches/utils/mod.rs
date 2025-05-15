use std::collections::VecDeque;

pub struct EvictionQueue<CacheKey>
where
    CacheKey: PartialEq,
{
    queue: VecDeque<CacheKey>,
    max_size: usize,
}

impl<CacheKey> EvictionQueue<CacheKey>
where
    CacheKey: PartialEq,
{
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn update(&mut self, key: CacheKey) {
        if let Some(pos) = self.queue.iter().position(|k| k == &key) {
            self.queue.remove(pos);
        }
        if self.queue.len() >= self.max_size {
            self.queue.pop_front();
        }
        self.queue.push_back(key);
    }

    pub fn get_oldest(&self) -> Option<&CacheKey> {
        self.queue.front()
    }
}
