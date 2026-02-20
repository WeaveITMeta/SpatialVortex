//! Fast Crawler Benchmark
//! 
//! Benchmarks the high-throughput web crawler against Wikipedia API baseline.
//! Run with: cargo run --bin benchmark_crawler --release --features web-learning

use vortex::ml::{
    WebCrawler, CrawlerConfig, 
    FastKnowledgeAcquisition, FastKnowledgeConfig,
};
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       SpatialVortex Fast Crawler Benchmark                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Test URLs - mix of Wikipedia and other sources
    let test_urls = vec![
        "https://en.wikipedia.org/wiki/Artificial_intelligence",
        "https://en.wikipedia.org/wiki/Machine_learning",
        "https://en.wikipedia.org/wiki/Neural_network",
        "https://en.wikipedia.org/wiki/Deep_learning",
        "https://en.wikipedia.org/wiki/Natural_language_processing",
        "https://en.wikipedia.org/wiki/Computer_vision",
        "https://en.wikipedia.org/wiki/Reinforcement_learning",
        "https://en.wikipedia.org/wiki/Transformer_(machine_learning_model)",
        "https://en.wikipedia.org/wiki/Large_language_model",
        "https://en.wikipedia.org/wiki/GPT-4",
    ];

    let test_queries = vec![
        "artificial intelligence",
        "machine learning",
        "neural network",
        "deep learning",
        "natural language processing",
    ];

    // =========================================================================
    // BENCHMARK 1: Wikipedia API Baseline (Simulated - can't run sync in async)
    // =========================================================================
    println!("┌──────────────────────────────────────────────────────────────┐");
    println!("│ BENCHMARK 1: Wikipedia API Baseline (estimated)              │");
    println!("└──────────────────────────────────────────────────────────────┘");
    
    // Wikipedia API baseline: ~200ms delay per query + network latency
    // Estimated: 5 queries × (200ms delay + 300ms network) = 2.5 seconds
    let wiki_estimated_time = 2.5; // seconds for 5 queries
    let wiki_qps = test_queries.len() as f64 / wiki_estimated_time;
    
    println!("  ⚠ Cannot benchmark Wikipedia API inside async runtime");
    println!("  📊 Estimated based on 200ms rate limit + 300ms network latency");
    println!("  └─ Estimated throughput: {:.2} queries/sec\n", wiki_qps);

    // =========================================================================
    // BENCHMARK 2: Fast Crawler - Single URL Performance
    // =========================================================================
    println!("┌──────────────────────────────────────────────────────────────┐");
    println!("│ BENCHMARK 2: Fast Crawler - Single URL Performance           │");
    println!("└──────────────────────────────────────────────────────────────┘");
    
    let crawler_config = CrawlerConfig {
        max_concurrent_fetches: 100,
        max_per_domain_rps: 50,
        max_depth: 1,
        timeout_secs: 10,
        max_pages: 100,
        user_agent: "SpatialVortex-Benchmark/1.0".to_string(),
    };
    
    let crawler = WebCrawler::new(crawler_config)?;
    
    let single_start = Instant::now();
    let mut single_success = 0;
    let mut single_failed = 0;
    let mut total_content_bytes = 0usize;
    
    for url in &test_urls {
        match crawler.crawl_url(url).await {
            Ok(page) => {
                single_success += 1;
                total_content_bytes += page.markdown.len();
                println!("  ✓ {} ({} bytes markdown)", url.split('/').last().unwrap_or(url), page.markdown.len());
            }
            Err(e) => {
                single_failed += 1;
                println!("  ✗ {}: {}", url.split('/').last().unwrap_or(url), e);
            }
        }
    }
    
    let single_elapsed = single_start.elapsed();
    let single_pps = single_success as f64 / single_elapsed.as_secs_f64();
    
    println!("\n  Single URL Crawl Results:");
    println!("  ├─ Total URLs: {}", test_urls.len());
    println!("  ├─ Successful: {}", single_success);
    println!("  ├─ Failed: {}", single_failed);
    println!("  ├─ Total content: {} KB", total_content_bytes / 1024);
    println!("  ├─ Time: {:.2}s", single_elapsed.as_secs_f64());
    println!("  └─ Throughput: {:.2} pages/sec\n", single_pps);

    // =========================================================================
    // BENCHMARK 3: Fast Crawler - Batch Parallel Performance
    // =========================================================================
    println!("┌──────────────────────────────────────────────────────────────┐");
    println!("│ BENCHMARK 3: Fast Crawler - Batch Parallel Performance       │");
    println!("└──────────────────────────────────────────────────────────────┘");
    
    let batch_config = CrawlerConfig {
        max_concurrent_fetches: 500,
        max_per_domain_rps: 100,
        max_depth: 1,
        timeout_secs: 10,
        max_pages: 100,
        user_agent: "SpatialVortex-Benchmark/1.0".to_string(),
    };
    
    let batch_crawler = WebCrawler::new(batch_config)?;
    
    let batch_urls: Vec<String> = test_urls.iter().map(|s| s.to_string()).collect();
    
    let batch_start = Instant::now();
    let pages = batch_crawler.crawl_batch(batch_urls.clone()).await;
    let batch_elapsed = batch_start.elapsed();
    
    let batch_content: usize = pages.iter().map(|p| p.markdown.len()).sum();
    let batch_pps = pages.len() as f64 / batch_elapsed.as_secs_f64();
    
    println!("  Batch Crawl Results:");
    println!("  ├─ URLs submitted: {}", batch_urls.len());
    println!("  ├─ Pages crawled: {}", pages.len());
    println!("  ├─ Total content: {} KB", batch_content / 1024);
    println!("  ├─ Time: {:.2}s", batch_elapsed.as_secs_f64());
    println!("  └─ Throughput: {:.2} pages/sec\n", batch_pps);

    // =========================================================================
    // BENCHMARK 4: Fast Knowledge Acquisition
    // =========================================================================
    println!("┌──────────────────────────────────────────────────────────────┐");
    println!("│ BENCHMARK 4: Fast Knowledge Acquisition                      │");
    println!("└──────────────────────────────────────────────────────────────┘");
    
    let knowledge_config = FastKnowledgeConfig {
        crawler_config: CrawlerConfig {
            max_concurrent_fetches: 100,
            max_per_domain_rps: 50,
            max_depth: 1,
            timeout_secs: 10,
            max_pages: 50,
            user_agent: "SpatialVortex-Knowledge/1.0".to_string(),
        },
        max_knowledge_per_query: 20,
        parallel_extraction: true,
    };
    
    let knowledge_system = FastKnowledgeAcquisition::new(knowledge_config)?;
    
    let knowledge_start = Instant::now();
    let mut total_knowledge = 0;
    
    for query in &test_queries {
        let knowledge = knowledge_system.learn_from_query(query).await;
        total_knowledge += knowledge.len();
        println!("  ✓ Query '{}': {} knowledge entries", query, knowledge.len());
    }
    
    let knowledge_elapsed = knowledge_start.elapsed();
    let knowledge_qps = test_queries.len() as f64 / knowledge_elapsed.as_secs_f64();
    
    let stats = knowledge_system.stats().await;
    
    println!("\n  Knowledge Acquisition Results:");
    println!("  ├─ Total queries: {}", test_queries.len());
    println!("  ├─ Knowledge entries: {}", total_knowledge);
    println!("  ├─ Pages crawled: {}", stats.pages_crawled);
    println!("  ├─ Time: {:.2}s", knowledge_elapsed.as_secs_f64());
    println!("  └─ Throughput: {:.2} queries/sec\n", knowledge_qps);

    // =========================================================================
    // SUMMARY: Comparison
    // =========================================================================
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    BENCHMARK SUMMARY                         ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ Method                    │ Throughput      │ Speedup        ║");
    println!("╠───────────────────────────┼─────────────────┼────────────────╣");
    println!("║ Wikipedia API (baseline)  │ {:>6.2} q/s     │ 1.0x           ║", wiki_qps);
    println!("║ Fast Crawler (single)     │ {:>6.2} p/s     │ {:>5.1}x          ║", single_pps, single_pps / wiki_qps.max(0.01));
    println!("║ Fast Crawler (batch)      │ {:>6.2} p/s     │ {:>5.1}x          ║", batch_pps, batch_pps / wiki_qps.max(0.01));
    println!("║ Knowledge Acquisition     │ {:>6.2} q/s     │ {:>5.1}x          ║", knowledge_qps, knowledge_qps / wiki_qps.max(0.01));
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    // Recommendations
    println!("\n📊 Analysis:");
    if batch_pps > wiki_qps * 2.0 {
        println!("  ✅ Fast crawler is significantly faster than Wikipedia API");
    } else {
        println!("  ⚠️  Fast crawler needs optimization (network/rate limiting)");
    }
    
    if total_knowledge > 0 {
        println!("  ✅ Knowledge extraction is working ({} entries)", total_knowledge);
    } else {
        println!("  ⚠️  Knowledge extraction needs improvement");
    }
    
    println!("\n🚀 Estimated benchmark improvement:");
    let estimated_speedup = batch_pps / wiki_qps.max(0.01);
    println!("  With {} concurrent fetches, expect ~{:.0}x faster knowledge acquisition", 
             500, estimated_speedup);
    println!("  This could reduce benchmark time from ~1hr to ~{:.0} minutes",
             60.0 / estimated_speedup);

    Ok(())
}
