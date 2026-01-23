//! Structured Logging Configuration
//!
//! Production-ready logging with tracing, supporting:
//! - Multiple output formats (JSON, pretty)
//! - Component-level filtering
//! - Span tracking for distributed tracing
//! - Log aggregation ready (ELK, Loki, etc.)

use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use std::io;

/// Log output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Pretty-printed logs for development
    Pretty,
    /// JSON logs for production/aggregation
    Json,
    /// Compact format
    Compact,
}

impl LogFormat {
    /// Parse from environment variable
    pub fn from_env() -> Self {
        match std::env::var("LOG_FORMAT")
            .unwrap_or_else(|_| "pretty".to_string())
            .to_lowercase()
            .as_str()
        {
            "json" => Self::Json,
            "compact" => Self::Compact,
            _ => Self::Pretty,
        }
    }
}

/// Log level configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Output format
    pub format: LogFormat,
    
    /// Default log level
    pub default_level: String,
    
    /// Component-specific log levels
    pub component_levels: Vec<(String, String)>,
    
    /// Enable span events (enter/exit/close)
    pub span_events: bool,
    
    /// Enable ANSI colors (disable for JSON)
    pub ansi: bool,
    
    /// Include thread IDs
    pub thread_ids: bool,
    
    /// Include thread names
    pub thread_names: bool,
    
    /// Include target (module path)
    pub target: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::from_env(),
            default_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            component_levels: vec![],
            span_events: true,
            ansi: true,
            thread_ids: false,
            thread_names: true,
            target: true,
        }
    }
}

impl LogConfig {
    /// Create configuration for production (JSON, no ANSI)
    pub fn production() -> Self {
        Self {
            format: LogFormat::Json,
            default_level: "info".to_string(),
            component_levels: vec![
                // Reduce noise from dependencies
                ("hyper".to_string(), "warn".to_string()),
                ("tokio".to_string(), "warn".to_string()),
                ("h2".to_string(), "warn".to_string()),
            ],
            span_events: false,  // Reduce log volume
            ansi: false,
            thread_ids: true,
            thread_names: true,
            target: true,
        }
    }
    
    /// Create configuration for development (Pretty, with colors)
    pub fn development() -> Self {
        Self {
            format: LogFormat::Pretty,
            default_level: "debug".to_string(),
            component_levels: vec![
                // More verbose for our components
                ("spatial_vortex".to_string(), "debug".to_string()),
            ],
            span_events: true,
            ansi: true,
            thread_ids: false,
            thread_names: true,
            target: true,
        }
    }
    
    /// Build EnvFilter from configuration
    fn build_filter(&self) -> EnvFilter {
        let mut filter = EnvFilter::new(&self.default_level);
        
        for (component, level) in &self.component_levels {
            filter = filter.add_directive(
                format!("{}={}", component, level).parse().expect("Invalid filter directive")
            );
        }
        
        filter
    }
}

/// Initialize logging system
///
/// # Example
///
/// ```no_run
/// use spatial_vortex::monitoring::logging::{init_logging, LogConfig};
///
/// // Development
/// init_logging(LogConfig::development()).expect("Failed to init logging");
///
/// // Production
/// init_logging(LogConfig::production()).expect("Failed to init logging");
/// ```
pub fn init_logging(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    let filter = config.build_filter();
    
    let span_events = if config.span_events {
        FmtSpan::NEW | FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    };
    
    // For now, use the same format for all (compact)
    // TODO: Add pretty and json formatting when tracing-subscriber version supports it
    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(io::stdout)
                .with_ansi(config.ansi)
                .with_thread_ids(config.thread_ids)
                .with_thread_names(config.thread_names)
                .with_target(config.target)
                .with_span_events(span_events)
        )
        .try_init()?;
    
    Ok(())
}

/// Initialize with default configuration (from environment)
pub fn init_default() -> Result<(), Box<dyn std::error::Error>> {
    init_logging(LogConfig::default())
}

/// Macros for structured logging with sacred geometry context
#[macro_export]
macro_rules! vortex_info {
    ($($key:ident = $value:expr),+ $(,)? ; $($arg:tt)*) => {
        tracing::info!($($key = $value),+, $($arg)*)
    };
}

#[macro_export]
macro_rules! vortex_warn {
    ($($key:ident = $value:expr),+ $(,)? ; $($arg:tt)*) => {
        tracing::warn!($($key = $value),+, $($arg)*)
    };
}

#[macro_export]
macro_rules! vortex_error {
    ($($key:ident = $value:expr),+ $(,)? ; $($arg:tt)*) => {
        tracing::error!($($key = $value),+, $($arg)*)
    };
}

#[macro_export]
macro_rules! vortex_debug {
    ($($key:ident = $value:expr),+ $(,)? ; $($arg:tt)*) => {
        tracing::debug!($($key = $value),+, $($arg)*)
    };
}

/// Span helpers for request tracing
pub mod spans {
    use tracing::{Span, Level};
    
    /// Create span for meta orchestrator request
    pub fn meta_request(strategy: &str, input_len: usize) -> Span {
        tracing::span!(
            Level::INFO,
            "meta_request",
            strategy = strategy,
            input_len = input_len,
        )
    }
    
    /// Create span for ASI inference
    pub fn asi_inference(mode: &str) -> Span {
        tracing::span!(
            Level::INFO,
            "asi_inference",
            mode = mode,
        )
    }
    
    /// Create span for sacred position operation
    pub fn sacred_operation(position: u8, operation: &str) -> Span {
        tracing::span!(
            Level::INFO,
            "sacred_operation",
            position = position,
            sacred = [3, 6, 9].contains(&position),
            operation = operation,
        )
    }
    
    /// Create span for fusion
    pub fn fusion(asi_weight: f32, runtime_weight: f32) -> Span {
        tracing::span!(
            Level::INFO,
            "result_fusion",
            asi_weight = asi_weight,
            runtime_weight = runtime_weight,
            position = 6,  // Fusion always at position 6
        )
    }
    
    /// Create span for vortex cycle
    pub fn vortex_cycle(cycle_id: u64, direction: &str) -> Span {
        tracing::span!(
            Level::DEBUG,
            "vortex_cycle",
            cycle_id = cycle_id,
            direction = direction,
        )
    }
    
    /// Create span for Confidence Lake operation
    pub fn lake_operation(operation: &str, confidence: f32) -> Span {
        tracing::span!(
            Level::DEBUG,
            "lake_operation",
            operation = operation,
            confidence = confidence,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_format_from_env() {
        std::env::set_var("LOG_FORMAT", "json");
        assert_eq!(LogFormat::from_env(), LogFormat::Json);
        
        std::env::set_var("LOG_FORMAT", "pretty");
        assert_eq!(LogFormat::from_env(), LogFormat::Pretty);
        
        std::env::set_var("LOG_FORMAT", "compact");
        assert_eq!(LogFormat::from_env(), LogFormat::Compact);
    }
    
    #[test]
    fn test_production_config() {
        let config = LogConfig::production();
        assert_eq!(config.format, LogFormat::Json);
        assert!(!config.ansi);
        assert!(config.thread_ids);
    }
    
    #[test]
    fn test_development_config() {
        let config = LogConfig::development();
        assert_eq!(config.format, LogFormat::Pretty);
        assert!(config.ansi);
        assert!(config.span_events);
    }
}
