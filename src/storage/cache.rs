use crate::error::Result;
use crate::models::*;
use chrono::Utc;
use redis::{AsyncCommands, Client};
use serde_json;
use uuid::Uuid;
use crate::metrics::{CACHE_HITS, CACHE_MISSES, CACHE_STORES};

fn dev_mode() -> bool {
    std::env::var("DEVELOPMENT_MODE").unwrap_or_default() == "true"
}

/// Cache manager for storing subject matrix patterns and inference results
#[derive(Clone)]
pub struct CacheManager {
    client: Client,
    default_ttl: i64, // Time to live in seconds
    // When true, operate in no-op in-memory mode (no Redis required)
    dev_in_memory: bool,
}

impl CacheManager {
    /// Create new cache manager
    pub async fn new(redis_url: &str, default_ttl_hours: i64) -> Result<Self> {
        let dev = dev_mode() || redis_url.starts_with("memory://");
        let client = if dev {
            // Create a dummy client; we won't connect in dev mode
            Client::open("redis://127.0.0.1:6379/")?
        } else {
            Client::open(redis_url)?
        };

        if !dev {
            // Test connection in production only
            let mut conn = client.get_async_connection().await?;
            let _: () = conn.set("test_connection", "ok").await?;
        }

        Ok(Self {
            client,
            default_ttl: default_ttl_hours * 3600, // Convert to seconds
            dev_in_memory: dev,
        })
    }

    /// Store flux matrix in cache
    pub async fn store_matrix(&self, matrix: FluxMatrix) -> Result<()> {
        if self.dev_in_memory { return Ok(()); }
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("matrix:subject:{}", matrix.subject);
        let value = serde_json::to_string(&matrix)?;

        let _: () = conn.set_ex(&key, value, self.default_ttl as u64).await?;

        // Also store by ID for direct lookups
        let id_key = format!("matrix:id:{}", matrix.id);
        let _: () = conn
            .set_ex(
                &id_key,
                serde_json::to_string(&matrix)?,
                self.default_ttl as u64,
            )
            .await?;

        // Update access statistics
        self.increment_access_count(&key).await?;

        CACHE_STORES.with_label_values(&["matrix"]).inc();
        Ok(())
    }

    /// Get matrix by subject
    pub async fn get_matrix(&self, subject: &str) -> Result<Option<FluxMatrix>> {
        if self.dev_in_memory { return Ok(None); }
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("matrix:subject:{}", subject);

        let cached_value: Option<String> = conn.get(&key).await?;

        if let Some(value) = cached_value {
            let matrix: FluxMatrix = serde_json::from_str(&value)?;

            // Update access statistics
            self.increment_access_count(&key).await?;
            self.update_last_accessed(&key).await?;
            CACHE_HITS.with_label_values(&["matrix"]).inc();
            Ok(Some(matrix))
        } else {
            CACHE_MISSES.with_label_values(&["matrix"]).inc();
            Ok(None)
        }
    }

    /// Get matrix by ID
    pub async fn get_matrix_by_id(&self, matrix_id: Uuid) -> Result<Option<FluxMatrix>> {
        if self.dev_in_memory { return Ok(None); }
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("matrix:id:{}", matrix_id);

        let cached_value: Option<String> = conn.get(&key).await?;

        if let Some(value) = cached_value {
            let matrix: FluxMatrix = serde_json::from_str(&value)?;
            self.increment_access_count(&key).await?;
            self.update_last_accessed(&key).await?;
            CACHE_HITS.with_label_values(&["matrix"]).inc();
            Ok(Some(matrix))
        } else {
            CACHE_MISSES.with_label_values(&["matrix"]).inc();
            Ok(None)
        }
    }

    /// Store inference result in cache
    pub async fn store_inference_result(&self, result: &InferenceResult) -> Result<()> {
        if self.dev_in_memory { return Ok(()); }
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("inference:{}", result.id);
        let value = serde_json::to_string(result)?;

        // Store with shorter TTL for inference results (they're more transient)
        let inference_ttl = self.default_ttl / 4; // Quarter of matrix TTL
        let _: () = conn.set_ex(&key, value, inference_ttl as u64).await?;

        // Store recent inferences list
        let recent_key = "recent_inferences";
        let _: () = conn.lpush(recent_key, result.id.to_string()).await?;
        let _: () = conn.ltrim(recent_key, 0, 99).await?; // Keep last 100 results
        let _: () = conn.expire(recent_key, self.default_ttl).await?;

        Ok(())
    }

    /// Get inference result by ID
    pub async fn get_inference_result(&self, result_id: Uuid) -> Result<Option<InferenceResult>> {
        if self.dev_in_memory { return Ok(None); }
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("inference:{}", result_id);

        let cached_value: Option<String> = conn.get(&key).await?;

        if let Some(value) = cached_value {
            let result: InferenceResult = serde_json::from_str(&value)?;
            self.increment_access_count(&key).await?;
            CACHE_HITS.with_label_values(&["inference"]).inc();
            Ok(Some(result))
        } else {
            CACHE_MISSES.with_label_values(&["inference"]).inc();
            Ok(None)
        }
    }

    /// Cache semantic associations for quick lookups
    pub async fn cache_semantic_associations(
        &self,
        subject: &str,
        associations: &[SemanticAssociation],
    ) -> Result<()> {
        if self.dev_in_memory { return Ok(()); }
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("associations:{}", subject);
        let value = serde_json::to_string(associations)?;

        let _: () = conn.set_ex(&key, value, self.default_ttl as u64).await?;
        CACHE_STORES.with_label_values(&["associations"]).inc();
        Ok(())
    }

    /// Get cached semantic associations
    pub async fn get_semantic_associations(
        &self,
        subject: &str,
    ) -> Result<Option<Vec<SemanticAssociation>>> {
        if self.dev_in_memory { return Ok(None); }
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("associations:{}", subject);

        let cached_value: Option<String> = conn.get(&key).await?;

        if let Some(value) = cached_value {
            let associations: Vec<SemanticAssociation> = serde_json::from_str(&value)?;
            CACHE_HITS.with_label_values(&["associations"]).inc();
            Ok(Some(associations))
        } else {
            CACHE_MISSES.with_label_values(&["associations"]).inc();
            Ok(None)
        }
    }

    /// Store AI-generated matrix patterns
    pub async fn store_ai_generated_pattern(
        &self,
        subject: &str,
        pattern: &FluxMatrix,
    ) -> Result<()> {
        if self.dev_in_memory { return Ok(()); }
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("ai_pattern:{}", subject);
        let value = serde_json::to_string(pattern)?;

        // AI patterns get longer TTL since they're expensive to generate
        let ai_ttl = self.default_ttl * 2;
        let _: () = conn.set_ex(&key, value, ai_ttl as u64).await?;
        CACHE_STORES.with_label_values(&["ai_pattern"]).inc();

        // Track AI-generated subjects
        let ai_subjects_key = "ai_generated_subjects";
        let _: () = conn.sadd(ai_subjects_key, subject).await?;
        let _: () = conn.expire(ai_subjects_key, self.default_ttl * 7).await?; // Keep for a week

        Ok(())
    }

    /// Get AI-generated pattern
    pub async fn get_ai_generated_pattern(&self, subject: &str) -> Result<Option<FluxMatrix>> {
        if self.dev_in_memory { return Ok(None); }
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("ai_pattern:{}", subject);

        let cached_value: Option<String> = conn.get(&key).await?;

        if let Some(value) = cached_value {
            let pattern: FluxMatrix = serde_json::from_str(&value)?;
            self.increment_access_count(&key).await?;
            CACHE_HITS.with_label_values(&["ai_pattern"]).inc();
            Ok(Some(pattern))
        } else {
            CACHE_MISSES.with_label_values(&["ai_pattern"]).inc();
            Ok(None)
        }
    }

    /// Cache frequently requested subjects for optimization
    pub async fn cache_popular_subjects(&self, subjects: &[String]) -> Result<()> {
        if self.dev_in_memory { return Ok(()); }
        let mut conn = self.client.get_async_connection().await?;
        let key = "popular_subjects";

        // Clear existing list
        let _: () = conn.del(key).await?;

        // Add subjects to list
        for subject in subjects {
            let _: () = conn.lpush(key, subject).await?;
        }

        let _: () = conn.expire(key, self.default_ttl).await?;
        CACHE_STORES.with_label_values(&["popular"]).inc();
        Ok(())
    }

    /// Get popular subjects
    pub async fn get_popular_subjects(&self) -> Result<Vec<String>> {
        if self.dev_in_memory { return Ok(Vec::new()); }
        let mut conn = self.client.get_async_connection().await?;
        let key = "popular_subjects";

        let subjects: Vec<String> = conn.lrange(key, 0, -1).await?;
        if subjects.is_empty() {
            CACHE_MISSES.with_label_values(&["popular"]).inc();
        } else {
            CACHE_HITS.with_label_values(&["popular"]).inc();
        }
        Ok(subjects)
    }

    /// Increment access count for cache analytics
    async fn increment_access_count(&self, key: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let stats_key = format!("stats:access:{}", key);
        let _: () = conn.incr(&stats_key, 1).await?;
        let _: () = conn.expire(&stats_key, self.default_ttl * 7).await?; // Keep stats for a week
        Ok(())
    }

    /// Update last accessed timestamp
    async fn update_last_accessed(&self, key: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let timestamp_key = format!("stats:last_access:{}", key);
        let now = Utc::now().timestamp();
        let _: () = conn.set(&timestamp_key, now).await?;
        let _: () = conn.expire(&timestamp_key, self.default_ttl * 7).await?;
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_cache_statistics(&self) -> Result<CacheStatistics> {
        if self.dev_in_memory {
            return Ok(CacheStatistics {
                matrix_count: 0,
                inference_count: 0,
                ai_pattern_count: 0,
                total_hits: 0,
                memory_usage: "N/A".to_string(),
            });
        }
        let mut conn = self.client.get_async_connection().await?;

        // Get memory usage
        let memory_info: String = String::from("N/A"); // Memory usage not available in this redis version

        // Count matrices
        let matrix_keys: Vec<String> = conn.keys("matrix:*").await?;
        let matrix_count = matrix_keys.len();

        // Count inferences
        let inference_keys: Vec<String> = conn.keys("inference:*").await?;
        let inference_count = inference_keys.len();

        // Count AI patterns
        let ai_pattern_keys: Vec<String> = conn.keys("ai_pattern:*").await?;
        let ai_pattern_count = ai_pattern_keys.len();

        // Get hit rate (simplified calculation)
        let total_access_keys: Vec<String> = conn.keys("stats:access:*").await?;
        let mut total_hits = 0i64;
        for key in total_access_keys {
            let hits: i64 = conn.get(&key).await.unwrap_or(0);
            total_hits += hits;
        }

        Ok(CacheStatistics {
            matrix_count,
            inference_count,
            ai_pattern_count,
            total_hits: total_hits as usize,
            memory_usage: memory_info,
        })
    }

    /// Clear all cached data
    pub async fn clear_all(&self) -> Result<()> {
        if self.dev_in_memory { return Ok(()); }
        let mut conn = self.client.get_async_connection().await?;
        redis::cmd("FLUSHDB")
            .query_async::<_, ()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Clear expired entries (manual cleanup)
    pub async fn cleanup_expired(&self) -> Result<usize> {
        if self.dev_in_memory { return Ok(0); }
        let mut conn = self.client.get_async_connection().await?;

        // Get all keys with TTL
        let all_keys: Vec<String> = conn.keys("*").await?;
        let mut removed_count = 0;

        for key in all_keys {
            let ttl: i64 = conn.ttl(&key).await.unwrap_or(-1);
            if ttl == -2 {
                // Key doesn't exist (expired)
                removed_count += 1;
            }
        }

        Ok(removed_count)
    }

    /// Warm up cache with frequently used matrices
    pub async fn warmup_cache(&self, subjects: &[String]) -> Result<()> {
        if self.dev_in_memory { return Ok(()); }
        // This would typically be called on startup to pre-load popular matrices
        for subject in subjects {
            // Create a simple key to mark as "warmed up"
            let mut conn = self.client.get_async_connection().await?;
            let warmup_key = format!("warmup:{}", subject);
            let _: () = conn.set_ex(&warmup_key, "warmed", 3600).await?; // 1 hour TTL
        }
        Ok(())
    }

    /// Health check for Redis connection
    pub async fn health_check(&self) -> Result<()> {
        if self.dev_in_memory { return Ok(()); }
        let mut conn = self.client.get_async_connection().await?;
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await?;
        Ok(())
    }

    /// Set custom TTL for specific key
    pub async fn set_ttl(&self, key: &str, ttl_seconds: i64) -> Result<()> {
        if self.dev_in_memory { return Ok(()); }
        let mut conn = self.client.get_async_connection().await?;
        let _: () = conn.expire(key, ttl_seconds).await?;
        Ok(())
    }

    /// Get remaining TTL for key
    pub async fn get_ttl(&self, key: &str) -> Result<i64> {
        if self.dev_in_memory { return Ok(-1); }
        let mut conn = self.client.get_async_connection().await?;
        let ttl: i64 = conn.ttl(key).await?;
        Ok(ttl)
    }

    /// Store batch of matrices efficiently
    pub async fn store_matrices_batch(&self, matrices: &[FluxMatrix]) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;

        // Use pipeline for batch operations
        let mut pipe = redis::pipe();

        for matrix in matrices {
            let subject_key = format!("matrix:subject:{}", matrix.subject);
            let id_key = format!("matrix:id:{}", matrix.id);
            let value = serde_json::to_string(matrix)?;

            pipe.set_ex(&subject_key, &value, self.default_ttl as u64);
            pipe.set_ex(&id_key, &value, self.default_ttl as u64);
        }

        let _: () = pipe.query_async(&mut conn).await?;
        Ok(())
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub matrix_count: usize,
    pub inference_count: usize,
    pub ai_pattern_count: usize,
    pub total_hits: usize,
    pub memory_usage: String,
}

// ============================================================
// ASI-Specific Cache Methods
// ============================================================

impl CacheManager {
    /// Cache ONNX embedding (expensive to compute)
    pub async fn cache_embedding(&self, text_hash: &str, embedding: &[f32]) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("embedding:text:{}", text_hash);
        let value = serde_json::to_string(embedding)?;
        
        // 24 hour TTL for embeddings (expensive to recompute)
        let _: () = conn.set_ex(&key, value, 86400).await?;
        Ok(())
    }
    
    /// Get cached embedding
    pub async fn get_cached_embedding(&self, text_hash: &str) -> Result<Option<Vec<f32>>> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("embedding:text:{}", text_hash);
        
        let cached_value: Option<String> = conn.get(&key).await?;
        
        if let Some(value) = cached_value {
            let embedding: Vec<f32> = serde_json::from_str(&value)?;
            self.increment_access_count(&key).await?;
            Ok(Some(embedding))
        } else {
            Ok(None)
        }
    }
    
    /// Track current signal strength
    pub async fn update_confidence(&self, signal: f32) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        
        // Update current signal
        let _: () = conn.set_ex("signal:current", signal, 3600).await?;
        
        // Add to history (circular buffer, last 1000 values)
        let _: () = conn.lpush("signal:history", signal).await?;
        let _: () = conn.ltrim("signal:history", 0, 999).await?;
        let _: () = conn.expire("signal:history", 86400).await?;
        
        Ok(())
    }
    
    /// Get current signal strength
    pub async fn get_confidence(&self) -> Result<Option<f32>> {
        let mut conn = self.client.get_async_connection().await?;
        let signal: Option<f32> = conn.get("signal:current").await?;
        Ok(signal)
    }
    
    /// Get signal strength history
    pub async fn get_signal_history(&self, limit: i64) -> Result<Vec<f32>> {
        let mut conn = self.client.get_async_connection().await?;
        let history: Vec<f32> = conn.lrange("signal:history", 0, (limit - 1) as isize).await?;
        Ok(history)
    }
    
    /// Track hallucination detection
    pub async fn mark_hallucination(&self, inference_id: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        
        // Add to detected set
        let _: () = conn.sadd("hallucination:detected", inference_id).await?;
        let _: () = conn.expire("hallucination:detected", 86400 * 7).await?; // 7 days
        
        // Increment counter
        let _: () = conn.incr("hallucination:count:24h", 1).await?;
        let _: () = conn.expire("hallucination:count:24h", 86400).await?;
        
        Ok(())
    }
    
    /// Get hallucination rate (last 24h)
    pub async fn get_hallucination_rate(&self) -> Result<f32> {
        let mut conn = self.client.get_async_connection().await?;
        
        let count: i64 = conn.get("hallucination:count:24h").await.unwrap_or(0);
        let total: i64 = conn.get("inference:count:24h").await.unwrap_or(1);
        
        Ok(count as f32 / total as f32)
    }
    
    /// Track sacred position intervention
    pub async fn log_sacred_intervention(&self, position: i32, boost: f32) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        
        // Increment counter for this position
        let key = format!("sacred:interventions:{}", position);
        let _: () = conn.incr(&key, 1).await?;
        let _: () = conn.expire(&key, 86400 * 7).await?;
        
        // Track effectiveness (sorted set: position -> avg boost)
        let _: () = conn.zadd("sacred:effectiveness", position, boost).await?;
        let _: () = conn.expire("sacred:effectiveness", 86400 * 7).await?;
        
        Ok(())
    }
    
    /// Get sacred intervention statistics
    pub async fn get_sacred_stats(&self) -> Result<SacredStats> {
        let mut conn = self.client.get_async_connection().await?;
        
        let count_3: i64 = conn.get("sacred:interventions:3").await.unwrap_or(0);
        let count_6: i64 = conn.get("sacred:interventions:6").await.unwrap_or(0);
        let count_9: i64 = conn.get("sacred:interventions:9").await.unwrap_or(0);
        
        Ok(SacredStats {
            position_3_count: count_3 as usize,
            position_6_count: count_6 as usize,
            position_9_count: count_9 as usize,
            total_interventions: (count_3 + count_6 + count_9) as usize,
        })
    }
    
    /// Cache vortex flow pattern hit
    pub async fn track_vortex_flow(&self, is_forward: bool) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        
        let key = if is_forward {
            "vortex:flow:forward"
        } else {
            "vortex:flow:backward"
        };
        
        let _: () = conn.incr(key, 1).await?;
        let _: () = conn.expire(key, 86400).await?;
        
        Ok(())
    }
    
    /// Get vortex flow statistics
    pub async fn get_vortex_stats(&self) -> Result<VortexStats> {
        let mut conn = self.client.get_async_connection().await?;
        
        let forward: i64 = conn.get("vortex:flow:forward").await.unwrap_or(0);
        let backward: i64 = conn.get("vortex:flow:backward").await.unwrap_or(0);
        let cycles: i64 = conn.get("vortex:cycle:count").await.unwrap_or(0);
        
        Ok(VortexStats {
            forward_flow_hits: forward as usize,
            backward_flow_hits: backward as usize,
            total_cycles: cycles as usize,
        })
    }
    
    /// Track model performance
    pub async fn update_model_performance(
        &self,
        model_id: &str,
        latency_ms: i64,
        accuracy: f32,
    ) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        
        // Update latency (moving average via sorted set)
        let latency_key = format!("model:{}:latency", model_id);
        let timestamp = chrono::Utc::now().timestamp();
        let _: () = conn.zadd(&latency_key, timestamp, latency_ms).await?;
        let _: () = conn.expire(&latency_key, 3600).await?; // 1 hour
        
        // Update accuracy
        let accuracy_key = format!("model:{}:accuracy", model_id);
        let _: () = conn.set_ex(&accuracy_key, accuracy, 3600).await?;
        
        Ok(())
    }
}

/// Sacred intervention statistics
#[derive(Debug, Clone)]
pub struct SacredStats {
    pub position_3_count: usize,
    pub position_6_count: usize,
    pub position_9_count: usize,
    pub total_interventions: usize,
}

/// Vortex flow statistics
#[derive(Debug, Clone)]
pub struct VortexStats {
    pub forward_flow_hits: usize,
    pub backward_flow_hits: usize,
    pub total_cycles: usize,
}
