/// Coding Agent Benchmark Tests
/// Based on standard programming challenges from LeetCode, HackerRank, and Project Euler
/// 
/// Test Categories:
/// 1. Easy: Basic algorithms and data structures
/// 2. Medium: Intermediate problem-solving
/// 3. Hard: Complex algorithms and optimizations

use spatial_vortex::agents::coding_agent::{CodingAgent, CodeRequest};

/// Easy Level Tests (LeetCode Easy)
#[tokio::test]
async fn test_two_sum() {
    let agent = CodingAgent::new().await.expect("Failed to create agent");
    
    let request = CodeRequest {
        description: "Write a Python function called two_sum that takes a list of integers and a target integer. \
                     Return indices of the two numbers that add up to the target. \
                     Example: two_sum([2, 7, 11, 15], 9) should return [0, 1]".to_string(),
        language: Some("python".to_string()),
        context: None,
    };
    
    let result = agent.generate_code(request).await;
    assert!(result.is_ok(), "Failed to generate two_sum: {:?}", result.err());
    
    let code = result.unwrap();
    assert!(code.code.contains("def two_sum"), "Missing function definition");
    assert_eq!(code.execution_result.as_ref().unwrap().exit_code, 0, "Execution failed");
}

#[tokio::test]
async fn test_palindrome_check() {
    let agent = CodingAgent::new().await.expect("Failed to create agent");
    
    let request = CodeRequest {
        description: "Write a Python function called is_palindrome that checks if a string is a palindrome. \
                     Ignore spaces and case. Example: is_palindrome('A man a plan a canal Panama') returns True".to_string(),
        language: Some("python".to_string()),
        context: None,
    };
    
    let result = agent.generate_code(request).await;
    assert!(result.is_ok(), "Failed to generate palindrome checker");
    
    let code = result.unwrap();
    assert!(code.code.contains("def is_palindrome"), "Missing function definition");
    assert_eq!(code.execution_result.as_ref().unwrap().exit_code, 0, "Execution failed");
}

#[tokio::test]
async fn test_reverse_string() {
    let agent = CodingAgent::new().await.expect("Failed to create agent");
    
    let request = CodeRequest {
        description: "Write a Python function called reverse_string that reverses a string in-place. \
                     Example: reverse_string('hello') returns 'olleh'".to_string(),
        language: Some("python".to_string()),
        context: None,
    };
    
    let result = agent.generate_code(request).await;
    assert!(result.is_ok(), "Failed to generate reverse_string");
    
    let code = result.unwrap();
    assert!(code.code.contains("def reverse_string"), "Missing function definition");
}

#[tokio::test]
async fn test_fibonacci() {
    let agent = CodingAgent::new().await.expect("Failed to create agent");
    
    let request = CodeRequest {
        description: "Write a Python function called fibonacci that returns the nth Fibonacci number. \
                     Use dynamic programming for efficiency. Example: fibonacci(10) returns 55".to_string(),
        language: Some("python".to_string()),
        context: None,
    };
    
    let result = agent.generate_code(request).await;
    assert!(result.is_ok(), "Failed to generate fibonacci");
    
    let code = result.unwrap();
    assert!(code.code.contains("def fibonacci"), "Missing function definition");
}

/// Medium Level Tests (LeetCode Medium)
#[tokio::test]
async fn test_merge_intervals() {
    let agent = CodingAgent::new().await.expect("Failed to create agent");
    
    let request = CodeRequest {
        description: "Write a Python function called merge_intervals that takes a list of intervals and merges overlapping ones. \
                     Example: merge_intervals([[1,3],[2,6],[8,10],[15,18]]) returns [[1,6],[8,10],[15,18]]".to_string(),
        language: Some("python".to_string()),
        context: None,
    };
    
    let result = agent.generate_code(request).await;
    assert!(result.is_ok(), "Failed to generate merge_intervals");
    
    let code = result.unwrap();
    assert!(code.code.contains("def merge_intervals"), "Missing function definition");
}

#[tokio::test]
async fn test_longest_substring() {
    let agent = CodingAgent::new().await.expect("Failed to create agent");
    
    let request = CodeRequest {
        description: "Write a Python function called longest_substring_without_repeating that finds the length of the longest substring without repeating characters. \
                     Example: longest_substring_without_repeating('abcabcbb') returns 3 (for 'abc')".to_string(),
        language: Some("python".to_string()),
        context: None,
    };
    
    let result = agent.generate_code(request).await;
    assert!(result.is_ok(), "Failed to generate longest_substring");
    
    let code = result.unwrap();
    assert!(code.code.contains("def longest_substring"), "Missing function definition");
}

#[tokio::test]
async fn test_binary_search_tree() {
    let agent = CodingAgent::new().await.expect("Failed to create agent");
    
    let request = CodeRequest {
        description: "Write a Python class called TreeNode and a function called is_valid_bst that validates if a binary tree is a valid binary search tree. \
                     In a BST, left subtree < node < right subtree for all nodes.".to_string(),
        language: Some("python".to_string()),
        context: None,
    };
    
    let result = agent.generate_code(request).await;
    assert!(result.is_ok(), "Failed to generate BST validator");
    
    let code = result.unwrap();
    assert!(code.code.contains("class TreeNode") || code.code.contains("def is_valid_bst"), "Missing class or function");
}

/// Hard Level Tests (LeetCode Hard)
#[tokio::test]
async fn test_median_two_sorted_arrays() {
    let agent = CodingAgent::new().await.expect("Failed to create agent");
    
    let request = CodeRequest {
        description: "Write a Python function called find_median_sorted_arrays that finds the median of two sorted arrays in O(log(m+n)) time. \
                     Example: find_median_sorted_arrays([1,3], [2]) returns 2.0".to_string(),
        language: Some("python".to_string()),
        context: None,
    };
    
    let result = agent.generate_code(request).await;
    assert!(result.is_ok(), "Failed to generate median finder");
    
    let code = result.unwrap();
    assert!(code.code.contains("def find_median_sorted_arrays"), "Missing function definition");
}

#[tokio::test]
async fn test_word_ladder() {
    let agent = CodingAgent::new().await.expect("Failed to create agent");
    
    let request = CodeRequest {
        description: "Write a Python function called word_ladder that finds the shortest transformation sequence from start word to end word, \
                     changing only one letter at a time. Each transformed word must exist in the word list. \
                     Example: word_ladder('hit', 'cog', ['hot','dot','dog','lot','log','cog']) returns 5".to_string(),
        language: Some("python".to_string()),
        context: None,
    };
    
    let result = agent.generate_code(request).await;
    assert!(result.is_ok(), "Failed to generate word_ladder");
    
    let code = result.unwrap();
    assert!(code.code.contains("def word_ladder"), "Missing function definition");
}

/// Multi-Language Tests
#[tokio::test]
async fn test_rust_quicksort() {
    let agent = CodingAgent::new().await.expect("Failed to create agent");
    
    let request = CodeRequest {
        description: "Write a Rust function that implements quicksort for a Vec<i32>. \
                     The function should be called quicksort and use in-place partitioning.".to_string(),
        language: Some("rust".to_string()),
        context: None,
    };
    
    let result = agent.generate_code(request).await;
    assert!(result.is_ok(), "Failed to generate Rust quicksort");
    
    let code = result.unwrap();
    assert!(code.code.contains("fn quicksort") || code.code.contains("pub fn quicksort"), "Missing function definition");
}

#[tokio::test]
async fn test_javascript_debounce() {
    let agent = CodingAgent::new().await.expect("Failed to create agent");
    
    let request = CodeRequest {
        description: "Write a JavaScript function called debounce that takes a function and delay in milliseconds. \
                     It should return a debounced version that only executes after the delay has passed without being called again.".to_string(),
        language: Some("javascript".to_string()),
        context: None,
    };
    
    let result = agent.generate_code(request).await;
    assert!(result.is_ok(), "Failed to generate JavaScript debounce");
    
    let code = result.unwrap();
    assert!(code.code.contains("function debounce") || code.code.contains("const debounce"), "Missing function definition");
}

/// Algorithm Optimization Tests
#[tokio::test]
async fn test_dynamic_programming() {
    let agent = CodingAgent::new().await.expect("Failed to create agent");
    
    let request = CodeRequest {
        description: "Write a Python function called coin_change that solves the coin change problem using dynamic programming. \
                     Given a list of coin denominations and a target amount, return the minimum number of coins needed. \
                     Example: coin_change([1,2,5], 11) returns 3 (11 = 5+5+1)".to_string(),
        language: Some("python".to_string()),
        context: None,
    };
    
    let result = agent.generate_code(request).await;
    assert!(result.is_ok(), "Failed to generate coin_change");
    
    let code = result.unwrap();
    assert!(code.code.contains("def coin_change"), "Missing function definition");
}

/// Integration Test: Complex Multi-Step Problem
#[tokio::test]
async fn test_complex_data_structure() {
    let agent = CodingAgent::new().await.expect("Failed to create agent");
    
    let request = CodeRequest {
        description: "Implement an LRU (Least Recently Used) Cache in Python with O(1) get and put operations. \
                     Use a combination of dictionary and doubly linked list. \
                     The class should be called LRUCache with methods get(key) and put(key, value).".to_string(),
        language: Some("python".to_string()),
        context: None,
    };
    
    let result = agent.generate_code(request).await;
    assert!(result.is_ok(), "Failed to generate LRU Cache");
    
    let code = result.unwrap();
    assert!(code.code.contains("class LRUCache"), "Missing class definition");
    assert!(code.code.contains("def get") || code.code.contains("def put"), "Missing required methods");
}
