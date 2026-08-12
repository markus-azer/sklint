use gray_matter::Matter;
use gray_matter::engine::YAML;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Skill {
    pub name: Option<String>,
    pub description: Option<String>,
    pub body: String,
    pub dir: PathBuf,
    pub files: Vec<PathBuf>,
}

#[derive(Deserialize, Default)]
struct FrontMatter {
    name: Option<String>,
    description: Option<String>,
}

impl Skill {
    pub fn parse(skill_md: &Path) -> std::io::Result<Skill> {
        let raw = fs::read_to_string(skill_md)?;
        let parsed = Matter::<YAML>::new().parse(&raw);
        let fm: FrontMatter = parsed
            .data
            .and_then(|pod| pod.deserialize().ok())
            .unwrap_or_default();
        let dir = skill_md
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let files = list_files(&dir);
        Ok(Skill {
            name: fm.name,
            description: fm.description,
            body: parsed.content,
            dir,
            files,
        })
    }
}

fn list_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for sub in ["scripts", "references"] {
        collect_files(&dir.join(sub), &mut out);
    }
    out
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

const RESOURCE_MARKERS: [&str; 2] = ["scripts/", "references/"];
const TRAILING_PUNCTUATION: [char; 4] = ['.', ',', ';', ':'];

pub fn referenced_paths(body: &str) -> Vec<String> {
    let mut refs = markdown_link_targets(body);
    refs.extend(bare_resource_paths(body));
    refs.into_iter()
        .map(|path| path.trim_end_matches(TRAILING_PUNCTUATION).to_string())
        .filter(|path| is_local_path(path))
        .collect()
}

fn markdown_link_targets(body: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut rest = body;
    while let Some(open) = rest.find("](") {
        rest = &rest[open + "](".len()..];
        let Some(close) = rest.find(')') else { break };
        targets.push(rest[..close].trim().to_string());
        rest = &rest[close + 1..];
    }
    targets
}

fn bare_resource_paths(body: &str) -> Vec<String> {
    body.split(is_token_boundary)
        .filter(|token| RESOURCE_MARKERS.iter().any(|marker| token.contains(marker)))
        .map(str::to_string)
        .collect()
}

fn is_token_boundary(c: char) -> bool {
    c.is_whitespace() || "`'\"()[]<>".contains(c)
}

fn is_local_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with("http")
        && !path.starts_with('#')
        && !path.starts_with("mailto:")
}
