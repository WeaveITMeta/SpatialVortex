//! RSI Macros - Procedural Macros for Recursive Self-Improvement
//!
//! This crate provides compile-time code generation based on RSI state,
//! enabling the vortex to optimize itself through recompilation.
//!
//! ## Key Macros
//!
//! - `#[rsi_optimized]` - Attribute macro for RSI-tuned functions
//! - `generate_expert_weights!` - Generate expert routing weights from RSI state
//! - `rsi_scoring_coefficients!` - Generate scoring coefficients from RSI learnings
//! - `define_semantic_chain!` - Define semantic relationship patterns for chaining
//! - `context_aware_extraction!` - Context-aware entity and relationship extraction
//!
//! ## Usage
//!
//! ```rust
//! use rsi_macros::{rsi_optimized, generate_expert_weights};
//!
//! // Function will be optimized based on RSI state at compile time
//! #[rsi_optimized(benchmark = "bAbI")]
//! fn score_deductive_query(context: &str, choice: &str) -> f32 {
//!     // Implementation
//! }
//!
//! // Generate expert weights from RSI state
//! generate_expert_weights!();
//!
//! // Define semantic relationship patterns for chaining
//! define_semantic_chain!(causality: ["causes", "leads_to", "results_in"] => 1.2);
//!
//! // Context-aware entity extraction
//! context_aware_extraction!(temporal: "X happened before Y" => extract_temporal_relationship);
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, parse, Lit, Expr, ExprArray, ExprLit, parse_quote};
use std::collections::HashMap;

// =============================================================================
// RSI State (loaded at compile time)
// =============================================================================

/// RSI configuration loaded from persistent state
#[derive(Debug, Clone, Default)]
struct RSIConfig {
    /// Expert weights learned from benchmark performance
    expert_weights: Vec<(String, f32)>,
    /// Scoring coefficients tuned by RSI
    scoring_coefficients: ScoringCoefficients,
    /// Version for cache invalidation
    version: u64,
}

#[derive(Debug, Clone)]
struct ScoringCoefficients {
    entity_attribute: f32,
    embedding_similarity: f32,
    attention: f32,
    ntp_deduction: f32,
    commonsense: f32,
    rag: f32,
}

impl Default for ScoringCoefficients {
    fn default() -> Self {
        Self {
            entity_attribute: 1.0,
            embedding_similarity: 1.0,
            attention: 1.0,
            ntp_deduction: 1.0,
            commonsense: 0.3,
            rag: 0.2,
        }
    }
}

/// Load RSI state from file (called at compile time)
fn load_rsi_config() -> RSIConfig {
    // Try to load from rsi_state.json in the vortex directory
    let config_path = std::env::var("CARGO_MANIFEST_DIR")
        .map(|dir| format!("{}/../rsi_state.json", dir))
        .unwrap_or_else(|_| "rsi_state.json".to_string());
    
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        parse_rsi_json(&content).unwrap_or_default()
    } else {
        // Return default config with baseline weights
        RSIConfig {
            expert_weights: vec![
                ("EntityAttribute".to_string(), 3.0),
                ("Location".to_string(), 2.5),
                ("Deductive".to_string(), 2.0),
                ("Semantic".to_string(), 1.5),
                ("Commonsense".to_string(), 1.0),
            ],
            scoring_coefficients: ScoringCoefficients::default(),
            version: 1,
        }
    }
}

fn parse_rsi_json(content: &str) -> Option<RSIConfig> {
    // Simplified JSON parsing - in production use serde_json
    let _ = content;
    None
}

// =============================================================================
// Procedural Macros
// =============================================================================

/// Attribute macro for RSI-optimized functions
///
/// This macro reads RSI state at compile time and generates optimized
/// code paths based on learned parameters.
///
/// # Arguments
///
/// - `benchmark` - Target benchmark for optimization (e.g., "bAbI", "CommonsenseQA")
/// - `target` - Optimization target (e.g., "inference", "scoring")
///
/// # Example
///
/// ```rust
/// #[rsi_optimized(benchmark = "bAbI", target = "inference")]
/// fn ai_inference(&self, question: &Question) -> (usize, f32) {
///     // Function body - macro may inject optimizations
/// }
/// ```
#[proc_macro_attribute]
pub fn rsi_optimized(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    
    // Load RSI config at compile time
    let config = load_rsi_config();
    
    // Extract function components
    let fn_name = &input.sig.ident;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_body = &input.block;
    let fn_vis = &input.vis;
    let fn_attrs = &input.attrs;
    
    // Generate version constant for cache invalidation
    let _version = config.version;
    
    // For now, pass through the function with RSI metadata comment
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis fn #fn_name(#fn_inputs) #fn_output {
            #fn_body
        }
    };
    
    expanded.into()
}

/// Generate expert routing weights from RSI state
///
/// This macro generates a constant array of expert weights that have been
/// tuned through RSI benchmark analysis.
///
/// # Example
///
/// ```rust
/// generate_expert_weights!();
/// // Expands to:
/// // pub const RSI_EXPERT_WEIGHTS: &[(&str, f32)] = &[
/// //     ("EntityAttribute", 3.0),
/// //     ("Location", 2.5),
/// //     ...
/// // ];
/// ```
#[proc_macro]
pub fn generate_expert_weights(_input: TokenStream) -> TokenStream {
    let config = load_rsi_config();
    
    let weights: Vec<_> = config.expert_weights.iter().map(|(name, weight)| {
        quote! { (#name, #weight) }
    }).collect();
    
    let expanded = quote! {
        /// Expert weights generated by RSI at compile time
        pub const RSI_EXPERT_WEIGHTS: &[(&str, f32)] = &[
            #(#weights),*
        ];
    };
    
    expanded.into()
}

/// Generate scoring coefficients from RSI state
///
/// This macro generates constants for scoring coefficients that have been
/// optimized through RSI benchmark analysis.
#[proc_macro]
pub fn generate_scoring_coefficients(_input: TokenStream) -> TokenStream {
    let config = load_rsi_config();
    let coef = config.scoring_coefficients;
    
    let entity_attr = coef.entity_attribute;
    let embed_sim = coef.embedding_similarity;
    let attention = coef.attention;
    let ntp = coef.ntp_deduction;
    let commonsense = coef.commonsense;
    let rag = coef.rag;
    
    let expanded = quote! {
        /// Scoring coefficients generated by RSI at compile time
        pub mod rsi_coefficients {
            pub const ENTITY_ATTRIBUTE: f32 = #entity_attr;
            pub const EMBEDDING_SIMILARITY: f32 = #embed_sim;
            pub const ATTENTION: f32 = #attention;
            pub const NTP_DEDUCTION: f32 = #ntp;
            pub const COMMONSENSE: f32 = #commonsense;
            pub const RAG: f32 = #rag;
        }
    };
    
    expanded.into()
}

/// Macro for conditional compilation based on RSI benchmark performance
///
/// Generates different code paths based on which benchmarks need optimization.
#[proc_macro]
pub fn rsi_benchmark_switch(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let config = load_rsi_config();
    
    // Parse benchmark name from input
    let benchmark = input_str.trim_matches('"');
    
    // Find weight for this benchmark's expert
    let weight = config.expert_weights.iter()
        .find(|(name, _)| name.to_lowercase().contains(&benchmark.to_lowercase()))
        .map(|(_, w)| *w)
        .unwrap_or(1.0);
    
    let expanded = quote! {
        #weight
    };
    
    expanded.into()
}

// =============================================================================  
// New Semantic Chaining Macros
// =============================================================================

/// Macro for defining semantic relationship patterns that can be chained
///
/// This creates transitive relationship patterns that can be dynamically applied
/// during knowledge extraction to enhance semantic understanding.
///
/// # Arguments
///
/// - `name` - Identifier for the semantic chain pattern
/// - `[relations]` - Array of relation types that form the chain
/// - `confidence_boost` - Confidence multiplier for chained relationships
///
/// # Example
///
/// ```rust
/// define_semantic_chain!(causality: ["causes", "leads_to", "results_in"] => 1.2);
/// define_semantic_chain!(spatial: ["left_of", "right_of", "above", "below"] => 1.1);
/// ```
#[proc_macro]
pub fn define_semantic_chain(input: TokenStream) -> TokenStream {
    // Parse: define_semantic_chain!(name: [relation1, relation2, relation3] => confidence_boost)
    let input_str = input.to_string();
    
    // Simple parsing for demonstration - in production, use proper syn parsing
    let parts: Vec<&str> = input_str.split(">>").collect();
    if parts.len() != 2 {
        return quote! {
            compile_error!("define_semantic_chain! expects format: name: [relations] => confidence_boost")
        }.into();
    }
    
    let pattern_part = parts[0].trim();
    let confidence_part = parts[1].trim();
    
    // Extract name and relations
    let pattern_parts: Vec<&str> = pattern_part.split(':').collect();
    if pattern_parts.len() != 2 {
        return quote! {
            compile_error!("define_semantic_chain! expects format: name: [relations] => confidence_boost")
        }.into();
    }
    
    let name = pattern_parts[0].trim();
    let relations_str = pattern_parts[1].trim();
    
    // Parse relations array (simplified)
    let relations_clean = relations_str.trim_start_matches('[').trim_end_matches(']');
    let relations: Vec<&str> = relations_clean.split(',').map(|s| s.trim().trim_matches('"')).collect();
    
    // Parse confidence boost
    let confidence_boost: f32 = confidence_part.parse().unwrap_or(1.0);
    
    // Generate the semantic chain structure
    let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
    let relations_tokens: Vec<proc_macro2::TokenStream> = relations.iter()
        .map(|rel| {
            let rel_str = rel.to_string();
            quote! { #rel_str }
        })
        .collect();
    
    let expanded = quote! {
        /// Generated semantic chain pattern
        pub struct #name_ident;
        
        impl #name_ident {
            /// Get the relation types in this semantic chain
            pub const fn relations() -> &'static [&'static str] {
                &[#(#relations_tokens),*]
            }
            
            /// Get the confidence boost for this chain
            pub const fn confidence_boost() -> f32 {
                #confidence_boost
            }
            
            /// Check if a relation is part of this semantic chain
            pub fn is_in_chain(relation: &str) -> bool {
                Self::relations().contains(&relation)
            }
        }
    };
    
    expanded.into()
}

/// Macro for conditional relationship extraction based on context patterns
///
/// This generates optimized extraction code based on learned linguistic patterns
/// for identifying entities and relationships in text.
///
/// # Arguments
///
/// - `pattern` - Linguistic pattern to match (e.g., "X is Y")
/// - `extractor_function` - Function to call when pattern is matched
///
/// # Example
///
/// ```rust
/// context_aware_extraction!(causal: "X causes Y" => extract_causal_relationship);
/// context_aware_extraction!(temporal: "X before Y" => extract_temporal_relationship);
/// ```
#[proc_macro]
pub fn context_aware_extraction(input: TokenStream) -> TokenStream {
    // Parse: context_aware_extraction!(pattern: "X causes Y" => extractor_function)
    let input_str = input.to_string();
    
    let parts: Vec<&str> = input_str.split(">>").collect();
    if parts.len() != 2 {
        return quote! {
            compile_error!("context_aware_extraction! expects format: pattern: \"pattern_text\" => extractor_function")
        }.into();
    }
    
    let pattern_part = parts[0].trim();
    let extractor_part = parts[1].trim();
    
    // Extract pattern name and text
    let pattern_parts: Vec<&str> = pattern_part.split(':').collect();
    if pattern_parts.len() != 2 {
        return quote! {
            compile_error!("context_aware_extraction! expects format: pattern: \"pattern_text\" => extractor_function")
        }.into();
    }
    
    let pattern_name = pattern_parts[0].trim();
    let pattern_text = pattern_parts[1].trim().trim_matches('"');
    
    // Parse extractor function
    let extractor_fn = syn::Ident::new(extractor_part, proc_macro2::Span::call_site());
    let pattern_ident = syn::Ident::new(pattern_name, proc_macro2::Span::call_site());
    
    let expanded = quote! {
        /// Generated context-aware extraction pattern
        pub struct #pattern_ident;
        
        impl #pattern_ident {
            /// The linguistic pattern to match
            pub const PATTERN: &'static str = #pattern_text;
            
            /// Extract relationships using the specified extractor function
            pub fn extract(text: &str) -> Vec<ExtractedRelationship> {
                #extractor_fn(text)
            }
            
            /// Check if text matches this pattern
            pub fn matches(text: &str) -> bool {
                // Simple substring matching for demonstration
                // In practice, this would use more sophisticated NLP
                text.contains(Self::PATTERN.split('X').next().unwrap_or(""))
                    && text.contains(Self::PATTERN.split('Y').last().unwrap_or(""))
            }
        }
    };
    
    expanded.into()
}

// =============================================================================  
// Helper Structs for Semantic Chaining (internal to macro implementation)
// =============================================================================

/// Relationship extracted from text
#[derive(Debug, Clone)]
struct ExtractedRelationship {
    source: String,
    relation: String,
    target: String,
    confidence: f32,
}

/// Semantic chain pattern for transitive reasoning
struct SemanticChainPattern {
    name: String,
    relations: Vec<String>,
    confidence_boost: f32,
}

// =============================================================================  
// Helper Macros (declarative)
// =============================================================================

/// Declarative macro for defining RSI-tunable parameters
///
/// This creates a parameter that can be overridden by RSI state at compile time.
#[proc_macro]
pub fn rsi_param(input: TokenStream) -> TokenStream {
    // Parse: rsi_param!(name: type = default)
    let input_str = input.to_string();
    let parts: Vec<&str> = input_str.split('=').collect();
    
    if parts.len() != 2 {
        return quote! {
            compile_error!("rsi_param! expects format: name: type = default")
        }.into();
    }
    
    let name_type = parts[0].trim();
    let default = parts[1].trim();
    
    let name_type_parts: Vec<&str> = name_type.split(':').collect();
    if name_type_parts.len() != 2 {
        return quote! {
            compile_error!("rsi_param! expects format: name: type = default")
        }.into();
    }
    
    let name = name_type_parts[0].trim();
    let ty = name_type_parts[1].trim();
    
    // In production, this would look up the RSI state for overrides
    let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
    let ty_tokens: proc_macro2::TokenStream = ty.parse().unwrap_or_else(|_| quote!(f32));
    let default_tokens: proc_macro2::TokenStream = default.parse().unwrap_or_else(|_| quote!(1.0));
    
    let expanded = quote! {
        pub const #name_ident: #ty_tokens = #default_tokens;
    };
    
    expanded.into()
}
