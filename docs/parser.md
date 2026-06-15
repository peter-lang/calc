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
returned position equals the token count (the whole input was consumed).

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
expression := expression "+" term
            | expression "-" term
            | "-" term                 (unary minus)
            | term

term       := term "*" exponent
            | term "/" exponent
            | term "to" unit           (unit conversion)
            | exponent

exponent   := atom "^" exponent        (right-associative; "**" also accepted)
            | atom

atom       := "(" expression ")"
            | num_unit                 (number with unit, possibly compound)
            | number                   (bare number, unitless)
            | unit                     (bare unit ⇒ quantity 1)

num_unit   := num_unit single_num_unit (compound, e.g. 5 m 10 cm)
            | single_num_unit

single_num_unit := number unit
```

Resulting precedence: `+ -` (lowest) < `* / to` < `^` < atoms. Unary minus is
handled at the `expression` level.

## Notable rules

- **`num_unit` (compound quantities).** Allows chains like `5 m 10 cm` or
  `1 h 30 min`. Each appended `single_num_unit` must share a `UnitType` with the
  accumulated left side (checked via `unit::common_type`); the pieces are folded
  together with `value_op::add`, so `5 m 10 cm` becomes `5 m + 10 cm` and
  evaluates to `5.1 m`.
- **`to` conversion.** `term "to" unit` builds a `value_op::conversion` node with
  the target unit (as a quantity-1 value) on the right.
- **Bare unit as `atom`.** A unit by itself parses as the value `1 <unit>`, so
  `EUR to USD` means "1 EUR to USD".
- **`expect_number` / `expect_unit`.** Leaf matchers that read a single literal
  or unit token. `expect_unit` is the big `Token → Unit` mapping, including the
  currency case, which validates the code against the sorted `CURRENCIES` array
  with `binary_search`.

## Adding or changing syntax

- **New operator** — add a token (see [lexer.md](lexer.md)), then add an
  alternative to the appropriate precedence rule, building a `Node::BinaryExpr`/
  `UnaryExpr` whose `op` is the matching `value_op` function. Add the operator to
  `debug.rs`'s `fmt_binary_op`/`fmt_unary_op` so the AST debug output names it.
- **New unit token** — extend `expect_unit`'s match arm (and the `Unit` enum /
  factors per [units.md](units.md)). No grammar change needed; `unit` is already
  an alternative in `atom` and `num_unit`.
- **New precedence level** — insert a rule between the existing ones and chain
  it; use `memoize_left_rec` if the rule references itself on the left, else
  `memoize`.
