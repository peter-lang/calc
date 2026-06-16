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
   holds an **operator enum** (`BinaryOp`/`UnaryOp`) whose `apply()` is called on
   the evaluated children. See [evaluation.md](evaluation.md).
4. **Display** — the resulting `Value` is printed via its `Display` impl,
   which delegates number formatting to [`Number`](numbers.md) and unit naming
   to `unit::get_unit_name`.

## Module map

| File | Responsibility |
|------|----------------|
| `src/main.rs` | Entry point: `config::init` then dispatch to `repl::run` (REPL) or `repl::run_once` (one-shot) |
| `src/repl.rs` | REPL loop, one-shot evaluation, `/config` meta-commands, and `rustyline` line-editing (completion + hints) |
| `src/config/mod.rs` | Config data types, defaults, load/persist, the live `RwLock<Config>`, `FormatSpec`/`apply_spec` |
| `src/config/registry.rs` | `REGISTRY` of settable keys: dotted path → getter/setter/completions; value parsers |
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
| `src/currency/mod.rs` | Live exchange-rate fetch (MNB SOAP), caching, conversion |
| `src/error.rs` | `CalcError` (via `thiserror`) |
| `src/files.rs` | Platform cache- and config-directory resolution |
| `src/debug.rs` | `Debug` impls for `Number` and `Node` (pretty-prints the AST) |

## The two run modes (`repl.rs`)

`main.rs` only initializes config and chooses a mode; both live in `repl.rs` and
share `evaluate_and_print` (eval → format with the current `FormatOptions` →
print), so output behaviour is defined in exactly one place.

- **REPL** (`repl::run`, no args): uses `rustyline` for line editing and
  persistent history (`history.txt` in the cache dir). Input is fed to the parser
  **incrementally** — each line's tokens are appended with `parser.extend(...)`,
  and `parser.parse()` is retried after every line. It returns `None` while the
  expression is still incomplete (e.g. an unclosed paren), and the prompt switches
  from `>> ` to `.. ` to signal continuation. `Ctrl-C`/`Ctrl-D` cancels a partial
  expression or, when the buffer is empty, exits.
- **One-shot** (`repl::run_once`, args present): `args[1..]` are joined with
  spaces, lexed, parsed, evaluated, and printed once.

Both modes share the same lexer/parser/eval path; only input handling differs.

## Configuration & `/config` meta-commands

Config is loaded once at startup into a process-global `RwLock<Config>`
(`config/mod.rs`); `config::current()` hands out a read guard and formatting reads
from it. The on-disk file (`conf.toml` in the config dir, or `$CALC_CONFIG`) is
bootstrapped from a commented template on first run.

In the REPL, a line beginning with `/` is intercepted **before** lexing (a calc
expression never starts with `/`) and routed to `handle_meta_command`:

- `/config` — print every settable key and its current value.
- `/config <key>` — print one key's value (`Hinter` also shows it inline as you
  finish typing a known key).
- `/config <key> <value>` — set it for this session (in-memory).
- `/config global <key> <value>` — set **and** persist the merged config back to
  the TOML file.

The settable keys live in `config/registry.rs` as a `REGISTRY` table mapping a
dotted path to a getter, a setter (with a value parser/validator), and the
completion candidates. `rustyline` TAB completion walks this table one dot-level
at a time; unknown keys and bad values produce a clear message and the loop
continues.

## Error handling

Every fallible operation returns `Result<_, CalcError>`
([`src/error.rs`](../src/error.rs)). `CalcError` carries both **domain errors**
(`DivByZero`, `DifferentUnitTypes`, `ConversionError`, `ExpByUnit`,
`OperateWithUnits`, `MissingUnit`) and **wrapped library errors** (`ureq`,
`rustyline`, `std::io`, `quick_xml`) via `#[from]`. At the REPL, an evaluation
error is printed and the loop continues; it is not fatal.

## Design notes worth knowing

- **Operators are enum variants.** `Node::BinaryExpr` stores `op: BinaryOp` (and
  `UnaryExpr` stores `UnaryOp`); each enum has `apply()` for evaluation and
  `symbol()` for debug output, both exhaustively matched in `value_op.rs`. This
  keeps the AST tiny while giving every node a known operator identity.
- **Exactness first.** Integer and rational arithmetic stay exact; the code only
  promotes to `f64` when it must (overflow, irrational result, or a float
  operand). See [numbers.md](numbers.md).
- **`Cargo.lock` is committed** — this is a binary crate, so the lockfile is
  tracked for reproducible builds.
