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

Operators in the AST are **enum variants** (`BinaryOp`/`UnaryOp` in `value_op`);
`eval()` calls `op.apply(...)`. A `Value` is a `Number`
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
- **Operators are the `BinaryOp`/`UnaryOp` enums** in `value_op.rs`; add a variant
  plus its `apply`/`symbol` arms (exhaustiveness-checked) when adding an operator.

## Engineering principles

These are the rules to follow for any change. They take precedence over matching
local style where the two conflict.

- **Encode requirements in the type system first.** Make illegal states
  unrepresentable and let invariants be checked at compile time (types, enums,
  `const` assertions). If a behaviour genuinely cannot be expressed statically,
  encode it in a test. **Do not** document an "unreachable"/"can't happen" path
  in a comment — restructure the types so the path is impossible, or assert it.
- **No risky constructs.** The whole point of using Rust is correctness through
  its type system. Avoid `unsafe`, and avoid panics on reachable paths
  (`unwrap`/`expect`/`panic!`/indexing that can go out of bounds / silent
  `as` truncation). Return `Result<_, CalcError>` instead. Existing spots that
  violate this (e.g. `unwrap`s in `lexer.rs`, `panic!` in `rational.rs::new` and
  `number.rs::to_rational`) are tech debt — migrate them when you touch that code
  rather than copying the pattern.
- **Code should be self-documenting; keep comments sparse.** Prefer clear names
  and structure over explanatory comments. Document higher-level behaviour in
  [`docs/`](docs/README.md), not in inline prose. When you add or change
  behaviour, update the relevant design doc.
- **Errors flow as `Result<_, CalcError>`**; the REPL prints eval errors and
  continues.
- **Keep arithmetic exact** (`Int`/`Rational`) and only fall back to `Float`
  when unavoidable — match the promotion pattern in `number_op.rs`.
- Small, dependency-light, std-plus-a-few-crates code; keep new code plain and
  explicit (no heavy abstraction).

## Testing strategy

- **Keep coverage high, but prefer high-level tests.** Default to
  integration/system tests that drive the program through the **main CLI
  interface** (`calc <expr>` → assert on printed output), since that exercises
  lex → parse → eval → format end to end.
- **Reach for unit tests only when the logic is complex enough that local
  edge-case coverage is worth it** (e.g. `Number` promotion in `number_op.rs`,
  `Rational` normalization, `Number` `Display` rounding, unit conversion
  factors). The existing `#[cfg(test)]` modules in `rational.rs` and `unit.rs`
  are examples of this justified-unit-test case.
- Before adding a unit test, ask whether the invariant could instead be a
  compile-time guarantee (see principles above) or covered by a CLI-level test.

## Planning (`docs/plans/`)

Any complex or multi-step task gets a written plan in [`docs/plans/`](docs/plans/)
before/while doing the work — this is also where future implementation and design
plans live, so they aren't lost.

- **Keep the plan updated as you go.** Tick off steps and record decisions /
  surprises as the task progresses, so the file always reflects current state and
  is a usable resume point.
- **Prioritize with a numeric prefix:** `1_`, `2_`, `3_`, … The number is the
  priority/order; lower runs first.
- **Need an immediate plan to do *right now*** without renumbering the queue? Use
  a `0_` or `__` prefix so it sorts ahead of the numbered backlog and you don't
  have to reprioritize everything else.
- **Delete a plan once it's finished** — completed plans don't linger; the git
  history keeps the record. (So don't link to a specific plan file from long-lived
  docs; it may be gone.)

A plan typically has: goal, a verification gate (how you'll know each step is
done), ordered steps with risk/notes, and a progress checklist used as the resume
point.

## When in doubt, read the design docs

Pull up the matching doc before changing these areas:

- Tokenizing / number suffixes / adding a unit spelling → [docs/lexer.md](docs/lexer.md)
- Grammar / precedence / operators / syntax → [docs/parser.md](docs/parser.md)
- Number types / exactness / **output formatting** → [docs/numbers.md](docs/numbers.md)
- Units / conversion factors / temperature → [docs/units.md](docs/units.md)
- Exchange rates / currency codes / caching → [docs/currency.md](docs/currency.md)
- AST / `eval` / unit-aware operators → [docs/evaluation.md](docs/evaluation.md)
- Overall data flow / module layout / error handling → [docs/architecture.md](docs/architecture.md)
