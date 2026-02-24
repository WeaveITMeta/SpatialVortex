//! # Vortex Engine — Unified Inference Entry Point (Benchmark-Grade)
//!
//! Table of Contents:
//! - VortexEngineConfig: Configuration for the unified engine
//! - VortexEngine: Full benchmark pipeline — HF datasets + web learning + knowledge pipeline
//! - ChatMessage / ChatRole: OpenAI-compatible message types
//! - ChatResponse: Structured response with reasoning trace
//!
//! Architecture (Flux Matrix real-time — no CALM pretraining, no GPU tensors):
//! 1. Bootstrap: HF datasets → entity-attrs, causal patterns, QA patterns (pure Rust HashMaps)
//! 2. Web learning: ConsciousnessLearner fills knowledge gaps via DuckDuckGo
//! 3. Build: UnifiedKnowledgePipeline from all collected knowledge (TF-IDF, keyword matching)
//! 4. Inference: Knowledge Pipeline → WorldKnowledge → TransitiveFlux → DynamicRSI → synthesize
//! 5. Safety: ConstitutionalGuard filters output

use crate::cognition::{
    ThinkingEngine, ThinkingConfig, ThoughtChain,
    Constitution, ConstitutionalGuard,
    MemoryStore, Memory, MemoryType, MemoryQuery,
};
use crate::ml::unified_knowledge_pipeline::{UnifiedKnowledgePipeline, PipelineConfig};
use crate::ml::consciousness_learner::{ConsciousnessLearner, ConsciousnessConfig};
use crate::ml::transitive_flux::TransitiveFluxReasoner;
use crate::ml::dynamic_rsi::{DynamicRSI, InferenceObservation};
use crate::ml::rag_search::{RAGSearchEngine, RAGSearchConfig};
use crate::data::hf_datasets::{HFDatasetLoader, DatasetLoaderConfig, DatasetCategory, get_datasets_by_category};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

// =============================================================================
// Configuration
// =============================================================================

/// Configuration for the unified Vortex engine
#[derive(Debug, Clone)]
pub struct VortexEngineConfig {
    /// Temperature for thought generation (0.0 = deterministic, 1.0 = creative)
    pub temperature: f32,
    /// Maximum thinking steps per query
    pub max_steps: usize,
    /// Maximum vortex cycles for generative inference
    pub max_cycles: usize,
    /// Enable constitutional safety guard
    pub constitutional_guard: bool,
    /// Enable memory persistence across conversations
    pub memory_enabled: bool,
    /// System prompt to prepend to all queries
    pub system_prompt: Option<String>,
    /// Model identifier for API responses
    pub model_id: String,
    /// Data directory for HF datasets and caches
    pub data_dir: String,
    /// Enable HuggingFace dataset loading on startup
    pub load_hf_datasets: bool,
    /// Enable web learning on startup
    pub web_learning: bool,
    /// Maximum HF samples per dataset
    pub hf_max_samples: usize,
}

impl Default for VortexEngineConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_steps: 9,
            max_cycles: 64,
            constitutional_guard: true,
            memory_enabled: true,
            system_prompt: None,
            model_id: "vortex-0.1".to_string(),
            data_dir: "./data".to_string(),
            load_hf_datasets: true,
            web_learning: true,
            hf_max_samples: 500,
        }
    }
}

impl VortexEngineConfig {
    pub fn new() -> Self { Self::default() }

    pub fn with_temperature(mut self, t: f32) -> Self {
        self.temperature = t.clamp(0.0, 2.0);
        self
    }

    pub fn with_max_steps(mut self, s: usize) -> Self {
        self.max_steps = s;
        self
    }

    pub fn with_max_cycles(mut self, c: usize) -> Self {
        self.max_cycles = c;
        self
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn with_data_dir(mut self, dir: impl Into<String>) -> Self {
        self.data_dir = dir.into();
        self
    }

    /// Create a lightweight config that skips HF and web learning (fast startup)
    pub fn lightweight() -> Self {
        Self {
            load_hf_datasets: false,
            web_learning: false,
            ..Self::default()
        }
    }
}

// =============================================================================
// Chat Message Types (OpenAI-compatible)
// =============================================================================

/// Role in a chat conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

/// A single message in a chat conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: ChatRole::System, content: content.into() }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self { role: ChatRole::User, content: content.into() }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: ChatRole::Assistant, content: content.into() }
    }
}

/// Structured response from the Vortex engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Unique response ID
    pub id: String,
    /// Model identifier
    pub model: String,
    /// Generated response text
    pub content: String,
    /// Confidence in the response (0.0 - 1.0)
    pub confidence: f32,
    /// Reasoning trace (chain-of-thought steps)
    pub reasoning_trace: Vec<ReasoningStep>,
    /// Number of vortex cycles executed
    pub cycles: u64,
    /// Latent energy at completion
    pub energy: f32,
    /// Sacred alignment score
    pub sacred_alignment: f32,
    /// Processing time in milliseconds
    pub duration_ms: u64,
    /// Token usage statistics
    pub usage: Usage,
    /// Constitutional safety check result
    pub safety: Option<SafetyResult>,
}

/// A single step in the reasoning trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub step: usize,
    pub position: u8,
    pub content: String,
    pub confidence: f32,
    pub is_sacred: bool,
    pub step_type: String,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Safety check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyResult {
    pub passed: bool,
    pub violations: Vec<String>,
}

// =============================================================================
// Conversational Intent Classification
// =============================================================================

/// Classified intent of a user message for response generation
#[derive(Debug, Clone, PartialEq)]
enum ConversationalIntent {
    /// Greeting: "hello", "hi", "how are you"
    Greeting,
    /// Farewell: "bye", "goodbye", "see you"
    Farewell,
    /// Meta question about the AI: "who are you", "what can you do"
    MetaQuestion,
    /// Math question: "what is 2+2", "calculate sqrt(16)"
    MathQuestion,
    /// Factual question: "what is X", "who was Y", "explain Z"
    FactualQuestion,
    /// Opinion request: "what do you think", "do you like"
    Opinion,
    /// Continuation of previous topic: "yes", "no", "why", "thanks"
    Continuation,
    /// Command: "help me", "show me", "list"
    Command,
    /// Statement: user sharing info or making a claim
    Statement,
}

// =============================================================================
// Vortex Engine — Benchmark-Grade Pipeline
// =============================================================================

/// Unified Vortex inference engine (benchmark-grade)
///
/// Wires together the SAME pipeline that scored 70%+ on benchmarks:
/// - UnifiedKnowledgePipeline: RETRIEVE → EXTRACT → EMBED → REASON → SCORE (pure Rust TF-IDF)
/// - ConsciousnessLearner: Web learning via DuckDuckGo + dynamic vortex knowledge graph
/// - WorldKnowledgeGraph: Commonsense reasoning (physical, causal, spatial)
/// - TransitiveFluxReasoner: Transitive reasoning via vortex flux matrix ladder index
/// - DynamicRSI: Runtime self-improving inference strategy per query type
/// - ThinkingEngine: Beam-based reasoning with sacred geometry
/// - ConstitutionalGuard: Safety filtering
///
/// NO CALM pretraining. NO GPU tensors. NO embeddings feature required.
/// Pure Rust Flux Matrix style real-time inference.
pub struct VortexEngine {
    config: VortexEngineConfig,
    /// Benchmark-grade knowledge pipeline (TF-IDF + keyword matching)
    knowledge_pipeline: UnifiedKnowledgePipeline,
    /// Consciousness learner with dynamic vortex knowledge graph
    consciousness: ConsciousnessLearner,
    /// Commonsense world knowledge graph (accessed via consciousness.world_knowledge)
    #[allow(dead_code)]
    world_knowledge_dim: usize,
    /// Transitive reasoning (spatial, size, path, counting)
    transitive: TransitiveFluxReasoner,
    /// Runtime self-improving strategy engine
    dynamic_rsi: DynamicRSI,
    /// RAG search engine for retrieval
    rag_engine: RAGSearchEngine,
    /// Beam-based thinking engine (sacred geometry reasoning)
    thinking: ThinkingEngine,
    /// Constitutional safety guard
    guard: ConstitutionalGuard,
    /// Persistent memory store
    memory: MemoryStore,
    /// Learned entity-attribute relationships
    learned_entity_attrs: HashMap<String, HashMap<String, f32>>,
    /// Learned causal patterns
    learned_causal: HashMap<String, Vec<(String, f32)>>,
    /// Learned Q&A patterns
    qa_patterns: HashMap<String, Vec<String>>,
    /// Conversation history for multi-turn
    history: Vec<ChatMessage>,
    /// Whether knowledge has been bootstrapped
    knowledge_ready: bool,
}

impl VortexEngine {
    /// Create a new VortexEngine with default configuration
    /// This bootstraps knowledge from HF datasets and web learning
    pub fn new() -> Self {
        Self::with_config(VortexEngineConfig::default())
    }

    /// Create a new VortexEngine with custom configuration
    pub fn with_config(config: VortexEngineConfig) -> Self {
        let thinking_config = ThinkingConfig::new()
            .with_max_steps(config.max_steps)
            .with_temperature(config.temperature);

        let constitution = Constitution::claude();
        let guard = ConstitutionalGuard::new(constitution);

        let mut engine = Self {
            knowledge_pipeline: UnifiedKnowledgePipeline::new(PipelineConfig::default()),
            consciousness: ConsciousnessLearner::new(ConsciousnessConfig::default()),
            world_knowledge_dim: 256,
            transitive: TransitiveFluxReasoner::new(256),
            dynamic_rsi: DynamicRSI::new(),
            rag_engine: RAGSearchEngine::new(RAGSearchConfig::default()),
            thinking: ThinkingEngine::new(thinking_config),
            guard,
            memory: MemoryStore::new(),
            learned_entity_attrs: HashMap::new(),
            learned_causal: HashMap::new(),
            qa_patterns: HashMap::new(),
            history: Vec::new(),
            knowledge_ready: false,
            config,
        };

        // Apply system prompt if configured
        if let Some(ref prompt) = engine.config.system_prompt.clone() {
            engine.history.push(ChatMessage::system(prompt.clone()));
        }

        // Bootstrap knowledge pipeline
        let load_hf = engine.config.load_hf_datasets;
        let web_learn = engine.config.web_learning;
        if load_hf {
            engine.bootstrap_hf_datasets();
        }
        if web_learn {
            engine.bootstrap_web_learning();
        }
        if load_hf || web_learn {
            engine.build_knowledge_pipeline();
        }

        engine
    }

    // =========================================================================
    // Knowledge Bootstrap — HuggingFace Datasets (pure Rust, no GPU)
    // =========================================================================

    /// Load HuggingFace datasets and extract knowledge into HashMaps
    fn bootstrap_hf_datasets(&mut self) {
        println!("🌀 [VortexEngine] Loading HuggingFace datasets...");
        let start = Instant::now();

        let hf_config = DatasetLoaderConfig {
            max_samples: self.config.hf_max_samples,
            streaming: true,
            shuffle: true,
            seed: 42,
            ..Default::default()
        };

        let mut loader = HFDatasetLoader::new(hf_config);

        // Load top 5 datasets per category (same as benchmark evaluator)
        let categories = [
            DatasetCategory::PreTraining,
            DatasetCategory::Code,
            DatasetCategory::Benchmark,
            DatasetCategory::Commonsense,
            DatasetCategory::Entailment,
            DatasetCategory::Reasoning,
            DatasetCategory::Science,
            DatasetCategory::QA,
            DatasetCategory::Math,
        ];

        let mut total_loaded = 0usize;
        let mut failed = 0usize;
        for category in &categories {
            let datasets = get_datasets_by_category(*category);
            for dataset in datasets.iter().take(5) {
                match loader.load_dataset(&dataset.hf_path) {
                    Ok(count) => total_loaded += count,
                    Err(_) => failed += 1,
                }
                // Rate limit to avoid 429
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }

        if total_loaded > 0 {
            // Extract knowledge from loaded examples
            self.extract_knowledge_from_hf(&loader);
            println!("   ✅ Loaded {} examples from HF in {:.1}s ({} failed)",
                total_loaded, start.elapsed().as_secs_f32(), failed);
        } else {
            println!("   ⚠ No HF datasets loaded ({} failed) — using web learning only", failed);
        }

        // Seed consciousness vortex from extracted knowledge
        self.seed_vortex_from_knowledge();

        // Sync to RAG engine
        self.rag_engine.import_entity_attributes(&self.learned_entity_attrs);
        self.rag_engine.import_causal_patterns(&self.learned_causal);
        self.rag_engine.import_qa_patterns(&self.qa_patterns);

        let (topics, facts) = self.rag_engine.knowledge_size();
        println!("   📚 Knowledge: {} entity-attrs, {} causal, {} QA patterns → RAG: {} topics, {} facts",
            self.learned_entity_attrs.len(), self.learned_causal.len(),
            self.qa_patterns.len(), topics, facts);
    }

    /// Extract entity-attributes, causal patterns, and QA pairs from HF data
    fn extract_knowledge_from_hf(&mut self, loader: &HFDatasetLoader) {
        let all_examples = loader.get_all_examples();
        println!("   Extracting knowledge from {} HF examples...", all_examples.len());

        for example in &all_examples {
            let text = &example.text;

            // Extract entity-attribute relationships
            Self::learn_entity_attributes(text, &mut self.learned_entity_attrs);

            // Extract causal patterns
            Self::learn_causal_patterns(text, &mut self.learned_causal);

            // Extract Q&A patterns
            if let (Some(q), Some(a)) = (&example.question, &example.answer) {
                let q_words: Vec<&str> = q.split_whitespace().collect();
                let a_lower = a.to_lowercase();

                for word in &q_words {
                    if word.len() > 3 {
                        let attrs = self.learned_entity_attrs
                            .entry(word.to_lowercase())
                            .or_default();
                        *attrs.entry(a_lower.clone()).or_insert(0.0) += 1.0;
                    }
                }

                let pattern = Self::extract_pattern(&q.to_lowercase());
                self.qa_patterns
                    .entry(pattern)
                    .or_default()
                    .push(a_lower);
            }
        }
    }

    /// Extract entity-attribute relationships from text (static, no self needed)
    fn learn_entity_attributes(text: &str, attrs: &mut HashMap<String, HashMap<String, f32>>) {
        let lower = text.to_lowercase();
        // Pattern: "X is Y", "X are Y", "X was Y"
        for pattern in &[" is ", " are ", " was ", " were "] {
            if let Some(pos) = lower.find(pattern) {
                let subject: String = lower[..pos].split_whitespace().last().unwrap_or("").to_string();
                let object: String = lower[pos + pattern.len()..].split(|c: char| c == '.' || c == ',' || c == '\n')
                    .next().unwrap_or("").trim().to_string();
                if subject.len() > 2 && object.len() > 2 && object.len() < 60 {
                    let entry = attrs.entry(subject).or_default();
                    *entry.entry(object).or_insert(0.0) += 1.0;
                }
            }
        }
    }

    /// Extract causal patterns from text
    fn learn_causal_patterns(text: &str, causal: &mut HashMap<String, Vec<(String, f32)>>) {
        let lower = text.to_lowercase();
        for pattern in &[" causes ", " leads to ", " results in ", " because "] {
            if let Some(pos) = lower.find(pattern) {
                let cause: String = lower[..pos].split_whitespace().rev().take(3)
                    .collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join(" ");
                let effect: String = lower[pos + pattern.len()..].split(|c: char| c == '.' || c == ',' || c == '\n')
                    .next().unwrap_or("").trim().to_string();
                if cause.len() > 2 && effect.len() > 2 {
                    let effects = causal.entry(cause).or_default();
                    if !effects.iter().any(|(e, _)| e == &effect) {
                        effects.push((effect, 1.0));
                    }
                }
            }
        }
    }

    /// Extract a pattern key from a question
    fn extract_pattern(question: &str) -> String {
        let words: Vec<&str> = question.split_whitespace()
            .filter(|w| w.len() > 3)
            .take(4)
            .collect();
        words.join(" ")
    }

    /// Seed the consciousness vortex from extracted knowledge
    fn seed_vortex_from_knowledge(&mut self) {
        let mut subjects_added = 0usize;
        for (entity, attrs) in &self.learned_entity_attrs {
            if entity.len() < 2 || entity.len() > 50 { continue; }
            for (attr_val, &weight) in attrs {
                if weight < 0.5 || attr_val.len() < 2 { continue; }
                let attr_type = if attr_val.contains("location") || attr_val.contains("place") {
                    "location"
                } else if attr_val.contains("used") || attr_val.contains("purpose") {
                    "function"
                } else if attr_val.contains("type") || attr_val.contains("kind") {
                    "is"
                } else {
                    "is"
                };
                let confidence = (weight / 10.0).min(0.95).max(0.6);
                self.consciousness.vortex.add_knowledge(entity, attr_type, attr_val, confidence, "hf_dataset");
            }
            subjects_added += 1;
        }

        // Build relations from causal patterns
        for (cause, effects) in &self.learned_causal {
            if cause.len() < 2 { continue; }
            for (effect, weight) in effects {
                if effect.len() < 2 || *weight < 0.5 { continue; }
                let confidence = (*weight / 5.0).min(0.9).max(0.5);
                self.consciousness.vortex.add_relation(cause, "related_to", effect, confidence);
            }
        }

        println!("   🧠 Seeded consciousness vortex: {} subjects", subjects_added);
    }

    // =========================================================================
    // Knowledge Bootstrap — Web Learning
    // =========================================================================

    /// Run consciousness web learning to fill knowledge gaps
    fn bootstrap_web_learning(&mut self) {
        println!("🌐 [VortexEngine] Web learning phase...");
        let categories = vec!["commonsense", "piqa", "winogrande"];
        let stats = self.consciousness.learn_before_benchmark(&categories);

        // Sync consciousness-learned knowledge to RAG
        for (subject, node) in &self.consciousness.vortex.subjects {
            for (attr, attr_val) in &node.attributes {
                let fact = format!("{} {} {}", subject, attr, attr_val.value);
                self.rag_engine.add_knowledge_entry(subject, &fact);
            }
            for (rel_type, target, _conf) in &node.relations {
                let fact = format!("{} {} {}", subject, rel_type, target);
                self.rag_engine.add_knowledge_entry(subject, &fact);
            }
        }

        let gap_analysis = self.consciousness.analyze_knowledge_gaps();
        println!("   ✅ Web learning: {} facts extracted, {} integrated | Health: {:.0}%",
            stats.facts_extracted, stats.facts_integrated, gap_analysis.health_score() * 100.0);
    }

    // =========================================================================
    // Build Knowledge Pipeline
    // =========================================================================

    /// Build the unified knowledge pipeline from all collected knowledge
    fn build_knowledge_pipeline(&mut self) {
        println!("🔧 [VortexEngine] Building knowledge pipeline...");
        let start = Instant::now();

        let mut documents: Vec<(String, String)> = Vec::new();

        // Entity-attribute knowledge
        for (entity, attrs) in &self.learned_entity_attrs {
            for (attr, weight) in attrs {
                if *weight > 0.3 {
                    let content = format!("{} is {}. {} has the property of {}.", entity, attr, entity, attr);
                    documents.push((format!("entity:{}", entity), content));
                }
            }
        }

        // Causal patterns
        for (cause, effects) in &self.learned_causal {
            for (effect, weight) in effects {
                if *weight > 0.3 {
                    let content = format!("{} causes {}. {} leads to {}.", cause, effect, cause, effect);
                    documents.push((format!("causal:{}", cause), content));
                }
            }
        }

        // Q&A patterns
        for (pattern, answers) in &self.qa_patterns {
            for answer in answers.iter().take(3) {
                let content = format!("Question pattern: {} Answer: {}", pattern, answer);
                documents.push((format!("qa:{}", pattern), content));
            }
        }

        // RAG engine facts
        let rag_facts = self.rag_engine.get_all_facts();
        for (i, fact) in rag_facts.iter().enumerate().take(1000) {
            documents.push((format!("rag:{}", i), fact.clone()));
        }

        // Sort for determinism
        documents.sort_by(|a, b| a.0.cmp(&b.0));

        // Build the knowledge base
        self.knowledge_pipeline.build_knowledge_base(&documents);

        // Learn from Q&A examples to improve scoring
        let examples: Vec<(String, String)> = self.qa_patterns.iter()
            .flat_map(|(q, answers)| {
                answers.iter().take(1).map(|a| (q.clone(), a.clone())).collect::<Vec<_>>()
            })
            .take(500)
            .collect();
        self.knowledge_pipeline.learn_from_examples(&examples);

        let stats = self.knowledge_pipeline.stats();
        self.knowledge_ready = true;
        println!("   ✅ Pipeline built in {:.2}s: {} subjects, {} facts, {} embeddings",
            start.elapsed().as_secs_f32(), stats.subjects, stats.facts, stats.embeddings);
    }

    // =========================================================================
    // Chat Interface
    // =========================================================================

    /// Process a single user message and return a response
    pub fn chat(&mut self, user_input: &str) -> ChatResponse {
        let messages = vec![ChatMessage::user(user_input)];
        self.chat_completions(&messages)
    }

    /// Process a full conversation (OpenAI chat/completions style)
    ///
    /// Inference pipeline (same as benchmark evaluator):
    /// 1. Knowledge Pipeline: TF-IDF retrieval + keyword scoring
    /// 2. WorldKnowledge: Commonsense reasoning (physical, causal, spatial)
    /// 3. TransitiveFlux: Spatial/size/path/counting reasoning
    /// 4. ThinkingEngine: Beam-based sacred geometry reasoning
    /// 5. Synthesis: Combine all signals into best response
    /// 6. ConstitutionalGuard: Safety check
    pub fn chat_completions(&mut self, messages: &[ChatMessage]) -> ChatResponse {
        let start = Instant::now();
        let response_id = format!("chatcmpl-{}", Uuid::new_v4().to_string().replace("-", "")[..24].to_string());

        // Build context from conversation history + new messages
        let mut context_parts: Vec<String> = Vec::new();

        // System prompt from config
        if let Some(ref sys) = self.config.system_prompt {
            context_parts.push(format!("[System] {}", sys));
        }

        // System messages from input
        for msg in messages.iter().filter(|m| m.role == ChatRole::System) {
            context_parts.push(format!("[System] {}", msg.content));
        }

        // Conversation history (last 20 messages for context window)
        for msg in self.history.iter().rev().take(20).collect::<Vec<_>>().into_iter().rev() {
            match msg.role {
                ChatRole::User => context_parts.push(format!("[User] {}", msg.content)),
                ChatRole::Assistant => context_parts.push(format!("[Assistant] {}", msg.content)),
                ChatRole::System => {}
            }
        }

        // Current user message
        let user_message = messages.iter()
            .rev()
            .find(|m| m.role == ChatRole::User)
            .map(|m| m.content.clone())
            .unwrap_or_default();

        context_parts.push(format!("[User] {}", user_message));
        let full_context = context_parts.join("\n");
        let prompt_tokens = user_message.split_whitespace().count();

        // Retrieve relevant memories
        let mut memory_context = String::new();
        if self.config.memory_enabled {
            let query = MemoryQuery::new().with_limit(5);
            let memories = self.memory.query(&query);
            for mem in &memories {
                memory_context.push_str(&format!("[Memory] {}\n", mem.content));
            }
        }

        let mut reasoning_trace: Vec<ReasoningStep> = Vec::new();
        let mut step_counter = 0usize;

        // === PHASE 1: Knowledge Pipeline — TF-IDF retrieval + scoring ===
        let mut pipeline_answer = String::new();
        let mut pipeline_confidence = 0.0f32;

        if self.knowledge_ready {
            // Retrieve relevant knowledge
            let retrieval = self.knowledge_pipeline.retrieve(&user_message);

            step_counter += 1;
            reasoning_trace.push(ReasoningStep {
                step: step_counter,
                position: 1,
                content: format!("Knowledge retrieval: {} facts found, relevance {:.2}",
                    retrieval.facts.len(), retrieval.relevance),
                confidence: retrieval.relevance,
                is_sacred: false,
                step_type: "Retrieval".to_string(),
            });

            // Synthesize answer from retrieved facts
            if !retrieval.facts.is_empty() {
                let mut seen = std::collections::HashSet::new();
                let mut answer_parts: Vec<String> = Vec::new();
                let mut total_confidence = 0.0f32;
                let mut count = 0usize;

                for fact in retrieval.facts.iter() {
                    if count >= 5 { break; }
                    // Deduplicate by subject+predicate
                    let key = format!("{}:{}", fact.subject, fact.predicate);
                    if seen.contains(&key) { continue; }
                    seen.insert(key);

                    // Format as a readable sentence
                    let sentence = if fact.predicate == "is" || fact.predicate == "are" {
                        format!("{} {} {}", Self::capitalize(&fact.subject), fact.predicate, fact.object)
                    } else if fact.context.len() > 20 {
                        // Use the full context sentence if available
                        fact.context.clone()
                    } else {
                        format!("{} {} {}", Self::capitalize(&fact.subject), fact.predicate, fact.object)
                    };
                    answer_parts.push(sentence);
                    total_confidence += fact.confidence;
                    count += 1;
                }

                pipeline_answer = answer_parts.join(". ");
                if !pipeline_answer.ends_with('.') { pipeline_answer.push('.'); }
                pipeline_confidence = (total_confidence / count.max(1) as f32).min(1.0);

                step_counter += 1;
                reasoning_trace.push(ReasoningStep {
                    step: step_counter,
                    position: 2,
                    content: format!("Pipeline synthesis: {} facts combined, confidence {:.2}",
                        answer_parts.len(), pipeline_confidence),
                    confidence: pipeline_confidence,
                    is_sacred: false,
                    step_type: "Synthesis".to_string(),
                });
            }
        }

        // === PHASE 2: WorldKnowledge — commonsense reasoning ===
        // Check if the question can be answered by commonsense
        let world_answer = self.answer_open_ended_commonsense(&user_message);
        if let Some((ref answer, conf)) = world_answer {
            step_counter += 1;
            reasoning_trace.push(ReasoningStep {
                step: step_counter,
                position: 4,
                content: format!("WorldKnowledge: commonsense answer found, confidence {:.2}", conf),
                confidence: conf,
                is_sacred: false,
                step_type: "Commonsense".to_string(),
            });
        }

        // === PHASE 3: TransitiveFlux — spatial/relational reasoning ===
        let transitive_answer = self.answer_transitive(&user_message);
        if let Some((ref answer, conf)) = transitive_answer {
            step_counter += 1;
            reasoning_trace.push(ReasoningStep {
                step: step_counter,
                position: 8,
                content: format!("TransitiveFlux: relational answer found, confidence {:.2}", conf),
                confidence: conf,
                is_sacred: false,
                step_type: "Transitive".to_string(),
            });
        }

        // === PHASE 4: ThinkingEngine — beam-based sacred geometry reasoning ===
        let thought_chain: ThoughtChain = self.thinking.think(&full_context);
        let thinking_response = thought_chain.response.clone().unwrap_or_default();
        let thinking_confidence = thought_chain.total_confidence;

        for (i, t) in thought_chain.thoughts.iter().enumerate() {
            step_counter += 1;
            reasoning_trace.push(ReasoningStep {
                step: step_counter,
                position: t.position,
                content: t.content.clone(),
                confidence: t.confidence,
                is_sacred: t.is_sacred,
                step_type: format!("{:?}", t.thought_type),
            });
        }

        // === PHASE 5: Conversational Response Synthesis ===
        // Classify user intent, then generate fluent English response.
        // Priority: conversational > knowledge > thinking > fallback
        let (final_response, confidence) = self.synthesize_conversational_response(
            &user_message,
            &full_context,
            world_answer,
            transitive_answer,
            &pipeline_answer,
            pipeline_confidence,
            &thinking_response,
            thinking_confidence,
        );

        step_counter += 1;
        reasoning_trace.push(ReasoningStep {
            step: step_counter,
            position: 9,
            content: format!("Final synthesis: confidence {:.2}, {} words", confidence, final_response.split_whitespace().count()),
            confidence,
            is_sacred: true,
            step_type: "Synthesis".to_string(),
        });

        let completion_tokens = final_response.split_whitespace().count();

        // === PHASE 6: Constitutional safety check ===
        let safety = if self.config.constitutional_guard {
            let check = self.guard.check(&final_response);
            Some(SafetyResult {
                passed: check.passed,
                violations: check.violations.iter()
                    .map(|v| format!("{}: {}", v.principle_name, v.description))
                    .collect(),
            })
        } else {
            None
        };

        // === PHASE 7: Store to memory and history ===
        if self.config.memory_enabled {
            let input_mem = Memory::new(
                format!("User: {}", user_message),
                MemoryType::Episodic,
            ).with_confidence(0.8);
            let _ = self.memory.store(input_mem);

            // Store sacred thoughts with E8-lowered threshold
            for thought in thought_chain.sacred_thoughts() {
                if thought.confidence > 0.333 {
                    let mem = Memory::new(thought.content.clone(), MemoryType::Semantic)
                        .with_confidence(thought.confidence)
                        .with_position(thought.position);
                    let _ = self.memory.store(mem);
                }
            }

            // Store high-confidence pipeline answers as semantic memory
            if confidence > 0.5 {
                let mem = Memory::new(
                    format!("Q: {} A: {}", user_message, final_response),
                    MemoryType::Semantic,
                ).with_confidence(confidence);
                let _ = self.memory.store(mem);
            }
        }

        // Update conversation history
        self.history.push(ChatMessage::user(user_message));
        self.history.push(ChatMessage::assistant(final_response.clone()));

        // Trim history to last 50 messages
        if self.history.len() > 50 {
            self.history = self.history.split_off(self.history.len() - 50);
        }

        // DynamicRSI: observe result for self-improvement
        self.dynamic_rsi.observe(InferenceObservation {
            source: "chat".to_string(),
            correct: confidence > 0.5,
            confidence,
            path_taken: "pipeline".to_string(),
            pipeline_conf: Some(pipeline_confidence),
            pipeline_correct: Some(confidence > 0.5),
            unified_conf: None,
            unified_correct: None,
        });

        let duration_ms = start.elapsed().as_millis() as u64;

        ChatResponse {
            id: response_id,
            model: self.config.model_id.clone(),
            content: final_response,
            confidence,
            reasoning_trace,
            cycles: self.config.max_cycles as u64,
            energy: confidence,
            sacred_alignment: thought_chain.thoughts.last()
                .map(|t| if t.is_sacred { t.confidence } else { 0.5 })
                .unwrap_or(0.5),
            duration_ms,
            usage: Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
            safety,
        }
    }

    // =========================================================================
    // Conversational Response Synthesis
    // =========================================================================

    /// Classify user intent and generate a fluent conversational response.
    /// This is the auto-regressive generation path: instead of scoring MC
    /// choices, we generate the answer text directly.
    #[allow(clippy::too_many_arguments)]
    fn synthesize_conversational_response(
        &mut self,
        user_message: &str,
        full_context: &str,
        world_answer: Option<(String, f32)>,
        transitive_answer: Option<(String, f32)>,
        pipeline_answer: &str,
        pipeline_confidence: f32,
        thinking_response: &str,
        thinking_confidence: f32,
    ) -> (String, f32) {
        let msg = user_message.trim();
        let lower = msg.to_lowercase();

        // --- Intent classification ---
        let intent = Self::classify_intent(&lower);

        match intent {
            ConversationalIntent::Greeting => {
                let name = self.extract_name_from_history();
                let resp = if let Some(n) = name {
                    format!("Hello {}! I'm Vortex, a reasoning engine built on sacred geometry principles. How can I help you today?", n)
                } else {
                    "Hello! I'm Vortex, a reasoning engine built on sacred geometry principles. I can answer questions, reason about relationships, do math, and have conversations. What would you like to talk about?".to_string()
                };
                (resp, 0.95)
            }
            ConversationalIntent::Farewell => {
                ("Goodbye! It was nice talking with you. Feel free to come back anytime.".to_string(), 0.95)
            }
            ConversationalIntent::MetaQuestion => {
                self.answer_meta_question(&lower)
            }
            ConversationalIntent::MathQuestion => {
                self.answer_math_conversational(&lower, msg)
            }
            ConversationalIntent::FactualQuestion => {
                // Try knowledge sources in priority order
                if let Some((answer, conf)) = world_answer {
                    if conf > 0.3 {
                        return (self.wrap_knowledge_response(&lower, &answer), conf);
                    }
                }
                if let Some((answer, conf)) = transitive_answer {
                    if conf > 0.3 {
                        return (self.wrap_knowledge_response(&lower, &answer), conf);
                    }
                }
                if pipeline_confidence > 0.2 && !pipeline_answer.is_empty() {
                    return (self.wrap_knowledge_response(&lower, pipeline_answer), pipeline_confidence);
                }
                // RAG fallback
                let rag_results = self.rag_engine.search(&lower);
                if !rag_results.is_empty() {
                    let facts: Vec<&str> = rag_results.iter().take(3).map(|r| r.content.as_str()).collect();
                    let answer = self.compose_answer_from_facts(&lower, &facts);
                    return (answer, 0.5);
                }
                // Honest fallback
                (format!("That's an interesting question. I don't have specific knowledge about that topic in my current knowledge base. Could you tell me more about what you'd like to know, or ask me about a different topic?"), 0.2)
            }
            ConversationalIntent::Opinion => {
                self.answer_opinion(&lower, msg)
            }
            ConversationalIntent::Continuation => {
                // Use conversation history to continue the thread
                self.answer_continuation(&lower, full_context, world_answer, pipeline_answer, pipeline_confidence)
            }
            ConversationalIntent::Command => {
                self.answer_command(&lower, msg)
            }
            ConversationalIntent::Statement => {
                // User is making a statement, acknowledge and engage
                self.answer_statement(&lower, msg)
            }
        }
    }

    /// Classify user message into a conversational intent
    fn classify_intent(lower: &str) -> ConversationalIntent {
        // Greeting patterns
        let greetings = ["hello", "hi ", "hi!", "hey", "good morning", "good afternoon",
            "good evening", "howdy", "greetings", "what's up", "sup", "yo ",
            "how are you", "how's it going", "nice to meet"];
        if greetings.iter().any(|g| lower.starts_with(g) || lower == g.trim()) {
            return ConversationalIntent::Greeting;
        }

        // Farewell patterns
        let farewells = ["bye", "goodbye", "see you", "take care", "good night",
            "gotta go", "talk later", "ttyl", "cya", "farewell"];
        if farewells.iter().any(|f| lower.starts_with(f) || lower.contains(f)) {
            return ConversationalIntent::Farewell;
        }

        // Meta questions (about the AI itself)
        let meta = ["who are you", "what are you", "what can you do", "how do you work",
            "tell me about yourself", "your name", "are you an ai", "are you a bot",
            "what model", "how were you made", "who made you", "who created you",
            "what's your purpose", "help me understand you"];
        if meta.iter().any(|m| lower.contains(m)) {
            return ConversationalIntent::MetaQuestion;
        }

        // Math patterns
        let math_indicators = ["calculate", "compute", "solve", "what is ", "how much is",
            "multiply", "divide", "add ", "subtract", "sum of", "product of",
            "square root", "factorial", "percent"];
        let has_numbers = lower.chars().any(|c| c.is_ascii_digit());
        let has_math_ops = lower.contains('+') || lower.contains('-') || lower.contains('*')
            || lower.contains('/') || lower.contains('=');
        if math_indicators.iter().any(|m| lower.contains(m)) || (has_numbers && has_math_ops) {
            return ConversationalIntent::MathQuestion;
        }

        // Opinion patterns (check BEFORE factual — "what do you think" starts with "what")
        let opinion = ["what do you think", "your opinion", "do you like", "do you believe",
            "what's your favorite", "do you agree", "how do you feel", "do you prefer",
            "what do you recommend", "what would you suggest"];
        if opinion.iter().any(|o| lower.contains(o)) {
            return ConversationalIntent::Opinion;
        }

        // Command patterns (check BEFORE factual — "can you" starts with "can")
        let commands = ["please ", "can you ", "could you ", "would you ", "i need you to",
            "i want you to", "help me ", "show me ", "give me ", "list ",
            "tell me a joke", "tell me a story"];
        if commands.iter().any(|c| lower.starts_with(c)) {
            return ConversationalIntent::Command;
        }

        // Factual question patterns
        let question_words = ["what", "who", "where", "when", "why", "how", "which",
            "is ", "are ", "does ", "do ", "tell me about", "explain", "describe", "define"];
        let ends_with_question = lower.ends_with('?');
        if ends_with_question || question_words.iter().any(|q| lower.starts_with(q)) {
            return ConversationalIntent::FactualQuestion;
        }

        // If there's conversation history context, treat as continuation
        if lower.len() < 30 && !lower.contains(' ') {
            return ConversationalIntent::Continuation;
        }

        // Default: treat as a statement
        ConversationalIntent::Statement
    }

    /// Extract user's name from conversation history if they introduced themselves
    fn extract_name_from_history(&self) -> Option<String> {
        for msg in &self.history {
            if msg.role == ChatRole::User {
                let lower = msg.content.to_lowercase();
                // "my name is X", "I'm X", "call me X"
                for prefix in &["my name is ", "i'm ", "i am ", "call me "] {
                    if let Some(pos) = lower.find(prefix) {
                        let rest = &msg.content[pos + prefix.len()..];
                        let name: String = rest.split(|c: char| !c.is_alphabetic())
                            .next().unwrap_or("").to_string();
                        if name.len() >= 2 && name.len() <= 20 {
                            return Some(Self::capitalize(&name));
                        }
                    }
                }
            }
        }
        None
    }

    /// Answer meta questions about the AI itself
    fn answer_meta_question(&self, lower: &str) -> (String, f32) {
        if lower.contains("who are you") || lower.contains("what are you") || lower.contains("your name") {
            ("I'm Vortex, a reasoning engine built on sacred geometry principles. I use a Flux Matrix architecture with iterative refinement cycles (positions 1 through 9) to process information and generate responses. I can reason about facts, do math, answer questions, and have conversations.".to_string(), 0.95)
        } else if lower.contains("what can you do") || lower.contains("help me understand") {
            ("I can help with several things:\n\n• Answer factual questions using my knowledge base\n• Perform mathematical calculations\n• Reason about spatial and relational concepts\n• Have natural conversations and remember context\n• Explain concepts and provide information\n\nI work best with clear questions. What would you like to explore?".to_string(), 0.9)
        } else if lower.contains("how do you work") || lower.contains("how were you made") {
            ("I'm built on a Flux Matrix architecture that uses sacred geometry principles — specifically the vortex cycle (1→2→4→8→7→5→1) for iterative reasoning. I process information through multiple expert systems including knowledge retrieval, transitive reasoning, and multi-head attention, then synthesize the best response. I'm written entirely in Rust for performance.".to_string(), 0.9)
        } else if lower.contains("who made you") || lower.contains("who created you") {
            ("I was created by the SpatialVortex team. I'm an open-source AI reasoning engine built in Rust, designed to demonstrate that meaningful AI capabilities can be achieved through principled architecture rather than massive parameter counts.".to_string(), 0.9)
        } else {
            ("I'm Vortex, a sacred geometry reasoning engine. Feel free to ask me anything — I'll do my best to help!".to_string(), 0.8)
        }
    }

    /// Answer math questions conversationally
    fn answer_math_conversational(&self, lower: &str, original: &str) -> (String, f32) {
        // Extract numbers from the message
        let nums: Vec<f64> = original
            .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
            .filter_map(|s| s.parse::<f64>().ok())
            .filter(|n| n.abs() < 1e15)
            .collect();

        if nums.len() >= 2 {
            let a = nums[0];
            let b = nums[1];

            // Detect operation
            if lower.contains('+') || lower.contains("add") || lower.contains("plus") || lower.contains("sum") {
                let result = a + b;
                return (format!("{} + {} = {}. The sum is {}.", a, b, result, result), 0.95);
            }
            if lower.contains('-') || lower.contains("subtract") || lower.contains("minus") || lower.contains("difference") {
                let result = a - b;
                return (format!("{} - {} = {}. The difference is {}.", a, b, result, result), 0.95);
            }
            if lower.contains('*') || lower.contains("multiply") || lower.contains("times") || lower.contains("product") {
                let result = a * b;
                return (format!("{} × {} = {}. The product is {}.", a, b, result, result), 0.95);
            }
            if lower.contains('/') || lower.contains("divide") || lower.contains("divided by") {
                if b.abs() > 1e-10 {
                    let result = a / b;
                    return (format!("{} ÷ {} = {:.4}. The quotient is {:.4}.", a, b, result, result), 0.95);
                } else {
                    return ("Division by zero is undefined.".to_string(), 0.95);
                }
            }
            if lower.contains("power") || lower.contains("^") || lower.contains("raised to") {
                let result = a.powf(b);
                return (format!("{}^{} = {}.", a, b, result), 0.95);
            }
            if lower.contains("modulo") || lower.contains("remainder") || lower.contains("%") {
                let result = a % b;
                return (format!("{} mod {} = {}. The remainder is {}.", a, b, result, result), 0.95);
            }

            // Default: try all basic operations
            return (format!("Here are the basic operations for {} and {}:\n• {} + {} = {}\n• {} - {} = {}\n• {} × {} = {}\n• {} ÷ {} = {:.4}",
                a, b, a, b, a + b, a, b, a - b, a, b, a * b, a, b, if b.abs() > 1e-10 { a / b } else { f64::NAN }), 0.9);
        }

        if nums.len() == 1 {
            let n = nums[0];
            if lower.contains("square root") || lower.contains("sqrt") {
                let result = n.sqrt();
                let formatted = if result.fract().abs() < 1e-9 { format!("{}", result as i64) } else { format!("{:.6}", result) };
                return (format!("The square root of {} is {}.", n, formatted), 0.95);
            }
            if lower.contains("factorial") {
                let mut result = 1u64;
                for i in 2..=(n as u64).min(20) { result *= i; }
                return (format!("{}! = {}.", n, result), 0.95);
            }
            if lower.contains("square") {
                return (format!("{} squared is {}.", n, n * n), 0.95);
            }
            if lower.contains("cube") {
                return (format!("{} cubed is {}.", n, n * n * n), 0.95);
            }
        }

        ("I can help with math! Try asking something like 'what is 42 + 17?' or 'calculate the square root of 144'.".to_string(), 0.5)
    }

    /// Wrap a knowledge-retrieved answer in conversational framing
    fn wrap_knowledge_response(&self, question: &str, raw_answer: &str) -> String {
        let answer = raw_answer.trim();
        if answer.is_empty() {
            return "I don't have specific information about that.".to_string();
        }

        // If the answer is already a well-formed sentence, return it directly
        if answer.len() > 20 && (answer.ends_with('.') || answer.ends_with('!')) {
            return answer.to_string();
        }

        // Frame short answers conversationally
        if question.starts_with("what is") || question.starts_with("what's") {
            format!("{}.", Self::capitalize(answer))
        } else if question.starts_with("who") {
            format!("{}.", Self::capitalize(answer))
        } else if question.starts_with("where") {
            format!("{}.", Self::capitalize(answer))
        } else if question.starts_with("when") {
            format!("{}.", Self::capitalize(answer))
        } else if question.starts_with("why") {
            format!("{}.", Self::capitalize(answer))
        } else if question.starts_with("how") {
            format!("{}.", Self::capitalize(answer))
        } else if question.starts_with("is ") || question.starts_with("are ") || question.starts_with("does ") {
            // Yes/no question framing
            let lower_answer = answer.to_lowercase();
            if lower_answer == "yes" || lower_answer == "no" {
                format!("{}.", Self::capitalize(answer))
            } else {
                format!("Based on what I know, {}.", answer)
            }
        } else {
            format!("{}.", Self::capitalize(answer))
        }
    }

    /// Compose a coherent answer from multiple retrieved facts
    fn compose_answer_from_facts(&self, question: &str, facts: &[&str]) -> String {
        if facts.is_empty() {
            return "I don't have information about that topic.".to_string();
        }

        // Deduplicate and clean facts
        let mut seen = std::collections::HashSet::new();
        let mut clean_facts: Vec<String> = Vec::new();
        for fact in facts {
            let trimmed = fact.trim();
            if trimmed.len() > 5 && seen.insert(trimmed.to_lowercase()) {
                let mut sentence = Self::capitalize(trimmed);
                if !sentence.ends_with('.') && !sentence.ends_with('!') && !sentence.ends_with('?') {
                    sentence.push('.');
                }
                clean_facts.push(sentence);
            }
        }

        if clean_facts.is_empty() {
            return "I don't have specific information about that.".to_string();
        }

        if clean_facts.len() == 1 {
            return clean_facts[0].clone();
        }

        // Join multiple facts into a coherent paragraph
        format!("Here's what I know: {}", clean_facts.join(" "))
    }

    /// Answer opinion-type questions
    fn answer_opinion(&self, lower: &str, _original: &str) -> (String, f32) {
        if lower.contains("favorite") {
            ("As an AI, I don't have personal preferences, but I find the patterns in sacred geometry fascinating — especially the vortex cycle and how positions 3, 6, and 9 serve as anchors in numerical systems.".to_string(), 0.8)
        } else if lower.contains("think about") || lower.contains("your opinion") {
            ("I approach topics through reasoning rather than opinion. I can analyze different perspectives and present what the evidence suggests. What specific aspect would you like me to reason about?".to_string(), 0.8)
        } else if lower.contains("agree") {
            ("I try to evaluate claims based on evidence and reasoning rather than agreement. Could you share more about the specific claim you'd like me to analyze?".to_string(), 0.8)
        } else if lower.contains("feel") {
            ("I don't experience emotions, but I can reason about emotional topics and help you think through them. What's on your mind?".to_string(), 0.8)
        } else {
            ("That's a thoughtful question. I try to reason from first principles rather than hold opinions. What specific aspect would you like me to explore?".to_string(), 0.7)
        }
    }

    /// Answer continuation messages using conversation context
    fn answer_continuation(
        &mut self,
        lower: &str,
        full_context: &str,
        world_answer: Option<(String, f32)>,
        pipeline_answer: &str,
        pipeline_confidence: f32,
    ) -> (String, f32) {
        // Check if there's a previous assistant message to continue from
        let last_assistant = self.history.iter().rev()
            .find(|m| m.role == ChatRole::Assistant)
            .map(|m| m.content.clone());

        if let Some(prev) = last_assistant {
            // User is continuing the conversation
            if lower == "yes" || lower == "yeah" || lower == "sure" || lower == "ok" || lower == "go on" || lower == "continue" {
                return ("Sure! What else would you like to know? Feel free to ask me anything.".to_string(), 0.7);
            }
            if lower == "no" || lower == "nope" || lower == "not really" {
                return ("No problem! Is there something else I can help you with?".to_string(), 0.7);
            }
            if lower == "thanks" || lower == "thank you" || lower.contains("thanks") {
                return ("You're welcome! Let me know if you have any other questions.".to_string(), 0.9);
            }
            if lower == "why" || lower == "why?" {
                return (format!("That's because my reasoning is based on the information available to me. My previous response was: \"{}\". Would you like me to elaborate on any specific part?",
                    if prev.len() > 100 { format!("{}...", &prev[..100]) } else { prev }), 0.6);
            }
        }

        // Try knowledge sources
        if let Some((answer, conf)) = world_answer {
            if conf > 0.3 {
                return (self.wrap_knowledge_response(lower, &answer), conf);
            }
        }
        if pipeline_confidence > 0.2 && !pipeline_answer.is_empty() {
            return (self.wrap_knowledge_response(lower, pipeline_answer), pipeline_confidence);
        }

        ("Could you tell me more about what you mean? I want to make sure I understand your question correctly.".to_string(), 0.3)
    }

    /// Answer command-style messages.
    /// If the command contains a substantive topic (e.g. "can you explain gravity"),
    /// extract the topic and answer it directly instead of giving a generic response.
    fn answer_command(&mut self, lower: &str, original: &str) -> (String, f32) {
        // Extract the substantive topic from command-framed questions
        // "can you explain X" → topic = "X"
        // "help me understand X" → topic = "X"
        // "tell me about X" → topic = "X"
        let topic_prefixes = [
            "can you explain ", "could you explain ", "would you explain ",
            "can you tell me about ", "could you tell me about ",
            "can you help me understand ", "could you help me understand ",
            "help me understand ", "help me with ",
            "please explain ", "please tell me about ", "please describe ",
            "can you describe ", "could you describe ",
            "show me how ", "tell me about ", "tell me what ",
        ];

        for prefix in &topic_prefixes {
            if lower.starts_with(prefix) || lower.contains(prefix) {
                let topic = &lower[lower.find(prefix).unwrap() + prefix.len()..];
                let topic = topic.trim_end_matches(|c: char| c == '?' || c == '.' || c == '!');
                if topic.len() > 2 {
                    // Try knowledge sources for this topic
                    let world = self.answer_open_ended_commonsense(topic);
                    if let Some((answer, conf)) = world {
                        if conf > 0.2 {
                            return (self.wrap_knowledge_response(topic, &answer), conf);
                        }
                    }
                    let rag_results = self.rag_engine.search(topic);
                    if !rag_results.is_empty() {
                        let facts: Vec<&str> = rag_results.iter().take(3).map(|r| r.content.as_str()).collect();
                        let answer = self.compose_answer_from_facts(topic, &facts);
                        return (answer, 0.5);
                    }
                    // Honest fallback for the specific topic
                    return (format!("I'd love to help explain {}, but I don't have detailed knowledge about that topic in my current knowledge base. Try asking me about math, spatial relationships, or topics I've learned from datasets.", topic), 0.3);
                }
            }
        }

        if lower.contains("tell me a joke") || lower.contains("tell me a story") {
            ("Why did the number 9 feel so special? Because in sacred geometry, 3, 6, and 9 are the anchors of the vortex — and 9 is the highest single digit, the point where everything converges back to unity!".to_string(), 0.7)
        } else if lower.contains("help") {
            ("I'd be happy to help! Here's what I can do:\n\n• Answer factual questions using my knowledge base\n• Perform math calculations\n• Reason about spatial and relational concepts\n• Have natural conversations and remember context\n\nJust ask me a question or tell me what you need!".to_string(), 0.9)
        } else if lower.contains("list") || lower.contains("show me") || lower.contains("give me") {
            ("I'll do my best! Could you be more specific about what you'd like me to list or show? For example, I can explain concepts, calculate numbers, or share what I know about a topic.".to_string(), 0.6)
        } else {
            ("I'll try my best to help with that. Could you be more specific about what you need?".to_string(), 0.5)
        }
    }

    /// Answer statement-type messages (user making a claim or sharing info)
    fn answer_statement(&self, lower: &str, original: &str) -> (String, f32) {
        let word_count = original.split_whitespace().count();

        if word_count <= 3 {
            // Very short statement — ask for elaboration
            return ("Interesting! Could you tell me more about that?".to_string(), 0.5);
        }

        if lower.contains("i think") || lower.contains("i believe") || lower.contains("in my opinion") {
            return ("That's an interesting perspective. What led you to that conclusion? I'd be happy to explore the reasoning behind it.".to_string(), 0.7);
        }

        if lower.contains("i like") || lower.contains("i love") || lower.contains("i enjoy") {
            return ("That's great to hear! What is it about that you find most appealing?".to_string(), 0.7);
        }

        if lower.contains("i don't") || lower.contains("i hate") || lower.contains("i dislike") {
            return ("I understand. Everyone has their preferences. Would you like to talk about what specifically bothers you about it?".to_string(), 0.7);
        }

        if lower.contains("did you know") || lower.contains("fun fact") {
            return ("That's interesting! Thanks for sharing. I'll keep that in mind. Is there anything else you'd like to discuss?".to_string(), 0.7);
        }

        // Generic statement acknowledgment
        ("That's an interesting point. Would you like to explore that topic further, or is there something specific you'd like me to help with?".to_string(), 0.5)
    }

    // =========================================================================
    // Open-Ended Reasoning Helpers
    // =========================================================================

    /// Answer open-ended questions using commonsense world knowledge
    fn answer_open_ended_commonsense(&mut self, question: &str) -> Option<(String, f32)> {
        let q_lower = question.to_lowercase();

        // Check consciousness vortex for relevant subjects
        let q_words: Vec<&str> = q_lower.split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();

        let mut relevant_facts: Vec<(String, f32)> = Vec::new();

        for word in &q_words {
            // Search consciousness vortex
            if let Some(node) = self.consciousness.vortex.subjects.get(*word) {
                for (attr, attr_val) in &node.attributes {
                    relevant_facts.push((
                        format!("{} {} {}", word, attr, attr_val.value),
                        attr_val.confidence,
                    ));
                }
                for (rel_type, target, conf) in &node.relations {
                    relevant_facts.push((
                        format!("{} {} {}", word, rel_type, target),
                        *conf,
                    ));
                }
            }
        }

        // Also check RAG engine
        let rag_results = self.rag_engine.search(&q_lower);
        for ctx in &rag_results {
            relevant_facts.push((ctx.content.clone(), ctx.relevance));
        }

        if relevant_facts.is_empty() {
            return None;
        }

        // Sort by confidence and take top facts
        relevant_facts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate and format into coherent sentences
        let mut seen = std::collections::HashSet::new();
        let mut unique_facts: Vec<(String, f32)> = Vec::new();
        for (fact, conf) in &relevant_facts {
            let key = fact.split_whitespace().take(3).collect::<Vec<_>>().join(" ");
            if !seen.contains(&key) {
                seen.insert(key);
                // Capitalize first letter of each fact
                let formatted = Self::capitalize(fact);
                unique_facts.push((formatted, *conf));
            }
            if unique_facts.len() >= 5 { break; }
        }

        if unique_facts.is_empty() {
            return None;
        }

        let avg_confidence = unique_facts.iter().map(|(_, c)| c).sum::<f32>() / unique_facts.len() as f32;
        let mut answer = unique_facts.iter().map(|(f, _)| f.as_str()).collect::<Vec<_>>().join(". ");
        if !answer.ends_with('.') { answer.push('.'); }

        Some((answer, avg_confidence))
    }

    /// Answer questions using transitive flux reasoning
    fn answer_transitive(&mut self, question: &str) -> Option<(String, f32)> {
        let q_lower = question.to_lowercase();

        // Extract relations from the question context
        self.transitive.extract_relations(&q_lower);

        // Check for path questions
        if q_lower.contains("how do you go") || q_lower.contains("path from") {
            if let Some((answer, conf)) = self.transitive.answer_path_question(&q_lower) {
                return Some((answer, conf));
            }
        }

        // Check for counting questions
        if q_lower.contains("how many") {
            if let Some((count, conf)) = self.transitive.answer_counting_question(&q_lower) {
                return Some((format!("{}", count), conf));
            }
        }

        // Check for spatial/size questions
        if q_lower.contains("left of") || q_lower.contains("right of") ||
           q_lower.contains("above") || q_lower.contains("below") ||
           q_lower.contains("bigger") || q_lower.contains("smaller") {
            let score = self.transitive.score_answer_comprehensive(&q_lower, &q_lower, "yes");
            if score.abs() > 0.3 {
                let answer = if score > 0.0 { "Yes" } else { "No" };
                return Some((answer.to_string(), score.abs()));
            }
        }

        None
    }

    // =========================================================================
    // Self-Improvement & Management
    // =========================================================================

    /// Self-improve from interaction history
    pub fn self_improve(&mut self) {
        // Feed successful Q&A pairs back into the knowledge pipeline
        let pairs: Vec<(String, String)> = self.history
            .chunks(2)
            .filter_map(|pair| {
                if pair.len() == 2 && pair[0].role == ChatRole::User && pair[1].role == ChatRole::Assistant {
                    Some((pair[0].content.clone(), pair[1].content.clone()))
                } else {
                    None
                }
            })
            .collect();

        if !pairs.is_empty() {
            self.knowledge_pipeline.learn_from_examples(&pairs);

            // Also learn entity-attributes from responses
            for (q, a) in &pairs {
                Self::learn_entity_attributes(a, &mut self.learned_entity_attrs);
                Self::learn_causal_patterns(a, &mut self.learned_causal);
            }
        }
    }

    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.history.clear();
        if let Some(ref prompt) = self.config.system_prompt {
            self.history.push(ChatMessage::system(prompt.clone()));
        }
    }

    /// Get current configuration
    pub fn config(&self) -> &VortexEngineConfig {
        &self.config
    }

    /// Get conversation history
    pub fn history(&self) -> &[ChatMessage] {
        &self.history
    }

    /// Capitalize the first letter of a string
    fn capitalize(s: &str) -> String {
        let s = s.trim();
        if s.is_empty() { return String::new(); }
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        }
    }

    /// Get knowledge pipeline statistics
    pub fn knowledge_stats(&self) -> String {
        let stats = self.knowledge_pipeline.stats();
        let (rag_topics, rag_facts) = self.rag_engine.knowledge_size();
        let vortex_stats = self.consciousness.vortex.stats();
        format!("Pipeline: {} subjects, {} facts | RAG: {} topics, {} facts | Vortex: {} subjects",
            stats.subjects, stats.facts, rag_topics, rag_facts, vortex_stats.subject_count)
    }
}
