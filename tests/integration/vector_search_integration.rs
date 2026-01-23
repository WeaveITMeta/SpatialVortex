/// Vector Search Integration Tests
/// 
/// Tests:
/// 1. Large-scale indexing (10K+ vectors)
/// 2. Search accuracy (recall rate)
/// 3. Position filtering
/// 4. ELP filtering
/// 5. Concurrent access
/// 6. Sacred position search

use spatial_vortex::vector_search::{VectorIndex, VectorMetadata, VECTOR_DIM, HNSWConfig};
use ndarray::Array1;
use std::sync::Arc;
use std::thread;

fn random_vector() -> Array1<f32> {
    Array1::from_iter((0..VECTOR_DIM).map(|_| rand::random::<f32>()))
}

fn normalized_vector(values: &[f32]) -> Array1<f32> {
    let mut arr = Array1::from_vec(values.to_vec());
    let norm = arr.dot(&arr).sqrt();
    if norm > 0.0 {
        arr = arr / norm;
    }
    arr
}

#[test]
fn test_large_scale_indexing() {
    let index = VectorIndex::new_default();
    
    // Index 10,000 vectors
    for i in 0..10000 {
        let vector = random_vector();
        let metadata = VectorMetadata {
            position: Some((i % 10) as u8),
            sacred: [3, 6, 9].contains(&((i % 10) as u8)),
            ethos: (i % 100) as f32 / 100.0,
            logos: (i % 50) as f32 / 50.0,
            pathos: (i % 25) as f32 / 25.0,
            created_at: std::time::SystemTime::now(),
        };
        
        index.add(format!("vec_{}", i), vector, metadata).unwrap();
    }
    
    let stats = index.stats();
    assert_eq!(stats.total_vectors, 10000);
    
    println!("✅ Indexed {} vectors", stats.total_vectors);
}

#[test]
fn test_search_accuracy() {
    let index = VectorIndex::new_default();
    
    // Create known vectors
    let base_vector = normalized_vector(&vec![1.0; VECTOR_DIM]);
    let metadata = VectorMetadata::default();
    
    // Add base vector
    index.add("base".to_string(), base_vector.clone(), metadata.clone()).unwrap();
    
    // Add similar vectors
    for i in 0..100 {
        let mut similar = base_vector.clone();
        // Add small noise
        for j in 0..VECTOR_DIM {
            similar[j] += (rand::random::<f32>() - 0.5) * 0.1;
        }
        index.add(format!("similar_{}", i), similar, metadata.clone()).unwrap();
    }
    
    // Add dissimilar vectors
    for i in 0..100 {
        let dissimilar = random_vector();
        index.add(format!("dissimilar_{}", i), dissimilar, metadata.clone()).unwrap();
    }
    
    // Search for base vector
    let results = index.search(&base_vector, 10).unwrap();
    
    // Top result should be the base vector itself
    assert_eq!(results[0].id, "base");
    assert!(results[0].score > 0.95);
    
    // Most results should be "similar_*"
    let similar_count = results.iter()
        .filter(|r| r.id.starts_with("similar_"))
        .count();
    
    assert!(similar_count >= 7, "Expected at least 7 similar results, got {}", similar_count);
    
    println!("✅ Search accuracy: {}/10 similar results", similar_count);
}

#[test]
fn test_position_filtering() {
    let index = VectorIndex::new_default();
    
    // Add vectors at different positions
    for pos in 0..10u8 {
        for i in 0..100 {
            let vector = random_vector();
            let metadata = VectorMetadata {
                position: Some(pos),
                sacred: [3, 6, 9].contains(&pos),
                ..Default::default()
            };
            index.add(format!("pos_{}_vec_{}", pos, i), vector, metadata).unwrap();
        }
    }
    
    // Search for position 3 (sacred)
    let query = random_vector();
    let results = index.search_by_position(&query, 20, 3).unwrap();
    
    // All results should be position 3
    for result in &results {
        assert_eq!(result.metadata.position, Some(3));
        assert!(result.metadata.sacred);
    }
    
    println!("✅ Position filter: All {} results at position 3", results.len());
}

#[test]
fn test_sacred_position_search() {
    let index = VectorIndex::new_default();
    
    // Add vectors at sacred positions (3, 6, 9)
    let sacred_positions = [3, 6, 9];
    for pos in sacred_positions {
        for i in 0..50 {
            let vector = random_vector();
            let metadata = VectorMetadata {
                position: Some(pos),
                sacred: true,
                ethos: 0.9,  // High ethos for sacred
                ..Default::default()
            };
            index.add(format!("sacred_{}_vec_{}", pos, i), vector, metadata).unwrap();
        }
    }
    
    // Add non-sacred vectors
    for pos in [0, 1, 2, 4, 5, 7, 8] {
        for i in 0..50 {
            let vector = random_vector();
            let metadata = VectorMetadata {
                position: Some(pos),
                sacred: false,
                ethos: 0.3,
                ..Default::default()
            };
            index.add(format!("normal_{}_vec_{}", pos, i), vector, metadata).unwrap();
        }
    }
    
    // Search each sacred position
    for sacred_pos in sacred_positions {
        let query = random_vector();
        let results = index.search_by_position(&query, 10, sacred_pos).unwrap();
        
        for result in &results {
            assert_eq!(result.metadata.position, Some(sacred_pos));
            assert!(result.metadata.sacred);
        }
        
        println!("✅ Sacred position {}: {} results", sacred_pos, results.len());
    }
}

#[test]
fn test_elp_filtering() {
    let index = VectorIndex::new_default();
    
    // Add vectors with varying ethos values
    for i in 0..100 {
        let vector = random_vector();
        let ethos = (i as f32) / 100.0;  // 0.0 to 0.99
        let metadata = VectorMetadata {
            ethos,
            logos: 0.5,
            pathos: 0.5,
            ..Default::default()
        };
        index.add(format!("vec_{}", i), vector, metadata).unwrap();
    }
    
    // Search with high ethos filter
    let query = random_vector();
    let results = index.search_by_elp(&query, 20, 0.8).unwrap();
    
    // All results should have ethos >= 0.8
    for result in &results {
        assert!(result.metadata.ethos >= 0.8, 
            "Expected ethos >= 0.8, got {}", result.metadata.ethos);
    }
    
    println!("✅ ELP filter: {} results with ethos >= 0.8", results.len());
}

#[test]
fn test_concurrent_search() {
    let index = Arc::new(VectorIndex::new_default());
    
    // Add 1000 vectors
    for i in 0..1000 {
        let vector = random_vector();
        let metadata = VectorMetadata {
            position: Some((i % 10) as u8),
            ..Default::default()
        };
        index.add(format!("vec_{}", i), vector, metadata).unwrap();
    }
    
    // Concurrent searches from multiple threads
    let mut handles = Vec::new();
    let num_threads = 8;
    let searches_per_thread = 100;
    
    for thread_id in 0..num_threads {
        let index_clone = Arc::clone(&index);
        
        let handle = thread::spawn(move || {
            let mut total_results = 0;
            for _ in 0..searches_per_thread {
                let query = random_vector();
                let results = index_clone.search(&query, 10).unwrap();
                total_results += results.len();
            }
            total_results
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    let mut total_searches = 0;
    for handle in handles {
        total_searches += handle.join().unwrap();
    }
    
    let expected = num_threads * searches_per_thread * 10;  // k=10
    assert!(total_searches > 0);
    
    println!("✅ Concurrent search: {} threads × {} searches = {} total results", 
        num_threads, searches_per_thread, total_searches);
}

#[test]
fn test_empty_index_search() {
    let index = VectorIndex::new_default();
    
    let query = random_vector();
    let results = index.search(&query, 10).unwrap();
    
    assert!(results.is_empty());
    println!("✅ Empty index search returns empty results");
}

#[test]
fn test_single_vector_search() {
    let index = VectorIndex::new_default();
    
    let vector = random_vector();
    let metadata = VectorMetadata::default();
    index.add("single".to_string(), vector.clone(), metadata).unwrap();
    
    let results = index.search(&vector, 5).unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "single");
    assert!(results[0].score > 0.99);  // Should be very similar to itself
    
    println!("✅ Single vector search: score = {:.4}", results[0].score);
}
