use spatial_vortex::ai::orchestrator::{ASIOrchestrator, ExecutionMode};
use spatial_vortex::metrics::ASI_EXPERT_SELECTED;

fn get_counter(expert: &str, reason: &str) -> u64 {
    ASI_EXPERT_SELECTED
        .get_metric_with_label_values(&[expert, reason])
        .map(|c| c.get())
        .unwrap_or(0)
}

#[tokio::test]
async fn moe_selects_rag_when_confidence_higher() {
    // Ensure MoE is enabled and permissive thresholds
    std::env::set_var("MOE_ENABLED", "true");
    std::env::set_var("MOE_MIN_CONFIDENCE", "0.4");
    std::env::set_var("MOE_MARGIN", "0.0");

    let mut asi = ASIOrchestrator::new().expect("orchestrator should initialize");

    // Craft input to favor RAGExpert (quotes + cite + enough words)
    let input = "\
        \"Truth\" in humanities often requires sources that we can cite and reference
        across disciplines to establish a coherent narrative of ethics, logic, and emotion
        that persists throughout history with reliable documentation and interpretation
    ";

    let before = get_counter("rag", "selected_by_moe");

    // Balanced mode: geometric + ml + experts
    let _result = asi.process(input, ExecutionMode::Balanced)
        .await
        .expect("process should succeed");

    let after = get_counter("rag", "selected_by_moe");
    assert!(after > before, "RAG should be selected by MoE when its confidence is higher (before={}, after={})", before, after);
}

#[tokio::test]
async fn moe_margin_keeps_baseline_when_gap_small() {
    // Enable MoE but require a significant margin to replace baseline
    std::env::set_var("MOE_ENABLED", "true");
    std::env::set_var("MOE_MIN_CONFIDENCE", "0.4");
    std::env::set_var("MOE_MARGIN", "0.5");

    let mut asi = ASIOrchestrator::new().expect("orchestrator should initialize");

    // Input that gives RAG a confidence boost but not enough to beat baseline + margin
    let input = "A modest scholarly paragraph that mentions a source but not strongly enough to vastly exceed the baseline confidence cite";

    let before_rag = get_counter("rag", "selected_by_moe");

    let _result = asi.process(input, ExecutionMode::Balanced)
        .await
        .expect("process should succeed");

    let after_rag = get_counter("rag", "selected_by_moe");
    assert_eq!(after_rag, before_rag, "RAG should not be selected when margin requirement is not met");
}
