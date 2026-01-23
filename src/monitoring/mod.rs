//! Production Monitoring Module
//!
//! Comprehensive monitoring, metrics, logging, and observability for SpatialVortex ASI.
//!
//! ## Components
//!
//! - **metrics**: Prometheus metrics for all system components
//! - **logging**: Structured logging with tracing
//! - **tracing**: Distributed tracing spans
//! - **alerts**: Alert rules and thresholds (future)

pub mod metrics;
pub mod logging;

pub use metrics::{VortexMetrics, VORTEX_METRICS};
pub use logging::{
    init_logging, init_default, LogConfig, LogFormat,
    spans,
};

// Re-export commonly used metrics
pub use metrics::{
    META_REQUESTS_TOTAL,
    META_DURATION,
    ASI_INFER_TOTAL,
    ASI_SACRED_HITS,
    LAKE_STORES_TOTAL,
    SIGNAL_STRENGTH,
    VCP_INTERVENTIONS,
    ERRORS_TOTAL,
};
