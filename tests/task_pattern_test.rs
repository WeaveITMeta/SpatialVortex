//! Task Pattern Tracker Tests

use spatial_vortex::asi::task_pattern_tracker::{
    TaskPatternTracker, TaskTrackerConfig, TaskAttempt, TaskResult,
    TaskCategory, ErrorType,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_task_category_detection() {
    assert_eq!(
        TaskCategory::from_description("Generate code for REST API"),
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
    
    assert_eq!(
        TaskCategory::from_description("Write unit tests"),
        TaskCategory::Testing
    );
    
    assert_eq!(
        TaskCategory::from_description("Optimize query performance"),
        TaskCategory::Optimization
    );
}

#[test]
fn test_error_type_detection() {
    assert_eq!(
        ErrorType::from_message("Syntax error at line 42"),
        ErrorType::SyntaxError
    );
    
    assert_eq!(
        ErrorType::from_message("Compilation failed with errors"),
        ErrorType::CompilationError
    );
    
    assert_eq!(
        ErrorType::from_message("Operation timed out after 30s"),
        ErrorType::TimeoutError
    );
    
    assert_eq!(
        ErrorType::from_message("Logic error: incorrect result"),
        ErrorType::LogicError
    );
    
    assert_eq!(
        ErrorType::from_message("Runtime panic: index out of bounds"),
        ErrorType::RuntimeError
    );
}

#[test]
fn test_pattern_detection() {
    let tracker = TaskPatternTracker::default();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    // Record multiple failures of same type
    for i in 0..5 {
        tracker.record_attempt(TaskAttempt {
            task_id: format!("task_{}", i),
            category: TaskCategory::CodeGeneration,
            description: "Generate API".to_string(),
            result: TaskResult::Failed,
            error_type: Some(ErrorType::SyntaxError),
            error_message: Some("Invalid syntax".to_string()),
            timestamp: now + i,
            duration_ms: 500,
            complexity: 5,
            retry_count: 0,
            strategy: "direct".to_string(),
        });
    }
    
    let patterns = tracker.get_patterns();
    assert!(!patterns.is_empty(), "Should detect pattern");
    
    let pattern = &patterns[0];
    assert_eq!(pattern.category, TaskCategory::CodeGeneration);
    assert_eq!(pattern.error_type, ErrorType::SyntaxError);
    assert!(pattern.failure_rate >= 0.5, "Failure rate should be >= 50%");
}

#[test]
fn test_stats_calculation() {
    let tracker = TaskPatternTracker::default();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    // Record mix of successes and failures
    for i in 0..10 {
        let result = if i % 3 == 0 {
            TaskResult::Failed
        } else {
            TaskResult::Success
        };
        
        tracker.record_attempt(TaskAttempt {
            task_id: format!("task_{}", i),
            category: TaskCategory::Testing,
            description: "Run tests".to_string(),
            result,
            error_type: if result == TaskResult::Failed {
                Some(ErrorType::TimeoutError)
            } else {
                None
            },
            error_message: None,
            timestamp: now + i,
            duration_ms: 1000,
            complexity: 3,
            retry_count: 0,
            strategy: "parallel".to_string(),
        });
    }
    
    let stats = tracker.get_stats();
    assert_eq!(stats.total_attempts, 10);
    assert_eq!(stats.failed, 4); // 0, 3, 6, 9
    assert_eq!(stats.successful, 6);
    assert!((stats.success_rate - 0.6).abs() < 0.01);
}

#[test]
fn test_category_stats() {
    let tracker = TaskPatternTracker::default();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    // Add code generation tasks
    for i in 0..5 {
        tracker.record_attempt(TaskAttempt {
            task_id: format!("codegen_{}", i),
            category: TaskCategory::CodeGeneration,
            description: "Generate code".to_string(),
            result: if i < 2 { TaskResult::Failed } else { TaskResult::Success },
            error_type: if i < 2 { Some(ErrorType::SyntaxError) } else { None },
            error_message: None,
            timestamp: now + i,
            duration_ms: 1000,
            complexity: 5,
            retry_count: 0,
            strategy: "direct".to_string(),
        });
    }
    
    let (total, successful, rate) = tracker.get_category_stats(&TaskCategory::CodeGeneration).unwrap();
    assert_eq!(total, 5);
    assert_eq!(successful, 3);
    assert!((rate - 0.6).abs() < 0.01);
}

#[test]
fn test_high_confidence_patterns() {
    let tracker = TaskPatternTracker::default();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    // Create strong pattern (10 failures)
    for i in 0..10 {
        tracker.record_attempt(TaskAttempt {
            task_id: format!("task_{}", i),
            category: TaskCategory::BugFix,
            description: "Fix bug".to_string(),
            result: TaskResult::Failed,
            error_type: Some(ErrorType::LogicError),
            error_message: Some("Logic error".to_string()),
            timestamp: now + i,
            duration_ms: 2000,
            complexity: 7,
            retry_count: 0,
            strategy: "direct".to_string(),
        });
    }
    
    let high_conf = tracker.get_high_confidence_patterns(0.7);
    assert!(!high_conf.is_empty(), "Should have high confidence patterns");
    
    for pattern in &high_conf {
        assert!(pattern.confidence >= 0.7);
    }
}

#[test]
fn test_retry_logic() {
    let tracker = TaskPatternTracker::default();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    let task_id = "retry_test";
    let strategy = "direct";
    
    // First attempt - should retry
    let (should_retry, alt) = tracker.should_retry(task_id, strategy);
    assert!(should_retry);
    assert!(alt.is_none()); // No alternative on first attempt
    
    // Record failure with same strategy
    tracker.record_attempt(TaskAttempt {
        task_id: task_id.to_string(),
        category: TaskCategory::CodeGeneration,
        description: "Test".to_string(),
        result: TaskResult::Failed,
        error_type: Some(ErrorType::SyntaxError),
        error_message: None,
        timestamp: now,
        duration_ms: 1000,
        complexity: 5,
        retry_count: 0,
        strategy: strategy.to_string(),
    });
    
    // Second attempt - should suggest alternative
    let (should_retry, alt) = tracker.should_retry(task_id, strategy);
    assert!(should_retry);
    assert!(alt.is_some()); // Should suggest alternative
}

#[test]
fn test_recent_failures() {
    let tracker = TaskPatternTracker::default();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    // Add mix of results
    for i in 0..10 {
        tracker.record_attempt(TaskAttempt {
            task_id: format!("task_{}", i),
            category: TaskCategory::Testing,
            description: "Test".to_string(),
            result: if i % 2 == 0 { TaskResult::Failed } else { TaskResult::Success },
            error_type: if i % 2 == 0 { Some(ErrorType::TimeoutError) } else { None },
            error_message: None,
            timestamp: now + i,
            duration_ms: 1000,
            complexity: 3,
            retry_count: 0,
            strategy: "test".to_string(),
        });
    }
    
    let recent = tracker.get_recent_failures(3);
    assert_eq!(recent.len(), 3);
    
    // Should be in reverse order (most recent first)
    assert_eq!(recent[0].task_id, "task_8");
    assert_eq!(recent[1].task_id, "task_6");
    assert_eq!(recent[2].task_id, "task_4");
}

#[test]
fn test_clear_history() {
    let tracker = TaskPatternTracker::default();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    // Add some attempts
    for i in 0..5 {
        tracker.record_attempt(TaskAttempt {
            task_id: format!("task_{}", i),
            category: TaskCategory::CodeGeneration,
            description: "Test".to_string(),
            result: TaskResult::Success,
            error_type: None,
            error_message: None,
            timestamp: now + i,
            duration_ms: 1000,
            complexity: 5,
            retry_count: 0,
            strategy: "test".to_string(),
        });
    }
    
    let stats_before = tracker.get_stats();
    assert_eq!(stats_before.total_attempts, 5);
    
    tracker.clear();
    
    let stats_after = tracker.get_stats();
    assert_eq!(stats_after.total_attempts, 0);
    assert!(tracker.get_patterns().is_empty());
}

#[test]
fn test_suggested_fixes() {
    let tracker = TaskPatternTracker::default();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    // Create pattern for code generation syntax errors
    for i in 0..5 {
        tracker.record_attempt(TaskAttempt {
            task_id: format!("task_{}", i),
            category: TaskCategory::CodeGeneration,
            description: "Generate code".to_string(),
            result: TaskResult::Failed,
            error_type: Some(ErrorType::SyntaxError),
            error_message: Some("Syntax error".to_string()),
            timestamp: now + i,
            duration_ms: 1000,
            complexity: 5,
            retry_count: 0,
            strategy: "direct".to_string(),
        });
    }
    
    let patterns = tracker.get_patterns();
    assert!(!patterns.is_empty());
    
    let pattern = &patterns[0];
    assert!(!pattern.suggested_fix.is_empty());
    assert!(pattern.suggested_fix.contains("syntax") || pattern.suggested_fix.contains("validation"));
}

#[test]
fn test_multiple_categories() {
    let tracker = TaskPatternTracker::default();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    // Add different categories
    for i in 0..3 {
        tracker.record_attempt(TaskAttempt {
            task_id: format!("codegen_{}", i),
            category: TaskCategory::CodeGeneration,
            description: "Generate".to_string(),
            result: TaskResult::Failed,
            error_type: Some(ErrorType::SyntaxError),
            error_message: None,
            timestamp: now + i,
            duration_ms: 1000,
            complexity: 5,
            retry_count: 0,
            strategy: "direct".to_string(),
        });
        
        tracker.record_attempt(TaskAttempt {
            task_id: format!("bugfix_{}", i),
            category: TaskCategory::BugFix,
            description: "Fix bug".to_string(),
            result: TaskResult::Failed,
            error_type: Some(ErrorType::LogicError),
            error_message: None,
            timestamp: now + 100 + i,
            duration_ms: 2000,
            complexity: 7,
            retry_count: 0,
            strategy: "direct".to_string(),
        });
    }
    
    let codegen_patterns = tracker.get_patterns_for_category(&TaskCategory::CodeGeneration);
    let bugfix_patterns = tracker.get_patterns_for_category(&TaskCategory::BugFix);
    
    assert!(!codegen_patterns.is_empty());
    assert!(!bugfix_patterns.is_empty());
}
