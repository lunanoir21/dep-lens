use std::path::{Path, PathBuf};
use std::process::ExitCode;

use dep_lens_core::{build_report, report, scanner, util};

const USAGE: &str = "\
dep-lens-core: scan dependencies for license risk

USAGE:
    dep-lens-core [OPTIONS]

OPTIONS:
    --path <DIR>       Project directory to scan (default: current directory)
    --format <FMT>     Output format: json, html, csv, markdown (default: json)
    --lang <LANG>      Language for reports: en, tr (default: en)
    --ignore <NAMES>   Comma-separated package names to exclude (repeatable)
    --project <NAME>   Override the detected project name
    --help             Show this help
    --version          Show version
";

struct Args {
    path: PathBuf,
    format: Format,
    lang: String,
    ignore: Vec<String>,
    project: Option<String>,
}

#[derive(PartialEq, Eq)]
enum Format {
    Json,
    Html,
    Csv,
    Markdown,
}

fn parse_args(argv: &[String]) -> Result<Option<Args>, String> {
    let mut args = Args {
        path: PathBuf::from("."),
        format: Format::Json,
        lang: "en".to_string(),
        ignore: Vec::new(),
        project: None,
    };
    let mut iter = argv.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print!("{USAGE}");
                return Ok(None);
            }
            "--version" | "-V" => {
                println!("dep-lens-core {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            "--path" => {
                let value = iter.next().ok_or("--path requires a value")?;
                args.path = PathBuf::from(value);
            }
            "--format" => {
                let value = iter.next().ok_or("--format requires a value")?;
                args.format = match value.as_str() {
                    "json" => Format::Json,
                    "html" => Format::Html,
                    "csv" => Format::Csv,
                    "markdown" | "md" => Format::Markdown,
                    other => return Err(format!("unsupported format: {other}")),
                };
            }
            "--lang" => {
                let value = iter.next().ok_or("--lang requires a value")?;
                args.lang = match value.as_str() {
                    "en" | "tr" => value.to_string(),
                    other => return Err(format!("unsupported language: {other}")),
                };
            }
            "--ignore" => {
                let value = iter.next().ok_or("--ignore requires a value")?;
                args.ignore.extend(
                    value
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string),
                );
            }
            "--project" => {
                let value = iter.next().ok_or("--project requires a value")?;
                args.project = Some(value.clone());
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(Some(args))
}

/// Best-effort project name: package.json name, then Cargo.toml package name,
/// then the directory basename.
fn detect_project_name(path: &Path) -> String {
    if let Ok(raw) = std::fs::read_to_string(path.join("package.json")) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(name) = value.get("name").and_then(|n| n.as_str()) {
                if !name.trim().is_empty() {
                    return name.trim().to_string();
                }
            }
        }
    }
    if let Ok(raw) = std::fs::read_to_string(path.join("Cargo.toml")) {
        if let Some(name) = cargo_toml_package_name(&raw) {
            return name;
        }
    }
    path.canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "unknown-project".to_string())
}

/// Minimal TOML scan: find `name = "..."` inside the `[package]` section.
fn cargo_toml_package_name(raw: &str) -> Option<String> {
    let mut in_package = false;
    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if in_package {
            if let Some(rest) = line.strip_prefix("name") {
                let rest = rest.trim_start();
                if let Some(rest) = rest.strip_prefix('=') {
                    let value = rest.trim().trim_matches('"');
                    if !value.is_empty() {
                        return Some(value.to_string());
                    }
                }
            }
        }
    }
    None
}

fn run() -> Result<(), String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let Some(args) = parse_args(&argv)? else {
        return Ok(());
    };
    if !args.path.is_dir() {
        return Err(format!("not a directory: {}", args.path.display()));
    }

    let npm_packages =
        scanner::npm::scan(&args.path).map_err(|e| format!("npm scan failed: {e}"))?;
    let cargo_packages = scanner::cargo::scan(&args.path).unwrap_or_else(|e| {
        eprintln!("warning: cargo scan skipped: {e}");
        Vec::new()
    });
    let go_packages = scanner::go::scan(&args.path).unwrap_or_else(|e| {
        eprintln!("warning: go scan skipped: {e}");
        Vec::new()
    });
    let python_packages = scanner::python::scan(&args.path).unwrap_or_else(|e| {
        eprintln!("warning: python scan skipped: {e}");
        Vec::new()
    });
    let ruby_packages = scanner::ruby::scan(&args.path).unwrap_or_else(|e| {
        eprintln!("warning: ruby scan skipped: {e}");
        Vec::new()
    });
    let php_packages = scanner::php::scan(&args.path).unwrap_or_else(|e| {
        eprintln!("warning: php scan skipped: {e}");
        Vec::new()
    });

    let mut packages = npm_packages;
    packages.extend(cargo_packages);
    packages.extend(go_packages);
    packages.extend(python_packages);
    packages.extend(ruby_packages);
    packages.extend(php_packages);

    let project = args
        .project
        .unwrap_or_else(|| detect_project_name(&args.path));
    let path_display = args
        .path
        .canonicalize()
        .unwrap_or_else(|_| args.path.clone())
        .display()
        .to_string();
    let report = build_report(
        &project,
        &path_display,
        &util::now_iso8601(),
        packages,
        &args.ignore,
    );

    let output = match args.format {
        Format::Json => report::json::render(&report).map_err(|e| e.to_string())?,
        Format::Html => report::html::render(&report, &args.lang),
        Format::Csv => report::csv::render(&report),
        Format::Markdown => report::markdown::render(&report),
    };
    println!("{output}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(2)
        }
    }
}
