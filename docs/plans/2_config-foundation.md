# Plan 2 — Configuration foundation

**Goal:** load, persist, and expose a user config file. This is the base that
plans 3–6 build on; it adds the mechanism, not yet much behaviour.

**Depends on:** nothing. **Blocks:** 3, 4, 5, 6.

## Decisions (agreed)

- **Format:** the full `toml` crate.
- **Default path:** `config_dir()/calc/conf.toml` → `~/.config/calc/conf.toml`,
  via a new `files::config()` helper (etcetera `BaseStrategy::config_dir()`).
- **Override:** env var `CALC_CONFIG=/path/to/conf.toml` wins over the default.
- **Missing file = defaults** (not an error). Malformed file = a `CalcError`.
- **First-run bootstrap:** if no file exists at the default path, write a default
  `conf.toml` (an embedded, commented template — not just serialized defaults, so
  it documents the options). Later plans extend the template (e.g. plan 4 adds the
  `fin` formatter). **On any invocation** (one-shot or REPL), not just the REPL.
- Every field uses `#[serde(default)]` + a `Default` impl, so partial/empty files
  are valid and "missing = defaults" is type-safe.

## Storage / access

Expose the parsed config process-wide. **Note:** plan 5 needs to mutate config at
runtime (`/config` session overrides + `--global` persist), so don't lock it into
a write-once `OnceLock`. Design the holder now to allow later override — e.g.
`RwLock<Config>` or a base + session-overlay — so plan 5 doesn't force a rewrite.
`Display for Number` (plan 3) reads the current config through this accessor.

## Steps

1. Add `toml` dep; create `src/config.rs` with `Config` (`#[serde(default)]`,
   `Default`) — minimal to start; plans 3/4/6 add the `[format]` / `[currency]`
   sub-tables.
2. `files::config()` helper.
3. Load: `CALC_CONFIG` else `files::config()`; parse; absent → `Default`; parse
   error → new `CalcError::ConfigError`.
4. Runtime holder + accessor (`config::current()`), revisable by plan 5.
5. First-run: write the default template to the default path when absent.

## Verification

`cargo build --all-targets` (0 warnings) + `cargo test`. New tests (isolated
`HOME`/temp like `tests/cli.rs`): default load, `CALC_CONFIG` temp file honoured,
missing file → defaults, malformed → error, first-run writes the template.

## Open questions

- Only bootstrap the default-location file, never when `CALC_CONFIG` is set?
  (Decided: write on any invocation; this sub-point is just about the `CALC_CONFIG`
  case.)

## Progress

- [ ] Step 1: `toml` dep + `Config`
- [ ] Step 2: `files::config()`
- [ ] Step 3: load logic + `ConfigError`
- [ ] Step 4: runtime holder + accessor
- [ ] Step 5: first-run template write
