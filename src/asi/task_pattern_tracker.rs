//! Task Pattern Tracker
//!
//! Tracks task attempts, detects failure patterns, and triggers self-improvement.
//! Enables the AI to learn from mistakes and adapt its problem-solving strategies.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Task attempt result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskResult {
    /// Task completed successfully
    Success,
    
    /// Task failed with error
    Failed,
    
    /// Task partially completed
    Partial,
    
    /// Task timed out
    Timeout,
    
    /// Task was skipped
    Skipped,
}

/// Task category for pattern grouping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskCategory {
    CodeGeneration,
    CodeRefactoring,
    BugFix,
    Testing,
    Documentation,
    Analysis,
    Optimization,
    Design,
    Other(String),
}

impl TaskCategory {
    pub fn from_description(desc: &str) -> Self {
        let lower = desc.to_lowercase();
        
        if lower.contains("generat") && lower.contains("code") {
            Self::CodeGeneration
        } else if lower.contains("refactor") {
            Self::CodeRefactoring
        } else if lower.contains("bug") || lower.contains("fix") {
            Self::BugFix
        } else if lower.contains("test") {
            Self::Testing
        } else if lower.contains("doc") {
            Self::Documentation
        } else if lower.contains("analyz") || lower.contains("review") {
            Self::Analysis
        } else if lower.contains("optim") || lower.contains("performance") {
            Self::Optimization
        } else if lower.contains("design") || lower.contains("architect") {
            Self::Design
        } else {
            Self::Other(desc.to_string())
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            Self::CodeGeneration => "code_generation",
            Self::CodeRefactoring => "code_refactoring",
            Self::BugFix => "bug_fix",
            Self::Testing => "testing",
            Self::Documentation => "documentation",
            Self::Analysis => "analysis",
            Self::Optimization => "optimization",
            Self::Design => "design",
            Self::Other(s) => s,
        }
    }
}

/// Error type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ErrorType {
    SyntaxError,
    LogicError,
    CompilationError,
    RuntimeError,
    TimeoutError,
    ResourceError,
    ValidationError,
    DependencyError,
    PermissionError,
    Unknown(String),
}

impl ErrorType {
    pub fn from_message(msg: &str) -> Self {
        let lower = msg.to_lowercase();
        
        if lower.contains("syntax") {
            Self::SyntaxError
        } else if lower.contains("logic") || lower.contains("incorrect") {
            Self::LogicError
        } else if lower.contains("compil") {
            Self::CompilationError
        } else if lower.contains("runtime") || lower.contains("panic") {
            Self::RuntimeError
        } else if lower.contains("timeout") || lower.contains("deadline") {
            Self::TimeoutError
        } else if lower.contains("memory") || lower.contains("resource") {
            Self::ResourceError
        } else if lower.contains("valid") || lower.contains("invalid") {
            Self::ValidationError
        } else if lower.contains("depend") || lower.contains("import") {
            Self::DependencyError
        } else if lower.contains("permission") || lower.contains("access") {
            Self::PermissionError
        } else {
            Self::Unknown(msg.to_string())
        }
    }
}

/// Task attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAttempt {
    /// Unique task ID
    pub task_id: String,
    
    /// Task category
    pub category: TaskCategory,
    
    /// Task description
    pub description: String,
    
    /// Attempt result
    pub result: TaskResult,
    
    /// Error type (if failed)
    pub error_type: Option<ErrorType>,
    
    /// Error message (if failed)
    pub error_message: Option<String>,
    
    /// Attempt timestamp
    pub timestamp: u64,
    
    /// Duration in milliseconds
    pub duration_ms: u64,
    
    /// Complexity estimate (1-10)
    pub complexity: u8,
    
    /// Retry attempt number (0 = first attempt)
    pub retry_count: u32,
    
    /// Strategy used
    pub strategy: String,
}

/// Detected failure pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    /// Pattern description
    pub description: String,
    
    /// Task category affected
    pub category: TaskCategory,
    
    /// Error type pattern
    pub error_type: ErrorType,
    
    /// Failure rate (0-1)
    pub failure_rate: f32,
    
    /// Total attempts
    pub total_attempts: usize,
    
    /// Failed attempts
    pub failed_attempts: usize,
    
    /// First failure timestamp
    pub first_seen: u64,
    
    /// Last failure timestamp
    pub last_seen: u64,
    
    /// Confidence score (0-1)
    pub confidence: f32,
    
    /// Suggested improvement
    pub suggested_fix: String,
}

/// Task statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub total_attempts: usize,
    pub successful: usize,
    pub failed: usize,
    pub partial: usize,
    pub timeout: usize,
    pub success_rate: f32,
    pub avg_duration_ms: f64,
    pub patterns_detected: usize,
}

/// Task pattern tracker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTrackerConfig {
    /// Maximum history size
    pub max_history: usize,
    
    /// Minimum attempts to detect pattern
    pub min_attempts_for_pattern: usize,
    
    /// Minimum failure rate to trigger (0-1)
    pub min_failure_rate: f32,
    
    /// Confidence threshold for pattern detection
    pub confidence_threshold: f32,
    
    /// Enable auto-improvement
    pub auto_improve: bool,
}

impl Default for TaskTrackerConfig {
    fn default() -> Self {
        Self {
            max_history: 1000,
            min_attempts_for_pattern: 3,
            min_failure_rate: 0.5, // 50% failure rate
            confidence_threshold: 0.7,
            auto_improve: true,
        }
    }
}

/// Task pattern tracker
pub struct TaskPatternTracker {
    config: Arc<RwLock<TaskTrackerConfig>>,
    
    /// Task attempt history
    attempts: Arc<RwLock<VecDeque<TaskAttempt>>>,
    
    /// Detected patterns
    patterns: Arc<RwLock<Vec<FailurePattern>>>,
    
    /// Category-specific statistics
    category_stats: Arc<RwLock<HashMap<TaskCategory, CategoryStats>>>,
}

#[derive(Debug, Clone)]
struct CategoryStats {
    total: usize,
    successful: usize,
    failed: usize,
    total_duration_ms: u64,
}

impl TaskPatternTracker {
    /// Create new task pattern tracker
    pub fn new(config: TaskTrackerConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            attempts: Arc::new(RwLock::new(VecDeque::new())),
            patterns: Arc::new(RwLock::new(Vec::new())),
            category_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Record a task attempt
    pub fn record_attempt(&self, attempt: TaskAttempt) {
        // Update category stats
        {
            let mut stats = self.category_stats.write();
            let cat_stats = stats.entry(attempt.category.clone()).or_insert(CategoryStats {
                total: 0,
                successful: 0,
                failed: 0,
                total_duration_ms: 0,
            });
            
            cat_stats.total += 1;
            cat_stats.total_duration_ms += attempt.duration_ms;
            
            match attempt.result {
                TaskResult::Success => cat_stats.successful += 1,
                TaskResult::Failed => cat_stats.failed += 1,
                _ => {}
            }
        }
        
        // Add to history
        {
            let mut attempts = self.attempts.write();
            attempts.push_back(attempt);
            
            let config = self.config.read();
            if attempts.len() > config.max_history {
                attempts.pop_front();
            }
        }
        
        // Analyze for patterns
        self.analyze_patterns();
    }
    
    /// Analyze task attempts for failure patterns
    pub fn analyze_patterns(&self) {
        let attempts = self.attempts.read();
        let config = self.config.read();
        
        if attempts.len() < config.min_attempts_for_pattern {
            return;
        }
        
        // Group by category and error type
        let mut groups: HashMap<(TaskCategory, ErrorType), Vec<&TaskAttempt>> = HashMap::new();
        
        for attempt in attempts.iter() {
            if attempt.result == TaskResult::Failed {
                if let Some(ref error_type) = attempt.error_type {
                    let key = (attempt.category.clone(), error_type.clone());
                    groups.entry(key).or_insert_with(Vec::new).push(attempt);
                }
            }
        }
        
        // Detect patterns
        let mut detected_patterns = Vec::new();
        
        for ((category, error_type), failures) in groups {
            if failures.len() < config.min_attempts_for_pattern {
                continue;
            }
            
            // Calculate failure rate for this category
            let category_attempts: Vec<&TaskAttempt> = attempts.iter()
                .filter(|a| a.category == category)
                .collect();
            
            if category_attempts.is_empty() {
                continue;
            }
            
            let failure_rate = failures.len() as f32 / category_attempts.len() as f32;
            
            if failure_rate < config.min_failure_rate {
                continue;
            }
            
            // Calculate confidence based on sample size and consistency
            let confidence = (failures.len() as f32 / 10.0).min(1.0) * failure_rate;
            
            if confidence < config.confidence_threshold {
                continue;
            }
            
            // Generate suggested fix
            let suggested_fix = Self::generate_fix_suggestion(&category, &error_type, failure_rate);
            
            let first_seen = failures.iter().map(|f| f.timestamp).min().unwrap_or(0);
            let last_seen = failures.iter().map(|f| f.timestamp).max().unwrap_or(0);
            
            detected_patterns.push(FailurePattern {
                description: format!(
                    "{} tasks fail {:.0}% of time with {:?}",
                    category.as_str(),
                    failure_rate * 100.0,
                    error_type
                ),
                category: category.clone(),
                error_type: error_type.clone(),
                failure_rate,
                total_attempts: category_attempts.len(),
                failed_attempts: failures.len(),
                first_seen,
                last_seen,
                confidence,
                suggested_fix,
            });
        }
        
        // Update patterns
        {
            let mut patterns = self.patterns.write();
            *patterns = detected_patterns;
        }
    }
    
    /// Generate fix suggestion based on pattern
    fn generate_fix_suggestion(category: &TaskCategory, error_type: &ErrorType, failure_rate: f32) -> String {
        match (category, error_type) {
            (TaskCategory::CodeGeneration, ErrorType::SyntaxError) => {
                "Add syntax validation step before code generation. Use AST parsing to verify correctness.".to_string()
            }
            (TaskCategory::CodeGeneration, ErrorType::CompilationError) => {
                "Improve type inference and dependency resolution. Add compilation check in validation phase.".to_string()
            }
            (TaskCategory::BugFix, ErrorType::LogicError) => {
                "Enhance bug analysis with more test cases. Add regression testing for similar bugs.".to_string()
            }
            (TaskCategory::Testing, ErrorType::TimeoutError) => {
                "Optimize test execution strategy. Parallelize tests or increase timeout thresholds.".to_string()
            }
            (TaskCategory::CodeRefactoring, ErrorType::CompilationError) => {
                "Add incremental refactoring with compilation checks at each step.".to_string()
            }
            (TaskCategory::Documentation, ErrorType::ValidationError) => {
                "Improve documentation template validation. Add format checking.".to_string()
            }
            (TaskCategory::Optimization, ErrorType::RuntimeError) => {
                "Add performance profiling before optimization. Validate optimizations don't break functionality.".to_string()
            }
            _ => {
                format!(
                    "Pattern detected: {:.0}% failure rate. Review and improve {} handling for {:?} errors.",
                    failure_rate * 100.0,
                    category.as_str(),
                    error_type
                )
            }
        }
    }
    
    /// Get all detected patterns
    pub fn get_patterns(&self) -> Vec<FailurePattern> {
        self.patterns.read().clone()
    }
    
    /// Get high-confidence patterns
    pub fn get_high_confidence_patterns(&self, threshold: f32) -> Vec<FailurePattern> {
        self.patterns.read()
            .iter()
            .filter(|p| p.confidence >= threshold)
            .cloned()
            .collect()
    }
    
    /// Get patterns for specific category
    pub fn get_patterns_for_category(&self, category: &TaskCategory) -> Vec<FailurePattern> {
        self.patterns.read()
            .iter()
            .filter(|p| &p.category == category)
            .cloned()
            .collect()
    }
    
    /// Get overall statistics
    pub fn get_stats(&self) -> TaskStats {
        let attempts = self.attempts.read();
        
        let total = attempts.len();
        let successful = attempts.iter().filter(|a| a.result == TaskResult::Success).count();
        let failed = attempts.iter().filter(|a| a.result == TaskResult::Failed).count();
        let partial = attempts.iter().filter(|a| a.result == TaskResult::Partial).count();
        let timeout = attempts.iter().filter(|a| a.result == TaskResult::Timeout).count();
        
        let success_rate = if total > 0 {
            successful as f32 / total as f32
        } else {
            0.0
        };
        
        let avg_duration_ms = if total > 0 {
            attempts.iter().map(|a| a.duration_ms).sum::<u64>() as f64 / total as f64
        } else {
            0.0
        };
        
        let patterns_detected = self.patterns.read().len();
        
        TaskStats {
            total_attempts: total,
            successful,
            failed,
            partial,
            timeout,
            success_rate,
            avg_duration_ms,
            patterns_detected,
        }
    }
    
    /// Get statistics for specific category
    pub fn get_category_stats(&self, category: &TaskCategory) -> Option<(usize, usize, f32)> {
        let stats = self.category_stats.read();
        stats.get(category).map(|s| {
            let success_rate = if s.total > 0 {
                s.successful as f32 / s.total as f32
            } else {
                0.0
            };
            (s.total, s.successful, success_rate)
        })
    }
    
    /// Get recent failures
    pub fn get_recent_failures(&self, count: usize) -> Vec<TaskAttempt> {
        let attempts = self.attempts.read();
        attempts.iter()
            .rev()
            .filter(|a| a.result == TaskResult::Failed)
            .take(count)
            .cloned()
            .collect()
    }
    
    /// Check if should retry task with different strategy
    pub fn should_retry(&self, task_id: &str, current_strategy: &str) -> (bool, Option<String>) {
        let attempts = self.attempts.read();
        
        // Find previous attempts for this task
        let task_attempts: Vec<&TaskAttempt> = attempts.iter()
            .filter(|a| a.task_id == task_id)
            .collect();
        
        if task_attempts.is_empty() {
            return (true, None); // First attempt
        }
        
        let retry_count = task_attempts.len();
        
        // Don't retry more than 3 times
        if retry_count >= 3 {
            return (false, None);
        }
        
        // Check if current strategy has failed before
        let strategy_failures = task_attempts.iter()
            .filter(|a| a.strategy == current_strategy && a.result == TaskResult::Failed)
            .count();
        
        if strategy_failures > 0 {
            // Suggest alternative strategy
            let alternative = self.suggest_alternative_strategy(
                &task_attempts[0].category,
                current_strategy
            );
            return (true, Some(alternative));
        }
        
        (true, None)
    }
    
    /// Suggest alternative strategy based on failures
    fn suggest_alternative_strategy(&self, category: &TaskCategory, failed_strategy: &str) -> String {
        match category {
            TaskCategory::CodeGeneration => {
                if failed_strategy.contains("direct") {
                    "incremental_with_validation".to_string()
                } else if failed_strategy.contains("incremental") {
                    "template_based".to_string()
                } else {
                    "ast_guided".to_string()
                }
            }
            TaskCategory::BugFix => {
                if failed_strategy.contains("direct") {
                    "test_driven".to_string()
                } else {
                    "root_cause_analysis".to_string()
                }
            }
            TaskCategory::Testing => {
                if failed_strategy.contains("parallel") {
                    "sequential".to_string()
                } else {
                    "parallel_with_timeout".to_string()
                }
            }
            _ => "alternative_approach".to_string(),
        }
    }
    
    /// Clear all history
    pub fn clear(&self) {
        self.attempts.write().clear();
        self.patterns.write().clear();
        self.category_stats.write().clear();
    }
}

impl Default for TaskPatternTracker {
    fn default() -> Self {
        Self::new(TaskTrackerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_category_detection() {
        assert_eq!(
            TaskCategory::from_description("Generate code for API"),
            TaskCategory::CodeGeneration
        );
        assert_eq!(
            TaskCategory::from_description("Fix bug in parser"),
            TaskCategory::BugFix
        );
        assert_eq!(
            TaskCategory::from_description("Refactor database layer"),
            TaskCategory::CodeRefactoring
        );
    }
    
    #[test]
    fn test_error_type_detection() {
        assert_eq!(
            ErrorType::from_message("Syntax error at line 42"),
            ErrorType::SyntaxError
        );
        assert_eq!(
            ErrorType::from_message("Compilation failed"),
            ErrorType::CompilationError
        );
        assert_eq!(
            ErrorType::from_message("Operation timed out"),
            ErrorType::TimeoutError
        );
    }
    
    #[test]
    fn test_pattern_detection() {
        let tracker = TaskPatternTracker::default();
        
        // Record multiple failures of same type
        for i in 0..5 {
            tracker.record_attempt(TaskAttempt {
                task_id: format!("task_{}", i),
                category: TaskCategory::CodeGeneration,
                description: "Generate API".to_string(),
                result: TaskResult::Failed,
                error_type: Some(ErrorType::SyntaxError),
                error_message: Some("Invalid syntax".to_string()),
                timestamp: 1000 + i * 100,
                duration_ms: 500,
                complexity: 5,
                retry_count: 0,
                strategy: "direct".to_string(),
            });
        }
        
        let patterns = tracker.get_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns[0].failure_rate > 0.5);
    }
    
    #[test]
    fn test_stats_calculation() {
        let tracker = TaskPatternTracker::default();
        
        // Record mix of successes and failures
        for i in 0..10 {
            let result = if i % 3 == 0 {
                TaskResult::Failed
            } else {
                TaskResult::Success
            };
            
            let result_clone = result.clone();
            tracker.record_attempt(TaskAttempt {
                task_id: format!("task_{}", i),
                category: TaskCategory::Testing,
                description: "Run tests".to_string(),
                result,
                error_type: if result_clone == TaskResult::Failed {
                    Some(ErrorType::TimeoutError)
                } else {
                    None
                },
                error_message: None,
                timestamp: 1000 + i * 100,
                duration_ms: 1000,
                complexity: 3,
                retry_count: 0,
                strategy: "parallel".to_string(),
            });
        }
        
        let stats = tracker.get_stats();
        assert_eq!(stats.total_attempts, 10);
        assert!(stats.success_rate > 0.5);
    }
}
