//! Performance profiling utilities for the project manager

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Performance metrics for a single operation
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    pub name: String,
    pub count: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub last_duration: Duration,
}

impl OperationMetrics {
    fn new(name: String, duration: Duration) -> Self {
        Self {
            name,
            count: 1,
            total_duration: duration,
            min_duration: duration,
            max_duration: duration,
            last_duration: duration,
        }
    }

    fn update(&mut self, duration: Duration) {
        self.count += 1;
        self.total_duration += duration;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);
        self.last_duration = duration;
    }

    pub fn average_duration(&self) -> Duration {
        if self.count > 0 {
            self.total_duration / self.count as u32
        } else {
            Duration::ZERO
        }
    }
}

/// Global performance profiler for tracking operation metrics
#[derive(Debug, Clone)]
pub struct Profiler {
    metrics: Arc<Mutex<HashMap<String, OperationMetrics>>>,
    enabled: bool,
}

impl Profiler {
    /// Create a new profiler instance
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            enabled: true,
        }
    }

    /// Create a disabled profiler (no-op for production)
    pub fn disabled() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            enabled: false,
        }
    }

    /// Start timing an operation
    pub fn start_operation(&self, name: &str) -> OperationTimer {
        OperationTimer::new(self.clone(), name.to_string(), self.enabled)
    }

    /// Record a completed operation
    pub fn record_operation(&self, name: &str, duration: Duration) {
        if !self.enabled {
            return;
        }

        if let Ok(mut metrics) = self.metrics.lock() {
            metrics
                .entry(name.to_string())
                .and_modify(|m| m.update(duration))
                .or_insert_with(|| OperationMetrics::new(name.to_string(), duration));
        }
    }

    /// Get metrics for a specific operation
    pub fn get_metrics(&self, name: &str) -> Option<OperationMetrics> {
        self.metrics
            .lock()
            .ok()?
            .get(name)
            .cloned()
    }

    /// Get all recorded metrics
    pub fn get_all_metrics(&self) -> HashMap<String, OperationMetrics> {
        self.metrics
            .lock()
            .unwrap_or_else(|_| std::process::abort())
            .clone()
    }

    /// Reset all metrics
    pub fn reset(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.clear();
        }
    }

    /// Generate a performance report
    pub fn generate_report(&self) -> String {
        let metrics = self.get_all_metrics();
        if metrics.is_empty() {
            return "No performance metrics recorded.".to_string();
        }

        let mut report = String::new();
        report.push_str("Performance Report\n");
        report.push_str("==================\n\n");

        let mut sorted_metrics: Vec<_> = metrics.values().collect();
        sorted_metrics.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));

        for metric in sorted_metrics {
            report.push_str(&format!(
                "Operation: {}\n\
                 Count: {}\n\
                 Total Time: {:?}\n\
                 Average: {:?}\n\
                 Min: {:?}\n\
                 Max: {:?}\n\
                 Last: {:?}\n\n",
                metric.name,
                metric.count,
                metric.total_duration,
                metric.average_duration(),
                metric.min_duration,
                metric.max_duration,
                metric.last_duration
            ));
        }

        report
    }

    /// Get the slowest operations
    pub fn get_slowest_operations(&self, limit: usize) -> Vec<OperationMetrics> {
        let metrics = self.get_all_metrics();
        let mut sorted_metrics: Vec<_> = metrics.into_values().collect();
        sorted_metrics.sort_by(|a, b| b.average_duration().cmp(&a.average_duration()));
        sorted_metrics.truncate(limit);
        sorted_metrics
    }

    /// Get operations with high total time
    pub fn get_high_impact_operations(&self, limit: usize) -> Vec<OperationMetrics> {
        let metrics = self.get_all_metrics();
        let mut sorted_metrics: Vec<_> = metrics.into_values().collect();
        sorted_metrics.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));
        sorted_metrics.truncate(limit);
        sorted_metrics
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer for measuring operation duration
pub struct OperationTimer {
    profiler: Profiler,
    operation_name: String,
    start_time: Instant,
    enabled: bool,
}

impl OperationTimer {
    fn new(profiler: Profiler, operation_name: String, enabled: bool) -> Self {
        Self {
            profiler,
            operation_name,
            start_time: Instant::now(),
            enabled,
        }
    }

    /// Finish timing and record the result
    pub fn finish(self) {
        if self.enabled {
            let duration = self.start_time.elapsed();
            self.profiler.record_operation(&self.operation_name, duration);
        }
    }

    /// Get elapsed time without finishing the timer
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        if self.enabled {
            let duration = self.start_time.elapsed();
            self.profiler.record_operation(&self.operation_name, duration);
        }
    }
}

/// Global profiler instance
static GLOBAL_PROFILER: std::sync::OnceLock<Profiler> = std::sync::OnceLock::new();

/// Get the global profiler instance
pub fn global_profiler() -> &'static Profiler {
    GLOBAL_PROFILER.get_or_init(|| {
        // Enable profiling in debug builds, disable in release builds
        if cfg!(debug_assertions) {
            Profiler::new()
        } else {
            Profiler::disabled()
        }
    })
}

/// Convenience macro for timing operations
#[macro_export]
macro_rules! profile_operation {
    ($name:expr, $code:block) => {{
        let _timer = $crate::profiling::global_profiler().start_operation($name);
        $code
    }};
}

/// Convenience macro for timing async operations
#[macro_export]
macro_rules! profile_async_operation {
    ($name:expr, $code:block) => {{
        let _timer = $crate::profiling::global_profiler().start_operation($name);
        $code
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_profiler_basic_functionality() {
        let profiler = Profiler::new();
        
        // Record some operations
        profiler.record_operation("test_op", Duration::from_millis(100));
        profiler.record_operation("test_op", Duration::from_millis(200));
        profiler.record_operation("another_op", Duration::from_millis(50));
        
        // Check metrics
        let metrics = profiler.get_metrics("test_op").unwrap();
        assert_eq!(metrics.count, 2);
        assert_eq!(metrics.total_duration, Duration::from_millis(300));
        assert_eq!(metrics.min_duration, Duration::from_millis(100));
        assert_eq!(metrics.max_duration, Duration::from_millis(200));
        assert_eq!(metrics.average_duration(), Duration::from_millis(150));
        
        let all_metrics = profiler.get_all_metrics();
        assert_eq!(all_metrics.len(), 2);
    }

    #[test]
    fn test_operation_timer() {
        let profiler = Profiler::new();
        profiler.reset(); // Clear any previous state
        
        {
            let timer = profiler.start_operation("manual_timer_test");
            thread::sleep(Duration::from_millis(10));
            timer.finish();
        }
        
        let metrics = profiler.get_metrics("manual_timer_test").unwrap();
        assert_eq!(metrics.count, 1);
        assert!(metrics.total_duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_operation_timer_auto_finish() {
        let profiler = Profiler::new();
        
        {
            let _timer = profiler.start_operation("auto_timer_test");
            thread::sleep(Duration::from_millis(10));
            // Timer automatically finishes when dropped
        }
        
        let metrics = profiler.get_metrics("auto_timer_test").unwrap();
        assert_eq!(metrics.count, 1);
        assert!(metrics.total_duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_disabled_profiler() {
        let profiler = Profiler::disabled();
        
        profiler.record_operation("disabled_test", Duration::from_millis(100));
        
        // Should have no metrics since profiler is disabled
        assert!(profiler.get_metrics("disabled_test").is_none());
        assert!(profiler.get_all_metrics().is_empty());
    }

    #[test]
    fn test_generate_report() {
        let profiler = Profiler::new();
        
        profiler.record_operation("fast_op", Duration::from_millis(10));
        profiler.record_operation("slow_op", Duration::from_millis(100));
        
        let report = profiler.generate_report();
        assert!(report.contains("Performance Report"));
        assert!(report.contains("fast_op"));
        assert!(report.contains("slow_op"));
    }

    #[test]
    fn test_slowest_operations() {
        let profiler = Profiler::new();
        
        profiler.record_operation("fast", Duration::from_millis(10));
        profiler.record_operation("medium", Duration::from_millis(50));
        profiler.record_operation("slow", Duration::from_millis(100));
        
        let slowest = profiler.get_slowest_operations(2);
        assert_eq!(slowest.len(), 2);
        assert_eq!(slowest[0].name, "slow");
        assert_eq!(slowest[1].name, "medium");
    }

    #[test]
    fn test_high_impact_operations() {
        let profiler = Profiler::new();
        
        // Operation with high frequency but low individual time
        for _ in 0..10 {
            profiler.record_operation("frequent", Duration::from_millis(10));
        }
        
        // Operation with low frequency but high individual time
        profiler.record_operation("expensive", Duration::from_millis(200));
        
        let high_impact = profiler.get_high_impact_operations(2);
        assert_eq!(high_impact.len(), 2);
        // "expensive" should be first (200ms total) over "frequent" (100ms total)
        assert_eq!(high_impact[0].name, "expensive");
        assert_eq!(high_impact[1].name, "frequent");
    }

    #[test]
    fn test_profile_operation_macro() {
        let result = profile_operation!("macro_test", {
            thread::sleep(Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
        
        let metrics = global_profiler().get_metrics("macro_test");
        if cfg!(debug_assertions) {
            assert!(metrics.is_some());
            let metrics = metrics.unwrap();
            assert_eq!(metrics.count, 1);
            assert!(metrics.total_duration >= Duration::from_millis(10));
        } else {
            // In release builds, profiling is disabled
            assert!(metrics.is_none());
        }
    }

    #[tokio::test]
    async fn test_profile_async_operation_macro() {
        let result = profile_async_operation!("async_macro_test", {
            tokio::time::sleep(Duration::from_millis(10)).await;
            "async_result"
        });
        
        assert_eq!(result, "async_result");
        
        let metrics = global_profiler().get_metrics("async_macro_test");
        if cfg!(debug_assertions) {
            assert!(metrics.is_some());
            let metrics = metrics.unwrap();
            assert_eq!(metrics.count, 1);
            assert!(metrics.total_duration >= Duration::from_millis(10));
        } else {
            // In release builds, profiling is disabled
            assert!(metrics.is_none());
        }
    }
}