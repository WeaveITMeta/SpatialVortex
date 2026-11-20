//! The purity invariant of `vortex-api`: nothing under `src/` touches files,
//! processes, sockets or environment variables. This scans the source text,
//! so a new I/O call fails here even before anyone reviews it.

use std::path::{Path, PathBuf};

const FORBIDDEN: &[&str] = &["std::fs", "std::process", "std::net", "std::env::", "File::open", "Command::new"];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let p = e.unwrap().path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn api_source_does_no_io() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    assert!(files.len() >= 20, "scanned only {} files under {}", files.len(), src.display());
    let mut hits = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap();
        for (n, line) in text.lines().enumerate() {
            for pat in FORBIDDEN {
                if line.contains(pat) {
                    hits.push(format!("{}:{}: `{pat}` in: {}", f.display(), n + 1, line.trim()));
                }
            }
        }
    }
    assert!(hits.is_empty(), "vortex-api must not do file, process, network or env I/O:\n{}", hits.join("\n"));
}
