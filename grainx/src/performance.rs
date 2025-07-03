use std::time::{Duration, Instant};
use std::collections::VecDeque;

pub struct PerformanceMonitor {
    frame_times: VecDeque<Duration>,
    last_frame_time: Instant,
    target_fps: f64,
    adaptive_refresh: bool,
    cpu_load_history: VecDeque<f32>,
}

impl PerformanceMonitor {
    pub fn new(target_fps: f64) -> Self {
        PerformanceMonitor {
            frame_times: VecDeque::with_capacity(60),
            last_frame_time: Instant::now(),
            target_fps,
            adaptive_refresh: true,
            cpu_load_history: VecDeque::with_capacity(10),
        }
    }

    pub fn start_frame(&mut self) {
        self.last_frame_time = Instant::now();
    }

    pub fn end_frame(&mut self) -> Duration {
        let frame_duration = self.last_frame_time.elapsed();
        self.frame_times.push_back(frame_duration);
        
        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }
        
        frame_duration
    }

    pub fn get_fps(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        
        let avg_duration: Duration = self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32;
        1.0 / avg_duration.as_secs_f64()
    }

    pub fn get_frame_time_ms(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        
        let avg_duration: Duration = self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32;
        avg_duration.as_secs_f64() * 1000.0
    }

    pub fn calculate_adaptive_refresh(&mut self, cpu_usage: f32) -> u64 {
        self.cpu_load_history.push_back(cpu_usage);
        if self.cpu_load_history.len() > 10 {
            self.cpu_load_history.pop_front();
        }

        if !self.adaptive_refresh {
            return (1000.0 / self.target_fps) as u64;
        }

        let avg_cpu = self.cpu_load_history.iter().sum::<f32>() / self.cpu_load_history.len() as f32;
        
        // Adaptive refresh based on system load
        let refresh_ms = if avg_cpu > 90.0 {
            2000 // Very slow refresh when system is overloaded
        } else if avg_cpu > 70.0 {
            1000 // Normal refresh
        } else if avg_cpu > 50.0 {
            500  // Fast refresh
        } else {
            250  // Very fast refresh when system is idle
        };

        refresh_ms
    }

    pub fn should_skip_frame(&self, cpu_usage: f32) -> bool {
        // Skip frames if CPU usage is very high to reduce system load
        cpu_usage > 95.0 && self.adaptive_refresh
    }

    pub fn get_performance_stats(&self) -> (f64, f64, bool) {
        (self.get_fps(), self.get_frame_time_ms(), self.adaptive_refresh)
    }

    pub fn toggle_adaptive_refresh(&mut self) {
        self.adaptive_refresh = !self.adaptive_refresh;
    }
}

// Memory pool for reducing allocations
pub struct MemoryPool<T> {
    pool: Vec<Vec<T>>,
    capacity: usize,
}

impl<T> MemoryPool<T> {
    pub fn new(capacity: usize) -> Self {
        MemoryPool {
            pool: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&mut self) -> Vec<T> {
        self.pool.pop().unwrap_or_else(|| Vec::with_capacity(self.capacity))
    }

    pub fn return_vec(&mut self, mut vec: Vec<T>) {
        if self.pool.len() < self.capacity {
            vec.clear();
            self.pool.push(vec);
        }
    }
}

// Efficient string formatting cache
pub struct StringCache {
    cache: std::collections::HashMap<String, String>,
    max_size: usize,
}

impl StringCache {
    pub fn new(max_size: usize) -> Self {
        StringCache {
            cache: std::collections::HashMap::new(),
            max_size,
        }
    }

    pub fn get_or_format<F>(&mut self, key: &str, formatter: F) -> &str
    where
        F: FnOnce() -> String,
    {
        if !self.cache.contains_key(key) {
            if self.cache.len() >= self.max_size {
                self.cache.clear(); // Simple eviction strategy
            }
            let formatted = formatter();
            self.cache.insert(key.to_string(), formatted);
        }
        &self.cache[key]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_performance_monitor_creation() {
        let perf = PerformanceMonitor::new(60.0);
        assert_eq!(perf.target_fps, 60.0);
        assert!(perf.adaptive_refresh);
    }

    #[test]
    fn test_adaptive_refresh_high_cpu() {
        let mut perf = PerformanceMonitor::new(60.0);
        let refresh_time = perf.calculate_adaptive_refresh(95.0);
        assert!(refresh_time >= 1000); // Should be slow when CPU is high
    }

    #[test]
    fn test_adaptive_refresh_low_cpu() {
        let mut perf = PerformanceMonitor::new(60.0);
        let refresh_time = perf.calculate_adaptive_refresh(20.0);
        assert!(refresh_time <= 500); // Should be fast when CPU is low
    }

    #[test]
    fn test_frame_skipping() {
        let perf = PerformanceMonitor::new(60.0);
        assert!(perf.should_skip_frame(96.0));
        assert!(!perf.should_skip_frame(50.0));
    }

    #[test]
    fn test_toggle_adaptive() {
        let mut perf = PerformanceMonitor::new(60.0);
        let initial = perf.adaptive_refresh;
        perf.toggle_adaptive_refresh();
        assert_ne!(initial, perf.adaptive_refresh);
    }

    #[test]
    fn test_memory_pool() {
        let mut pool: MemoryPool<i32> = MemoryPool::new(10);
        let vec1 = pool.get();
        assert!(vec1.is_empty());
        
        let vec2 = vec![1, 2, 3];
        pool.return_vec(vec2);
        
        let vec3 = pool.get();
        assert!(vec3.is_empty()); // Should be cleared when returned
    }

    #[test]
    fn test_string_cache() {
        let mut cache = StringCache::new(5);
        let result1 = cache.get_or_format("test", || "formatted".to_string()).to_string();
        let result2 = cache.get_or_format("test", || "different".to_string()).to_string();
        
        assert_eq!(result1, "formatted");
        assert_eq!(result2, "formatted"); // Should return cached value
    }
}