# Architecture & data flow

## The pipeline

```
                 src/parser/lexer.rs      src/parser/parser.rs    src/node.rs
raw string  ───▶  Lexer::parse  ───▶  [Token]  ───▶  Parser::parse  ───▶  Node  ───▶  Node::eval  ───▶  Value
                  (one big regex)      token.rs       (packrat PEG)         (AST)       (recursive)        value.rs
```

1. **Lex** — `Lexer::parse` turns the input string into an iterator of
   [`Token`](../src/parser/token.rs)s using a single combined regex. See
   [lexer.md](lexer.md).
2. **Parse** — `Parser` consumes the token vector and produces a
   [`Node`](../src/node.rs) tree. It is a memoizing (packrat) parser that also
   supports left recursion, so the grammar can be written naturally. See
   [parser.md](parser.md).
3. **Evaluate** — `Node::eval` walks the tree bottom-up. Each operator node
   holds a **function pointer** (e.g. `value_op::add`) that is applied to the
   evaluated children. See [evaluation.md](evaluation.md).
4. **Display** — the resulting `Value` is printed via its `Display` impl,
   which delegates number formatting to [`Number`](numbers.md) and unit naming
   to `unit::get_unit_name`.

## Module map

| File | Responsibility |
|------|----------------|
| `src/main.rs` | Entry point: REPL loop and one-shot CLI mode |
| `src/parser/lexer.rs` | String → tokens (regex table) |
| `src/parser/token.rs` | `Token` enum, currency name list & regex |
| `src/parser/parser.rs` | Tokens → AST (`Node`); grammar lives here |
| `src/parser/mod.rs` | Re-exports the three parser submodules |
| `src/node.rs` | `Node` AST type and `eval()` |
| `src/value.rs` | `Value` = `Number` + optional `Unit`; conversions from primitives; `Display` |
| `src/value_op.rs` | Operators on `Value`s (combine numbers **and** units) |
| `src/number.rs` | `Number` enum (`Int`/`Rational`/`Float`); output formatting |
| `src/number_op.rs` | Arithmetic on `Number`s with type promotion |
| `src/rational.rs` | Exact `Rational` (num/den) with gcd normalization |
| `src/unit.rs` | `Unit`/`UnitType` enums, conversion factors, `convert`, unit combining |
| `src/currency.rs` | Live exchange-rate fetch (MNB SOAP), caching, conversion |
| `src/error.rs` | `CalcError` (via `thiserror`) |
| `src/files.rs` | Platform cache-directory resolution |
| `src/debug.rs` | `Debug` impls for `Number` and `Node` (pretty-prints the AST) |

## The two run modes (`main.rs`)

- **REPL** (no args): uses `rustyline` for line editing and persistent history
  (`history.txt` in the cache dir). Input is fed to the parser **incrementally**
  — each line's tokens are appended with `parser.extend(...)`, and `parser.parse()`
  is retried after every line. It returns `None` while the expression is still
  incomplete (e.g. an unclosed paren), and the prompt switches from `>> ` to
  `.. ` to signal continuation. `Ctrl-C`/`Ctrl-D` cancels a partial expression
  or, when the buffer is empty, exits.
- **One-shot** (args present): `args[1..]` are joined with spaces, lexed,
  parsed, evaluated, and printed once.

Both modes share the same lexer/parser/eval path; only input handling differs.

## Error handling

Every fallible operation returns `Result<_, CalcError>`
([`src/error.rs`](../src/error.rs)). `CalcError` carries both **domain errors**
(`DivByZero`, `DifferentUnitTypes`, `ConversionError`, `ExpByUnit`,
`OperateWithUnits`, `MissingUnit`) and **wrapped library errors** (`ureq`,
`rustyline`, `std::io`, `quick_xml`) via `#[from]`. At the REPL, an evaluation
error is printed and the loop continues; it is not fatal.

## Design notes worth knowing

- **Operators are function pointers, not enum variants.** `Node::BinaryExpr`
  stores `op: fn(Value, Value) -> Result<Value, CalcError>`. This keeps the AST
  tiny and the evaluator trivial, but it means a node has no human-readable
  operator name — `debug.rs` reconstructs the symbol by comparing the pointer
  against each known `value_op` function.
- **Exactness first.** Integer and rational arithmetic stay exact; the code only
  promotes to `f64` when it must (overflow, irrational result, or a float
  operand). See [numbers.md](numbers.md).
- **`Cargo.lock` is committed** — this is a binary crate, so the lockfile is
  tracked for reproducible builds.
