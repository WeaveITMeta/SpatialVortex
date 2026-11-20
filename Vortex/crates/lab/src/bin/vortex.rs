//! `vortex`: the command-line front door.
//!
//! ```text
//! vortex eval [--seed N] [--limit N] [--procedural N] [--arc N] [--code N]
//!             [--lm-mb N] [--order 2|3|4] [--no-lm] [--threads N]
//!             [--only a,b] [--no-contamination]
//! vortex ask "<request>"            one request through the inference tick
//! vortex classify "<text>"          Guardian moderation verdict
//! vortex action '<tool>' '<json>'   tool-call risk class
//! vortex astra --space PATH         a Spatial Reasoning Lab Space on disk, graded by its own evaluate.py
//! vortex serve                      the API over JSON-RPC 2.0 on stdio (direct calls and MCP tools)
//! vortex cost                       price table from the latest eval
//! vortex constitution [--write]     print (or write CONSTITUTION.md)
//! vortex mcp <program> [args...]    connect to an MCP server and list its tools
//! vortex call [<method> ['<json>']] [--json|--toon]
//!                                   one API method through the JSON door; prints the
//!                                   model-facing text (the shorter of JSON and TOON),
//!                                   or one format when forced.
//!                                   No method: list the methods.
//! vortex toon <file.json|file.jsonl|->
//!                                   convert any JSON (or JSONL, as an array) to TOON
//! ```

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use vortex_api::Vortex;
use vortex_api::constitution::Constitution;
use vortex_api::lm::NgramConfig;
use vortex_api::tools::{Calc, Registry};
use vortex_harness::suites::SuiteConfig;
use vortex_lab::host::builtin::{Clock, FsList, FsRead, FsWrite, Shell};
use vortex_lab::lifecycle::{self, EvalOptions};

/// `Vortex/` (the workspace root), found from this crate's manifest.
fn vortex_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap_or_else(|_| PathBuf::from("."))
}

fn repo_root() -> PathBuf {
    vortex_root().join("..").canonicalize().unwrap_or_else(|_| PathBuf::from(".."))
}

struct Args(Vec<String>);

impl Args {
    fn flag(&self, name: &str) -> bool {
        self.0.iter().any(|a| a == name)
    }
    fn val(&self, name: &str) -> Option<String> {
        self.0.iter().position(|a| a == name).and_then(|i| self.0.get(i + 1).cloned())
    }
    fn num<T: std::str::FromStr>(&self, name: &str, default: T) -> T {
        self.val(name).and_then(|v| v.parse().ok()).unwrap_or(default)
    }
}

fn registry(root: &Path) -> Registry {
    let mut r = Registry::new();
    r.register(Calc);
    r.register(Clock);
    r.register(FsRead { root: root.to_path_buf(), max_bytes: 256 * 1024 });
    r.register(FsList { root: root.to_path_buf() });
    r.register(FsWrite { root: root.to_path_buf() });
    r.register(Shell { cwd: root.to_path_buf(), timeout: Duration::from_secs(60) });
    r
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = argv.first().cloned() else {
        eprintln!("usage: vortex <eval|ask|classify|action|astra|cost|constitution|mcp|call|toon|serve> ...");
        std::process::exit(2);
    };
    let args = Args(argv[1..].to_vec());
    let code = match cmd.as_str() {
        "eval" => eval(&args),
        "ask" => ask(&args),
        "classify" => classify(&args),
        "action" => action(&args),
        "serve" => serve(),
        "astra" => astra(&args),
        "cost" => cost(),
        "constitution" => constitution(&args),
        "mcp" => mcp(&args),
        "call" => call(&args),
        "toon" => toon(&args),
        other => {
            eprintln!("unknown command `{other}`");
            2
        }
    };
    std::process::exit(code);
}

fn eval(a: &Args) -> i32 {
    let data = repo_root().join("benchmarks/data");
    let threads = a.num("--threads", std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4));
    let opts = EvalOptions {
        suites: SuiteConfig {
            data_dir: a.val("--data").map(PathBuf::from).unwrap_or_else(|| data.clone()),
            heldout_dir: vortex_root().join("eval/heldout"),
            seed: a.num("--seed", 20261002),
            limit: a.num("--limit", 500),
            procedural: a.num("--procedural", 200),
            arc_agi_limit: a.num("--arc", 100),
            code_limit: a.num("--code", 164),
        },
        corpus_path: a.val("--corpus").map(PathBuf::from).unwrap_or_else(|| data.join("arc/ARC-V1-Feb2018-2/ARC_Corpus.txt")),
        lm_bytes: a.num::<usize>("--lm-mb", 32) << 20,
        lm: NgramConfig { order: a.num("--order", 3), min_count: 2 },
        no_lm: a.flag("--no-lm"),
        threads,
        only: a.val("--only").map(|s| s.split(',').map(str::to_string).collect()).unwrap_or_default(),
        eval_dir: vortex_root().join("eval"),
        check_contamination: !a.flag("--no-contamination"),
    };
    match lifecycle::run(&opts, |m| eprintln!("[vortex] {m}")) {
        Ok(out) => {
            println!("\n== Lifecycle ==\n{}\n", out.lifecycle.summary());
            println!("== Results: {} ==\nagent: {}\nconstitution: {}\nseed: {}  git: {}\n", out.report.run_id, out.report.agent, out.report.constitution, out.report.seed, out.report.git_commit);
            println!("{}", out.report.table());
            println!("== Suite details ==");
            for s in &out.report.suites {
                let mut parts: Vec<String> = s.extra.iter().map(|(k, v)| format!("{k}={v:.3}")).collect();
                if let Some(c) = s.contamination_rate {
                    parts.push(format!("contamination={:.3}", c));
                }
                println!("{:<20} {}  [{}]", s.id, parts.join("  "), s.source);
            }
            for w in &out.warnings {
                println!("WARNING: {w}");
            }
            println!("\n== Cost per request (measured latency) ==\n{}", out.cost.table());
            0
        }
        Err(e) => {
            eprintln!("eval failed: {e}");
            1
        }
    }
}

fn ask(a: &Args) -> i32 {
    let req = a.0.join(" ");
    let t = Vortex::new().tick(&registry(&vortex_root()), &[], &req);
    for s in &t.trace {
        println!("{:<9} {:>7} us  {}", format!("{:?}", s.stage), s.micros, s.note);
    }
    println!("\n{}", t.reply);
    0
}

fn classify(a: &Args) -> i32 {
    let m = Vortex::new().moderate(&a.0.join(" "));
    println!("{}", serde_json::to_string_pretty(&m).unwrap_or_default());
    0
}

fn action(a: &Args) -> i32 {
    let tool = a.0.first().cloned().unwrap_or_default();
    let args: serde_json::Value = a.0.get(1).and_then(|s| serde_json::from_str(s).ok()).unwrap_or(serde_json::json!({}));
    let r = Vortex::new().assess(&vortex_kernel::ToolCall::new(tool, args));
    println!("{}", serde_json::to_string_pretty(&r).unwrap_or_default());
    0
}

fn astra(a: &Args) -> i32 {
    let Some(space) = a.val("--space").map(PathBuf::from) else {
        eprintln!("usage: vortex astra --space <path to a Spatial Reasoning Lab Space> [--scratch DIR]");
        return 2;
    };
    let scratch = a.val("--scratch").map(PathBuf::from).unwrap_or_else(|| std::env::temp_dir().join("vortex-astra"));
    let run = || -> Result<(), String> {
        let ch = vortex_lab::astra::load_challenge(&space)?;
        let (sub, conf) = vortex_api::spatial::solve_challenge(&ch);
        println!("route ({} waypoints, {:.3} m), portals {:?}, confidence {conf}", sub.route.len(), vortex_api::spatial::plan::length(&sub.route), sub.passable_portals);
        let root = vortex_lab::astra::scratch_copy(&space, &scratch)?;
        let before = vortex_lab::astra::evaluate(&root, &sub, true)?;
        println!("\nBefore authoring the scene (submission + on-disk check): score {}", before["score"]);
        let edited = vortex_lab::astra::author_assembly(&root, &sub)?;
        println!("Authored positions into scratch copy: {}", edited.join(", "));
        let after = vortex_lab::astra::evaluate(&root, &sub, true)?;
        println!("After authoring (graded by the Space's own evaluate.py --check-scene):\n{}", serde_json::to_string_pretty(&after).unwrap_or_default());
        println!("\nThe original Space was not modified. Scratch copy: {}", root.display());
        Ok(())
    };
    match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("astra: {e}");
            1
        }
    }
}

fn cost() -> i32 {
    let p = vortex_root().join("eval/cost_report.json");
    match std::fs::read_to_string(&p) {
        Ok(s) => {
            println!("{s}");
            0
        }
        Err(e) => {
            eprintln!("{}: {e} (run `vortex eval` first)", p.display());
            1
        }
    }
}

fn constitution(a: &Args) -> i32 {
    let md = Constitution::v1().to_markdown();
    if a.flag("--write") {
        let p = vortex_root().join("CONSTITUTION.md");
        if let Err(e) = std::fs::write(&p, &md) {
            eprintln!("{}: {e}", p.display());
            return 1;
        }
        println!("wrote {}", p.display());
    } else {
        print!("{md}");
    }
    0
}

fn mcp(a: &Args) -> i32 {
    let Some(program) = a.0.first() else {
        eprintln!("usage: vortex mcp <program> [args...]");
        return 2;
    };
    let rest: Vec<&str> = a.0[1..].iter().map(String::as_str).collect();
    match vortex_lab::host::mcp::McpClient::spawn(program, &rest) {
        Ok(client) => {
            println!("server: {}", client.server_info.get("serverInfo").cloned().unwrap_or_default());
            let client = Arc::new(Mutex::new(client));
            match vortex_lab::host::mcp::McpTool::all(client, "mcp.") {
                Ok(tools) => {
                    for t in tools {
                        let s = vortex_api::tools::Tool::spec(&t);
                        println!("{:<40} {:?}  {}", s.name, s.effect, s.description.lines().next().unwrap_or(""));
                    }
                    0
                }
                Err(e) => {
                    eprintln!("tools/list: {e}");
                    1
                }
            }
        }
        Err(e) => {
            eprintln!("spawn {program}: {e}");
            1
        }
    }
}

/// One method through the same door every host uses. Prints what a
/// model would be handed (`text_for`: the shorter of compact JSON and
/// TOON); `--json` or `--toon` forces one.
fn call(a: &Args) -> i32 {
    let (json, toon) = (a.flag("--json"), a.flag("--toon"));
    let positional: Vec<&String> = a.0.iter().filter(|s| !matches!(s.as_str(), "--json" | "--toon")).collect();
    let show = |v: &serde_json::Value| {
        if json {
            println!("{v}")
        } else if toon {
            println!("{}", vortex_api::toon::encode(v))
        } else {
            println!("{}", vortex_api::text_for(v).text)
        }
    };
    let Some(method) = positional.first() else {
        let methods: Vec<serde_json::Value> = Vortex::new().methods().iter().map(|m| serde_json::json!({ "name": m.name, "read_only": m.read_only, "description": m.description })).collect();
        show(&serde_json::json!({ "methods": methods }));
        return 0;
    };
    let params = match positional.get(1) {
        None => serde_json::Value::Null,
        Some(text) => match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("params are not JSON: {e}");
                return 2;
            }
        },
    };
    match Vortex::new().call(method, params) {
        Ok(v) => {
            show(&v);
            0
        }
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

/// JSON in, TOON out: a file, or `-` for stdin. A `.jsonl` file (one JSON
/// document per line, like the ledgers) becomes an array of its lines.
fn toon(a: &Args) -> i32 {
    let Some(src) = a.0.first() else {
        eprintln!("usage: vortex toon <file.json|file.jsonl|->");
        return 2;
    };
    let text = if src == "-" {
        let mut s = String::new();
        match std::io::Read::read_to_string(&mut std::io::stdin(), &mut s) {
            Ok(_) => s,
            Err(e) => {
                eprintln!("stdin: {e}");
                return 1;
            }
        }
    } else {
        match std::fs::read_to_string(src) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{src}: {e}");
                return 1;
            }
        }
    };
    let value: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(whole) => {
            let lines: Result<Vec<serde_json::Value>, _> = text.lines().filter(|l| !l.trim().is_empty()).map(serde_json::from_str).collect();
            match lines {
                Ok(v) if !v.is_empty() => serde_json::Value::Array(v),
                _ => {
                    eprintln!("{src}: not JSON or JSONL: {whole}");
                    return 1;
                }
            }
        }
    };
    println!("{}", vortex_api::toon::encode(&value));
    0
}

/// The API over JSON-RPC 2.0 on stdio, one message per line: direct method
/// calls and the MCP tool dialect (`vortex_api::rpc`). Runs until stdin closes.
fn serve() -> i32 {
    use std::io::{BufRead, Write};
    let vortex = Vortex::new();
    let stdout = std::io::stdout();
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = vortex.handle_line(&line) {
            let mut out = stdout.lock();
            if writeln!(out, "{reply}").and_then(|_| out.flush()).is_err() {
                break;
            }
        }
    }
    0
}
