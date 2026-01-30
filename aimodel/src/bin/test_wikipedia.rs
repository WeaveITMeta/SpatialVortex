use aimodel::ml::web_knowledge::{DuckDuckGoScraper, WebScraperConfig};

#[tokio::main]
async fn main() {
    let config = WebScraperConfig {
        timeout_secs: 30,
        max_results: 5,
        request_delay_ms: 0,
    };
    
    let mut scraper = DuckDuckGoScraper::new(config);
    
    println!("Testing Wikipedia API...");
    
    let test_queries = vec![
        "hamburger",
        "commonsense knowledge",
        "physical properties",
    ];
    
    for query in test_queries {
        println!("\n=== Testing query: {} ===", query);
        match scraper.search(query).await {
            Ok(results) => {
                println!("✓ Success! Got {} results", results.len());
                for (i, result) in results.iter().enumerate() {
                    println!("  [{}] {}", i + 1, result.title);
                    println!("      URL: {}", result.url);
                    println!("      Snippet: {}...", &result.snippet.chars().take(100).collect::<String>());
                }
            }
            Err(e) => {
                println!("✗ Error: {}", e);
            }
        }
    }
}
