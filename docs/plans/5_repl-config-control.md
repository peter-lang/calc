# Plan 5 — REPL configuration & session control

**Goal:** change config live in the REPL via `/`-commands, with session-scoped or
global (persisted) effect, and TAB completion of the available options.

**Depends on:** plan 2 (config holder must be runtime-mutable — flagged there).
Most useful once plans 3/4 give options worth toggling.

## Behaviour

- The REPL intercepts lines starting with `/` as **meta-commands**, before
  lexing. Unambiguous: a calc expression can't start with `/` (mid-expression `/`
  is `Div`). This is handled in `main.rs`, not the grammar.
- `/config <key.path> <value>` — set a config value for the **session**
  (in-memory override), e.g. `/config format.precision 6`.
- `/config --global <key.path> <value>` — also **persist** to `conf.toml`.
- `/config <key.path>` / `/config` — show current value(s).
- **TAB completion:** after `/config`, propose valid `key.path`s (and enum/bool
  values where applicable) via a rustyline `Completer`/`Helper` (rustyline 18).

## Design

- **Mutable config:** revisit plan 2's holder — needs session overrides over the
  loaded base. Likely `RwLock<Config>` or base + overlay map. `--global`
  serializes the merged config back to `conf.toml` (toml `to_string`).
- **Key registry:** completion + `/config` validation need to enumerate keys and
  their value types. Use a hand-maintained registry (or a small derive) mapping
  dotted paths → setter/validator, so unknown keys / bad values give clear errors.
- Session overrides are the same mechanism plan 4's `| fmt` uses transiently —
  reuse the `FormatOptions` merge.

## Steps

1. REPL command dispatch for `/`-prefixed lines (`main.rs`).
2. Runtime-mutable config + `/config <key> <value>` (parse, validate, set).
3. `--global` persistence: serialize merged config to `conf.toml`.
4. rustyline `Completer` proposing config key paths (+ values) on TAB.
5. Tests: drive the REPL via stdin (existing harness) — `/config format.precision 6`
   then an expression confirms the override; unit-test the completer/registry.

## Open questions

- Key-path syntax & value typing (dotted paths; how to parse/validate values).
- How the registry is built (manual table vs derive macro vs serde reflection).
- Should `/config --global` rewrite the whole file (losing comments) or edit in
  place (argues for `toml_edit`, a heavier dep)?

## Progress

- [ ] Step 1: `/`-command dispatch
- [ ] Step 2: mutable config + `/config` set
- [ ] Step 3: `--global` persist
- [ ] Step 4: TAB completion
- [ ] Step 5: tests
