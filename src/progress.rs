//! Progress tracking for long-running operations

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;

/// Progress status for an operation
#[derive(Debug, Clone)]
pub enum ProgressStatus {
    NotStarted,
    InProgress { current: u64, total: Option<u64> },
    Completed { total: u64 },
    Failed { error: String },
}

/// Progress update message
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub operation_id: String,
    pub status: ProgressStatus,
    pub message: Option<String>,
    pub timestamp: Instant,
}

/// Progress tracker for monitoring operation progress
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    sender: mpsc::UnboundedSender<ProgressUpdate>,
    operations: Arc<RwLock<std::collections::HashMap<String, ProgressUpdate>>>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> (Self, mpsc::UnboundedReceiver<ProgressUpdate>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let operations = Arc::new(RwLock::new(std::collections::HashMap::new()));

        let tracker = Self {
            sender,
            operations,
        };

        (tracker, receiver)
    }

    /// Start tracking an operation
    pub async fn start_operation(&self, operation_id: &str, message: Option<&str>) {
        let update = ProgressUpdate {
            operation_id: operation_id.to_string(),
            status: ProgressStatus::NotStarted,
            message: message.map(|s| s.to_string()),
            timestamp: Instant::now(),
        };

        self.operations.write().await.insert(operation_id.to_string(), update.clone());
        let _ = self.sender.send(update);
    }

    /// Update operation progress
    pub async fn update_progress(&self, operation_id: &str, current: u64, total: Option<u64>, message: Option<&str>) {
        let update = ProgressUpdate {
            operation_id: operation_id.to_string(),
            status: ProgressStatus::InProgress { current, total },
            message: message.map(|s| s.to_string()),
            timestamp: Instant::now(),
        };

        self.operations.write().await.insert(operation_id.to_string(), update.clone());
        let _ = self.sender.send(update);
    }

    /// Mark operation as completed
    pub async fn complete_operation(&self, operation_id: &str, total: u64, message: Option<&str>) {
        let update = ProgressUpdate {
            operation_id: operation_id.to_string(),
            status: ProgressStatus::Completed { total },
            message: message.map(|s| s.to_string()),
            timestamp: Instant::now(),
        };

        self.operations.write().await.insert(operation_id.to_string(), update.clone());
        let _ = self.sender.send(update);
    }

    /// Mark operation as failed
    pub async fn fail_operation(&self, operation_id: &str, error: &str) {
        let update = ProgressUpdate {
            operation_id: operation_id.to_string(),
            status: ProgressStatus::Failed { error: error.to_string() },
            message: None,
            timestamp: Instant::now(),
        };

        self.operations.write().await.insert(operation_id.to_string(), update.clone());
        let _ = self.sender.send(update);
    }

    /// Get current status of an operation
    pub async fn get_status(&self, operation_id: &str) -> Option<ProgressUpdate> {
        self.operations.read().await.get(operation_id).cloned()
    }

    /// Get all active operations
    pub async fn get_all_operations(&self) -> Vec<ProgressUpdate> {
        self.operations.read().await.values().cloned().collect()
    }

    /// Clean up completed or failed operations older than the specified duration
    pub async fn cleanup_old_operations(&self, max_age: Duration) {
        let now = Instant::now();
        let mut operations = self.operations.write().await;
        
        operations.retain(|_, update| {
            match &update.status {
                ProgressStatus::Completed { .. } | ProgressStatus::Failed { .. } => {
                    now.duration_since(update.timestamp) <= max_age
                }
                _ => true, // Keep in-progress operations
            }
        });
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new().0
    }
}

/// Progress indicator that can be used for tracking file operations
#[derive(Debug)]
pub struct OperationProgress {
    tracker: ProgressTracker,
    operation_id: String,
    total_steps: Option<u64>,
    current_step: u64,
}

impl OperationProgress {
    /// Create a new operation progress tracker
    pub fn new(tracker: ProgressTracker, operation_id: String, total_steps: Option<u64>) -> Self {
        Self {
            tracker,
            operation_id,
            total_steps,
            current_step: 0,
        }
    }

    /// Start the operation
    pub async fn start(&self, message: Option<&str>) {
        self.tracker.start_operation(&self.operation_id, message).await;
    }

    /// Advance to the next step
    pub async fn advance(&mut self, message: Option<&str>) {
        self.current_step += 1;
        self.tracker.update_progress(
            &self.operation_id, 
            self.current_step, 
            self.total_steps, 
            message
        ).await;
    }

    /// Set current step directly
    pub async fn set_progress(&mut self, current: u64, message: Option<&str>) {
        self.current_step = current;
        self.tracker.update_progress(
            &self.operation_id, 
            self.current_step, 
            self.total_steps, 
            message
        ).await;
    }

    /// Complete the operation
    pub async fn complete(&self, message: Option<&str>) {
        let total = self.total_steps.unwrap_or(self.current_step);
        self.tracker.complete_operation(&self.operation_id, total, message).await;
    }

    /// Fail the operation
    pub async fn fail(&self, error: &str) {
        self.tracker.fail_operation(&self.operation_id, error).await;
    }

    /// Get progress percentage (0-100)
    pub fn get_percentage(&self) -> Option<f64> {
        self.total_steps.map(|total| {
            if total > 0 {
                (self.current_step as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        })
    }
}

/// Progress reporter for displaying progress to users
pub struct ProgressReporter {
    receiver: mpsc::UnboundedReceiver<ProgressUpdate>,
    enabled: bool,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(receiver: mpsc::UnboundedReceiver<ProgressUpdate>, enabled: bool) -> Self {
        Self { receiver, enabled }
    }

    /// Start the progress reporting loop
    pub async fn start_reporting(&mut self) {
        if !self.enabled {
            return;
        }

        // Cleanup interval for removing old operations
        let mut cleanup_interval = interval(Duration::from_secs(300)); // 5 minutes

        loop {
            tokio::select! {
                update = self.receiver.recv() => {
                    match update {
                        Some(update) => self.handle_progress_update(update).await,
                        None => break, // Channel closed
                    }
                }
                _ = cleanup_interval.tick() => {
                    tracing::debug!("Progress cleanup tick");
                }
            }
        }
    }

    /// Handle a single progress update
    async fn handle_progress_update(&self, update: ProgressUpdate) {
        match &update.status {
            ProgressStatus::NotStarted => {
                let message = update.message.as_deref().unwrap_or("Starting operation");
                tracing::info!("🚀 {}: {}", update.operation_id, message);
            }
            ProgressStatus::InProgress { current, total } => {
                let progress_str = if let Some(total) = total {
                    let percentage = (*current as f64 / *total as f64) * 100.0;
                    format!("[{}/{}] ({:.1}%)", current, total, percentage)
                } else {
                    format!("[{}]", current)
                };
                
                let message = update.message.as_deref().unwrap_or("In progress");
                tracing::info!("⏳ {}: {} {}", update.operation_id, progress_str, message);
            }
            ProgressStatus::Completed { total } => {
                let message = update.message.as_deref().unwrap_or("Completed successfully");
                tracing::info!("✅ {}: {} (total: {})", update.operation_id, message, total);
            }
            ProgressStatus::Failed { error } => {
                tracing::error!("❌ {}: Failed - {}", update.operation_id, error);
            }
        }
    }

    /// Generate a progress report for all operations
    pub fn generate_report(operations: &[ProgressUpdate]) -> String {
        if operations.is_empty() {
            return "No operations in progress.".to_string();
        }

        let mut report = String::new();
        report.push_str("Progress Report\n");
        report.push_str("===============\n\n");

        for update in operations {
            report.push_str(&format!("Operation: {}\n", update.operation_id));
            
            match &update.status {
                ProgressStatus::NotStarted => {
                    report.push_str("Status: Not Started\n");
                }
                ProgressStatus::InProgress { current, total } => {
                    if let Some(total) = total {
                        let percentage = (*current as f64 / *total as f64) * 100.0;
                        report.push_str(&format!("Status: In Progress [{}/{}] ({:.1}%)\n", current, total, percentage));
                    } else {
                        report.push_str(&format!("Status: In Progress [{}]\n", current));
                    }
                }
                ProgressStatus::Completed { total } => {
                    report.push_str(&format!("Status: Completed (total: {})\n", total));
                }
                ProgressStatus::Failed { error } => {
                    report.push_str(&format!("Status: Failed - {}\n", error));
                }
            }
            
            if let Some(message) = &update.message {
                report.push_str(&format!("Message: {}\n", message));
            }
            
            let elapsed = update.timestamp.elapsed();
            report.push_str(&format!("Elapsed: {:.2}s\n", elapsed.as_secs_f64()));
            report.push_str("\n");
        }

        report
    }
}

/// Global progress tracker instance
static GLOBAL_PROGRESS_TRACKER: std::sync::OnceLock<ProgressTracker> = std::sync::OnceLock::new();

/// Get the global progress tracker instance
pub fn global_progress_tracker() -> &'static ProgressTracker {
    GLOBAL_PROGRESS_TRACKER.get_or_init(|| {
        ProgressTracker::new().0
    })
}

/// Convenience macro for tracking operation progress
#[macro_export]
macro_rules! track_progress {
    ($operation_id:expr, $total_steps:expr, $code:block) => {{
        let tracker = $crate::progress::global_progress_tracker();
        let mut progress = $crate::progress::OperationProgress::new(
            tracker.clone(),
            $operation_id.to_string(),
            Some($total_steps)
        );
        
        progress.start(None).await;
        
        let result = async {
            $code
        }.await;
        
        match &result {
            Ok(_) => progress.complete(Some("Operation completed successfully")).await,
            Err(e) => progress.fail(&e.to_string()).await,
        }
        
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_progress_tracker_basic() {
        let (tracker, mut receiver) = ProgressTracker::new();
        
        // Start operation
        tracker.start_operation("test_op", Some("Starting test")).await;
        
        // Check that we received the update
        let update = receiver.recv().await.unwrap();
        assert_eq!(update.operation_id, "test_op");
        assert!(matches!(update.status, ProgressStatus::NotStarted));
        assert_eq!(update.message, Some("Starting test".to_string()));
        
        // Update progress
        tracker.update_progress("test_op", 5, Some(10), Some("Half way")).await;
        
        let update = receiver.recv().await.unwrap();
        assert_eq!(update.operation_id, "test_op");
        match update.status {
            ProgressStatus::InProgress { current, total } => {
                assert_eq!(current, 5);
                assert_eq!(total, Some(10));
            }
            _ => panic!("Expected InProgress status"),
        }
        
        // Complete operation
        tracker.complete_operation("test_op", 10, Some("Finished")).await;
        
        let update = receiver.recv().await.unwrap();
        assert_eq!(update.operation_id, "test_op");
        match update.status {
            ProgressStatus::Completed { total } => {
                assert_eq!(total, 10);
            }
            _ => panic!("Expected Completed status"),
        }
    }

    #[tokio::test]
    async fn test_operation_progress() {
        let (tracker, _receiver) = ProgressTracker::new();
        let mut progress = OperationProgress::new(tracker.clone(), "test_op".to_string(), Some(3));
        
        progress.start(Some("Starting")).await;
        
        // Check initial status
        let status = tracker.get_status("test_op").await.unwrap();
        assert!(matches!(status.status, ProgressStatus::NotStarted));
        
        // Advance progress
        progress.advance(Some("Step 1")).await;
        progress.advance(Some("Step 2")).await;
        
        let status = tracker.get_status("test_op").await.unwrap();
        match status.status {
            ProgressStatus::InProgress { current, total } => {
                assert_eq!(current, 2);
                assert_eq!(total, Some(3));
            }
            _ => panic!("Expected InProgress status"),
        }
        
        // Test percentage calculation (allow for floating point precision)
        let percentage = progress.get_percentage().unwrap();
        assert!((percentage - 66.66666666666667).abs() < 0.0001);
        
        // Complete
        progress.complete(Some("All done")).await;
        
        let status = tracker.get_status("test_op").await.unwrap();
        assert!(matches!(status.status, ProgressStatus::Completed { total: 3 }));
    }

    #[tokio::test]
    async fn test_progress_cleanup() {
        let (tracker, _receiver) = ProgressTracker::new();
        
        // Add some operations
        tracker.start_operation("op1", None).await;
        tracker.complete_operation("op2", 10, None).await;
        tracker.fail_operation("op3", "Test error").await;
        
        // Should have 3 operations
        let operations = tracker.get_all_operations().await;
        assert_eq!(operations.len(), 3);
        
        // Wait a bit and cleanup with very short max age
        sleep(Duration::from_millis(10)).await;
        tracker.cleanup_old_operations(Duration::from_millis(5)).await;
        
        // Should still have the in-progress operation
        let operations = tracker.get_all_operations().await;
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].operation_id, "op1");
    }

    #[tokio::test]
    async fn test_progress_reporter() {
        let (tracker, receiver) = ProgressTracker::new();
        let mut reporter = ProgressReporter::new(receiver, true);
        
        // Start the reporter in the background
        let reporter_task = tokio::spawn(async move {
            // Run for a short time then exit
            tokio::time::timeout(Duration::from_millis(100), reporter.start_reporting()).await.ok();
        });
        
        // Generate some progress updates
        tracker.start_operation("test", Some("Testing")).await;
        tracker.update_progress("test", 5, Some(10), Some("Half done")).await;
        tracker.complete_operation("test", 10, Some("Finished")).await;
        
        // Wait for reporter to process
        sleep(Duration::from_millis(50)).await;
        
        // Clean up
        reporter_task.abort();
    }

    #[tokio::test]
    async fn test_progress_report_generation() {
        let updates = vec![
            ProgressUpdate {
                operation_id: "op1".to_string(),
                status: ProgressStatus::InProgress { current: 5, total: Some(10) },
                message: Some("Working on it".to_string()),
                timestamp: Instant::now(),
            },
            ProgressUpdate {
                operation_id: "op2".to_string(),
                status: ProgressStatus::Completed { total: 20 },
                message: Some("All done".to_string()),
                timestamp: Instant::now(),
            },
        ];
        
        let report = ProgressReporter::generate_report(&updates);
        
        assert!(report.contains("Progress Report"));
        assert!(report.contains("op1"));
        assert!(report.contains("op2"));
        assert!(report.contains("In Progress [5/10]"));
        assert!(report.contains("Completed"));
        assert!(report.contains("Working on it"));
        assert!(report.contains("All done"));
    }
}