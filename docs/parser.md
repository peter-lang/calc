# Parser & grammar

File: [`src/parser/parser.rs`](../src/parser/parser.rs)

## Overview

The parser is a hand-written **packrat (memoizing) recursive-descent parser**
with explicit support for **left recursion**. It consumes the `Vec<Token>`
accumulated by the REPL/CLI and produces a [`Node`](../src/node.rs) AST, or
`None` if the tokens don't form a complete, fully-consumed expression.

Each rule is a method that takes a start position `pos: usize` and returns:

```rust
enum Match<T> { Ok(T, usize), Err }   // Ok(value, next_position)
```

`Parser::parse` calls the top rule `expression(0)` and only succeeds if the
returned position equals the token count (or a trailing `| formatter` clause
consumes the remainder). It returns `Option<(Node, Option<FormatSpec>)>`: the
AST node and an optional per-expression format override.

## Memoization and left recursion

Two helpers wrap each rule:

- **`memoize`** — plain packrat caching keyed by `(pos, rule_name)`. Used by
  rules that are not left-recursive (`atom`, `exponent`).
- **`memoize_left_rec`** — the "seed-and-grow" algorithm for **left-recursive**
  rules. It seeds the memo with `Err`, then repeatedly re-runs the rule, each
  time letting it consume one more level, until the match stops growing. Used by
  `expression`, `term`, and `num_unit`.

This is why the grammar can be written in its natural left-associative form
(e.g. `expression -> expression + term`) without infinite recursion. The memo
table is cleared at the start of every `parse()` call.

> Performance note: `parse()` clears and rebuilds memos on each call, and
> `Match`/`Node` are cloned freely. There are `// TODO` markers about reusing
> previous computations. Inputs are short, so this hasn't mattered.

## Grammar (precedence, loosest → tightest)

```
input      := expression ("|" formatter [precision])?

expression := expression "+" term
            | expression "-" term
            | "-" term                 (unary minus)
            | term

term       := term "*" exponent
            | term "/" exponent
            | term "to" unit           (unit conversion)
            | term unit                (implicit ×1·unit, e.g. (2*3) eur)
            | exponent

exponent   := atom "^" exponent        (right-associative; "**" also accepted)
            | atom

atom       := "(" expression ")"
            | "ans"                    (last result, initially 0)
            | num_unit                 (number with unit, possibly compound)
            | number                   (bare number, unitless)
            | unit                     (bare unit ⇒ quantity 1)

num_unit   := single_num_unit+         (same-group quantities summed)
single_num_unit := number unit

formatter  := "fixed" | "float" | "sci" | "fin" | "financial" | "rat" | "rational"
precision  := integer (0–255)
```

Resulting precedence: `+ -` (lowest) < `* / to` < `^` < atoms. Unary minus is
handled at the `expression` level. The `|` clause is not part of the expression
grammar — it is matched by `parse_format_clause` after the expression consumes
all it can.

## Notable rules

- **`num_unit` (compound quantities).** The "feet-and-inches" notation. Adjacent
  quantities are summed when their units share a **compound group**
  (`unit::compound_group`): `{m, cm}`, `{ft, in}`, `{h, min, s}`. The pieces fold
  left with `value_op::add`, so `5 m 10 cm` → `5.1 m` and `1 h 30 min 15 s`
  chains N-way. Units outside any group (`5 kg 10 g`) don't compound and fail to
  parse. User-defined groups are a planned config feature.
- **Bare-unit multiplication.** Juxtaposing a value with a bare unit is implicit
  multiplication by `1·unit` (`term := term unit`, multiplication precedence), so
  `(2*3) eur` → `6 eur` and `1/2 eur` → `0.5 eur`. Two united values therefore
  multiply and error if incompatible; raising a united value to a power is also
  rejected — see [units.md](units.md).
- **`to` conversion.** `term "to" unit` builds a `BinaryOp::Conversion` node with
  the target unit (as a quantity-1 value) on the right.
- **Bare unit as `atom`.** A unit by itself parses as the value `1 <unit>`, so
  `EUR to USD` means "1 EUR to USD".
- **`ans`.** `Parser` carries `ans: Value`, initialized to `0` and updated by
  `set_ans` (called from `main.rs`) after each successful eval. When `atom`
  sees `KwAns` it immediately substitutes `Node::Value(self.ans.clone())`, so
  `ans` is resolved at parse time and `eval()` sees only a plain value. Eval
  errors do not update `ans`.
- **`expect_number` / `expect_unit`.** Leaf matchers that read a single literal
  or unit token. `expect_unit` is the big `Token → Unit` mapping, including the
  currency case, which validates the code against the sorted `CURRENCIES` array
  with `binary_search`.
- **`parse_format_clause` / `expect_precision`.** Called by `parse()` after a
  successful expression parse, when tokens remain. Tries `Token::Pipe` followed
  by a formatter keyword (`KwFixed`, `KwFloat`, `KwSci`, `KwFin`, `KwRat`),
  then an optional `LitInt(0–255)` precision
  override. Returns `Option<(FormatSpec, next_pos)>`. An unknown name or
  leftover tokens after the clause cause `parse()` to return `None`.

## `FormatSpec`

```rust
pub enum FormatSpec {
    Fixed     { precision: Option<u8> },
    Float,
    Sci       { precision: Option<u8> },
    Rational,
    Financial { precision: Option<u8> },
}
```

Defined in `config.rs` alongside `FormatOptions`. The spec is not attached to
the `Node` or evaluated by `eval()` — it is purely a rendering hint. `main.rs`
calls `config::apply_spec(&guard.format, &spec)` to produce a one-off
`FormatOptions` that overrides `repr` (and the relevant precision field if `N`
was given), then passes it to `format_value`. See [numbers.md](numbers.md).

## Adding or changing syntax

- **New operator** — add a token (see [lexer.md](lexer.md)), then add an
  alternative to the appropriate precedence rule, building a `Node::BinaryExpr`/
  `UnaryExpr` whose `op` is the matching `BinaryOp`/`UnaryOp` variant. Add the
  variant and its `apply`/`symbol` arms in `value_op.rs` (the compiler enforces
  exhaustiveness, so the debug output names it automatically).
- **New unit token** — extend `expect_unit`'s match arm (and the `Unit` enum /
  factors per [units.md](units.md)). No grammar change needed; `unit` is already
  an alternative in `atom` and `num_unit`.
- **New precedence level** — insert a rule between the existing ones and chain
  it; use `memoize_left_rec` if the rule references itself on the left, else
  `memoize`.
