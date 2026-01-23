/// CLI tool for generating new subject modules
///
/// Usage: cargo run --bin subject_cli -- <subject_name>
/// Example: cargo run --bin subject_cli -- "Chemistry"
use spatial_vortex::{ai_integration::AIModelIntegration, subject_generator::SubjectGenerator};
use std::env;

#[tokio::main]
async fn main() {
    println!("\n=== SpatialVortex Subject Generator CLI ===\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: Subject name required\n");
        println!("Usage: {} <subject_name>", args[0]);
        println!("\nExamples:");
        println!("  {} \"Chemistry\"", args[0]);
        println!("  {} \"Biology\"", args[0]);
        println!("  {} \"Economics\"", args[0]);
        println!("  {} \"Psychology\"", args[0]);
        std::process::exit(1);
    }

    let subject_name = args[1..].join(" ");

    // Check for API key
    let api_key = env::var("GROK_API_KEY").ok();
    let endpoint = env::var("GROK_ENDPOINT")
        .ok()
        .or_else(|| Some("https://api.x.ai/v1/chat/completions".to_string()));

    if api_key.is_none() {
        eprintln!("\nWarning: GROK_API_KEY not set. AI-powered generation unavailable.");
        eprintln!("Set the environment variable to enable dynamic subject generation.\n");
        eprintln!("Example: export GROK_API_KEY=your_api_key_here\n");
        std::process::exit(1);
    }

    println!("Configuration:");
    println!(
        "  API Key: {}",
        if api_key.is_some() { "Set" } else { "Not set" }
    );
    println!("  Endpoint: {}", endpoint.as_ref().unwrap());
    println!("  Subject: {}\n", subject_name);

    // Initialize AI integration
    let ai_integration = AIModelIntegration::new(api_key, endpoint);

    // Create subject generator
    let generator = SubjectGenerator::new(ai_integration, None);

    // Generate the subject
    match generator.create_subject(&subject_name).await {
        Ok(()) => {
            println!(
                "\n[SUCCESS] Subject '{}' generated successfully!",
                subject_name
            );
            println!("\nNext steps:");
            println!(
                "  1. Review the generated file: src/subjects/{}.rs",
                subject_name.to_lowercase().replace(" ", "_")
            );
            println!("  2. Run: cargo fmt");
            println!("  3. Run: cargo check");
            println!("  4. Rebuild and test your application\n");
        }
        Err(e) => {
            eprintln!("\n[ERROR] Failed to generate subject: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("  - Verify your API key is correct");
            eprintln!("  - Check your internet connection");
            eprintln!("  - Ensure the subject name is valid");
            eprintln!("  - Check if the subject file already exists\n");
            std::process::exit(1);
        }
    }
}
