//! HuggingFace Dataset Loader
//!
//! Automatic downloading and streaming of datasets from HuggingFace Hub.
//! Supports priority datasets: FineWeb-Edu, GSM8K, MMLU, ProofPile-2, etc.

use crate::data::models::BeamTensor;
use hf_hub::api::sync::Api;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::{BufRead, BufReader};

/// Response from HuggingFace Datasets Server API
#[derive(Debug, Deserialize)]
struct HFDatasetResponse {
    rows: Vec<HFRow>,
}

#[derive(Debug, Deserialize)]
struct HFRow {
    row: serde_json::Value,
}

// =============================================================================
// Dataset Registry
// =============================================================================

/// Known HuggingFace datasets with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    pub hf_path: String,
    pub name: String,
    pub category: DatasetCategory,
    pub split: String,
    pub estimated_tokens: u64,
    pub license: String,
    pub priority: u8, // 1 = highest
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DatasetCategory {
    PreTraining,
    Reasoning,
    Math,
    Code,
    Benchmark,
    // Vision,      // Disabled - requires image processing
    // Multimodal,  // Disabled - requires multi-modal processing
    Science,
    QA,
    Entailment,   // NLI/RTE for deductive reasoning
    Commonsense,  // ConceptNet, ATOMIC for world knowledge
}

/// All 105 HuggingFace datasets organized by category
pub fn get_priority_datasets() -> Vec<DatasetInfo> {
    vec![
        // =============================================================================
        // 1. Large-Scale Text/Language Corpora (1-20)
        // =============================================================================
        DatasetInfo { hf_path: "HuggingFaceFW/fineweb".to_string(), name: "FineWeb".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 15_000_000_000, license: "ODC-BY".to_string(), priority: 1 },
        DatasetInfo { hf_path: "HuggingFaceFW/fineweb-edu".to_string(), name: "FineWeb-Edu".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 5_400_000_000, license: "ODC-BY".to_string(), priority: 2 },
        DatasetInfo { hf_path: "allenai/dolma".to_string(), name: "Dolma".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 3_000_000_000_000, license: "ODC-BY".to_string(), priority: 3 },
        DatasetInfo { hf_path: "togethercomputer/RedPajama-Data-1T".to_string(), name: "RedPajama".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 1_200_000_000_000, license: "Apache-2.0".to_string(), priority: 4 },
        DatasetInfo { hf_path: "EleutherAI/the_pile".to_string(), name: "The Pile".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 825_000_000_000, license: "MIT".to_string(), priority: 5 },
        DatasetInfo { hf_path: "allenai/c4".to_string(), name: "C4".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 750_000_000_000, license: "ODC-BY".to_string(), priority: 6 },
        DatasetInfo { hf_path: "oscar-corpus/OSCAR-2109".to_string(), name: "OSCAR".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 400_000_000_000, license: "CC-BY".to_string(), priority: 7 },
        DatasetInfo { hf_path: "EleutherAI/bookcorpus".to_string(), name: "BookCorpus".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 985_000_000, license: "MIT".to_string(), priority: 8 },
        DatasetInfo { hf_path: "wikipedia".to_string(), name: "Wikipedia".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 6_000_000_000, license: "CC-BY-SA".to_string(), priority: 9 },
        DatasetInfo { hf_path: "skylion007/openwebtext".to_string(), name: "OpenWebText".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 8_000_000_000, license: "MIT".to_string(), priority: 10 },
        DatasetInfo { hf_path: "allenai/cc-news".to_string(), name: "CC-News".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 76_000_000_000, license: "CC-BY".to_string(), priority: 11 },
        DatasetInfo { hf_path: "allenai/mc4".to_string(), name: "MC4".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 100_000_000_000, license: "ODC-BY".to_string(), priority: 12 },
        DatasetInfo { hf_path: "bigscience-data/roots".to_string(), name: "ROOTS".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 1_600_000_000_000, license: "Apache-2.0".to_string(), priority: 13 },
        DatasetInfo { hf_path: "tiiuae/falcon-refinedweb".to_string(), name: "RefinedWeb".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 600_000_000_000, license: "ODC-BY".to_string(), priority: 14 },
        DatasetInfo { hf_path: "nguyentito/CultureX".to_string(), name: "CultureX".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 50_000_000_000, license: "CC-BY".to_string(), priority: 15 },
        DatasetInfo { hf_path: "open-web-math/open-web-math".to_string(), name: "OpenWebMath".to_string(), category: DatasetCategory::Math, split: "train".to_string(), estimated_tokens: 14_700_000_000, license: "ODC-BY".to_string(), priority: 16 },
        DatasetInfo { hf_path: "cerebras/SlimPajama-627B".to_string(), name: "SlimPajama".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 627_000_000_000, license: "Apache-2.0".to_string(), priority: 17 },
        DatasetInfo { hf_path: "Skywork/SkyPile-150B".to_string(), name: "SkyPile".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 150_000_000_000, license: "Apache-2.0".to_string(), priority: 18 },
        DatasetInfo { hf_path: "EleutherAI/webtext2".to_string(), name: "WebText2".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 40_000_000_000, license: "MIT".to_string(), priority: 19 },
        DatasetInfo { hf_path: "allenai/cc-stories".to_string(), name: "CC-Stories".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 31_000_000_000, license: "CC-BY".to_string(), priority: 20 },

        // =============================================================================
        // 2. Code and Programming Datasets (21-30)
        // =============================================================================
        DatasetInfo { hf_path: "openai_humaneval".to_string(), name: "HumanEval".to_string(), category: DatasetCategory::Code, split: "test".to_string(), estimated_tokens: 164, license: "MIT".to_string(), priority: 21 },
        DatasetInfo { hf_path: "bigcode/the-stack".to_string(), name: "The Stack".to_string(), category: DatasetCategory::Code, split: "train".to_string(), estimated_tokens: 3_000_000_000_000, license: "Various".to_string(), priority: 22 },
        DatasetInfo { hf_path: "codeparrot/github-code".to_string(), name: "GitHub Code".to_string(), category: DatasetCategory::Code, split: "train".to_string(), estimated_tokens: 1_000_000_000_000, license: "Various".to_string(), priority: 23 },
        DatasetInfo { hf_path: "hendrycks/apps".to_string(), name: "APPS".to_string(), category: DatasetCategory::Code, split: "train".to_string(), estimated_tokens: 10_000, license: "MIT".to_string(), priority: 24 },
        DatasetInfo { hf_path: "microsoft/CodeXGLUE".to_string(), name: "CodeXGLUE".to_string(), category: DatasetCategory::Code, split: "train".to_string(), estimated_tokens: 50_000, license: "MIT".to_string(), priority: 25 },
        DatasetInfo { hf_path: "google-research/mbpp".to_string(), name: "MBPP".to_string(), category: DatasetCategory::Code, split: "train".to_string(), estimated_tokens: 974, license: "CC-BY".to_string(), priority: 26 },
        DatasetInfo { hf_path: "microsoft/ds1000".to_string(), name: "DS-1000".to_string(), category: DatasetCategory::Code, split: "test".to_string(), estimated_tokens: 1_000, license: "MIT".to_string(), priority: 27 },
        DatasetInfo { hf_path: "bigcode-project/bigcodebench".to_string(), name: "BigCodeBench".to_string(), category: DatasetCategory::Code, split: "test".to_string(), estimated_tokens: 5_000, license: "Apache-2.0".to_string(), priority: 28 },
        DatasetInfo { hf_path: "bigcode/starcoderdata".to_string(), name: "StarCoderData".to_string(), category: DatasetCategory::Code, split: "train".to_string(), estimated_tokens: 250_000_000_000, license: "Various".to_string(), priority: 29 },
        DatasetInfo { hf_path: "thebigpile/python-stackoverflow".to_string(), name: "Python StackOverflow".to_string(), category: DatasetCategory::Code, split: "train".to_string(), estimated_tokens: 10_000_000, license: "CC-BY-SA".to_string(), priority: 30 },

        // =============================================================================
        // 3. Vision and Image Datasets (31-40) - DISABLED
        // =============================================================================
        // Vision datasets disabled - requires image processing infrastructure
        // Re-enable when adding vision capabilities

        // =============================================================================
        // 4. Multimodal Datasets (41-50) - DISABLED  
        // =============================================================================
        // Multimodal datasets disabled - requires multi-modal processing
        // Re-enable when adding audio/video/image capabilities

        // =============================================================================
        // 5. Science, Math, and Reasoning Datasets (51-60)
        // =============================================================================
        DatasetInfo { hf_path: "openai/gsm8k".to_string(), name: "GSM8K".to_string(), category: DatasetCategory::Math, split: "train".to_string(), estimated_tokens: 8_500, license: "MIT".to_string(), priority: 51 },
        DatasetInfo { hf_path: "hendrycks/math".to_string(), name: "MATH".to_string(), category: DatasetCategory::Math, split: "train".to_string(), estimated_tokens: 12_500, license: "MIT".to_string(), priority: 52 },
        DatasetInfo { hf_path: "ccdv/arxiv-summarization".to_string(), name: "arXiv".to_string(), category: DatasetCategory::Science, split: "train".to_string(), estimated_tokens: 215_913, license: "CC-BY".to_string(), priority: 53 },
        DatasetInfo { hf_path: "qiaojin/PubMedQA".to_string(), name: "PubMedQA".to_string(), category: DatasetCategory::Science, split: "train".to_string(), estimated_tokens: 211_000, license: "MIT".to_string(), priority: 54 },
        DatasetInfo { hf_path: "GBaker/MedQA-USMLE-4-options".to_string(), name: "MedQA".to_string(), category: DatasetCategory::Science, split: "train".to_string(), estimated_tokens: 50_000, license: "Apache-2.0".to_string(), priority: 55 },
        DatasetInfo { hf_path: "allenai/sciq".to_string(), name: "SciQ".to_string(), category: DatasetCategory::Science, split: "train".to_string(), estimated_tokens: 13_679, license: "CC-BY".to_string(), priority: 56 },
        DatasetInfo { hf_path: "allenai/ai2_arc".to_string(), name: "ARC".to_string(), category: DatasetCategory::Benchmark, split: "train".to_string(), estimated_tokens: 7_800, license: "Apache-2.0".to_string(), priority: 57 },
        DatasetInfo { hf_path: "allenai/openbookqa".to_string(), name: "OpenBookQA".to_string(), category: DatasetCategory::Benchmark, split: "train".to_string(), estimated_tokens: 5_957, license: "Apache-2.0".to_string(), priority: 58 },
        DatasetInfo { hf_path: "EleutherAI/proof-pile-2".to_string(), name: "Proof-Pile-2".to_string(), category: DatasetCategory::Math, split: "default".to_string(), estimated_tokens: 55_000_000_000, license: "Apache-2.0".to_string(), priority: 59 },
        DatasetInfo { hf_path: "wenhu/theoremqa".to_string(), name: "TheoremQA".to_string(), category: DatasetCategory::Math, split: "test".to_string(), estimated_tokens: 800, license: "MIT".to_string(), priority: 60 },

        // =============================================================================
        // 6. Additional Diverse Datasets (61-105)
        // =============================================================================
        DatasetInfo { hf_path: "cais/mmlu".to_string(), name: "MMLU".to_string(), category: DatasetCategory::Benchmark, split: "test".to_string(), estimated_tokens: 14_042, license: "MIT".to_string(), priority: 61 },
        DatasetInfo { hf_path: "glue".to_string(), name: "GLUE".to_string(), category: DatasetCategory::Benchmark, split: "train".to_string(), estimated_tokens: 100_000, license: "Various".to_string(), priority: 62 },
        DatasetInfo { hf_path: "super_glue".to_string(), name: "SuperGLUE".to_string(), category: DatasetCategory::Benchmark, split: "train".to_string(), estimated_tokens: 50_000, license: "Various".to_string(), priority: 62 },
        DatasetInfo { hf_path: "rajpurkar/squad".to_string(), name: "SQuAD".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 87_599, license: "CC-BY-SA".to_string(), priority: 63 },
        DatasetInfo { hf_path: "cnn_dailymail".to_string(), name: "CNN/Daily Mail".to_string(), category: DatasetCategory::PreTraining, split: "train".to_string(), estimated_tokens: 300_000, license: "Apache-2.0".to_string(), priority: 64 },
        DatasetInfo { hf_path: "hendrycks/ethics".to_string(), name: "Ethics".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 130_000, license: "MIT".to_string(), priority: 65 },
        DatasetInfo { hf_path: "truthful_qa".to_string(), name: "TruthfulQA".to_string(), category: DatasetCategory::Benchmark, split: "validation".to_string(), estimated_tokens: 817, license: "Apache-2.0".to_string(), priority: 66 },
        DatasetInfo { hf_path: "Rowan/hellaswag".to_string(), name: "HellaSwag".to_string(), category: DatasetCategory::Benchmark, split: "train".to_string(), estimated_tokens: 70_000, license: "MIT".to_string(), priority: 67 },
        DatasetInfo { hf_path: "ybisk/piqa".to_string(), name: "PIQA".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 16_113, license: "AFL-3.0".to_string(), priority: 68 },
        DatasetInfo { hf_path: "piqa".to_string(), name: "PIQA-alt".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 16_113, license: "AFL-3.0".to_string(), priority: 168 },
        DatasetInfo { hf_path: "sap2019/socialiqa".to_string(), name: "SocialIQA".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 33_410, license: "CC-BY".to_string(), priority: 69 },
        DatasetInfo { hf_path: "allenai/cosmosqa".to_string(), name: "CosmosQA".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 25_262, license: "CC-BY".to_string(), priority: 70 },
        DatasetInfo { hf_path: "quac".to_string(), name: "QuAC".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 83_568, license: "CC-BY-SA".to_string(), priority: 71 },
        DatasetInfo { hf_path: "stanfordnlp/coqa".to_string(), name: "CoQA".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 108_647, license: "Various".to_string(), priority: 72 },
        DatasetInfo { hf_path: "allenai/drop".to_string(), name: "DROP".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 77_409, license: "Apache-2.0".to_string(), priority: 73 },
        DatasetInfo { hf_path: "google/boolq".to_string(), name: "BoolQ".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 9_427, license: "CC-BY-SA".to_string(), priority: 74 },
        DatasetInfo { hf_path: "multi_rc".to_string(), name: "MultiRC".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 27_243, license: "Various".to_string(), priority: 75 },
        DatasetInfo { hf_path: "record".to_string(), name: "ReCoRD".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 100_730, license: "Various".to_string(), priority: 76 },
        DatasetInfo { hf_path: "hotpot_qa".to_string(), name: "HotpotQA".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 90_447, license: "CC-BY-SA".to_string(), priority: 77 },
        DatasetInfo { hf_path: "trivia_qa".to_string(), name: "TriviaQA".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 138_384, license: "Apache-2.0".to_string(), priority: 78 },
        DatasetInfo { hf_path: "natural_questions".to_string(), name: "Natural Questions".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 307_373, license: "CC-BY-SA".to_string(), priority: 79 },
        DatasetInfo { hf_path: "allenai/scitail".to_string(), name: "SciTail".to_string(), category: DatasetCategory::Science, split: "train".to_string(), estimated_tokens: 23_596, license: "Apache-2.0".to_string(), priority: 80 },
        DatasetInfo { hf_path: "race".to_string(), name: "RACE".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 87_866, license: "Custom".to_string(), priority: 81 },
        DatasetInfo { hf_path: "dream".to_string(), name: "DREAM".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 6_116, license: "Custom".to_string(), priority: 82 },
        DatasetInfo { hf_path: "allenai/qasc".to_string(), name: "QASC".to_string(), category: DatasetCategory::Science, split: "train".to_string(), estimated_tokens: 8_134, license: "CC-BY".to_string(), priority: 83 },
        DatasetInfo { hf_path: "quoref".to_string(), name: "Quoref".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 19_399, license: "CC-BY-SA".to_string(), priority: 84 },
        DatasetInfo { hf_path: "ropes".to_string(), name: "ROPES".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 10_924, license: "CC-BY".to_string(), priority: 85 },
        DatasetInfo { hf_path: "wiki_qa".to_string(), name: "WikiQA".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 20_360, license: "Custom".to_string(), priority: 86 },
        DatasetInfo { hf_path: "duorc".to_string(), name: "DuoRC".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 186_089, license: "MIT".to_string(), priority: 87 },
        DatasetInfo { hf_path: "cb".to_string(), name: "CB".to_string(), category: DatasetCategory::Benchmark, split: "train".to_string(), estimated_tokens: 250, license: "CC-BY".to_string(), priority: 88 },
        DatasetInfo { hf_path: "copa".to_string(), name: "COPA".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 400, license: "BSD".to_string(), priority: 89 },
        DatasetInfo { hf_path: "rte".to_string(), name: "RTE".to_string(), category: DatasetCategory::Benchmark, split: "train".to_string(), estimated_tokens: 2_490, license: "Various".to_string(), priority: 90 },
        DatasetInfo { hf_path: "wic".to_string(), name: "WiC".to_string(), category: DatasetCategory::Benchmark, split: "train".to_string(), estimated_tokens: 5_428, license: "CC-BY-NC".to_string(), priority: 91 },
        DatasetInfo { hf_path: "wsc".to_string(), name: "WSC".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 554, license: "CC-BY".to_string(), priority: 92 },
        DatasetInfo { hf_path: "pdp".to_string(), name: "PDP".to_string(), category: DatasetCategory::Reasoning, split: "test".to_string(), estimated_tokens: 60, license: "Custom".to_string(), priority: 93 },
        DatasetInfo { hf_path: "dpr".to_string(), name: "DPR".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 58_880, license: "CC-BY-SA".to_string(), priority: 94 },
        DatasetInfo { hf_path: "fiqa".to_string(), name: "FiQA".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 5_500, license: "Custom".to_string(), priority: 95 },
        DatasetInfo { hf_path: "tweet_qa".to_string(), name: "TweetQA".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 10_692, license: "CC-BY".to_string(), priority: 96 },
        DatasetInfo { hf_path: "newsqa".to_string(), name: "NewsQA".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 92_549, license: "Custom".to_string(), priority: 97 },
        DatasetInfo { hf_path: "search_qa".to_string(), name: "SearchQA".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 140_461, license: "Apache-2.0".to_string(), priority: 98 },
        DatasetInfo { hf_path: "quail".to_string(), name: "QuAIL".to_string(), category: DatasetCategory::QA, split: "train".to_string(), estimated_tokens: 10_246, license: "CC-BY-SA".to_string(), priority: 99 },
        DatasetInfo { hf_path: "biomrc".to_string(), name: "BioMRC".to_string(), category: DatasetCategory::Science, split: "train".to_string(), estimated_tokens: 812_752, license: "Apache-2.0".to_string(), priority: 100 },
        DatasetInfo { hf_path: "clutrr".to_string(), name: "CLUTRR".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 10_000, license: "MIT".to_string(), priority: 101 },
        DatasetInfo { hf_path: "logiqa".to_string(), name: "LogiQA".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 7_376, license: "Custom".to_string(), priority: 102 },
        DatasetInfo { hf_path: "anli".to_string(), name: "Abductive NLI".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 169_654, license: "CC-BY-NC".to_string(), priority: 103 },
        DatasetInfo { hf_path: "art".to_string(), name: "ART".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 20_000, license: "CC-BY".to_string(), priority: 104 },
        DatasetInfo { hf_path: "causalnet".to_string(), name: "CausalNet".to_string(), category: DatasetCategory::Reasoning, split: "train".to_string(), estimated_tokens: 11_000_000, license: "MIT".to_string(), priority: 105 },

        // =============================================================================
        // 7. Entailment/NLI Datasets for Deductive Reasoning (106-115)
        // =============================================================================
        DatasetInfo { hf_path: "stanfordnlp/snli".to_string(), name: "SNLI".to_string(), category: DatasetCategory::Entailment, split: "train".to_string(), estimated_tokens: 550_152, license: "CC-BY-SA".to_string(), priority: 106 },
        DatasetInfo { hf_path: "nyu-mll/multi_nli".to_string(), name: "MultiNLI".to_string(), category: DatasetCategory::Entailment, split: "train".to_string(), estimated_tokens: 392_702, license: "Various".to_string(), priority: 107 },
        DatasetInfo { hf_path: "facebook/anli".to_string(), name: "Adversarial NLI".to_string(), category: DatasetCategory::Entailment, split: "train_r1".to_string(), estimated_tokens: 163_000, license: "CC-BY-NC".to_string(), priority: 108 },
        DatasetInfo { hf_path: "ynie/xnli".to_string(), name: "XNLI".to_string(), category: DatasetCategory::Entailment, split: "train".to_string(), estimated_tokens: 392_000, license: "CC-BY-NC".to_string(), priority: 109 },
        DatasetInfo { hf_path: "mteb/sickr-sts".to_string(), name: "SICK".to_string(), category: DatasetCategory::Entailment, split: "test".to_string(), estimated_tokens: 10_000, license: "CC-BY-NC-SA".to_string(), priority: 110 },
        DatasetInfo { hf_path: "scitail".to_string(), name: "SciTail-NLI".to_string(), category: DatasetCategory::Entailment, split: "train".to_string(), estimated_tokens: 27_000, license: "Apache-2.0".to_string(), priority: 111 },
        DatasetInfo { hf_path: "hans".to_string(), name: "HANS".to_string(), category: DatasetCategory::Entailment, split: "train".to_string(), estimated_tokens: 30_000, license: "MIT".to_string(), priority: 112 },
        DatasetInfo { hf_path: "paws".to_string(), name: "PAWS".to_string(), category: DatasetCategory::Entailment, split: "train".to_string(), estimated_tokens: 49_000, license: "Custom".to_string(), priority: 113 },
        DatasetInfo { hf_path: "qqp".to_string(), name: "QQP".to_string(), category: DatasetCategory::Entailment, split: "train".to_string(), estimated_tokens: 400_000, license: "Custom".to_string(), priority: 114 },
        DatasetInfo { hf_path: "mrpc".to_string(), name: "MRPC".to_string(), category: DatasetCategory::Entailment, split: "train".to_string(), estimated_tokens: 5_800, license: "Custom".to_string(), priority: 115 },

        // =============================================================================
        // 8. Commonsense Knowledge Datasets (116-125)
        // =============================================================================
        DatasetInfo { hf_path: "conceptnet5/conceptnet5".to_string(), name: "ConceptNet5".to_string(), category: DatasetCategory::Commonsense, split: "train".to_string(), estimated_tokens: 34_000, license: "CC-BY-SA".to_string(), priority: 116 },
        DatasetInfo { hf_path: "allenai/ai2_arc".to_string(), name: "ARC-Easy".to_string(), category: DatasetCategory::Commonsense, split: "train".to_string(), estimated_tokens: 2_251, license: "CC-BY".to_string(), priority: 117 },
        DatasetInfo { hf_path: "allenai/winogrande".to_string(), name: "WinoGrande".to_string(), category: DatasetCategory::Commonsense, split: "train".to_string(), estimated_tokens: 40_398, license: "Apache-2.0".to_string(), priority: 118 },
        DatasetInfo { hf_path: "allenai/swag".to_string(), name: "SWAG".to_string(), category: DatasetCategory::Commonsense, split: "train".to_string(), estimated_tokens: 73_546, license: "MIT".to_string(), priority: 119 },
        DatasetInfo { hf_path: "tau/commonsense_qa".to_string(), name: "CommonsenseQA".to_string(), category: DatasetCategory::Commonsense, split: "train".to_string(), estimated_tokens: 9_741, license: "MIT".to_string(), priority: 120 },
        DatasetInfo { hf_path: "Rowan/hellaswag".to_string(), name: "HellaSwag-CS".to_string(), category: DatasetCategory::Commonsense, split: "train".to_string(), estimated_tokens: 39_905, license: "MIT".to_string(), priority: 121 },
        DatasetInfo { hf_path: "allenai/openbookqa".to_string(), name: "OpenBookQA-CS".to_string(), category: DatasetCategory::Commonsense, split: "train".to_string(), estimated_tokens: 4_957, license: "Apache-2.0".to_string(), priority: 122 },
        DatasetInfo { hf_path: "codah".to_string(), name: "CODAH".to_string(), category: DatasetCategory::Commonsense, split: "train".to_string(), estimated_tokens: 2_800, license: "MIT".to_string(), priority: 123 },
        DatasetInfo { hf_path: "glucose".to_string(), name: "GLUCOSE".to_string(), category: DatasetCategory::Commonsense, split: "train".to_string(), estimated_tokens: 670_000, license: "CC-BY".to_string(), priority: 124 },
        DatasetInfo { hf_path: "event2mind".to_string(), name: "Event2Mind".to_string(), category: DatasetCategory::Commonsense, split: "train".to_string(), estimated_tokens: 57_000, license: "CC-BY".to_string(), priority: 125 },
    ]
}

/// Get datasets by category
pub fn get_datasets_by_category(category: DatasetCategory) -> Vec<DatasetInfo> {
    get_priority_datasets().into_iter()
        .filter(|d| d.category == category)
        .collect()
}

/// Get top N priority datasets
pub fn get_top_priority_datasets(n: usize) -> Vec<DatasetInfo> {
    let mut datasets = get_priority_datasets();
    datasets.sort_by_key(|d| d.priority);
    datasets.into_iter().take(n).collect()
}

// =============================================================================
// Dataset Loader
// =============================================================================

/// Configuration for dataset loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetLoaderConfig {
    /// Cache directory for downloaded datasets
    pub cache_dir: PathBuf,
    /// Maximum samples to load per dataset (0 = all)
    pub max_samples: usize,
    /// Enable streaming mode (don't download full dataset)
    pub streaming: bool,
    /// Shuffle data
    pub shuffle: bool,
    /// Random seed for shuffling
    pub seed: u64,
}

impl Default for DatasetLoaderConfig {
    fn default() -> Self {
        Self {
            cache_dir: PathBuf::from("./hf_cache"),
            max_samples: 10000, // Start small for testing
            streaming: true,
            shuffle: true,
            seed: 42,
        }
    }
}

/// A single training example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingExample {
    pub text: String,
    pub source: String,
    pub category: DatasetCategory,
    /// Optional: question for Q&A datasets
    pub question: Option<String>,
    /// Optional: answer for Q&A datasets
    pub answer: Option<String>,
}

impl TrainingExample {
    /// Convert text to BeamTensor sequence with word-level tokenization
    pub fn to_beams(&self, max_len: usize) -> Vec<BeamTensor> {
        let text = self.question.as_ref()
            .map(|q| format!("{} {}", q, self.answer.as_deref().unwrap_or("")))
            .unwrap_or_else(|| self.text.clone());
        
        // Word-level tokenization - each word becomes a BeamTensor
        let words: Vec<&str> = text.split_whitespace().take(max_len).collect();
        
        words.iter().enumerate()
            .map(|(idx, word)| {
                let mut beam = BeamTensor::default();
                
                // Store the actual word for embeddings
                beam.word = word.to_string();
                
                // Encode word bytes into digits (first 9 bytes)
                let bytes = word.as_bytes();
                for (i, &b) in bytes.iter().take(9).enumerate() {
                    beam.digits[i] = (b as f32) / 255.0;
                }
                
                // Word length feature
                if bytes.len() > 9 {
                    beam.digits[8] = (bytes.len() as f32 / 20.0).min(1.0);
                }
                
                beam.confidence = 0.8;
                beam.position = ((idx % 9) + 1) as u8;
                beam
            })
            .collect()
    }
}

/// HuggingFace Dataset Loader
/// 
/// Downloads and streams datasets from HuggingFace Hub.
/// Uses HTTP API for parquet files when available.
pub struct HFDatasetLoader {
    config: DatasetLoaderConfig,
    datasets: Vec<DatasetInfo>,
    loaded_examples: HashMap<String, Vec<TrainingExample>>,
}

impl HFDatasetLoader {
    pub fn new(config: DatasetLoaderConfig) -> Self {
        Self {
            config,
            datasets: get_priority_datasets(),
            loaded_examples: HashMap::new(),
        }
    }

    /// Load all priority datasets
    pub fn load_all(&mut self) -> Result<usize, String> {
        let mut total = 0;
        let datasets = self.datasets.clone();
        
        for dataset in &datasets {
            match self.load_dataset(&dataset.hf_path) {
                Ok(count) => {
                    println!("   ✓ Loaded {} examples from {}", count, dataset.name);
                    total += count;
                }
                Err(e) => {
                    println!("   ⚠ Failed to load {}: {}", dataset.name, e);
                }
            }
        }
        
        Ok(total)
    }

    /// Load a specific dataset by HF path
    /// Tries real HuggingFace download first, falls back to synthetic examples
    pub fn load_dataset(&mut self, hf_path: &str) -> Result<usize, String> {
        // Find dataset info
        let info = self.datasets.iter()
            .find(|d| d.hf_path == hf_path)
            .cloned()
            .ok_or_else(|| format!("Unknown dataset: {}", hf_path))?;

        // Try to load from HuggingFace Hub first
        let examples = match self.load_from_hf_hub(&info) {
            Ok(ex) if !ex.is_empty() => {
                println!("      ✓ Downloaded {} examples from HF: {}", ex.len(), info.name);
                ex
            }
            Ok(_) => {
                println!("      ⚠ HF returned empty for {}, using synthetic", info.name);
                self.generate_examples(&info)?
            }
            Err(e) => {
                println!("      ⚠ HF download failed for {}: {}", info.name, e);
                // Fall back to synthetic examples
                self.generate_examples(&info)?
            }
        };
        
        let count = examples.len();
        self.loaded_examples.insert(hf_path.to_string(), examples);
        Ok(count)
    }
    
    /// Load dataset from HuggingFace Datasets Server API
    fn load_from_hf_hub(&self, info: &DatasetInfo) -> Result<Vec<TrainingExample>, String> {
        // Use HuggingFace Datasets Server REST API (more reliable than file downloads)
        let max_samples = if self.config.max_samples == 0 { 100 } else { self.config.max_samples.min(100) };
        
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Client error: {}", e))?;
        
        // Dataset-specific configs (many HF datasets require specific config names)
        let configs = self.get_dataset_configs(&info.hf_path);
        
        for config in &configs {
            let url = format!(
                "https://datasets-server.huggingface.co/rows?dataset={}&config={}&split={}&offset=0&length={}",
                info.hf_path, config, info.split, max_samples
            );
            
            if let Ok(response) = client.get(&url)
                .header("User-Agent", "SpatialVortex/1.0")
                .send() 
            {
                if response.status().is_success() {
                    return self.parse_hf_api_response(response, info);
                }
            }
        }
        
        Err(format!("No valid config found for {}", info.name))
    }
    
    /// Get dataset-specific config names to try
    fn get_dataset_configs(&self, hf_path: &str) -> Vec<&'static str> {
        match hf_path {
            // ARC has ARC-Easy and ARC-Challenge configs
            "allenai/ai2_arc" => vec!["ARC-Easy", "ARC-Challenge"],
            // WinoGrande configs
            "allenai/winogrande" => vec!["winogrande_xl", "winogrande_l", "winogrande_m"],
            // SWAG config
            "allenai/swag" => vec!["regular", "full"],
            // HellaSwag
            "Rowan/hellaswag" => vec!["default"],
            // OpenBookQA
            "allenai/openbookqa" => vec!["main", "additional"],
            // SICK
            "mteb/sickr-sts" => vec!["default"],
            // Ethics
            "hendrycks/ethics" => vec!["commonsense", "deontology", "justice", "utilitarianism", "virtue"],
            // ANLI - facebook/anli uses plain_text config with train_r1/r2/r3 splits
            "facebook/anli" => vec!["plain_text"],
            // arXiv
            "ccdv/arxiv-summarization" => vec!["document", "section"],
            // PubMedQA
            "qiaojin/PubMedQA" => vec!["pqa_labeled", "pqa_unlabeled", "pqa_artificial"],
            // MedQA (replacement for BioASQ)
            "bigbio/med_qa" => vec!["med_qa_en_source", "med_qa_en_bigbio_qa"],
            // SocialIQA
            "allenai/social_i_qa" => vec!["default"],
            "sap2019/socialiqa" => vec!["default"],
            // PIQA
            "piqa" => vec!["plain_text"],
            "ybisk/piqa" => vec!["plain_text", "default"],
            // CosmosQA
            "allenai/cosmosqa" => vec!["default"],
            // DROP
            "drop" => vec!["default"],
            "allenai/drop" => vec!["default"],
            // XNLI - needs language config
            "facebook/xnli" => vec!["en", "all_languages"],
            // CoQA
            "stanfordnlp/coqa" => vec!["default"],
            // MultiRC
            "aps/super_glue" => vec!["multirc"],
            "super_glue" => vec!["multirc"],
            "multi_rc" => vec!["default"],
            // MATH
            "lighteval/MATH" => vec!["default", "all"],
            "hendrycks/math" => vec!["default"],
            // TheoremQA
            "TIGER-Lab/TheoremQA" => vec!["default"],
            "wenhu/theoremqa" => vec!["default"],
            // SciTail
            "allenai/scitail" => vec!["snli_format", "tsv_format"],
            // ConceptNet5 (replacement for LAMA)
            "conceptnet5/conceptnet5" => vec!["conceptnet5"],
            // OpenWebMath (replacement for ProofPile)
            "open-web-math/open-web-math" => vec!["default"],
            // Default fallback configs
            _ => vec!["default", "main", "plain_text", "en", "all"],
        }
    }
    
    /// Parse HuggingFace Datasets Server API response
    fn parse_hf_api_response(&self, response: reqwest::blocking::Response, info: &DatasetInfo) -> Result<Vec<TrainingExample>, String> {
        let data: HFDatasetResponse = response.json()
            .map_err(|e| format!("JSON parse error: {}", e))?;
        
        let examples: Vec<TrainingExample> = data.rows.iter()
            .map(|row| self.json_to_example(&row.row, info))
            .filter(|ex| !ex.text.is_empty() || ex.question.is_some())
            .collect();
        
        Ok(examples)
    }
    
    /// Load dataset from file (fallback for hf-hub file downloads)
    #[allow(dead_code)]
    fn load_from_hf_file(&self, info: &DatasetInfo) -> Result<Vec<TrainingExample>, String> {
        // Initialize HF Hub API
        let api = Api::new().map_err(|e| format!("HF API error: {}", e))?;
        
        // Try to get the dataset repo
        let repo = api.dataset(info.hf_path.clone());
        
        // Common dataset file patterns to try
        let file_patterns = [
            format!("{}.jsonl", info.split),
            format!("data/{}.jsonl", info.split),
            format!("{}.json", info.split),
            "train.jsonl".to_string(),
        ];
        
        for pattern in &file_patterns {
            if let Ok(path) = repo.get(pattern) {
                return self.parse_dataset_file(&path, info);
            }
        }
        
        Err(format!("Could not find data file for {}", info.name))
    }
    
    /// Parse a downloaded dataset file into TrainingExamples
    fn parse_dataset_file(&self, path: &PathBuf, info: &DatasetInfo) -> Result<Vec<TrainingExample>, String> {
        let file = std::fs::File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut examples = Vec::new();
        let max_samples = if self.config.max_samples == 0 { 100_000 } else { self.config.max_samples };
        
        // Try JSONL format (one JSON object per line)
        for line in reader.lines().take(max_samples) {
            let line = line.map_err(|e| format!("Read error: {}", e))?;
            if line.trim().is_empty() { continue; }
            
            // Parse JSON and extract fields based on common patterns
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                let example = self.json_to_example(&json, info);
                if example.text.len() > 0 || example.question.is_some() {
                    examples.push(example);
                }
            }
        }
        
        Ok(examples)
    }
    
    /// Convert a JSON object to a TrainingExample based on common field patterns
    fn json_to_example(&self, json: &serde_json::Value, info: &DatasetInfo) -> TrainingExample {
        // Common field names for different dataset types
        let text = json.get("text")
            .or_else(|| json.get("content"))
            .or_else(|| json.get("sentence"))
            .or_else(|| json.get("passage"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let question = json.get("question")
            .or_else(|| json.get("query"))
            .or_else(|| json.get("prompt"))
            .or_else(|| json.get("input"))
            .or_else(|| json.get("premise"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let answer = json.get("answer")
            .or_else(|| json.get("response"))
            .or_else(|| json.get("output"))
            .or_else(|| json.get("label"))
            .or_else(|| json.get("hypothesis"))
            .or_else(|| json.get("target"))
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(n) = v.as_i64() {
                    Some(n.to_string())
                } else {
                    None
                }
            });
        
        TrainingExample {
            text,
            source: info.name.clone(),
            category: info.category,
            question,
            answer,
        }
    }

    /// Generate training examples (simulated - would be real HF download)
    fn generate_examples(&self, info: &DatasetInfo) -> Result<Vec<TrainingExample>, String> {
        // If max_samples is 0, use full dataset (capped at reasonable size for memory)
        let count = if self.config.max_samples == 0 {
            // Use estimated tokens but cap at 100K for memory efficiency
            (info.estimated_tokens as usize).min(100_000)
        } else {
            self.config.max_samples.min(info.estimated_tokens as usize)
        };
        
        let examples: Vec<TrainingExample> = match info.category {
            DatasetCategory::Math => {
                // Generate math-style examples with varied operations
                (0..count).map(|i| {
                    let a = (i % 100) + 1;
                    let b = ((i * 7) % 100) + 1;
                    let op = match i % 4 {
                        0 => ("+", a + b),
                        1 => ("-", if a > b { a - b } else { b - a }),
                        2 => ("*", a * b),
                        _ => ("/", if b > 0 { a / b.max(1) } else { a }),
                    };
                    TrainingExample {
                        text: String::new(),
                        source: info.name.clone(),
                        category: info.category,
                        question: Some(format!("What is {} {} {}?", a, op.0, b)),
                        answer: Some(format!("{}", op.1)),
                    }
                }).collect()
            }
            DatasetCategory::Benchmark => {
                // Generate benchmark-style Q&A
                (0..count).map(|i| {
                    TrainingExample {
                        text: String::new(),
                        source: info.name.clone(),
                        category: info.category,
                        question: Some(format!("Question {}: What is the capital of country {}?", i, i % 50)),
                        answer: Some(format!("Answer {}", i % 50)),
                    }
                }).collect()
            }
            DatasetCategory::Code => {
                // Generate code examples in multiple languages
                (0..count).map(|i| {
                    let lang = match i % 4 {
                        0 => format!("fn example_{}() -> i32 {{\n    let x = {};\n    let y = {};\n    x + y\n}}", i, i % 100, (i * 3) % 100),
                        1 => format!("def example_{}():\n    x = {}\n    y = {}\n    return x + y", i, i % 100, (i * 3) % 100),
                        2 => format!("function example{}() {{\n    const x = {};\n    const y = {};\n    return x + y;\n}}", i, i % 100, (i * 3) % 100),
                        _ => format!("public int example{}() {{\n    int x = {};\n    int y = {};\n    return x + y;\n}}", i, i % 100, (i * 3) % 100),
                    };
                    TrainingExample {
                        text: lang,
                        source: info.name.clone(),
                        category: info.category,
                        question: None,
                        answer: None,
                    }
                }).collect()
            }
            DatasetCategory::Reasoning => {
                // Generate reasoning examples with logical chains
                (0..count).map(|i| {
                    let premises = vec![
                        format!("If A then B. A is true."),
                        format!("All X are Y. Z is an X."),
                        format!("Either P or Q. Not P."),
                        format!("If it rains, the ground is wet. The ground is wet."),
                    ];
                    let conclusions = vec!["Therefore B.", "Therefore Z is Y.", "Therefore Q.", "It may have rained."];
                    let idx = i % premises.len();
                    TrainingExample {
                        text: String::new(),
                        source: info.name.clone(),
                        category: info.category,
                        question: Some(premises[idx].clone()),
                        answer: Some(conclusions[idx].to_string()),
                    }
                }).collect()
            }
            DatasetCategory::Science => {
                // Generate science Q&A examples
                (0..count).map(|i| {
                    let topics = vec![
                        ("What is the chemical formula for water?", "H2O"),
                        ("What is the speed of light?", "299,792,458 meters per second"),
                        ("What is DNA?", "Deoxyribonucleic acid, the molecule carrying genetic instructions"),
                        ("What is photosynthesis?", "The process plants use to convert light into energy"),
                        ("What is gravity?", "A fundamental force that attracts objects with mass"),
                    ];
                    let idx = i % topics.len();
                    TrainingExample {
                        text: String::new(),
                        source: info.name.clone(),
                        category: info.category,
                        question: Some(topics[idx].0.to_string()),
                        answer: Some(topics[idx].1.to_string()),
                    }
                }).collect()
            }
            DatasetCategory::QA => {
                // Generate general Q&A examples
                (0..count).map(|i| {
                    TrainingExample {
                        text: String::new(),
                        source: info.name.clone(),
                        category: info.category,
                        question: Some(format!("Context: The {} is located in region {}. Question: Where is the {}?", 
                            i % 100, i % 10, i % 100)),
                        answer: Some(format!("Region {}", i % 10)),
                    }
                }).collect()
            }
            // Vision and Multimodal categories disabled - requires specialized processing
            // DatasetCategory::Vision => { ... }
            // DatasetCategory::Multimodal => { ... }
            DatasetCategory::Entailment => {
                // Generate NLI/entailment examples for deductive reasoning
                // Format: premise, hypothesis, label (entailment/neutral/contradiction)
                (0..count).map(|i| {
                    let examples = vec![
                        ("A man is playing a guitar.", "A person is making music.", "entailment"),
                        ("A woman is reading a book.", "A woman is sleeping.", "contradiction"),
                        ("Children are playing in the park.", "People are outdoors.", "entailment"),
                        ("The cat is on the mat.", "The mat is under the cat.", "entailment"),
                        ("It is raining outside.", "The weather is sunny.", "contradiction"),
                        ("All birds can fly.", "Penguins can fly.", "neutral"),
                        ("The restaurant is expensive.", "The food costs a lot.", "entailment"),
                        ("She finished her homework.", "She started her homework.", "neutral"),
                        ("The door is open.", "The door is closed.", "contradiction"),
                        ("He drives to work.", "He has a car.", "entailment"),
                    ];
                    let idx = i % examples.len();
                    let (premise, hypothesis, label) = examples[idx];
                    TrainingExample {
                        text: format!("Premise: {} Hypothesis: {}", premise, hypothesis),
                        source: info.name.clone(),
                        category: info.category,
                        question: Some(format!("Does the premise entail the hypothesis? Premise: '{}' Hypothesis: '{}'", premise, hypothesis)),
                        answer: Some(label.to_string()),
                    }
                }).collect()
            }
            DatasetCategory::Commonsense => {
                // Generate commonsense knowledge examples (ConceptNet/ATOMIC style)
                // Format: head, relation, tail for knowledge graph triples
                (0..count).map(|i| {
                    let knowledge = vec![
                        ("dog", "IsA", "animal", "A dog is a type of animal."),
                        ("bird", "CapableOf", "fly", "Birds are capable of flying."),
                        ("ice", "HasProperty", "cold", "Ice has the property of being cold."),
                        ("PersonX eats food", "xEffect", "PersonX is not hungry", "After eating, a person is not hungry."),
                        ("PersonX goes to school", "xIntent", "to learn", "People go to school to learn."),
                        ("fire", "Causes", "heat", "Fire causes heat."),
                        ("rain", "Causes", "wet ground", "Rain causes the ground to be wet."),
                        ("sleep", "HasPrerequisite", "tired", "Being tired is a prerequisite for sleep."),
                        ("PersonX helps PersonY", "xAttr", "kind", "Helping others shows kindness."),
                        ("car", "UsedFor", "transportation", "Cars are used for transportation."),
                        ("knife", "UsedFor", "cutting", "Knives are used for cutting."),
                        ("book", "UsedFor", "reading", "Books are used for reading."),
                        ("PersonX wins the game", "xReact", "happy", "Winning makes a person happy."),
                        ("PersonX loses the game", "xReact", "sad", "Losing makes a person sad."),
                        ("water", "HasProperty", "liquid", "Water is a liquid at room temperature."),
                    ];
                    let idx = i % knowledge.len();
                    let (head, relation, tail, explanation) = knowledge[idx];
                    TrainingExample {
                        text: format!("[{}, {}, {}] {}", head, relation, tail, explanation),
                        source: info.name.clone(),
                        category: info.category,
                        question: Some(format!("What is the {} of '{}'?", relation, head)),
                        answer: Some(tail.to_string()),
                    }
                }).collect()
            }
            DatasetCategory::PreTraining => {
                // Pre-training style text with varied content
                (0..count).map(|i| {
                    let topics = vec![
                        "Machine learning models learn patterns from data through iterative optimization.",
                        "Neural networks process information through layers of interconnected nodes.",
                        "Natural language processing enables computers to understand human language.",
                        "Deep learning has revolutionized computer vision and speech recognition.",
                        "Transformers use attention mechanisms to process sequential data efficiently.",
                    ];
                    TrainingExample {
                        text: format!(
                            "Training example {} from {}: {} {}",
                            i, info.name, topics[i % topics.len()],
                            "Training involves adjusting weights to minimize loss functions."
                        ),
                        source: info.name.clone(),
                        category: info.category,
                        question: None,
                        answer: None,
                    }
                }).collect()
            }
        };
        
        Ok(examples)
    }

    /// Get all loaded examples as training pairs
    pub fn get_training_pairs(&self, max_seq_len: usize) -> Vec<(Vec<BeamTensor>, Vec<BeamTensor>)> {
        let mut pairs = Vec::new();
        
        for examples in self.loaded_examples.values() {
            for example in examples {
                let beams = example.to_beams(max_seq_len);
                if beams.len() >= 2 {
                    // Input is all but last, target is shifted by 1
                    let input = beams[..beams.len()-1].to_vec();
                    let target = beams[1..].to_vec();
                    pairs.push((input, target));
                }
            }
        }
        
        // Shuffle if configured
        if self.config.shuffle {
            use rand::seq::SliceRandom;
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::seed_from_u64(self.config.seed);
            pairs.shuffle(&mut rng);
        }
        
        pairs
    }

    /// Get examples by category
    pub fn get_by_category(&self, category: DatasetCategory) -> Vec<&TrainingExample> {
        self.loaded_examples.values()
            .flat_map(|examples| examples.iter())
            .filter(|e| e.category == category)
            .collect()
    }

    /// Get dataset statistics
    pub fn stats(&self) -> DatasetStats {
        let mut total_examples = 0;
        let mut by_category: HashMap<DatasetCategory, usize> = HashMap::new();
        
        for examples in self.loaded_examples.values() {
            total_examples += examples.len();
            for example in examples {
                *by_category.entry(example.category).or_insert(0) += 1;
            }
        }
        
        DatasetStats {
            total_examples,
            datasets_loaded: self.loaded_examples.len(),
            by_category,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatasetStats {
    pub total_examples: usize,
    pub datasets_loaded: usize,
    pub by_category: HashMap<DatasetCategory, usize>,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_loader() {
        let config = DatasetLoaderConfig {
            max_samples: 100,
            ..Default::default()
        };
        let mut loader = HFDatasetLoader::new(config);
        
        let count = loader.load_dataset("openai/gsm8k").unwrap();
        assert_eq!(count, 100);
        
        let stats = loader.stats();
        assert_eq!(stats.datasets_loaded, 1);
        assert_eq!(stats.total_examples, 100);
    }

    #[test]
    fn test_training_pairs() {
        let config = DatasetLoaderConfig {
            max_samples: 50,
            ..Default::default()
        };
        let mut loader = HFDatasetLoader::new(config);
        loader.load_dataset("openai/gsm8k").unwrap();
        
        let pairs = loader.get_training_pairs(64);
        assert!(!pairs.is_empty());
    }

    #[test]
    fn test_priority_datasets() {
        let datasets = get_priority_datasets();
        // 106 datasets: 86 original + 10 Entailment + 10 Commonsense
        assert_eq!(datasets.len(), 106, "Should have 106 datasets (including Entailment and Commonsense)");
        
        // Check key datasets are included
        assert!(datasets.iter().any(|d| d.name == "GSM8K"));
        assert!(datasets.iter().any(|d| d.name == "FineWeb"));
        assert!(datasets.iter().any(|d| d.name == "The Stack"));
        assert!(datasets.iter().any(|d| d.name == "SQuAD"));
        // New entailment datasets
        assert!(datasets.iter().any(|d| d.name == "SNLI"));
        assert!(datasets.iter().any(|d| d.name == "MultiNLI"));
        // New commonsense datasets
        assert!(datasets.iter().any(|d| d.name == "ConceptNet"));
        assert!(datasets.iter().any(|d| d.name == "ATOMIC"));
    }
    
    #[test]
    fn test_datasets_by_category() {
        let code_datasets = get_datasets_by_category(DatasetCategory::Code);
        assert!(code_datasets.len() >= 10, "Should have at least 10 code datasets");
        
        let math_datasets = get_datasets_by_category(DatasetCategory::Math);
        assert!(math_datasets.len() >= 4, "Should have at least 4 math datasets");
    }
    
    #[test]
    fn test_top_priority_datasets() {
        let top_10 = get_top_priority_datasets(10);
        assert_eq!(top_10.len(), 10);
        assert_eq!(top_10[0].priority, 1);
    }
}
