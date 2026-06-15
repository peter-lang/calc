# CLAUDE.md

Guidance for working in this repo. Full documentation lives in [`docs/`](docs/README.md);
this file is the orientation + "where do I change X" map.

## What this is

`calc` is a Rust command-line calculator with three notable features beyond
arithmetic:

- **Exact rational arithmetic** — integers/fractions stay exact, only falling
  back to `f64` when forced.
- **Physical units** — length, area, volume, mass, temperature, time, with
  conversions (`5 m 10 cm to cm`, `95 f to c`).
- **Live currency conversion** — exchange rates fetched from the Hungarian
  National Bank (MNB), cached on disk.

Runs as a REPL (`cargo run`) or one-shot (`cargo run -- 2^10`).

## Build / run / test

```bash
cargo build
cargo run                 # REPL (rustyline, persistent history)
cargo run -- <expr>       # evaluate once, e.g. cargo run -- 1/2 + 1/3
cargo test                # tests live in rational.rs and unit.rs
```

`Cargo.lock` is committed (binary crate).

## The pipeline (always think in these stages)

```
text → Lexer → [Token] → Parser → Node (AST) → eval() → Value → Display
```

Operators in the AST are **function pointers** into `value_op` — there is no
operator enum. `eval()` just applies the stored `fn`. A `Value` is a `Number`
(`Int`/`Rational`/`Float`) plus an optional `Unit`.

## Module map

| File | Role | Docs |
|------|------|------|
| `src/main.rs` | REPL + one-shot entry point | [architecture](docs/architecture.md) |
| `src/parser/lexer.rs` | string → tokens (one combined regex) | [lexer](docs/lexer.md) |
| `src/parser/token.rs` | `Token` enum, currency list/regex | [lexer](docs/lexer.md) |
| `src/parser/parser.rs` | tokens → AST; **grammar** | [parser](docs/parser.md) |
| `src/node.rs` | `Node` AST + `eval()` | [evaluation](docs/evaluation.md) |
| `src/value.rs` / `value_op.rs` | `Value` and unit-aware operators | [evaluation](docs/evaluation.md) |
| `src/number.rs` / `number_op.rs` | `Number`, arithmetic, **output formatting** | [numbers](docs/numbers.md) |
| `src/rational.rs` | exact `Rational` | [numbers](docs/numbers.md) |
| `src/unit.rs` | units, conversion factors, combining rules | [units](docs/units.md) |
| `src/currency.rs` | MNB fetch, caching, HUF-base conversion | [currency](docs/currency.md) |
| `src/error.rs` | `CalcError` (`thiserror`) | — |
| `src/files.rs` | cache-dir paths (history, rates) | [currency](docs/currency.md) |
| `src/debug.rs` | `Debug` for `Number`/`Node` (AST dump) | [evaluation](docs/evaluation.md) |

## Where to change X

- **Output formatting** (precision, `k`/`m` suffixes, sci-notation, the `…`
  approximation marker): `Display for Number` in `src/number.rs`. See
  [numbers](docs/numbers.md).
- **Add a unit**: token + regex in `lexer.rs`/`token.rs`, then `Unit` enum and
  the four parallel matches — `get_default_factor`, `get_unit_name`,
  `get_unit_type` (`unit.rs`) and `expect_unit` (`parser.rs`). Checklist in
  [units](docs/units.md).
- **Add a currency**: update **both** `CURRENCIES` (keep sorted, bump length)
  and `CURRENCIES_PATTERN` in `token.rs`. See [currency](docs/currency.md).
- **Add/change an operator or syntax**: add a token, extend the right precedence
  rule in `parser.rs`, point its `Node` at a `value_op` fn, and register the fn
  in `debug.rs`'s `fmt_*_op`. See [parser](docs/parser.md) and
  [evaluation](docs/evaluation.md).
- **How an operator treats units**: `src/value_op.rs`. **How numbers promote/stay
  exact**: `src/number_op.rs`.
- **Conversion logic** (factors / temperature / currency dispatch): `unit.rs`
  `convert` / `get_default_factor` / `convert_temp`.

## Gotchas

- **Longest-match lexing.** The lexer is one big regex alternation; order in the
  `PATTERNS` table matters (3-char units before 2-char before 1-char, floats
  before ints). Adding a token in the wrong spot makes it unreachable.
- **`m` is overloaded** — number suffix "million" vs. unit "metre"; resolved by
  lexer context.
- **Parser keeps the natural grammar via a left-recursion-aware packrat parser**
  (`memoize_left_rec`). The memo table is rebuilt every `parse()` (there are
  `// TODO`s about reuse).
- **No derived/compound units.** `100 km / 2 h` errors (`OperateWithUnits`);
  multiplying/dividing two united values is rejected by `unit::single`.
- **Currency needs network the first time per day**; otherwise it serves a cached
  `rates.xml` (valid if dated today/yesterday).
- **Four parallel `match`es over `Unit`** must stay in sync; a missing
  `get_default_factor` arm silently yields `ConversionError` at runtime.
- **Operators are fn pointers**, so `debug.rs` identifies them by pointer
  comparison — new operators must be added there to print as anything but `?`.

## Conventions

- Errors flow as `Result<_, CalcError>`; REPL prints eval errors and continues.
- Prefer keeping arithmetic exact (`Int`/`Rational`) and only fall back to
  `Float` when unavoidable — match the existing promotion pattern in
  `number_op.rs`.
- This is small, dependency-light, std-plus-a-few-crates code; keep new code in
  the same plain, explicit style (no heavy abstraction).
