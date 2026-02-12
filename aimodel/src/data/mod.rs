//! Data Models Module
//!
//! Core data structures for sacred geometry:
//! - BeamTensor, FluxMatrix
//! - Attributes system (ELP compatibility)
//! - HuggingFace dataset loading
//! - Live benchmark evaluation
//! - Real benchmark loading from actual data files

pub mod attributes;
pub mod models;
pub mod hf_datasets;
pub mod benchmark_eval;
pub mod real_benchmarks;
pub mod inference_audit;

pub use attributes::{Attributes, AttributeValue, AttributeAccessor, Tags};
#[allow(deprecated)]
pub use models::{BeamTensor, ELPTensor, FluxMatrix, BeadTensor};
pub use hf_datasets::{
    HFDatasetLoader, DatasetLoaderConfig, DatasetInfo, DatasetCategory,
    TrainingExample, DatasetStats, get_priority_datasets,
    get_datasets_by_category, get_top_priority_datasets,
};
pub use benchmark_eval::{
    BenchmarkEvaluator, BenchmarkEvalResult, BenchmarkQuestion,
    get_mmlu_questions, get_gsm8k_questions, get_arc_questions,
};
pub use real_benchmarks::{
    RealBenchmarkEvaluator, RealBenchmarkResult, RealBenchmarkQuestion,
    load_commonsenseqa, load_squad, load_babi,
    load_mmlu, load_gsm8k, load_arc, load_hellaswag, load_truthfulqa, load_humaneval,
    load_swebench,
};
