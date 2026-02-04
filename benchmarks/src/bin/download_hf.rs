//! Rust downloader for HuggingFace-hosted benchmark datasets (no Python)
//! Usage:
//!   cargo run --bin download_hf -- [--dataset NAME] [--out DIR]
//!   cargo run --bin download_hf -- --list

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;

#[derive(Clone, Debug)]
struct Dataset {
    name: &'static str,
    url: &'static str,
    filename: &'static str,
}

const DATASETS: &[Dataset] = &[
    // Abstract reasoning
    Dataset {
        name: "truthfulqa",
        url: "https://huggingface.co/datasets/domenicrosati/TruthfulQA/resolve/main/TruthfulQA.csv",
        filename: "TruthfulQA.csv",
    },
    Dataset {
        name: "infinitebench",
        url: "https://huggingface.co/datasets/xinrongzhang2022/InfiniteBench/resolve/main/test.jsonl",
        filename: "infinitebench.jsonl",
    },
    Dataset {
        name: "gaia",
        url: "https://huggingface.co/datasets/gaia-benchmark/GAIA/resolve/main/gaia_test.jsonl",
        filename: "gaia.jsonl",
    },
    // Reasoning and commonsense
    Dataset {
        name: "bbh",
        url: "https://huggingface.co/datasets/suzgunmirac/BIG-Bench-Hard/resolve/main/bbh.json",
        filename: "bbh.json",
    },
    Dataset {
        name: "winogrande",
        url: "https://huggingface.co/datasets/allenai/winogrande/resolve/main/winogrande_1.1/train_xl.jsonl",
        filename: "winogrande_1.1.jsonl",
    },
    Dataset {
        name: "musr",
        url: "https://huggingface.co/datasets/TAUR-Lab/MuSR/resolve/main/musr.json",
        filename: "musr.json",
    },
    Dataset {
        name: "ifeval",
        url: "https://huggingface.co/datasets/google/IFEval/resolve/main/ifeval.jsonl",
        filename: "ifeval.jsonl",
    },
    Dataset {
        name: "drop",
        url: "https://huggingface.co/datasets/allenai/drop/resolve/main/drop_dataset_dev.json",
        filename: "drop_dataset_dev.json",
    },
    Dataset {
        name: "mbpp",
        url: "https://huggingface.co/datasets/google-research-datasets/mbpp/resolve/main/mbpp.jsonl",
        filename: "mbpp.jsonl",
    },
    Dataset {
        name: "aqua",
        url: "https://huggingface.co/datasets/google/AQuA-RAT/resolve/main/AQuA.json",
        filename: "AQuA.json",
    },
    Dataset {
        name: "hle",
        url: "https://huggingface.co/datasets/cais/hle/resolve/main/hle.csv",
        filename: "hle.csv",
    },
    // Coding
    Dataset {
        name: "swe_bench_verified",
        url: "https://huggingface.co/datasets/princeton-nlp/SWE-bench_Verified/resolve/main/data/test.jsonl",
        filename: "swe_verified.jsonl",
    },
    Dataset {
        name: "swe_bench_lite",
        url: "https://huggingface.co/datasets/princeton-nlp/SWE-bench_Lite/resolve/main/data/test-00000-of-00001.parquet",
        filename: "swe_lite.parquet",
    },
];

fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let mut selected: Option<String> = None;
    let mut out_dir = PathBuf::from("benchmarks/data");

    // simple flag parsing
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dataset" => {
                if i + 1 >= args.len() {
                    eprintln!("--dataset requires a value");
                    std::process::exit(1);
                }
                selected = Some(args[i + 1].clone());
                i += 2;
            }
            "--out" => {
                if i + 1 >= args.len() {
                    eprintln!("--out requires a value");
                    std::process::exit(1);
                }
                out_dir = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--list" => {
                println!("Available datasets:");
                for ds in DATASETS {
                    println!("  - {} -> {}", ds.name, ds.filename);
                }
                return Ok(());
            }
            other => {
                eprintln!("Unknown arg: {}", other);
                std::process::exit(1);
            }
        }
    }

    fs::create_dir_all(&out_dir)?;
    let client = Client::builder()
        .user_agent(USER_AGENT.as_str())
        .build()?;

    let mut success = 0usize;
    let mut failed = Vec::new();

    for ds in DATASETS {
        if let Some(sel) = &selected {
            if ds.name != sel {
                continue;
            }
        }

        let out_path = out_dir.join(ds.filename);
        if out_path.exists() {
            println!("SKIP {} (already exists)", ds.name);
            continue;
        }

        print!("Downloading {} -> {}... ", ds.name, out_path.display());
        io::stdout().flush().ok();

        match download_file(&client, ds.url, &out_path) {
            Ok(_) => {
                println!("OK");
                success += 1;
            }
            Err(e) => {
                println!("FAIL ({})", e);
                failed.push(ds.name);
            }
        }
    }

    println!("\n====================================");
    println!("Downloaded: {}", success);
    if !failed.is_empty() {
        println!("Failed: {} -> {}", failed.len(), failed.join(", "));
        std::process::exit(1);
    } else {
        println!("All requested datasets downloaded");
    }

    Ok(())
}

fn download_file(client: &Client, url: &str, path: &Path) -> anyhow::Result<()> {
    let resp = client.get(url).send()?;
    if !resp.status().is_success() {
        anyhow::bail!("HTTP {}", resp.status());
    }
    let bytes = resp.bytes()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, &bytes)?;
    Ok(())
}
