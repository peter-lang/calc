# Plan 1 — Dependency upgrades

**Goal:** bring all dependencies up to current versions and align with current
community standards, accepting breaking changes and fixing the resulting
breakage. Work **one dependency at a time**, verifying after each.

**Status:** in progress (paused). See [Progress](#progress) at the bottom.

Plans in `docs/plans/` are numbered by priority; this is priority 1.

## Verification gate (run after every step)

```bash
cargo build
cargo test
# manual CLI smoke tests (cover lex → parse → eval → format end to end):
cargo run -- "1/2 + 1/3"        # 0.833333…
cargo run -- "5 m 10 cm to cm"  # 510 cm
cargo run -- "95 f to c"        # 35 C
cargo run -- "2^10"             # 1024
# currency path needs network the first time per day:
cargo run -- "100 EUR to USD"
```

A step is "done" only when build + tests pass and the smoke tests are unchanged.

## Prerequisites (DONE in working tree) — make tests a real safety net

Before changing any dependency, the suite must compile, pass, and actually cover
behaviour so upgrades can't regress it silently:

1. **Fixed test compilation.** The test target **never compiled** on `main`: the
   three conversion tests in `src/unit.rs` used `assert_eq!(result, Ok(Number::…))`
   on a `Result<Number, CalcError>`, which requires `CalcError: PartialEq`.
   `CalcError` wraps non-`PartialEq` errors (`io::Error`, `ureq::Error`, …) and
   cannot derive it. Fixed by comparing the unwrapped `Number`.
2. **Added CLI characterization tests** (`tests/cli.rs`). 12 end-to-end tests
   (plus 1 ignored currency smoke test) drive the built binary and assert on
   stdout, locking in current output for arithmetic, rationals, precedence,
   number formatting, every unit family's conversions, compound quantities,
   unit-arithmetic rules, and error messages. Zero new dependencies (std
   `Command` + `env!("CARGO_BIN_EXE_calc")`). Golden values were captured from
   the current binary. This is the regression gate for every step below.

## Audit (June 2026)

| Crate | Current | Latest | Decision |
|-------|---------|--------|----------|
| `serde` | 1.0.203 | 1.0.228 | ✅ healthy — minor bump |
| `regex` | 1.10.5 | 1.12.4 | ✅ healthy (rust-lang) — minor bump |
| `chrono` | 0.4.38 | 0.4.45 | ✅ **keep** — actively maintained; `NaiveDate`/`Utc` usage is idiomatic. (`time` is the alternative but switching is churn without benefit.) — minor bump |
| `thiserror` | 1.0.61 | 2.0.18 | ⚠️ upgrade to v2 (dtolnay; current standard) |
| `quick-xml` | 0.32 | 0.40 | ⚠️ upgrade (8 breaking 0.x releases behind) |
| `ureq` | 2.9.7 | 3.3.0 | ⚠️ upgrade to v3 (rewrite) |
| `rustyline` | 14.0 | 18.0 | ⚠️ upgrade (4 majors behind) |
| `variant_count` | 1.1.0 | 1.2.0 | 🔄 **swap → `strum` `EnumCount`** — niche single-purpose crate (~0.8M recent dl) vs `strum` the de-facto standard (~104M) |
| `directories` | 5.0.1 | 6.0.0 | 🔄 **swap → `etcetera`** — original repo archived since 2020; `etcetera` is the actively-maintained modern standard (~22M recent dl, updated Oct 2025). Note: relocates cache files (`history.txt`, `rates.xml`). |

## Ordered steps (low → high risk)

Each step = edit `Cargo.toml` (and source if needed) → run the verification gate
→ commit on its own.

### 1. Minor bumps — `serde`, `regex`, `chrono` (DONE in working tree)
Semver-compatible, no source changes. Grouped because none can break.

### 2. `thiserror` 1 → 2  — low risk
- Bump version. v2 is largely source-compatible.
- Watch: `#[error(transparent)]` and `#[from]` semantics (our usage is simple —
  expect zero code changes). v2 raised MSRV to 1.61.
- Files: `Cargo.toml`, possibly `src/error.rs`.

### 3. `variant_count` → `strum` `EnumCount`  — low/medium risk
- Replace dep: `strum = { version = "0.28", features = ["derive"] }`; drop
  `variant_count`.
- `src/parser/token.rs`: `#[derive(VariantCount)]` → `#[derive(strum::EnumCount)]`.
- `src/parser/lexer.rs`: `Token::VARIANT_COUNT` → `Token::COUNT` (the `EnumCount`
  trait const). Add `use strum::EnumCount;` where the const is referenced.
- Verify the `[(…); Token::COUNT]` table length still type-checks.

### 4. `directories` → `etcetera`  — medium risk
- Replace dep: `etcetera = "0.11"`; drop `directories`.
- `src/files.rs`: rewrite `cache()` using etcetera's base strategy, e.g.
  `choose_base_strategy()?.cache_dir().join("calc").join(name)`, keeping the
  `create_dir_all` behaviour. Aim to preserve the logical location
  (`~/Library/Caches/calc`, `~/.cache/calc`).
- Note in commit: cache-file location may change for existing users.
- Honor engineering principles: replace the existing `.unwrap()`s here with
  proper `Result`/`CalcError` flow if practical (cache() currently returns
  `PathBuf` and panics; consider returning `Result`).

### 5. `quick-xml` 0.32 → 0.40  — medium risk
- Bump version; confirm serde feature names (currently `["serde", "serialize"]`).
- `src/currency.rs`: verify `de::from_reader` / `de::from_str` signatures and the
  `@attr` / `$value` rename attributes still deserialize the MNB response.
- Test the currency path against a real/cached `rates.xml`.

### 6. `ureq` 2 → 3  — high risk (rewrite)
- Bump version. Expect API changes in `src/currency.rs::fetch_current_rate_xml`:
  - request building: `ureq::post(url).set(k, v)` → `.header(k, v)` /
    `.config()`,
  - body send: `.send_bytes(&b)` → `.send(&b)` (or `.send(body)`),
  - response read: `response.into_reader()` → `response.into_body().into_reader()`
    (or `.body_mut().as_reader()`).
- Check `CalcError::RequestError(#[from] ureq::Error)` still matches v3's error
  type (`src/error.rs`).

### 7. `rustyline` 14 → 18  — medium risk
- Bump version. Verify in `src/main.rs`: `DefaultEditor::new`, `readline`,
  `add_history_entry`, `load_history`, `save_history`, and the
  `ReadlineError::{Interrupted, Eof}` variants. API has been stable across these
  majors; expect little or no change.

## Notes / decisions

- **Keep `chrono`** (not switching to `time`).
- **Two crate swaps** are the only community-standards changes; everything else is
  version bumps.
- Follow the repo engineering principles while fixing breakage: prefer
  `Result<_, CalcError>` over panics/`unwrap`, keep changes minimal and
  self-documenting, update the relevant `docs/` page if behaviour changes (e.g.
  [docs/currency.md](../currency.md) for the cache-location change, and update the
  dependency notes in [CLAUDE.md](../../CLAUDE.md) / Cargo).

## Progress

- [x] Prerequisite: fix test compilation (`src/unit.rs`) — *working tree, uncommitted*
- [x] Prerequisite: CLI characterization tests (`tests/cli.rs`) — *working tree, uncommitted*
- [x] Step 1: minor bumps `serde`/`regex`/`chrono` — *working tree, uncommitted*
- [x] Step 2: `thiserror` 1 → 2 — *no source changes; transparent*
- [ ] Step 3: `variant_count` → `strum`
- [ ] Step 4: `directories` → `etcetera`
- [ ] Step 5: `quick-xml` 0.32 → 0.40
- [ ] Step 6: `ureq` 2 → 3
- [ ] Step 7: `rustyline` 14 → 18

> Resume point: commit the prerequisite test fix and Step 1 (currently staged in
> the working tree), then continue from Step 2.
