use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn cmd(name: &str) -> Command {
    let mut c = Command::cargo_bin("sklint").unwrap();
    c.arg(fixture(name));
    c.env("NO_COLOR", "1");
    c
}

fn temp_dir(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("sklint-{tag}-{}", std::process::id()))
}

#[test]
fn pass_is_exit_zero() {
    cmd("pass/good-skill")
        .assert()
        .success()
        .stdout(contains("✓ good-skill"));
}

#[test]
fn name_mismatch_fails() {
    cmd("fail-name-mismatch")
        .assert()
        .failure()
        .stdout(contains("does not match folder"));
}

#[test]
fn over_budget_fails() {
    cmd("fail-budget")
        .assert()
        .failure()
        .stdout(contains("budget"));
}

#[test]
fn dead_link_fails() {
    cmd("fail-dead-link")
        .assert()
        .failure()
        .stdout(contains("dead-link"))
        .stdout(contains("missing.md"));
}

#[test]
fn missing_when_phrase_warns() {
    cmd("warn-desc-when")
        .assert()
        .success()
        .stdout(contains("desc-when"));
}

#[test]
fn unreferenced_file_warns_orphan() {
    cmd("warn-orphan")
        .assert()
        .success()
        .stdout(contains("orphan"))
        .stdout(contains("unused.sh"));
}

#[test]
fn script_without_verb_warns() {
    cmd("warn-script-intent")
        .assert()
        .success()
        .stdout(contains("script-intent"))
        .stdout(contains("build.sh"));
}

#[test]
fn filler_phrase_warns_mush() {
    cmd("warn-mush").assert().success().stdout(contains("mush"));
}

#[test]
fn config_promotes_mush_to_error() {
    cmd("severity-error")
        .assert()
        .failure()
        .stdout(contains("mush"));
}

#[test]
fn config_turns_rule_off() {
    cmd("rule-off")
        .assert()
        .success()
        .stdout(contains("desc-when").not());
}

#[test]
fn references_folder_softens_budget_to_warn() {
    cmd("budget-refs-warn")
        .assert()
        .success()
        .stdout(contains("budget"));
}

#[test]
fn description_over_byte_limit_but_under_char_limit_is_clean() {
    cmd("desc-chars")
        .assert()
        .success()
        .stdout(contains("desc-length").not());
}

#[test]
fn multi_skill_reports_each_and_totals() {
    cmd("multi-skill")
        .assert()
        .failure()
        .stdout(contains("alpha"))
        .stdout(contains("beta"))
        .stdout(contains("does not match folder"))
        .stdout(contains("2 skills"));
}

#[test]
fn invalid_config_exits_2() {
    cmd("invalid-config")
        .assert()
        .code(2)
        .stderr(contains("invalid sklint.toml"));
}

#[test]
fn unreadable_skill_md_exits_2() {
    let dir = temp_dir("unreadable");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("demo")).unwrap();
    std::fs::write(dir.join("demo/SKILL.md"), [0xff, 0xfe, 0xff]).unwrap();

    let assert = Command::cargo_bin("sklint")
        .unwrap()
        .arg(&dir)
        .env("NO_COLOR", "1")
        .assert();

    let _ = std::fs::remove_dir_all(&dir);
    assert.code(2).stderr(contains("cannot read"));
}

#[cfg(unix)]
#[test]
fn symlink_cycle_terminates() {
    let dir = temp_dir("symlink-cycle");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("demo/scripts")).unwrap();
    std::fs::write(
        dir.join("demo/SKILL.md"),
        "---\nname: demo\ndescription: Use when testing.\n---\nBody.\n",
    )
    .unwrap();
    std::os::unix::fs::symlink(&dir, dir.join("demo/scripts/loop")).unwrap();

    let out = Command::cargo_bin("sklint")
        .unwrap()
        .arg(&dir)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();

    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.code().is_some(),
        "process should terminate normally, not crash on a symlink cycle"
    );
}

#[test]
fn no_skills_exits_2() {
    let dir = temp_dir("empty");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let assert = Command::cargo_bin("sklint")
        .unwrap()
        .arg(&dir)
        .env("NO_COLOR", "1")
        .assert();

    let _ = std::fs::remove_dir_all(&dir);
    assert.code(2).stderr(contains("no SKILL.md"));
}

const VALID_SKILL: &str = "---\nname: widget\ndescription: Use when widgeting.\n---\nBody.\n";

#[test]
fn no_arg_lints_default_claude_skills() {
    let dir = temp_dir("default-loc");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(".claude/skills/widget")).unwrap();
    std::fs::write(dir.join(".claude/skills/widget/SKILL.md"), VALID_SKILL).unwrap();

    let assert = Command::cargo_bin("sklint")
        .unwrap()
        .current_dir(&dir)
        .env("NO_COLOR", "1")
        .assert();

    let _ = std::fs::remove_dir_all(&dir);
    assert
        .success()
        .stdout(contains("widget"))
        .stdout(contains("1 skills"));
}

#[test]
fn config_paths_override_default_location() {
    let dir = temp_dir("cfg-paths");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("myskills/widget")).unwrap();
    std::fs::write(dir.join("myskills/widget/SKILL.md"), VALID_SKILL).unwrap();
    std::fs::create_dir_all(dir.join(".claude/skills/decoy")).unwrap();
    std::fs::write(dir.join(".claude/skills/decoy/SKILL.md"), VALID_SKILL).unwrap();
    std::fs::write(dir.join("sklint.toml"), "paths = [\"myskills\"]\n").unwrap();

    let assert = Command::cargo_bin("sklint")
        .unwrap()
        .current_dir(&dir)
        .env("NO_COLOR", "1")
        .assert();

    let _ = std::fs::remove_dir_all(&dir);
    assert
        .stdout(contains("1 skills"))
        .stdout(contains("decoy").not());
}

#[test]
fn help_renders() {
    Command::cargo_bin("sklint")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("sklint"));
}

#[test]
fn output_is_deterministic() {
    let first = cmd("multi-skill").output().unwrap().stdout;
    for _ in 0..4 {
        let next = cmd("multi-skill").output().unwrap().stdout;
        assert_eq!(
            first, next,
            "output differs between runs (iteration order leaked into stdout)"
        );
    }
}
