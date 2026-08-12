mod config;
mod rules;
mod skill;
mod token;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use owo_colors::{OwoColorize, Stream};

use config::{Config, Severity};
use rules::{Finding, lint};
use skill::Skill;

const EXIT_OK: u8 = 0;
const EXIT_FINDINGS: u8 = 1;
const EXIT_ERROR: u8 = 2;

const RULE_COL_WIDTH: usize = 13;

const PASS_MARK: &str = "✓";
const FAIL_MARK: &str = "✗";
const SUMMARY_SEP: &str = " · ";

#[derive(Parser)]
#[command(name = "sklint", version, about)]
struct Cli {
    path: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("sklint: {err:#}");
            ExitCode::from(EXIT_ERROR)
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    let root = cli.path.unwrap_or_else(|| PathBuf::from("."));

    let cfg = Config::load(&root).map_err(|e| anyhow::anyhow!("invalid sklint.toml: {e}"))?;
    let skills = discover(&root);

    if skills.is_empty() {
        anyhow::bail!("no SKILL.md found under {}", root.display());
    }

    let mut errors = 0usize;
    let mut warns = 0usize;
    let mut read_failed = false;
    for skill_md in &skills {
        let parsed = match Skill::parse(skill_md) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("cannot read {}: {e}", skill_md.display());
                read_failed = true;
                continue;
            }
        };
        let findings = lint(&parsed, &cfg);
        report(&parsed, &findings);
        errors += findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count();
        warns += findings
            .iter()
            .filter(|f| f.severity == Severity::Warn)
            .count();
    }

    let code = if read_failed {
        EXIT_ERROR
    } else if errors > 0 {
        EXIT_FINDINGS
    } else {
        EXIT_OK
    };
    summary(skills.len(), errors, warns, code);
    Ok(ExitCode::from(code))
}

fn discover(root: &Path) -> Vec<PathBuf> {
    if root.join("SKILL.md").is_file() {
        return vec![root.join("SKILL.md")];
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out.sort();
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            walk(&path, out);
        } else if path.file_name().is_some_and(|n| n == "SKILL.md") {
            out.push(path);
        }
    }
}

fn skill_label(skill: &Skill) -> String {
    skill
        .name
        .clone()
        .or_else(|| {
            skill
                .dir
                .file_name()
                .and_then(|n| n.to_str())
                .map(String::from)
        })
        .unwrap_or_else(|| skill.dir.display().to_string())
}

fn report(skill: &Skill, findings: &[Finding]) {
    let label = skill_label(skill);

    if findings.is_empty() {
        println!(
            "{} {label}",
            PASS_MARK.if_supports_color(Stream::Stdout, |t| t.green())
        );
        return;
    }

    println!(
        "{} {label}",
        FAIL_MARK.if_supports_color(Stream::Stdout, |t| t.red())
    );
    for f in findings {
        let tag = match f.severity {
            Severity::Error => "error"
                .if_supports_color(Stream::Stdout, |t| t.red())
                .to_string(),
            Severity::Warn => "warn "
                .if_supports_color(Stream::Stdout, |t| t.yellow())
                .to_string(),
            Severity::Off => continue,
        };
        println!(
            "    {tag}  {:<width$} {}",
            f.rule,
            f.msg,
            width = RULE_COL_WIDTH
        );
    }
    println!();
}

fn summary(count: usize, errors: usize, warns: usize, code: u8) {
    let mut parts = vec![format!("{count} skills")];
    if errors > 0 {
        parts.push(format!("{errors} error{}", plural(errors)));
    }
    if warns > 0 {
        parts.push(format!("{warns} warn{}", plural(warns)));
    }
    parts.push(format!("exit {code}"));
    println!("{}", parts.join(SUMMARY_SEP));
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}
