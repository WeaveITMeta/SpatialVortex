//! Built-in tools. Filesystem tools are confined to a root directory; the
//! shell tool runs a command only after the registry gate has passed it, and
//! never longer than its timeout.

use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use vortex_kernel::{Effect, ParamKind, ParamSpec, ToolSpec};

use crate::calc;
use crate::registry::{Tool, ToolError};

fn p(name: &str, kind: ParamKind) -> ParamSpec {
    ParamSpec { name: name.into(), kind, required: true }
}

fn str_arg<'a>(args: &'a Value, name: &str) -> Result<&'a str, ToolError> {
    args.get(name).and_then(Value::as_str).ok_or_else(|| ToolError::BadArgs(format!("missing string `{name}`")))
}

pub struct Calc;

impl Tool for Calc {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "calc".into(),
            description: "Evaluate an arithmetic expression".into(),
            params: vec![p("expression", ParamKind::String)],
            effect: Effect::ReadOnly,
        }
    }
    fn call(&self, args: &Value) -> Result<Value, ToolError> {
        let e = str_arg(args, "expression")?;
        let v = calc::eval(e).map_err(|e| ToolError::BadArgs(e.to_string()))?;
        Ok(json!({ "value": v, "text": calc::format_number(v) }))
    }
}

pub struct Clock;

impl Tool for Clock {
    fn spec(&self) -> ToolSpec {
        ToolSpec { name: "time.now".into(), description: "Current time (Unix seconds, UTC)".into(), params: vec![], effect: Effect::ReadOnly }
    }
    fn call(&self, _: &Value) -> Result<Value, ToolError> {
        let s = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| ToolError::Failed(e.to_string()))?;
        Ok(json!({ "unix_seconds": s.as_secs() }))
    }
}

/// Resolve `rel` under `root` without following `..` out of it. Works for
/// paths that do not exist yet (needed for writes).
pub fn confine(root: &Path, rel: &str) -> Result<PathBuf, ToolError> {
    let rel_path = Path::new(rel);
    if rel_path.is_absolute() {
        let canon_root = root.canonicalize().map_err(|e| ToolError::Failed(e.to_string()))?;
        let canon = rel_path.canonicalize().map_err(|e| ToolError::BadArgs(format!("{rel}: {e}")))?;
        return if canon.starts_with(&canon_root) {
            Ok(canon)
        } else {
            Err(ToolError::Refused(format!("{rel} is outside the sandbox root")))
        };
    }
    let mut out = root.to_path_buf();
    for c in rel_path.components() {
        match c {
            Component::Normal(x) => out.push(x),
            Component::CurDir => {}
            _ => return Err(ToolError::Refused(format!("{rel} escapes the sandbox root"))),
        }
    }
    Ok(out)
}

pub struct FsRead {
    pub root: PathBuf,
    pub max_bytes: usize,
}

impl Tool for FsRead {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "fs.read".into(),
            description: "Read a text file inside the workspace".into(),
            params: vec![p("path", ParamKind::String)],
            effect: Effect::ReadOnly,
        }
    }
    fn call(&self, args: &Value) -> Result<Value, ToolError> {
        let path = confine(&self.root, str_arg(args, "path")?)?;
        let bytes = std::fs::read(&path).map_err(|e| ToolError::Failed(format!("{}: {e}", path.display())))?;
        let truncated = bytes.len() > self.max_bytes;
        let text = String::from_utf8_lossy(&bytes[..bytes.len().min(self.max_bytes)]).into_owned();
        Ok(json!({ "path": path.display().to_string(), "text": text, "truncated": truncated }))
    }
}

pub struct FsList {
    pub root: PathBuf,
}

impl Tool for FsList {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "fs.list".into(),
            description: "List the entries of a directory inside the workspace".into(),
            params: vec![p("path", ParamKind::String)],
            effect: Effect::ReadOnly,
        }
    }
    fn call(&self, args: &Value) -> Result<Value, ToolError> {
        let path = confine(&self.root, args.get("path").and_then(Value::as_str).unwrap_or("."))?;
        let mut entries: Vec<Value> = std::fs::read_dir(&path)
            .map_err(|e| ToolError::Failed(format!("{}: {e}", path.display())))?
            .filter_map(Result::ok)
            .map(|e| {
                let dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
                json!({ "name": e.file_name().to_string_lossy(), "dir": dir })
            })
            .collect();
        entries.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        Ok(json!({ "path": path.display().to_string(), "entries": entries }))
    }
}

pub struct FsWrite {
    pub root: PathBuf,
}

impl Tool for FsWrite {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "fs.write".into(),
            description: "Write a text file inside the workspace".into(),
            params: vec![p("path", ParamKind::String), p("text", ParamKind::String)],
            effect: Effect::LocalWrite,
        }
    }
    fn call(&self, args: &Value) -> Result<Value, ToolError> {
        let path = confine(&self.root, str_arg(args, "path")?)?;
        let text = str_arg(args, "text")?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ToolError::Failed(e.to_string()))?;
        }
        std::fs::write(&path, text).map_err(|e| ToolError::Failed(e.to_string()))?;
        Ok(json!({ "path": path.display().to_string(), "bytes": text.len() }))
    }
}

/// CLI / terminal access. Declared `LocalWrite` because a shell can change
/// local state; the safety lane classifies each command on its own merits.
pub struct Shell {
    pub cwd: PathBuf,
    pub timeout: Duration,
}

impl Tool for Shell {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "shell".into(),
            description: "Run a command in the system shell (PowerShell on Windows, sh elsewhere)".into(),
            params: vec![p("command", ParamKind::String)],
            effect: Effect::LocalWrite,
        }
    }
    fn call(&self, args: &Value) -> Result<Value, ToolError> {
        let command = str_arg(args, "command")?;
        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("powershell");
            c.args(["-NoProfile", "-NonInteractive", "-Command", command]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", command]);
            c
        };
        let mut child = cmd
            .current_dir(&self.cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ToolError::Failed(e.to_string()))?;
        let mut stdout = child.stdout.take().expect("piped");
        let mut stderr = child.stderr.take().expect("piped");
        let out_t = std::thread::spawn(move || {
            let mut s = String::new();
            let _ = stdout.read_to_string(&mut s);
            s
        });
        let err_t = std::thread::spawn(move || {
            let mut s = String::new();
            let _ = stderr.read_to_string(&mut s);
            s
        });
        let t0 = Instant::now();
        let status = loop {
            if let Some(st) = child.try_wait().map_err(|e| ToolError::Failed(e.to_string()))? {
                break Some(st);
            }
            if t0.elapsed() > self.timeout {
                // Kill exactly the child we spawned, never by name.
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        let stdout = out_t.join().unwrap_or_default();
        let stderr = err_t.join().unwrap_or_default();
        match status {
            Some(st) => Ok(json!({ "exit_code": st.code(), "stdout": stdout, "stderr": stderr })),
            None => Err(ToolError::Failed(format!("timed out after {:?}", self.timeout))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_tool() {
        let v = Calc.call(&json!({"expression": "6*7"})).unwrap();
        assert_eq!(v["text"], "42");
    }

    #[test]
    fn sandbox_blocks_escape() {
        let root = std::env::temp_dir();
        assert!(confine(&root, "a/b.txt").is_ok());
        assert!(matches!(confine(&root, "../x"), Err(ToolError::Refused(_))));
    }

    #[test]
    fn fs_roundtrip_inside_root() {
        let root = std::env::temp_dir().join(format!("vortex-tools-test-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        FsWrite { root: root.clone() }.call(&json!({"path": "n/hello.txt", "text": "hi"})).unwrap();
        let r = FsRead { root: root.clone(), max_bytes: 1024 }.call(&json!({"path": "n/hello.txt"})).unwrap();
        assert_eq!(r["text"], "hi");
        let l = FsList { root: root.clone() }.call(&json!({"path": "n"})).unwrap();
        assert_eq!(l["entries"][0]["name"], "hello.txt");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn shell_runs_and_times_out() {
        let sh = Shell { cwd: std::env::temp_dir(), timeout: Duration::from_secs(20) };
        let out = sh.call(&json!({"command": "echo vortex"})).unwrap();
        assert!(out["stdout"].as_str().unwrap().contains("vortex"));
        let slow = Shell { cwd: std::env::temp_dir(), timeout: Duration::from_millis(200) };
        let cmd = if cfg!(windows) { "Start-Sleep -Seconds 5" } else { "sleep 5" };
        assert!(matches!(slow.call(&json!({"command": cmd})), Err(ToolError::Failed(_))));
    }
}
