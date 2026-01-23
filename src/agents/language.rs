//! Programming language detection and definitions

use crate::agents::error::Result;
use serde::{Deserialize, Serialize};

/// Supported programming languages (24+)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    // Systems Programming
    Rust,
    Cpp,
    C,
    Zig,
    
    // Scripting & Dynamic
    Python,
    Ruby,
    Elixir,
    
    // Web & JavaScript Ecosystem
    JavaScript,
    TypeScript,
    
    // Functional
    Haskell,
    OCaml,
    FSharp,
    Julia,
    
    // JVM
    Java,
    Kotlin,
    Scala,
    
    // .NET
    CSharp,
    
    // Compiled Multi-Target
    Nim,
    Haxe,
    
    // Modern Systems
    Go,
    Swift,
    
    // Domain-Specific
    SQL,
    GLSL,      // OpenGL shaders
    WGSL,      // WebGPU shaders
    WASM,      // WebAssembly
    
    // Scripting & Shell
    PowerShell,
    Bash,
    
    // Statistical & Scientific
    R,
    
    // Web & Legacy
    PHP,
    Perl,
    Lua,
    
    // Academic & Niche
    Scheme,
    Racket,
    CommonLisp,
    
    // Mobile & Specialized
    Dart,
    Erlang,
}

impl Language {
    /// Get Docker image for this language
    pub fn docker_image(&self) -> &'static str {
        match self {
            Language::Python => "python:3.11-slim",
            Language::Elixir => "elixir:1.15-alpine",
            Language::TypeScript => "node:20-alpine",
            Language::Nim => "nimlang/nim:2.0.0-alpine",
            Language::Haxe => "haxe:4.3-alpine",
            Language::Go => "golang:1.21-alpine",
            Language::Rust => "rust:1.75-alpine",
            Language::Ruby => "ruby:3.2-alpine",
            Language::Kotlin => "zenika/kotlin:1.9-jdk17",
            Language::JavaScript => "node:20-alpine",
            Language::Swift => "swift:5.9",
            Language::Zig => "euantorano/zig:0.11.0",
            Language::Cpp => "gcc:13-alpine",
            Language::C => "gcc:13-alpine",
            Language::Java => "openjdk:21-slim",
            Language::Scala => "hseeberger/scala-sbt:11.0.15_1.7.1_2.13.9",
            Language::CSharp => "mcr.microsoft.com/dotnet/sdk:8.0",
            Language::Haskell => "haskell:9.4-slim",
            Language::OCaml => "ocaml/opam:alpine",
            Language::FSharp => "mcr.microsoft.com/dotnet/sdk:8.0",
            Language::Julia => "julia:latest",
            Language::SQL => "postgres:16-alpine",
            Language::GLSL | Language::WGSL => "alpine:latest",
            Language::WASM => "emscripten/emsdk:latest",
            Language::PowerShell => "mcr.microsoft.com/powershell:lts-alpine-3.18",
            Language::Bash => "bash:5.2-alpine3.18",
            Language::R => "r-base:4.3",
            Language::PHP => "php:8.3-cli-alpine",
            Language::Perl => "perl:5.38-slim",
            Language::Lua => "nickblah/lua:5.4-alpine",
            Language::Scheme => "schemers/racket:8.11-full",
            Language::Racket => "racket/racket:8.11-full",
            Language::CommonLisp => "daewok/sbcl:2.3.10-alpine",
            Language::Dart => "dart:stable",
            Language::Erlang => "erlang:26-alpine",
        }
    }
    
    /// Get file extension
    pub fn extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::Python => "py",
            Language::JavaScript => "js",
            Language::TypeScript => "ts",
            Language::Go => "go",
            Language::Java => "java",
            Language::Kotlin => "kt",
            Language::Swift => "swift",
            Language::Cpp => "cpp",
            Language::C => "c",
            Language::Zig => "zig",
            Language::Nim => "nim",
            Language::Haxe => "hx",
            Language::Ruby => "rb",
            Language::Elixir => "ex",
            Language::Scala => "scala",
            Language::CSharp => "cs",
            Language::FSharp => "fs",
            Language::Haskell => "hs",
            Language::OCaml => "ml",
            Language::Julia => "jl",
            Language::SQL => "sql",
            Language::GLSL => "glsl",
            Language::WGSL => "wgsl",
            Language::WASM => "wat",
            Language::PowerShell => "ps1",
            Language::Bash => "sh",
            Language::R => "r",
            Language::PHP => "php",
            Language::Perl => "pl",
            Language::Lua => "lua",
            Language::Scheme => "scm",
            Language::Racket => "rkt",
            Language::CommonLisp => "lisp",
            Language::Dart => "dart",
            Language::Erlang => "erl",
        }
    }
    
    /// Get language name
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::Kotlin => "Kotlin",
            Language::Swift => "Swift",
            Language::Cpp => "C++",
            Language::C => "C",
            Language::Zig => "Zig",
            Language::Nim => "Nim",
            Language::Haxe => "Haxe",
            Language::Ruby => "Ruby",
            Language::Elixir => "Elixir",
            Language::Scala => "Scala",
            Language::CSharp => "C#",
            Language::FSharp => "F#",
            Language::Haskell => "Haskell",
            Language::OCaml => "OCaml",
            Language::Julia => "Julia",
            Language::SQL => "SQL",
            Language::GLSL => "GLSL",
            Language::WGSL => "WGSL",
            Language::WASM => "WebAssembly",
            Language::PowerShell => "PowerShell",
            Language::Bash => "Bash",
            Language::R => "R",
            Language::PHP => "PHP",
            Language::Perl => "Perl",
            Language::Lua => "Lua",
            Language::Scheme => "Scheme",
            Language::Racket => "Racket",
            Language::CommonLisp => "Common Lisp",
            Language::Dart => "Dart",
            Language::Erlang => "Erlang",
        }
    }
}

/// Language detector using keyword matching
pub struct LanguageDetector;

impl LanguageDetector {
    pub fn new() -> Self {
        Self
    }
    
    /// Detect language from task description
    pub fn detect(&self, task: &str) -> Result<Language> {
        let task_lower = task.to_lowercase();
        
        // Keyword matching (order matters - check specific before general)
        let keywords = vec![
            ("typescript", Language::TypeScript),
            ("javascript", Language::JavaScript),
            ("elixir", Language::Elixir),
            ("nim", Language::Nim),
            ("haxe", Language::Haxe),
            ("rust", Language::Rust),
            ("c++", Language::Cpp),
            ("cpp", Language::Cpp),
            ("zig", Language::Zig),
            ("c lang", Language::C),
            (" c ", Language::C),
            ("python", Language::Python),
            ("ruby", Language::Ruby),
            ("haskell", Language::Haskell),
            ("ocaml", Language::OCaml),
            ("f#", Language::FSharp),
            ("fsharp", Language::FSharp),
            ("kotlin", Language::Kotlin),
            ("java", Language::Java),
            ("scala", Language::Scala),
            ("c#", Language::CSharp),
            ("csharp", Language::CSharp),
            ("golang", Language::Go),
            ("go lang", Language::Go),
            (" go ", Language::Go),
            ("go ", Language::Go),  // Match "Go " at start of string
            ("swift", Language::Swift),
            ("sql", Language::SQL),
            ("glsl", Language::GLSL),
            ("shader", Language::GLSL),
            ("wgsl", Language::WGSL),
            ("webgpu", Language::WGSL),
            ("wasm", Language::WASM),
            ("webassembly", Language::WASM),
            ("powershell", Language::PowerShell),
            ("bash", Language::Bash),
            ("shell", Language::Bash),
            (" r ", Language::R),
            ("r lang", Language::R),
            ("php", Language::PHP),
            ("perl", Language::Perl),
            ("lua", Language::Lua),
            ("scheme", Language::Scheme),
            ("racket", Language::Racket),
            ("common lisp", Language::CommonLisp),
            ("lisp", Language::CommonLisp),
            ("dart", Language::Dart),
            ("flutter", Language::Dart),
            ("erlang", Language::Erlang),
        ];
        
        for (keyword, lang) in keywords {
            if task_lower.contains(keyword) {
                return Ok(lang);
            }
        }
        
        // Default to Python for general tasks
        Ok(Language::Python)
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_language_detection() {
        let detector = LanguageDetector::new();
        
        assert_eq!(detector.detect("Write Python code for").unwrap(), Language::Python);
        assert_eq!(detector.detect("Create a Rust function").unwrap(), Language::Rust);
        assert_eq!(detector.detect("TypeScript async function").unwrap(), Language::TypeScript);
        assert_eq!(detector.detect("Elixir GenServer").unwrap(), Language::Elixir);
        assert_eq!(detector.detect("Nim binary search tree").unwrap(), Language::Nim);
        assert_eq!(detector.detect("Go worker pool").unwrap(), Language::Go);
    }
    
    #[test]
    fn test_docker_images() {
        assert_eq!(Language::Python.docker_image(), "python:3.11-slim");
        assert_eq!(Language::Rust.docker_image(), "rust:1.75-alpine");
        assert_eq!(Language::Elixir.docker_image(), "elixir:1.15-alpine");
    }
    
    #[test]
    fn test_extensions() {
        assert_eq!(Language::Python.extension(), "py");
        assert_eq!(Language::Rust.extension(), "rs");
        assert_eq!(Language::TypeScript.extension(), "ts");
    }
}
