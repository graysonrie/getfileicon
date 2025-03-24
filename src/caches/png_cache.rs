use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock as TokioRwLock;

use crate::image::Image;

#[derive(Hash, Eq, PartialEq, Clone)]
struct CacheKey {
    path: String,
    width: u32,
    height: u32,
}

struct CacheEntry {
    image: Image,
    access_count: u32,
    last_accessed: Instant,
}

struct EvictionQueue {
    queue: VecDeque<CacheKey>,
    max_size: usize,
}

impl EvictionQueue {
    fn new(max_size: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn update(&mut self, key: CacheKey) {
        if let Some(pos) = self.queue.iter().position(|k| k == &key) {
            self.queue.remove(pos);
        }
        if self.queue.len() >= self.max_size {
            self.queue.pop_front();
        }
        self.queue.push_back(key);
    }

    fn get_oldest(&self) -> Option<&CacheKey> {
        self.queue.front()
    }
}

/// A cache for PNG images. Safe to use across threads.
pub struct PngCache {
    cache: Arc<TokioRwLock<HashMap<CacheKey, CacheEntry>>>,
    eviction_queue: Arc<TokioRwLock<EvictionQueue>>,
    max_size: usize,
}

impl PngCache {
    pub fn new(max_size: usize) -> Self {
        let cache = Arc::new(TokioRwLock::new(HashMap::new()));
        let eviction_queue = Arc::new(TokioRwLock::new(EvictionQueue::new(max_size)));

        // Spawn cleanup task with separate locks to prevent deadlocks
        let cache_clone = Arc::clone(&cache);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                Self::cleanup_old_entries(&cache_clone).await;
            }
        });

        Self {
            cache,
            eviction_queue,
            max_size,
        }
    }

    pub async fn get(&self, path: &str, width: u32, height: u32) -> Option<Arc<Image>> {
        tracing::debug!(
            "Cache get request for path: {}, size: {}x{}",
            path,
            width,
            height
        );
        let key = CacheKey {
            path: path.to_string(),
            width,
            height,
        };

        // First try a read lock
        let image = {
            tracing::debug!("Attempting initial read lock check");
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&key) {
                tracing::debug!("Cache hit on first read lock check");
                Some(Arc::new(entry.image.clone()))
            } else {
                tracing::debug!("Cache miss on first read lock check");
                None
            }
        };

        if let Some(image) = image {
            tracing::debug!("Found image in cache, updating access metrics");
            // Always acquire cache lock first, then queue lock
            let mut cache = self.cache.write().await;
            let mut queue = self.eviction_queue.write().await;

            // Update both access count and queue
            if let Some(entry) = cache.get_mut(&key) {
                entry.access_count += 1;
                entry.last_accessed = Instant::now();
                tracing::debug!("Updated access count to: {}", entry.access_count);
            }
            queue.update(key.clone());
            tracing::debug!("Updated eviction queue");

            return Some(image);
        }

        tracing::debug!("Image not found in cache, attempting to load from file");
        // Not found, acquire write locks in consistent order
        let mut cache = self.cache.write().await;
        let mut queue = self.eviction_queue.write().await;

        // Double-check after acquiring write lock
        if let Some(entry) = cache.get(&key) {
            tracing::debug!("Found image in cache after write lock (race condition)");
            return Some(Arc::new(entry.image.clone()));
        }

        // Create new image
        tracing::debug!("Loading new image from file");
        match Image::try_new_from_file(path, width, height) {
            Ok(image) => {
                if cache.len() >= self.max_size {
                    tracing::debug!(
                        "Cache full ({} entries), evicting oldest entry",
                        cache.len()
                    );
                    // Use the eviction queue to determine what to remove
                    if let Some(old_key) = queue.get_oldest() {
                        cache.remove(old_key);
                        tracing::debug!("Evicted entry for path: {}", old_key.path);
                    }
                }

                let image = Arc::new(image);
                cache.insert(
                    key.clone(),
                    CacheEntry {
                        image: (*image).clone(),
                        access_count: 1,
                        last_accessed: Instant::now(),
                    },
                );
                queue.update(key);
                tracing::debug!("Successfully added new image to cache");
                Some(image)
            }
            Err(e) => {
                tracing::error!("Failed to create image: {}", e);
                None
            }
        }
    }

    // Optional: Add methods to get statistics
    pub async fn get_stats(&self, path: &str, width: u32, height: u32) -> Option<(u32, Instant)> {
        let key = CacheKey {
            path: path.to_string(),
            width,
            height,
        };
        self.cache
            .read()
            .await
            .get(&key)
            .map(|entry| (entry.access_count, entry.last_accessed))
    }

    pub async fn len(&self) -> usize {
        self.cache.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.cache.read().await.is_empty()
    }

    async fn cleanup_old_entries(cache: &Arc<TokioRwLock<HashMap<CacheKey, CacheEntry>>>) {
        let now = Instant::now();

        // First get the keys to remove
        let keys_to_remove: Vec<CacheKey> = {
            let cache = cache.read().await;
            cache
                .iter()
                .filter(|(_, entry)| {
                    now.duration_since(entry.last_accessed) >= Duration::from_secs(3600)
                })
                .map(|(key, _)| key.clone())
                .collect()
        };

        // Then remove them with write locks
        if !keys_to_remove.is_empty() {
            let mut cache = cache.write().await;

            for key in keys_to_remove {
                cache.remove(&key);
                // Note: We don't need to update the queue here as the keys are already removed
            }
        }
    }
}
