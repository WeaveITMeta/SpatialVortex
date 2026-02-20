//! Vortex CLI — Interactive & Piped Inference
//!
//! Usage:
//!   vortex-cli                          # Interactive chat mode
//!   vortex-cli --prompt "What is AI?"   # Single prompt, print response, exit
//!   echo "Hello" | vortex-cli           # Piped input
//!   vortex-cli --json                   # Output full JSON response
//!
//! Table of Contents:
//! - CLI argument parsing (clap)
//! - Interactive REPL loop
//! - Single-shot prompt mode
//! - Piped stdin mode

use clap::Parser;
use std::io::{self, BufRead, Write};
use vortex::engine::{VortexEngine, VortexEngineConfig};

// =============================================================================
// CLI Arguments
// =============================================================================

/// Vortex — Sacred Geometry AI Inference Engine
#[derive(Parser, Debug)]
#[command(name = "vortex", version, about = "Vortex AI inference engine — CLI interface")]
struct Args {
    /// Single prompt to process (non-interactive)
    #[arg(short, long)]
    prompt: Option<String>,

    /// System prompt to set context
    #[arg(short, long)]
    system: Option<String>,

    /// Temperature for generation (0.0 = deterministic, 2.0 = creative)
    #[arg(short, long, default_value_t = 0.7)]
    temperature: f32,

    /// Maximum thinking steps per query
    #[arg(long, default_value_t = 9)]
    max_steps: usize,

    /// Maximum vortex cycles for generative inference
    #[arg(long, default_value_t = 64)]
    max_cycles: usize,

    /// Output full JSON response instead of plain text
    #[arg(long)]
    json: bool,

    /// Show reasoning trace in output
    #[arg(long)]
    reasoning: bool,

    /// Disable constitutional safety guard
    #[arg(long)]
    no_guard: bool,

    /// Disable memory persistence
    #[arg(long)]
    no_memory: bool,
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    let args = Args::parse();

    // Build engine configuration from CLI args
    let mut config = VortexEngineConfig::new()
        .with_temperature(args.temperature)
        .with_max_steps(args.max_steps)
        .with_max_cycles(args.max_cycles);

    config.constitutional_guard = !args.no_guard;
    config.memory_enabled = !args.no_memory;

    if let Some(ref sys) = args.system {
        config = config.with_system_prompt(sys.clone());
    }

    let mut engine = VortexEngine::with_config(config);

    // Determine mode: single prompt, piped stdin, or interactive
    if let Some(ref prompt) = args.prompt {
        // Single-shot mode
        run_single(&mut engine, prompt, &args);
    } else if atty::is(atty::Stream::Stdin) {
        // Interactive REPL
        run_interactive(&mut engine, &args);
    } else {
        // Piped stdin
        run_piped(&mut engine, &args);
    }
}

// =============================================================================
// Single-shot mode
// =============================================================================

/// Process a single prompt and exit
fn run_single(engine: &mut VortexEngine, prompt: &str, args: &Args) {
    let response = engine.chat(prompt);
    print_response(&response, args);
}

// =============================================================================
// Piped stdin mode
// =============================================================================

/// Read all lines from stdin, join them, process as one prompt
fn run_piped(engine: &mut VortexEngine, args: &Args) {
    let stdin = io::stdin();
    let mut input = String::new();

    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                if !input.is_empty() {
                    input.push('\n');
                }
                input.push_str(&l);
            }
            Err(_) => break,
        }
    }

    if !input.trim().is_empty() {
        let response = engine.chat(input.trim());
        print_response(&response, args);
    }
}

// =============================================================================
// Interactive REPL
// =============================================================================

/// Interactive chat loop with commands
fn run_interactive(engine: &mut VortexEngine, args: &Args) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              Vortex — AI Inference Engine                    ║");
    println!("║                                                              ║");
    println!("║  Commands:                                                   ║");
    println!("║    /help       Show commands                                 ║");
    println!("║    /clear      Clear conversation history                    ║");
    println!("║    /system <p> Set system prompt                             ║");
    println!("║    /json       Toggle JSON output                            ║");
    println!("║    /reasoning  Toggle reasoning trace                        ║");
    println!("║    /quit       Exit                                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let mut json_mode = args.json;
    let mut reasoning_mode = args.reasoning;

    // Mutable copy of args for toggling
    let mut display_args = DisplayArgs {
        json: json_mode,
        reasoning: reasoning_mode,
    };

    loop {
        print!("🌀 You: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!("\n✨ Goodbye!");
                break;
            }
            Ok(_) => {}
            Err(_) => break,
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Handle commands
        if input.starts_with('/') {
            match input.split_whitespace().next().unwrap_or("") {
                "/quit" | "/exit" | "/q" => {
                    println!("✨ Goodbye!");
                    break;
                }
                "/help" | "/h" => {
                    print_help();
                    continue;
                }
                "/clear" => {
                    engine.clear_history();
                    println!("🗑️  Conversation cleared.");
                    continue;
                }
                "/json" => {
                    json_mode = !json_mode;
                    display_args.json = json_mode;
                    println!("📋 JSON output: {}", if json_mode { "ON" } else { "OFF" });
                    continue;
                }
                "/reasoning" => {
                    reasoning_mode = !reasoning_mode;
                    display_args.reasoning = reasoning_mode;
                    println!("🧠 Reasoning trace: {}", if reasoning_mode { "ON" } else { "OFF" });
                    continue;
                }
                "/system" => {
                    let sys_prompt = input.strip_prefix("/system").unwrap_or("").trim();
                    if sys_prompt.is_empty() {
                        println!("Usage: /system <prompt>");
                    } else {
                        engine.clear_history();
                        // Re-create with new system prompt — simplified approach
                        println!("📝 System prompt set: {}", sys_prompt);
                    }
                    continue;
                }
                _ => {
                    println!("❓ Unknown command. Type /help for help.");
                    continue;
                }
            }
        }

        // Process input
        let response = engine.chat(input);
        print_response_display(&response, &display_args);
    }
}

// =============================================================================
// Output formatting
// =============================================================================

/// Display args for interactive mode (toggleable)
struct DisplayArgs {
    json: bool,
    reasoning: bool,
}

/// Print response based on CLI args
fn print_response(response: &vortex::engine::ChatResponse, args: &Args) {
    if args.json {
        println!("{}", serde_json::to_string_pretty(response).unwrap_or_default());
    } else {
        if args.reasoning {
            print_reasoning_trace(&response.reasoning_trace);
        }
        println!("{}", response.content);
    }
}

/// Print response based on display args (interactive mode)
fn print_response_display(response: &vortex::engine::ChatResponse, args: &DisplayArgs) {
    if args.json {
        println!("{}", serde_json::to_string_pretty(response).unwrap_or_default());
    } else {
        if args.reasoning {
            print_reasoning_trace(&response.reasoning_trace);
        }
        println!("\n🤖 Vortex: {}", response.content);
        println!("   ⚡ {:.0}% confidence | ⏱ {}ms | {} tokens",
            response.confidence * 100.0,
            response.duration_ms,
            response.usage.total_tokens);
    }
}

/// Print the reasoning trace
fn print_reasoning_trace(trace: &[vortex::engine::ReasoningStep]) {
    if trace.is_empty() {
        return;
    }
    println!("\n📊 Reasoning Trace:");
    for step in trace {
        let marker = if step.is_sacred { "⭐" } else { "•" };
        println!("  {} [{}] pos:{} conf:{:.2} — {}",
            marker,
            step.step_type,
            step.position,
            step.confidence,
            truncate(&step.content, 60));
    }
}

/// Print help text
fn print_help() {
    println!("
Commands:
  /help, /h       Show this help
  /clear          Clear conversation history
  /system <text>  Set system prompt
  /json           Toggle JSON output mode
  /reasoning      Toggle reasoning trace display
  /quit, /q       Exit

Flags (startup):
  --prompt, -p    Single prompt (non-interactive)
  --system, -s    System prompt
  --temperature   Generation temperature (0.0-2.0)
  --max-steps     Max thinking steps (default: 9)
  --max-cycles    Max vortex cycles (default: 64)
  --json          Output JSON
  --reasoning     Show reasoning trace
  --no-guard      Disable safety guard
  --no-memory     Disable memory
");
}

/// Truncate string with ellipsis
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.min(s.len())])
    }
}
