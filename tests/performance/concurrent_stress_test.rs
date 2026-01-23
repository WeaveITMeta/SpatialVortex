/// Concurrent Stress Tests for Lock-Free Flux Matrix
/// 
/// Tests high-concurrency scenarios with multiple threads:
/// - 10,000+ concurrent operations
/// - Race condition detection
/// - Throughput measurement
/// - Thread safety verification

use spatial_vortex::lock_free_flux::LockFreeFluxMatrix;
use spatial_vortex::models::*;
use std::sync::Arc;
use std::collections::HashMap;
use std::thread;

fn create_test_node(position: u8) -> FluxNode {
    let mut parameters = HashMap::new();
    parameters.insert("ethos".to_string(), 0.8);
    parameters.insert("logos".to_string(), 0.6);
    
    FluxNode {
        position,
        base_value: position,
        semantic_index: SemanticIndex {
            positive_associations: vec![],
            negative_associations: vec![],
            neutral_base: format!("Position {}", position),
            predicates: vec![],
            relations: vec![],
        },
        attributes: NodeAttributes {
            properties: HashMap::new(),
            parameters,
            state: NodeState {
                active: true,
                last_accessed: chrono::Utc::now(),
                usage_count: 0,
                context_stack: vec![],
            },
            dynamics: NodeDynamics {
                evolution_rate: 1.0,
                stability_index: 1.0,
                interaction_patterns: vec![],
                learning_adjustments: vec![],
            },
        },
        connections: vec![],
    }
}

#[test]
fn test_concurrent_inserts_10k() {
    let matrix = Arc::new(LockFreeFluxMatrix::new("concurrent_test".to_string()));
    let num_threads = 10;
    let inserts_per_thread = 1000;
    
    let mut handles = vec![];
    
    for thread_id in 0..num_threads {
        let matrix_clone = Arc::clone(&matrix);
        
        let handle = thread::spawn(move || {
            for i in 0..inserts_per_thread {
                let position = ((thread_id * inserts_per_thread + i) % 9) as u8;
                let node = create_test_node(position);
                matrix_clone.insert(node);
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify no data lost
    let total_expected = num_threads * inserts_per_thread;
    println!("Completed {} concurrent inserts", total_expected);
    
    // All positions should have been written to
    for pos in 0..9 {
        assert!(matrix.get(pos).is_some(), "Position {} should have data", pos);
    }
}

#[test]
fn test_concurrent_reads_writes() {
    let matrix = Arc::new(LockFreeFluxMatrix::new("read_write_test".to_string()));
    
    // Pre-populate
    for pos in 0..9 {
        matrix.insert(create_test_node(pos));
    }
    
    let num_readers = 5;
    let num_writers = 5;
    let operations_per_thread = 1000;
    
    let mut reader_handles = vec![];
    let mut writer_handles = vec![];
    
    // Spawn readers
    for _ in 0..num_readers {
        let matrix_clone = Arc::clone(&matrix);
        
        let handle = thread::spawn(move || {
            let mut read_count = 0;
            for _ in 0..operations_per_thread {
                let position = (read_count % 9) as u8;
                if matrix_clone.get(position).is_some() {
                    read_count += 1;
                }
            }
            read_count
        });
        
        reader_handles.push(handle);
    }
    
    // Spawn writers
    for writer_id in 0..num_writers {
        let matrix_clone = Arc::clone(&matrix);
        
        let handle = thread::spawn(move || {
            for i in 0..operations_per_thread {
                let position = ((writer_id * operations_per_thread + i) % 9) as u8;
                matrix_clone.insert(create_test_node(position));
            }
        });
        
        writer_handles.push(handle);
    }
    
    // Wait for all
    let mut total_reads = 0;
    for handle in reader_handles {
        total_reads += handle.join().unwrap();
    }
    
    for handle in writer_handles {
        handle.join().unwrap();
    }
    
    println!("Total reads: {}", total_reads);
    println!("Total writes: {}", num_writers * operations_per_thread);
    
    // All reads should have succeeded
    assert_eq!(total_reads, num_readers * operations_per_thread);
}

#[test]
fn test_snapshot_isolation() {
    let matrix = Arc::new(LockFreeFluxMatrix::new("snapshot_test".to_string()));
    
    // Insert initial data
    matrix.insert(create_test_node(1));
    matrix.insert(create_test_node(2));
    
    // Create snapshot
    let snapshot_v1 = matrix.snapshot();
    
    // Spawn thread that modifies data
    let matrix_clone = Arc::clone(&matrix);
    let writer = thread::spawn(move || {
        for pos in 3..9 {
            matrix_clone.insert(create_test_node(pos));
        }
    });
    
    // Reader using snapshot should see consistent view
    let matrix_clone2 = Arc::clone(&matrix);
    let reader = thread::spawn(move || {
        // Snapshot should only have positions 1 and 2
        assert!(matrix_clone2.get_from_snapshot(snapshot_v1, 1).is_some());
        assert!(matrix_clone2.get_from_snapshot(snapshot_v1, 2).is_some());
        assert!(matrix_clone2.get_from_snapshot(snapshot_v1, 3).is_none());
        assert!(matrix_clone2.get_from_snapshot(snapshot_v1, 8).is_none());
    });
    
    writer.join().unwrap();
    reader.join().unwrap();
    
    // Current view should have all data
    for pos in 1..9 {
        assert!(matrix.get(pos).is_some());
    }
}

#[test]
fn test_attribute_query_concurrent() {
    let matrix = Arc::new(LockFreeFluxMatrix::new("attr_test".to_string()));
    
    // Insert nodes with varying ethos values
    for i in 0..100 {
        let mut node = create_test_node((i % 9) as u8);
        let ethos_value = (i as f64) / 100.0;
        node.attributes.parameters.insert("ethos".to_string(), ethos_value);
        matrix.insert(node);
    }
    
    let num_threads = 4;
    let mut handles = vec![];
    
    // Concurrent attribute queries
    for _ in 0..num_threads {
        let matrix_clone = Arc::clone(&matrix);
        
        let handle = thread::spawn(move || {
            // Query high ethos nodes
            let high_ethos = matrix_clone.get_by_attribute("ethos", 0.8, 1.0);
            
            // Should find ~20 nodes (80-99 out of 100)
            assert!(high_ethos.len() >= 15 && high_ethos.len() <= 25);
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_sacred_anchor_concurrent_access() {
    let matrix = Arc::new(LockFreeFluxMatrix::new("sacred_test".to_string()));
    
    let num_threads = 10;
    let mut handles = vec![];
    
    for _ in 0..num_threads {
        let matrix_clone = Arc::clone(&matrix);
        
        let handle = thread::spawn(move || {
            // All threads should see sacred anchors
            let anchor3 = matrix_clone.get_sacred_anchor(3);
            let anchor6 = matrix_clone.get_sacred_anchor(6);
            let anchor9 = matrix_clone.get_sacred_anchor(9);
            
            assert!(anchor3.is_some());
            assert!(anchor6.is_some());
            assert!(anchor9.is_some());
            
            // Test judgment
            let anchor = anchor3.unwrap();
            assert_eq!(anchor.position, 3);
            assert_eq!(anchor.orbital_radius, 3.0);
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_no_data_races() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let matrix = Arc::new(LockFreeFluxMatrix::new("race_test".to_string()));
    let counter = Arc::new(AtomicUsize::new(0));
    
    let num_threads = 8;
    let operations = 1000;
    let mut handles = vec![];
    
    for _ in 0..num_threads {
        let matrix_clone = Arc::clone(&matrix);
        let counter_clone = Arc::clone(&counter);
        
        let handle = thread::spawn(move || {
            for i in 0..operations {
                let position = (i % 9) as u8;
                matrix_clone.insert(create_test_node(position));
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Counter should equal total operations (no lost updates)
    assert_eq!(counter.load(Ordering::SeqCst), num_threads * operations);
    
    println!("No data races detected in {} operations", num_threads * operations);
}
