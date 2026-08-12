use crate::config::{Config, Severity};
use crate::skill::{Skill, referenced_paths};
use crate::token::tokens;

// Rule ids. These double as the keys users set severities on in sklint.toml.
const RULE_NAME: &str = "name";
const RULE_DESC_LENGTH: &str = "desc-length";
const RULE_DESC_WHEN: &str = "desc-when";
const RULE_BUDGET: &str = "budget";
const RULE_DEAD_LINK: &str = "dead-link";
const RULE_ORPHAN: &str = "orphan";
const RULE_SCRIPT_INTENT: &str = "script-intent";
const RULE_MUSH: &str = "mush";

pub struct Finding {
    pub rule: &'static str,
    pub severity: Severity,
    pub msg: String,
}

pub type Check = fn(&Skill, &Config) -> Vec<Finding>;

pub const CHECKS: &[Check] = &[
    name,
    description,
    budget,
    dead_links,
    orphans,
    script_intent,
    mush,
];

pub fn lint(s: &Skill, cfg: &Config) -> Vec<Finding> {
    CHECKS.iter().flat_map(|c| c(s, cfg)).collect()
}

fn finding(rule: &'static str, sev: Severity, msg: impl Into<String>) -> Vec<Finding> {
    match sev {
        Severity::Off => Vec::new(),
        _ => vec![Finding {
            rule,
            severity: sev,
            msg: msg.into(),
        }],
    }
}

fn name(s: &Skill, cfg: &Config) -> Vec<Finding> {
    let sev = cfg.severity(RULE_NAME, Severity::Error);
    if sev == Severity::Off {
        return Vec::new();
    }
    let Some(name) = &s.name else {
        return finding(RULE_NAME, sev, "missing name in frontmatter");
    };

    let mut messages = Vec::new();
    let name_len = name.chars().count();
    if name_len > cfg.thresholds.name_max {
        messages.push(format!(
            "name is {name_len} chars, over {}",
            cfg.thresholds.name_max
        ));
    }
    if !is_kebab(name) {
        messages.push("name is not kebab-case".to_string());
    }
    if let Some(folder) = s.dir.file_name().and_then(|f| f.to_str()) {
        if folder != name {
            messages.push(format!("name '{name}' does not match folder '{folder}'"));
        }
    }

    messages
        .into_iter()
        .map(|msg| Finding {
            rule: RULE_NAME,
            severity: sev,
            msg,
        })
        .collect()
}

fn is_kebab(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && !s.ends_with('-')
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn description(s: &Skill, cfg: &Config) -> Vec<Finding> {
    let len_sev = cfg.severity(RULE_DESC_LENGTH, Severity::Error);
    let Some(desc) = &s.description else {
        return finding(
            RULE_DESC_LENGTH,
            len_sev,
            "missing description in frontmatter",
        );
    };

    let mut out = Vec::new();
    let desc_len = desc.chars().count();
    if desc_len > cfg.thresholds.desc_max {
        out.extend(finding(
            RULE_DESC_LENGTH,
            len_sev,
            format!(
                "description is {desc_len} chars, over {}",
                cfg.thresholds.desc_max
            ),
        ));
    }
    let when_sev = cfg.severity(RULE_DESC_WHEN, Severity::Warn);
    if !says_when(desc) {
        out.extend(finding(
            RULE_DESC_WHEN,
            when_sev,
            "description may not say when to use it",
        ));
    }
    out
}

fn says_when(desc: &str) -> bool {
    let lower = desc.to_lowercase();
    ["use when", "when you", "when the", "whenever", "trigger"]
        .iter()
        .any(|p| lower.contains(p))
}

fn budget(s: &Skill, cfg: &Config) -> Vec<Finding> {
    let default = if has_references(s) {
        Severity::Warn
    } else {
        Severity::Error
    };
    let sev = cfg.severity(RULE_BUDGET, default);
    if sev == Severity::Off {
        return Vec::new();
    }

    let mut out = Vec::new();
    let lines = s.body.lines().count();
    if lines > cfg.thresholds.body_max_lines {
        out.push(Finding {
            rule: RULE_BUDGET,
            severity: sev,
            msg: format!(
                "body is {lines} lines, over {}",
                cfg.thresholds.body_max_lines
            ),
        });
    }
    let token_count = tokens(&s.body);
    if token_count > cfg.thresholds.body_max_tokens {
        out.push(Finding {
            rule: RULE_BUDGET,
            severity: sev,
            msg: format!(
                "body is ~{token_count} tokens, over {}",
                cfg.thresholds.body_max_tokens
            ),
        });
    }
    out
}

fn has_references(s: &Skill) -> bool {
    s.files
        .iter()
        .any(|f| f.components().any(|c| c.as_os_str() == "references"))
}

fn dead_links(s: &Skill, cfg: &Config) -> Vec<Finding> {
    let sev = cfg.severity(RULE_DEAD_LINK, Severity::Error);
    if sev == Severity::Off {
        return Vec::new();
    }

    let unique_paths: std::collections::BTreeSet<String> =
        referenced_paths(&s.body).into_iter().collect();

    unique_paths
        .into_iter()
        .filter(|p| !s.dir.join(p).exists())
        .flat_map(|p| {
            finding(
                RULE_DEAD_LINK,
                sev,
                format!("referenced path does not exist: {p}"),
            )
        })
        .collect()
}

fn orphans(s: &Skill, cfg: &Config) -> Vec<Finding> {
    let sev = cfg.severity(RULE_ORPHAN, Severity::Warn);
    if sev == Severity::Off {
        return Vec::new();
    }
    s.files
        .iter()
        .filter_map(|f| f.file_name().and_then(|n| n.to_str()).map(String::from))
        .filter(|fname| !fname.is_empty() && !s.body.contains(fname.as_str()))
        .flat_map(|fname| {
            finding(
                RULE_ORPHAN,
                sev,
                format!("file never referenced by the body: {fname}"),
            )
        })
        .collect()
}

const SCRIPT_VERBS: [&str; 6] = ["run", "execute", "read", "call", "invoke", "./"];

fn script_intent(s: &Skill, cfg: &Config) -> Vec<Finding> {
    let sev = cfg.severity(RULE_SCRIPT_INTENT, Severity::Warn);
    if sev == Severity::Off {
        return Vec::new();
    }
    s.files
        .iter()
        .filter(|f| f.components().any(|c| c.as_os_str() == "scripts"))
        .filter_map(|f| f.file_name().and_then(|n| n.to_str()).map(String::from))
        .filter(|fname| {
            let mentions: Vec<&str> = s
                .body
                .lines()
                .filter(|l| l.contains(fname.as_str()))
                .collect();
            !mentions.is_empty()
                && !mentions.iter().any(|l| {
                    let lower = l.to_lowercase();
                    SCRIPT_VERBS.iter().any(|v| lower.contains(v))
                })
        })
        .flat_map(|fname| {
            finding(
                RULE_SCRIPT_INTENT,
                sev,
                format!("script '{fname}' is mentioned with no run or read verb nearby"),
            )
        })
        .collect()
}

const FILLER_PHRASES: [&str; 8] = [
    "as an ai",
    "please note",
    "it is important to note",
    "in today's world",
    "leverage synergies",
    "best practices dictate",
    "simply put",
    "needless to say",
];

fn mush(s: &Skill, cfg: &Config) -> Vec<Finding> {
    let sev = cfg.severity(RULE_MUSH, Severity::Warn);
    if sev == Severity::Off {
        return Vec::new();
    }
    let lower = s.body.to_lowercase();
    FILLER_PHRASES
        .iter()
        .filter(|p| lower.contains(*p))
        .flat_map(|p| finding(RULE_MUSH, sev, format!("generic filler phrase: \"{p}\"")))
        .collect()
}
