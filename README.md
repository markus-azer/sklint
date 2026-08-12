# sklint

A static analyzer for agent skills. It reads every `SKILL.md` in a folder and reports the ones that break skill-authoring best practices.

## Install

```sh
# npm (any Node project). Downloads a prebuilt binary. Command is `sklint`.
pnpm add -D @markusazer/sklint
# or
npm i -D @markusazer/sklint

# from source (Rust)
cargo install sklint
```

Prebuilt binaries cover Linux x64/arm64, macOS x64/arm64, and Windows x64.

## Usage

```sh
sklint path/to/skills
```

```
✓ good-skill
✗ big-skill
    error  budget        body is 812 lines, over 500
    warn   desc-when     description may not say when to use it

2 skills · 1 error · 1 warn · exit 1
```

## How it works

Each skill is parsed once into its frontmatter, body, and bundled files. Every rule reads those parts and reports findings at one of three levels: `error`, `warn`, or `off`. The exit code follows the worst finding, so it fits into CI or a pre-commit hook.

## Checks

| rule | catches |
|------|---------|
| `name` | missing name, not kebab-case, or not matching the folder |
| `desc-length` | description over 1024 characters |
| `desc-when` | description that never says *when* to use the skill |
| `budget` | body over 500 lines (a warning if a `references/` folder exists) |
| `dead-link` | a referenced path that does not exist |
| `orphan` | a `scripts/` or `references/` file the body never mentions |
| `script-intent` | a script named with no run or read verb nearby |
| `mush` | generic filler phrases |

## Configure

Add an optional `sklint.toml` next to the skills. It tunes thresholds and per-rule severity.

```toml
[thresholds]
body_max_lines = 300

[rules]
mush = "off"          # off | warn | error
desc-when = "error"
```

Every key is optional. Anything you leave out keeps its default, and no file means all defaults.

## Exit codes

- `0` all skills clean
- `1` at least one error-level finding
- `2` could not run: no `SKILL.md` found, or a file could not be read

## License

MIT.
