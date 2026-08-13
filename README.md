# sklint

A static analyzer for agent skills. It reads every `SKILL.md` in a folder and reports the ones that break skill-authoring best practices.

## Install

```sh
# prebuilt binary, no toolchain (macOS/Linux)
curl -fsSL https://raw.githubusercontent.com/markus-azer/sklint/main/install.sh | sh

# npm, as a global command
npm i -g @markusazer/sklint

# from source (Rust)
cargo install sklint
```

Or run it once without installing:

```sh
npx @markusazer/sklint path/to/skills
```

The command is always `sklint`. Prebuilt binaries cover Linux x64/arm64, macOS x64/arm64, and Windows x64.

## Usage

```sh
sklint                 # lint the configured paths (default .claude/skills)
sklint path/to/skills  # or point it at any directory
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

Add an optional `sklint.toml`. It sets where skills live, plus thresholds and per-rule severity.

```toml
paths = [".claude/skills"]   # directories to scan (default)

[thresholds]
body_max_lines = 300

[rules]
mush = "off"          # off | warn | error
desc-when = "error"
```

Every key is optional. Anything you leave out keeps its default. No file means all defaults. A path passed on the command line overrides `paths`.

## Exit codes

- `0` all skills clean
- `1` at least one error-level finding
- `2` could not run: no `SKILL.md` found, or a file could not be read

## License

MIT.
