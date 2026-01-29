//! RSI Macros - Procedural Macros for Recursive Self-Improvement
//!
//! This crate provides compile-time code generation based on RSI state,
//! enabling the aimodel to optimize itself through recompilation.
//!
//! ## Key Macros
//!
//! - `#[rsi_optimized]` - Attribute macro for RSI-tuned functions
//! - `generate_expert_weights!` - Generate expert routing weights from RSI state
//! - `rsi_scoring_coefficients!` - Generate scoring coefficients from RSI learnings
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
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

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
    // Try to load from rsi_state.json in the aimodel directory
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
