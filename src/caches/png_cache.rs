use std::collections::HashMap;
use std::time::Instant;

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

pub struct PngCache {
    cache: HashMap<CacheKey, CacheEntry>,
    max_size: usize,
}

impl PngCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    pub fn get(&mut self, path: &str, width: u32, height: u32) -> Option<&Image> {
        let key = CacheKey {
            path: path.to_string(),
            width,
            height,
        };

        if self.cache.contains_key(&key) {
            // Update access statistics
            if let Some(entry) = self.cache.get_mut(&key) {
                entry.access_count += 1;
                entry.last_accessed = Instant::now();
            }
            self.cache.get(&key).map(|entry| &entry.image)
        } else {
            match Image::try_new_from_file(path, width, height) {
                Ok(image) => {
                    // Check if we need to remove least accessed items
                    if self.cache.len() >= self.max_size {
                        self.remove_least_accessed();
                    }

                    // Insert new entry
                    self.cache.insert(
                        key.clone(),
                        CacheEntry {
                            image,
                            access_count: 1,
                            last_accessed: Instant::now(),
                        },
                    );
                    self.cache.get(&key).map(|entry| &entry.image)
                }
                Err(e) => {
                    println!("Error: {}", e);
                    None
                }
            }
        }
    }

    fn remove_least_accessed(&mut self) {
        // Find the key with the lowest access count and oldest access time
        if let Some(key_to_remove) = self
            .cache
            .iter()
            .min_by(|a, b| {
                // First compare by access count
                let count_cmp = a.1.access_count.cmp(&b.1.access_count);
                if count_cmp != std::cmp::Ordering::Equal {
                    count_cmp
                } else {
                    // If access counts are equal, compare by last accessed time
                    a.1.last_accessed.cmp(&b.1.last_accessed)
                }
            })
            .map(|(k, _)| k.clone())
        {
            self.cache.remove(&key_to_remove);
        }
    }

    // Optional: Add methods to get statistics
    pub fn get_stats(&self, path: &str, width: u32, height: u32) -> Option<(u32, Instant)> {
        let key = CacheKey {
            path: path.to_string(),
            width,
            height,
        };
        self.cache
            .get(&key)
            .map(|entry| (entry.access_count, entry.last_accessed))
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}
