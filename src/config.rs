//! Configuration Management
//!
//! Load configuration from TOML file or environment variables

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Complete application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    #[serde(default)]
    pub onnx: OnnxConfig,
    #[serde(default)]
    pub confidence_lake: ConfidenceLakeConfig,
    #[serde(default)]
    pub voice: VoiceConfig,
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_workers")]
    pub workers: usize,
    #[serde(default = "default_true")]
    pub enable_cors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    #[serde(default = "default_timeout")]
    pub connection_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub url: String,
    #[serde(default = "default_ttl")]
    pub ttl_hours: i64,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OnnxConfig {
    pub model_path: String,
    pub tokenizer_path: String,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceLakeConfig {
    pub path: String,
    #[serde(default = "default_lake_size")]
    pub size_mb: usize,
    #[serde(default = "default_true")]
    pub encryption_enabled: bool,
    pub encryption_key: Option<String>,
    #[serde(default = "default_signal_threshold")]
    pub signal_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    #[serde(default = "default_channels")]
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiConfig {
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    #[serde(default = "default_true")]
    pub onnx_enabled: bool,
    #[serde(default = "default_true")]
    pub lake_enabled: bool,
    #[serde(default = "default_true")]
    pub voice_enabled: bool,
}

// Default value functions
fn default_host() -> String { "127.0.0.1".to_string() }
fn default_port() -> u16 { 8080 }
fn default_workers() -> usize { 4 }
fn default_pool_size() -> u32 { 10 }
fn default_timeout() -> u64 { 30 }
fn default_ttl() -> i64 { 24 }
fn default_batch_size() -> usize { 32 }
fn default_lake_size() -> usize { 1024 }
fn default_signal_threshold() -> f64 { 0.6 }
fn default_sample_rate() -> u32 { 44100 }
fn default_buffer_size() -> usize { 1024 }
fn default_channels() -> u16 { 1 }
fn default_log_level() -> String { "info".to_string() }
fn default_log_format() -> String { "json".to_string() }
fn default_true() -> bool { true }

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            workers: default_workers(),
            enable_cors: default_true(),
        }
    }
}

impl Default for ConfidenceLakeConfig {
    fn default() -> Self {
        Self {
            path: "./data/confidence_lake.mmap".to_string(),
            size_mb: default_lake_size(),
            encryption_enabled: default_true(),
            encryption_key: None,
            signal_threshold: default_signal_threshold(),
        }
    }
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            sample_rate: default_sample_rate(),
            buffer_size: default_buffer_size(),
            channels: default_channels(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            onnx_enabled: default_true(),
            lake_enabled: default_true(),
            voice_enabled: default_true(),
        }
    }
}

impl Config {
    /// Load configuration from TOML file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spatial_vortex::config::Config;
    ///
    /// let config = Config::from_file("config.toml")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path.as_ref())
            .context("Failed to read config file")?;
        
        let config: Config = toml::from_str(&contents)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }
    
    /// Load configuration from file with environment variable overrides
    ///
    /// Environment variables take precedence over config file values.
    /// Format: SPATIALVORTEX_<SECTION>_<KEY> (e.g., SPATIALVORTEX_SERVER_PORT=8080)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use spatial_vortex::config::Config;
    ///
    /// let config = Config::load("config.toml")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut config = Self::from_file(path)?;
        
        // Override with environment variables
        config.apply_env_overrides();
        
        Ok(config)
    }
    
    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        // Server overrides
        if let Ok(host) = std::env::var("SPATIALVORTEX_SERVER_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = std::env::var("SPATIALVORTEX_SERVER_PORT") {
            if let Ok(p) = port.parse() {
                self.server.port = p;
            }
        }
        if let Ok(workers) = std::env::var("SPATIALVORTEX_SERVER_WORKERS") {
            if let Ok(w) = workers.parse() {
                self.server.workers = w;
            }
        }
        
        // Database overrides
        if let Ok(url) = std::env::var("SPATIALVORTEX_DATABASE_URL") {
            self.database.url = url;
        }
        if let Ok(url) = std::env::var("DATABASE_URL") {
            self.database.url = url;
        }
        
        // Cache overrides
        if let Ok(url) = std::env::var("SPATIALVORTEX_CACHE_URL") {
            self.cache.url = url;
        }
        if let Ok(url) = std::env::var("REDIS_URL") {
            self.cache.url = url;
        }
        
        // ONNX overrides
        if let Ok(path) = std::env::var("SPATIALVORTEX_ONNX_MODEL_PATH") {
            self.onnx.model_path = path;
        }
        if let Ok(path) = std::env::var("SPATIALVORTEX_ONNX_TOKENIZER_PATH") {
            self.onnx.tokenizer_path = path;
        }
        
        // Confidence Lake overrides
        if let Ok(key) = std::env::var("SPATIALVORTEX_LAKE_ENCRYPTION_KEY") {
            self.confidence_lake.encryption_key = Some(key);
        }
        
        // AI overrides
        if let Ok(key) = std::env::var("SPATIALVORTEX_AI_API_KEY") {
            self.ai.api_key = Some(key);
        }
        
        // Logging overrides
        if let Ok(level) = std::env::var("SPATIALVORTEX_LOG_LEVEL") {
            self.logging.level = level;
        }
        if let Ok(level) = std::env::var("RUST_LOG") {
            self.logging.level = level;
        }
    }
    
    /// Create a default configuration
    pub fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/spatialvortex".to_string(),
                pool_size: default_pool_size(),
                connection_timeout_seconds: default_timeout(),
            },
            cache: CacheConfig {
                url: "redis://127.0.0.1:6379/".to_string(),
                ttl_hours: default_ttl(),
                pool_size: default_pool_size(),
            },
            onnx: OnnxConfig::default(),
            confidence_lake: ConfidenceLakeConfig::default(),
            voice: VoiceConfig::default(),
            ai: AiConfig::default(),
            logging: LoggingConfig::default(),
            features: FeatureFlags::default(),
        }
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Check required fields
        if self.database.url.is_empty() {
            anyhow::bail!("Database URL is required");
        }
        
        if self.cache.url.is_empty() {
            anyhow::bail!("Cache URL is required");
        }
        
        if self.features.lake_enabled && self.confidence_lake.encryption_enabled {
            if self.confidence_lake.encryption_key.is_none() {
                anyhow::bail!("Encryption key required when lake encryption is enabled");
            }
        }
        
        if self.features.onnx_enabled {
            if self.onnx.model_path.is_empty() {
                anyhow::bail!("ONNX model path required when ONNX is enabled");
            }
            if self.onnx.tokenizer_path.is_empty() {
                anyhow::bail!("ONNX tokenizer path required when ONNX is enabled");
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.workers, 4);
    }
    
    #[test]
    fn test_env_overrides() {
        std::env::set_var("SPATIALVORTEX_SERVER_PORT", "9000");
        
        let mut config = Config::default();
        config.apply_env_overrides();
        
        assert_eq!(config.server.port, 9000);
        
        std::env::remove_var("SPATIALVORTEX_SERVER_PORT");
    }
}
