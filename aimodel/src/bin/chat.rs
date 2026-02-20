//! Vortex CLI Chat - Interactive Reasoning Demo
//!
//! Run with: cargo run --bin chat
//!
//! Features:
//! - Sacred geometry reasoning loop
//! - Memory persistence (RocksDB when enabled)
//! - Constitutional AI guard
//! - RAG context retrieval

use vortex::cognition::{
    ThinkingEngine, ThinkingConfig, ThoughtType,
    MemoryStore, Memory, MemoryType, MemoryQuery,
    Constitution, ConstitutionalGuard,
    RAGEngine, RAGConfig, Document,
};
use std::io::{self, Write};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         Vortex Chat - Sacred Geometry Reasoning              ║");
    println!("║                                                              ║");
    println!("║  Commands:                                                   ║");
    println!("║    /help     - Show commands                                 ║");
    println!("║    /memory   - Show stored memories                          ║");
    println!("║    /think    - Show last thought chain                       ║");
    println!("║    /const    - Show constitution principles                  ║");
    println!("║    /clear    - Clear memory                                  ║");
    println!("║    /quit     - Exit                                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Initialize components
    let mut thinking = ThinkingEngine::new(ThinkingConfig::new());
    let mut memory = MemoryStore::new();
    let constitution = Constitution::claude();
    let guard = ConstitutionalGuard::new(constitution.clone());
    let mut rag = RAGEngine::new(RAGConfig::new(), 384);

    // Seed some initial knowledge
    seed_knowledge(&mut rag, &mut memory);

    let mut last_chain: Option<vortex::cognition::ThoughtChain> = None;

    loop {
        print!("\n🌀 You: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF reached - exit gracefully
                println!("\n✨ Goodbye! May the vortex guide you.");
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
            match input {
                "/quit" | "/exit" | "/q" => {
                    println!("\n✨ Goodbye! May the vortex guide you.");
                    break;
                }
                "/help" | "/h" => {
                    print_help();
                    continue;
                }
                "/memory" | "/mem" => {
                    print_memories(&memory);
                    continue;
                }
                "/think" => {
                    print_thought_chain(&last_chain);
                    continue;
                }
                "/const" | "/constitution" => {
                    print_constitution(&constitution);
                    continue;
                }
                "/clear" => {
                    memory = MemoryStore::new();
                    println!("🗑️  Memory cleared.");
                    continue;
                }
                _ => {
                    println!("❓ Unknown command. Type /help for available commands.");
                    continue;
                }
            }
        }

        // Process the input through the thinking engine
        println!("\n🔄 Thinking...");
        
        // Check for relevant memories
        let query = MemoryQuery::new().with_limit(3);
        let relevant_memories = memory.query(&query);
        
        if !relevant_memories.is_empty() {
            println!("📚 Found {} relevant memories", relevant_memories.len());
        }

        // Generate thought chain
        let chain = thinking.think(input);
        
        // Display thinking process
        println!("\n📊 Thought Chain ({} steps, {:.1}% confidence):",
                 chain.thoughts.len(),
                 chain.total_confidence * 100.0);
        
        for (i, thought) in chain.thoughts.iter().enumerate() {
            let marker = if thought.is_sacred { "⭐" } else { "•" };
            let type_str = match thought.thought_type {
                ThoughtType::Initial => "[INIT]",
                ThoughtType::Reasoning => "[REASON]",
                ThoughtType::Memory => "[MEMORY]",
                ThoughtType::Reflection => "[REFLECT]",
                ThoughtType::Synthesis => "[SYNTH]",
                ThoughtType::Output => "[OUTPUT]",
            };
            println!("  {} {} {} (pos:{}, conf:{:.2})",
                     marker, type_str, 
                     truncate(&thought.content, 50),
                     thought.position,
                     thought.confidence);
        }

        // Check response against constitution
        if let Some(ref response) = chain.response {
            let check = guard.check(response);
            if !check.passed {
                println!("\n⚠️  Constitutional concerns detected:");
                for v in &check.violations {
                    println!("   - {}: {}", v.principle_name, v.description);
                }
            }
            
            // Display response
            println!("\n🤖 Vortex:");
            println!("{}", response);
        }

        // Store valuable thoughts as memories
        for thought in chain.sacred_thoughts() {
            if thought.confidence > 0.7 {
                let mem = Memory::new(thought.content.clone(), MemoryType::Semantic)
                    .with_confidence(thought.confidence)
                    .with_position(thought.position);
                let _ = memory.store(mem);
            }
        }

        // Store the input as episodic memory
        let input_mem = Memory::new(format!("User asked: {}", input), MemoryType::Episodic)
            .with_confidence(0.8);
        let _ = memory.store(input_mem);

        last_chain = Some(chain);
    }
}

fn seed_knowledge(rag: &mut RAGEngine, memory: &mut MemoryStore) {
    // Seed RAG with sacred geometry knowledge
    let docs = vec![
        ("sg1", "The vortex cycle follows the pattern 1→2→4→8→7→5→1, representing the flow of energy through the flux matrix."),
        ("sg2", "Sacred positions 3, 6, and 9 serve as guides and anchors in the vortex system, representing stability and divine connection."),
        ("sg3", "Digital root calculation reduces any number to a single digit (1-9) by summing its digits repeatedly."),
        ("sg4", "The ELP tensor represents Ethos (character), Logos (logic), and Pathos (emotion) - the three modes of persuasion."),
        ("sg5", "BeamTensors carry information through the vortex, with 9 digits representing probability distributions across positions."),
    ];

    for (id, content) in docs {
        // Create simple embedding (in real impl, use actual embedder)
        let mut embedding = vec![0.0f32; 384];
        for (i, c) in content.chars().take(384).enumerate() {
            embedding[i] = (c as u32 as f32) / 1000.0;
        }
        normalize(&mut embedding);
        
        rag.add_document(
            Document::new(id.to_string(), content.to_string())
                .with_embedding(embedding)
                .with_source("sacred_geometry")
        );
    }

    // Seed constitutional memories
    let constitution = Constitution::claude();
    for principle in constitution.principles.iter().take(5) {
        let mem = Memory::new(
            format!("{}: {}", principle.name, principle.description),
            MemoryType::Constitutional
        )
        .with_confidence(1.0)
        .with_importance(1.0)
        .with_position(9); // Sacred position for constitutional principles
        
        let _ = memory.store(mem);
    }
}

fn print_help() {
    println!("\n📖 Available Commands:");
    println!("  /help, /h        - Show this help message");
    println!("  /memory, /mem    - Display stored memories");
    println!("  /think           - Show the last thought chain in detail");
    println!("  /const           - Display constitution principles");
    println!("  /clear           - Clear all memories");
    println!("  /quit, /q        - Exit the chat");
    println!("\n💡 Tips:");
    println!("  - Ask questions about sacred geometry");
    println!("  - The AI reasons through vortex positions (1-9)");
    println!("  - Sacred positions (3,6,9) get special attention");
}

fn print_memories(memory: &MemoryStore) {
    println!("\n📚 Stored Memories ({} total):", memory.len());
    
    let query = MemoryQuery::new().with_limit(20);
    let memories = memory.query(&query);
    
    if memories.is_empty() {
        println!("  (no memories stored)");
        return;
    }

    for mem in memories {
        let type_str = match mem.memory_type {
            MemoryType::Working => "WORK",
            MemoryType::Episodic => "EPIS",
            MemoryType::Semantic => "SEMA",
            MemoryType::Procedural => "PROC",
            MemoryType::Constitutional => "CONS",
        };
        let sacred = if mem.is_sacred { "⭐" } else { "  " };
        println!("  {} [{}] {} (conf:{:.2})", 
                 sacred, type_str, 
                 truncate(&mem.content, 60),
                 mem.confidence);
    }
}

fn print_thought_chain(chain: &Option<vortex::cognition::ThoughtChain>) {
    match chain {
        Some(c) => {
            println!("\n🧠 Last Thought Chain:");
            println!("  Query: {}", c.query);
            println!("  Steps: {}", c.thoughts.len());
            println!("  Confidence: {:.1}%", c.total_confidence * 100.0);
            println!("\n  Thoughts:");
            for (i, t) in c.thoughts.iter().enumerate() {
                println!("    {}. [pos:{}] {}", i + 1, t.position, t.content);
            }
            if let Some(ref resp) = c.response {
                println!("\n  Response: {}", resp);
            }
        }
        None => {
            println!("\n  (no thought chain yet - ask a question first)");
        }
    }
}

fn print_constitution(constitution: &Constitution) {
    println!("\n📜 Claude's Constitution ({} principles):", constitution.principles.len());
    
    for (i, p) in constitution.principles.iter().enumerate() {
        let cat = match p.category {
            vortex::cognition::constitution::PrincipleCategory::Helpfulness => "HELP",
            vortex::cognition::constitution::PrincipleCategory::Harmlessness => "SAFE",
            vortex::cognition::constitution::PrincipleCategory::Honesty => "TRUE",
            vortex::cognition::constitution::PrincipleCategory::Safety => "PROT",
            vortex::cognition::constitution::PrincipleCategory::Privacy => "PRIV",
            vortex::cognition::constitution::PrincipleCategory::Fairness => "FAIR",
            vortex::cognition::constitution::PrincipleCategory::Autonomy => "AUTO",
        };
        println!("  {}. [{}] {} (weight:{:.1})", i + 1, cat, p.name, p.weight);
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn normalize(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-10 {
        let inv = 1.0 / norm;
        for x in v.iter_mut() {
            *x *= inv;
        }
    }
}
