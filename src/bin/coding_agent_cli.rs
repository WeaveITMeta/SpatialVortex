//! Command-line interface for the SpatialVortex coding agent
//!
//! Usage:
//!   coding_agent_cli "Write a Python function to sort a list"
//!   coding_agent_cli "Solve x^2 + 3x + 2 = 0" --language rust --execute
//!   coding_agent_cli "Create a React button" --flux-position 6 --no-correct

use clap::Parser;
use spatial_vortex::agents::{CodingAgent, AgentConfig, LLMConfig, LLMBackend};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "coding_agent_cli")]
#[command(about = "SpatialVortex Multi-Language Coding Agent", long_about = None)]
#[command(version = "0.7.0")]
struct Args {
    /// Task description or coding request
    #[arg(value_name = "TASK")]
    task: String,
    
    /// Programming language (auto-detect if not provided)
    #[arg(short, long)]
    language: Option<String>,
    
    /// Execute the generated code in Docker sandbox
    #[arg(short, long, default_value = "false")]
    execute: bool,
    
    /// Enable self-correction loop (up to 3 attempts)
    #[arg(short = 'c', long, default_value = "true")]
    correct: bool,
    
    /// Disable self-correction
    #[arg(long)]
    no_correct: bool,
    
    /// Force specific flux position (3=Ethos, 6=Pathos, 9=Logos)
    #[arg(short = 'p', long)]
    flux_position: Option<u8>,
    
    /// LLM backend (ollama, openai, anthropic)
    #[arg(short = 'b', long, default_value = "ollama")]
    backend: String,
    
    /// Model name for LLM backend
    #[arg(short, long)]
    model: Option<String>,
    
    /// Ollama server URL
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_url: String,
    
    /// OpenAI API key (or set OPENAI_API_KEY env var)
    #[arg(long, env = "OPENAI_API_KEY")]
    openai_api_key: Option<String>,
    
    /// Anthropic API key (or set ANTHROPIC_API_KEY env var)
    #[arg(long, env = "ANTHROPIC_API_KEY")]
    anthropic_api_key: Option<String>,
    
    /// Temperature for LLM generation (0.0-1.0, default 0.2)
    #[arg(short, long, default_value = "0.2")]
    temperature: f32,
    
    /// Maximum tokens for LLM generation
    #[arg(long, default_value = "2048")]
    max_tokens: usize,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Output format (text, json)
    #[arg(short = 'f', long, default_value = "text")]
    output_format: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Banner
    if args.verbose {
        println!("🤖 SpatialVortex Coding Agent v0.7.0");
        println!("{}\n", "=".repeat(60));
    }
    
    // Configure agent
    let agent_config = AgentConfig {
        max_correction_attempts: if args.no_correct { 1 } else { 3 },
        enable_self_correction: args.correct && !args.no_correct,
        enable_flux_routing: args.flux_position.is_none(),
        use_enhanced_prompts: true,  // Use enhanced prompts by default
        memory_limit: "256m".to_string(),
        cpu_limit: "0.5".to_string(),
    };
    
    // Configure LLM backend
    let llm_backend = match args.backend.to_lowercase().as_str() {
        "ollama" => {
            let model = args.model.unwrap_or_else(|| "codellama:13b".to_string());
            if args.verbose {
                println!("📡 Using Ollama: {}", model);
                println!("   URL: {}", args.ollama_url);
            }
            LLMBackend::Ollama {
                url: args.ollama_url,
                model,
            }
        }
        "openai" => {
            let api_key = args.openai_api_key
                .ok_or_else(|| anyhow::anyhow!("OpenAI API key required. Use --openai-api-key or set OPENAI_API_KEY"))?;
            let model = args.model.unwrap_or_else(|| "gpt-4".to_string());
            if args.verbose {
                println!("📡 Using OpenAI: {}", model);
            }
            LLMBackend::OpenAI { api_key, model }
        }
        "anthropic" => {
            let api_key = args.anthropic_api_key
                .ok_or_else(|| anyhow::anyhow!("Anthropic API key required. Use --anthropic-api-key or set ANTHROPIC_API_KEY"))?;
            let model = args.model.unwrap_or_else(|| "claude-3-sonnet-20240229".to_string());
            if args.verbose {
                println!("📡 Using Anthropic: {}", model);
            }
            LLMBackend::Anthropic { api_key, model }
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown backend: {}. Use: ollama, openai, or anthropic", args.backend));
        }
    };
    
    let llm_config = LLMConfig {
        backend: llm_backend,
        temperature: args.temperature,
        max_tokens: args.max_tokens,
        ..Default::default()
    };
    
    // Create agent
    if args.verbose {
        println!("🔧 Initializing agent...\n");
    }
    
    let agent = CodingAgent::with_llm(agent_config, llm_config)?;
    
    // Execute task
    if args.verbose {
        println!("📝 Task: {}", args.task);
        if let Some(lang) = &args.language {
            println!("🔤 Language: {}", lang);
        } else {
            println!("🔤 Language: Auto-detect");
        }
        if let Some(pos) = args.flux_position {
            println!("🌀 Flux Position: {} (forced)", pos);
        } else {
            println!("🌀 Flux Routing: Enabled");
        }
        println!();
    }
    
    let start = Instant::now();
    
    let result = agent.execute_task(&args.task).await?;
    
    let elapsed = start.elapsed();
    
    // Output results
    match args.output_format.as_str() {
        "json" => {
            // JSON output
            let json = serde_json::json!({
                "success": result.execution.as_ref().map(|e| e.success).unwrap_or(true),
                "language": result.language.name(),
                "flux_position": result.flux_position,
                "attempts": result.attempts,
                "elapsed_ms": elapsed.as_millis(),
                "code": result.code,
                "stdout": result.execution.as_ref().map(|e| e.stdout.clone()),
                "stderr": result.execution.as_ref().map(|e| e.stderr.clone()),
                "exit_code": result.execution.as_ref().and_then(|e| e.exit_code),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            // Text output
            if args.verbose {
                println!("{}", "=".repeat(60));
                println!("✅ Task completed in {:.2?}", elapsed);
                println!("{}", "=".repeat(60));
            }
            
            println!("\n📊 Results:");
            println!("   Language: {}", result.language.name());
            println!("   Flux Position: {:?}", result.flux_position);
            println!("   Attempts: {}", result.attempts);
            println!("   Time: {:.2?}", elapsed);
            
            println!("\n📝 Generated Code:");
            println!("```{}", result.language.extension());
            println!("{}", result.code);
            println!("```");
            
            if let Some(exec) = &result.execution {
                println!("\n🔧 Execution:");
                if exec.success {
                    println!("   Status: ✅ Success");
                    if !exec.stdout.is_empty() {
                        println!("\n   Output:");
                        for line in exec.stdout.lines() {
                            println!("   {}", line);
                        }
                    }
                } else {
                    println!("   Status: ❌ Failed");
                    if !exec.stderr.is_empty() {
                        println!("\n   Error:");
                        for line in exec.stderr.lines() {
                            println!("   {}", line);
                        }
                    }
                }
                if let Some(code) = exec.exit_code {
                    println!("   Exit Code: {}", code);
                }
            }
            
            if args.verbose {
                println!("\n{}", "=".repeat(60));
            }
        }
    }
    
    Ok(())
}
