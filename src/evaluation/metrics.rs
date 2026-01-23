use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Core evaluation metrics for the ASI Orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetrics {
    /// Context integrity score (0.0-1.0)
    pub context_integrity: f32,
    /// Grounding score based on RAG retrieval (0.0-1.0)
    pub grounding_score: f32,
    /// Hallucination risk trend over session
    pub hallucination_risk_trend: Vec<f32>,
    /// Controller compliance rate (0.0-1.0)
    pub controller_compliance: f32,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Cost per request (tokens/credits)
    pub cost_per_request: f32,
    /// Session-level metrics
    pub session_metrics: SessionMetrics,
}

/// Session-scoped metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Number of turns in session
    pub turn_count: u64,
    /// Compression efficiency ratio
    pub compression_efficiency: f32,
    /// Memory usage ratio
    pub memory_usage_ratio: f32,
    /// Checkpoint hit rate
    pub checkpoint_hit_rate: f32,
}

/// Scorecard for evaluation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationScorecard {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub metrics: EvaluationMetrics,
    pub kpi_summary: KPISummary,
    pub recommendations: Vec<String>,
}

/// KPI summary for quick assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KPISummary {
    /// Overall health score (0.0-1.0)
    pub overall_health: f32,
    /// Risk level (Low, Medium, High, Critical)
    pub risk_level: RiskLevel,
    /// Performance grade (A-F)
    pub performance_grade: Grade,
    /// Key strengths
    pub strengths: Vec<String>,
    /// Areas for improvement
    pub improvements: Vec<String>,
}

/// Risk level classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Performance grade
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Grade {
    A,
    B,
    C,
    D,
    F,
}

impl EvaluationMetrics {
    pub fn new() -> Self {
        Self {
            context_integrity: 0.0,
            grounding_score: 0.0,
            hallucination_risk_trend: Vec::new(),
            controller_compliance: 0.0,
            avg_latency_ms: 0.0,
            cost_per_request: 0.0,
            session_metrics: SessionMetrics::new(),
        }
    }

    /// Calculate overall health score
    pub fn health_score(&self) -> f32 {
        let weights = [
            (self.context_integrity, 0.25),
            (self.grounding_score, 0.20),
            (1.0 - self.avg_hallucination_risk(), 0.25),
            (self.controller_compliance, 0.15),
            (self.latency_score(), 0.10),
            (self.session_metrics.compression_efficiency, 0.05),
        ];

        weights.iter().map(|(score, weight)| score * weight).sum()
    }

    /// Average hallucination risk from trend
    pub fn avg_hallucination_risk(&self) -> f32 {
        if self.hallucination_risk_trend.is_empty() {
            return 0.0;
        }
        self.hallucination_risk_trend.iter().sum::<f32>() / self.hallucination_risk_trend.len() as f32
    }

    /// Latency score (lower is better, normalized to 0-1)
    pub fn latency_score(&self) -> f32 {
        // Target: <500ms for good score
        (1.0 - (self.avg_latency_ms / 1000.0).min(1.0)) as f32
    }

    /// Determine risk level
    pub fn risk_level(&self) -> RiskLevel {
        let risk = self.avg_hallucination_risk();
        match risk {
            r if r < 0.1 => RiskLevel::Low,
            r if r < 0.3 => RiskLevel::Medium,
            r if r < 0.5 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }

    /// Determine performance grade
    pub fn performance_grade(&self) -> Grade {
        let score = self.health_score();
        match score {
            s if s >= 0.9 => Grade::A,
            s if s >= 0.8 => Grade::B,
            s if s >= 0.7 => Grade::C,
            s if s >= 0.6 => Grade::D,
            _ => Grade::F,
        }
    }
}

impl SessionMetrics {
    pub fn new() -> Self {
        Self {
            turn_count: 0,
            compression_efficiency: 1.0,
            memory_usage_ratio: 0.0,
            checkpoint_hit_rate: 0.0,
        }
    }

    /// Update checkpoint hit rate
    pub fn update_checkpoint_rate(&mut self, total_turns: u64, checkpoint_turns: u64) {
        self.checkpoint_hit_rate = if total_turns > 0 {
            checkpoint_turns as f32 / total_turns as f32
        } else {
            0.0
        };
    }
}

impl EvaluationScorecard {
    pub fn new(session_id: String) -> Self {
        let metrics = EvaluationMetrics::new();
        let kpi_summary = KPISummary::from_metrics(&metrics);

        Self {
            session_id,
            timestamp: Utc::now(),
            metrics,
            kpi_summary,
            recommendations: Vec::new(),
        }
    }

    /// Generate recommendations based on metrics
    pub fn generate_recommendations(&mut self) {
        self.recommendations.clear();

        if self.metrics.context_integrity < 0.7 {
            self.recommendations.push("Improve context compression and preservation".to_string());
        }

        if self.metrics.grounding_score < 0.6 {
            self.recommendations.push("Enhance RAG retrieval quality and relevance".to_string());
        }

        if self.metrics.avg_hallucination_risk() > 0.3 {
            self.recommendations.push("Strengthen VCP hallucination detection".to_string());
        }

        if self.metrics.controller_compliance < 0.8 {
            self.recommendations.push("Review controller intervention logic".to_string());
        }

        if self.metrics.avg_latency_ms > 1000.0 {
            self.recommendations.push("Optimize generation pipeline performance".to_string());
        }
    }
}

impl KPISummary {
    pub fn from_metrics(metrics: &EvaluationMetrics) -> Self {
        let overall_health = metrics.health_score();
        let risk_level = metrics.risk_level();
        let performance_grade = metrics.performance_grade();

        let mut strengths = Vec::new();
        let mut improvements = Vec::new();

        // Analyze strengths
        if metrics.context_integrity > 0.8 {
            strengths.push("Strong context integrity".to_string());
        }
        if metrics.grounding_score > 0.8 {
            strengths.push("Excellent grounding quality".to_string());
        }
        if metrics.controller_compliance > 0.9 {
            strengths.push("High controller compliance".to_string());
        }
        if metrics.avg_latency_ms < 500.0 {
            strengths.push("Low latency performance".to_string());
        }

        // Analyze improvements
        if metrics.context_integrity < 0.6 {
            improvements.push("Context integrity needs improvement".to_string());
        }
        if metrics.grounding_score < 0.6 {
            improvements.push("Grounding quality below threshold".to_string());
        }
        if metrics.avg_hallucination_risk() > 0.3 {
            improvements.push("Hallucination risk too high".to_string());
        }
        if metrics.controller_compliance < 0.7 {
            improvements.push("Controller compliance insufficient".to_string());
        }

        Self {
            overall_health,
            risk_level,
            performance_grade,
            strengths,
            improvements,
        }
    }
}

/// Aggregated metrics across multiple sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub total_sessions: u64,
    pub avg_context_integrity: f32,
    pub avg_grounding_score: f32,
    pub avg_hallucination_risk: f32,
    pub avg_controller_compliance: f32,
    pub avg_latency_ms: f64,
    pub risk_distribution: HashMap<RiskLevel, u64>,
    pub grade_distribution: HashMap<Grade, u64>,
}

impl AggregatedMetrics {
    pub fn from_scorecards(scorecards: &[EvaluationScorecard]) -> Self {
        let total_sessions = scorecards.len() as u64;
        if total_sessions == 0 {
            return Self::empty();
        }

        let context_integrity_sum: f32 = scorecards.iter().map(|s| s.metrics.context_integrity).sum();
        let grounding_score_sum: f32 = scorecards.iter().map(|s| s.metrics.grounding_score).sum();
        let hallucination_risk_sum: f32 = scorecards.iter().map(|s| s.metrics.avg_hallucination_risk()).sum();
        let controller_compliance_sum: f32 = scorecards.iter().map(|s| s.metrics.controller_compliance).sum();
        let latency_sum: f64 = scorecards.iter().map(|s| s.metrics.avg_latency_ms).sum();

        let mut risk_distribution = HashMap::new();
        let mut grade_distribution = HashMap::new();

        for scorecard in scorecards {
            *risk_distribution.entry(scorecard.kpi_summary.risk_level.clone()).or_insert(0) += 1;
            *grade_distribution.entry(scorecard.kpi_summary.performance_grade.clone()).or_insert(0) += 1;
        }

        Self {
            total_sessions,
            avg_context_integrity: context_integrity_sum / total_sessions as f32,
            avg_grounding_score: grounding_score_sum / total_sessions as f32,
            avg_hallucination_risk: hallucination_risk_sum / total_sessions as f32,
            avg_controller_compliance: controller_compliance_sum / total_sessions as f32,
            avg_latency_ms: latency_sum / total_sessions as f64,
            risk_distribution,
            grade_distribution,
        }
    }

    pub fn empty() -> Self {
        Self {
            total_sessions: 0,
            avg_context_integrity: 0.0,
            avg_grounding_score: 0.0,
            avg_hallucination_risk: 0.0,
            avg_controller_compliance: 0.0,
            avg_latency_ms: 0.0,
            risk_distribution: HashMap::new(),
            grade_distribution: HashMap::new(),
        }
    }
}
