//! `vortex`: the command-line front door.
//!
//! ```text
//! vortex eval [--seed N] [--limit N] [--procedural N] [--arc N] [--code N]
//!             [--lm-mb N] [--order 2|3|4] [--no-lm] [--threads N]
//!             [--only a,b] [--no-contamination]
//! vortex ask "<request>"            one request through the inference tick
//! vortex classify "<text>"          Guardian moderation verdict
//! vortex action '<tool>' '<json>'   tool-call risk class
//! vortex spaces                     Eustress Universes, AI Spaces, engine status
//! vortex astra [--space PATH]       real Astra Space, graded by its own evaluate.py
//! vortex cost                       price table from the latest eval
//! vortex constitution [--write]     print (or write CONSTITUTION.md)
//! vortex mcp <program> [args...]    connect to an MCP server and list its tools
//! ```

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use vortex_constitution::Constitution;
use vortex_harness::suites::SuiteConfig;
use vortex_llm::NgramConfig;
use vortex_orchestrator::lifecycle::{self, EvalOptions};
use vortex_tools::Registry;
use vortex_tools::builtin::{Calc, Clock, FsList, FsRead, FsWrite, Shell};

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
        eprintln!("usage: vortex <eval|ask|classify|action|spaces|astra|cost|constitution|mcp> ...");
        std::process::exit(2);
    };
    let args = Args(argv[1..].to_vec());
    let code = match cmd.as_str() {
        "eval" => eval(&args),
        "ask" => ask(&args),
        "classify" => classify(&args),
        "action" => action(&args),
        "spaces" => spaces(),
        "astra" => astra(&args),
        "cost" => cost(),
        "constitution" => constitution(&args),
        "mcp" => mcp(&args),
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
            println!("\n== Eustress AI Mode lifecycle ==\n{}\n", out.lifecycle.summary());
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
    let c = Constitution::v1();
    let t = vortex_pipeline::tick(&c, &registry(&vortex_root()), &[], &req);
    for s in &t.trace {
        println!("{:<9} {:>7} us  {}", format!("{:?}", s.stage), s.micros, s.note);
    }
    println!("\n{}", t.reply);
    0
}

fn classify(a: &Args) -> i32 {
    let m = vortex_safety::moderate(&a.0.join(" "));
    println!("{}", serde_json::to_string_pretty(&m).unwrap_or_default());
    0
}

fn action(a: &Args) -> i32 {
    let tool = a.0.first().cloned().unwrap_or_default();
    let args: serde_json::Value = a.0.get(1).and_then(|s| serde_json::from_str(s).ok()).unwrap_or(serde_json::json!({}));
    let r = vortex_safety::assess_action(&vortex_kernel::ToolCall::new(tool, args));
    println!("{}", serde_json::to_string_pretty(&r).unwrap_or_default());
    0
}

fn spaces() -> i32 {
    let root = vortex_eustress::default_root();
    match vortex_eustress::universes(&root) {
        Ok(all) => {
            println!("Eustress root: {}\n", root.display());
            for u in &all {
                println!("{:<22} {:>3} spaces  engine {}", u.name, u.spaces.len(), if u.engine_live { "LIVE" } else { "off" });
            }
            println!("\nAI research / evaluation Spaces:");
            for (u, s) in vortex_eustress::ai_research_spaces(&all) {
                println!("  {u} / {s}");
            }
            0
        }
        Err(e) => {
            eprintln!("{}: {e}", root.display());
            1
        }
    }
}

fn astra(a: &Args) -> i32 {
    let space = a.val("--space").map(PathBuf::from).unwrap_or_else(|| vortex_eustress::default_root().join("Astra/Spaces/Spatial Reasoning Lab"));
    let scratch = a.val("--scratch").map(PathBuf::from).unwrap_or_else(|| std::env::temp_dir().join("vortex-astra"));
    let run = || -> Result<(), String> {
        let ch = vortex_eustress::astra::load_challenge(&space)?;
        let (sub, conf) = vortex_spatial::solve_challenge(&ch);
        println!("route ({} waypoints, {:.3} m), portals {:?}, confidence {conf}", sub.route.len(), vortex_spatial::plan::length(&sub.route), sub.passable_portals);
        let root = vortex_eustress::astra::scratch_copy(&space, &scratch)?;
        let before = vortex_eustress::astra::evaluate(&root, &sub, true)?;
        println!("\nBefore authoring the scene (submission + on-disk check): score {}", before["score"]);
        let edited = vortex_eustress::astra::author_assembly(&root, &sub)?;
        println!("Authored positions into scratch copy: {}", edited.join(", "));
        let after = vortex_eustress::astra::evaluate(&root, &sub, true)?;
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
    match vortex_tools::mcp::McpClient::spawn(program, &rest) {
        Ok(client) => {
            println!("server: {}", client.server_info.get("serverInfo").cloned().unwrap_or_default());
            let client = Arc::new(Mutex::new(client));
            match vortex_tools::mcp::McpTool::all(client, "mcp.") {
                Ok(tools) => {
                    for t in tools {
                        let s = vortex_tools::Tool::spec(&t);
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
