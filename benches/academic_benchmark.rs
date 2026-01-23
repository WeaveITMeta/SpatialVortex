/// Academic Benchmarks for ParallelFusion v0.8.4
/// Tests the actual HTTP API endpoint (POST /api/v1/process)
/// Results saved to benchmarks/data/

use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use chrono::Utc;
use reqwest::Client;
use std::io::{self, Write};

#[derive(Debug, Serialize, Deserialize)]
struct CommonsenseQAItem {
    id: String,
    question: Question,
    answerKey: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Question {
    stem: String,
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    label: String,
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkResults {
    version: String,
    model: String,
    timestamp: String,
    dataset: String,
    total_questions: usize,
    correct: usize,
    accuracy_percent: f64,
    target_accuracy: String,
    status: String,
    samples: Vec<TestSample>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestSample {
    id: String,
    question: String,
    choices: Vec<String>,
    correct_answer: String,
    model_answer: String,
    confidence: f32,
    correct: bool,
}

#[derive(Debug, Serialize)]
struct UnifiedRequest {
    input: String,
}

#[derive(Debug, Deserialize)]
struct UnifiedResponse {
    result: String,
    confidence: f32,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> anyhow::Result<()> {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║   ParallelFusion v0.8.4 Academic Benchmark (Real AI)           ║");
    println!("║   Dataset: CommonsenseQA Dev Set                               ║");
    println!("║   Backend: Ollama (llama3.2) via ASI Orchestrator              ║");
    println!("║   Endpoint: POST http://localhost:7000/api/v1/process          ║");
    println!("║   Target: 97-99% accuracy                                      ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Check if API server is running
    let api_url = "http://localhost:7000/api/v1/process";
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(600))  // 10 minutes per request
        .build()?;
    
    println!("🔍 Checking API server at http://localhost:7000...");
    match client.get("http://localhost:7000/api/v1/health").send().await {
        Ok(_) => println!("✅ API server is running\n"),
        Err(_) => {
            eprintln!("❌ API server not running!");
            eprintln!("   Start it with: cargo run --release --bin api_server");
            return Ok(());
        }
    }

    // Load CommonsenseQA dataset
    let dataset_path = Path::new("benchmarks/data/commonsenseqa/dev.jsonl");
    if !dataset_path.exists() {
        eprintln!("❌ Dataset not found: {}", dataset_path.display());
        eprintln!("   Run: cd benchmarks && ./scripts/download_datasets.ps1");
        return Ok(());
    }

    let content = fs::read_to_string(dataset_path)?;
    let questions: Vec<CommonsenseQAItem> = content
        .lines()
        .take(50)  // Test on 50 questions for reasonable runtime
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    println!("📊 Testing on {} questions...\n", questions.len());

    let mut correct = 0;
    let mut samples = Vec::new();

    for (idx, item) in questions.iter().enumerate() {
        print!("  [{}/{}] Processing: {}... ", idx + 1, questions.len(), item.id);
        io::stdout().flush().unwrap();  // Force immediate output
        
        // Format question with choices
        let choices_text: Vec<String> = item.question.choices.iter()
            .map(|c| format!("{}) {}", c.label, c.text))
            .collect();
        
        let prompt = format!(
            "Question: {}\n\nChoices:\n{}\n\nAnswer with just the letter (A, B, C, D, or E):",
            item.question.stem,
            choices_text.join("\n")
        );

        // Make HTTP POST request to ParallelFusion API
        let request = UnifiedRequest { 
            input: prompt,
        };
        let start_time = std::time::Instant::now();
        
        let response = client
            .post(api_url)
            .json(&request)
            .send()
            .await?;
        
        // Check if response is OK
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            eprintln!("\n❌ API Error:");
            eprintln!("   Status: {}", status);
            eprintln!("   Body: {}", body);
            return Err(anyhow::anyhow!("API returned error: {}", status));
        }
        
        // Try to parse JSON response
        let body_text = response.text().await?;
        let result: UnifiedResponse = match serde_json::from_str(&body_text) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("\n❌ JSON Parse Error:");
                eprintln!("   Error: {}", e);
                eprintln!("   Response body: {}", body_text);
                return Err(anyhow::anyhow!("Failed to parse response as JSON"));
            }
        };
        let duration = start_time.elapsed();
        let model_answer = result.result.trim().to_uppercase();
        
        print!("[{:.1}s] ", duration.as_secs_f32());
        io::stdout().flush().unwrap();
        
        // Extract just the letter
        let model_letter = model_answer
            .chars()
            .find(|c| matches!(c, 'A'..='E'))
            .map(|c| c.to_string())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        let is_correct = model_letter == item.answerKey;
        if is_correct {
            correct += 1;
            println!("✅ ({:.0}% conf)", result.confidence * 100.0);
        } else {
            println!("❌ Expected: {}, Got: {} ({:.0}% conf)", 
                item.answerKey, model_letter, result.confidence * 100.0);
        }

        samples.push(TestSample {
            id: item.id.clone(),
            question: item.question.stem.clone(),
            choices: item.question.choices.iter().map(|c| c.text.clone()).collect(),
            correct_answer: item.answerKey.clone(),
            model_answer: model_letter,
            confidence: result.confidence,  // Already f32
            correct: is_correct,
        });
    }

    let accuracy = (correct as f64 / questions.len() as f64) * 100.0;
    let status = if accuracy >= 97.0 {
        "✅ PASSED (Target: 97-99%)".to_string()
    } else if accuracy >= 90.0 {
        "⚠️  GOOD (Target: 97-99%)".to_string()
    } else {
        "❌ NEEDS IMPROVEMENT (Target: 97-99%)".to_string()
    };

    let results = BenchmarkResults {
        version: "0.8.4".to_string(),
        model: "ParallelFusion Ensemble".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        dataset: "CommonsenseQA Dev".to_string(),
        total_questions: questions.len(),
        correct,
        accuracy_percent: accuracy,
        target_accuracy: "97-99%".to_string(),
        status: status.clone(),
        samples,
    };

    // Print summary
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                    BENCHMARK RESULTS                             ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!("\n📊 Results:");
    println!("   Total Questions: {}", questions.len());
    println!("   Correct: {}", correct);
    println!("   Accuracy: {:.2}%", accuracy);
    println!("   Target: 97-99%");
    println!("   Status: {}\n", status);

    // Save results
    let output_path = format!(
        "benchmarks/data/parallel_fusion_v0.8.4_academic_{}.json",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let json = serde_json::to_string_pretty(&results)?;
    fs::write(&output_path, json)?;
    println!("💾 Full results saved to: {}\n", output_path);

    Ok(())
}
