# Plan 4 — `|` format operator

**Goal:** per-calculation output formatting via a trailing `|` clause.

**Depends on:** plan 3 (formatting engine), plan 2 (config + default template).

## Behaviour

A trailing `| <formatter> [N]` overrides formatting for that one result, where
the optional integer `N` overrides the precision for that expression only:

| Input | Output |
|-------|--------|
| `1/3 \| rat` (or `rational`) | `1/3` |
| `1/3 \| fixed 6` | `0.333333…` |
| `1/1000000 \| sci` | `1e-6` |
| `1234567 \| fin` | `1.23…m` |
| `1234567 \| fin 0` | `1m` |
| `1500000.5 \| float` | `1.5000…e6` |
| `1500000.5 \| fixed` | `1500000.5` |

- **`|` is a new token** — confirmed unused in the lexer today.
- Built-in formatters mirror `NumberRepr` from `config.rs`:
  `fixed`, `float`, `sci`, `rat`/`rational`, `fin`.
- Optional trailing integer sets precision for that expression:
  `| sci 6`, `| fin 2`, `| fixed 4`.
- `float` and `fixed` are distinct: `float` honours `float.sci_upgrade` thresholds;
  `fixed` is always fixed-point with no upgrade.

## Design

`FormatSpec` is defined in terms of `NumberRepr` (from `config.rs`):

```rust
enum FormatSpec {
    Fixed     { precision: Option<u8> },
    Float,
    Sci       { precision: Option<u8> },
    Rational,
    Financial { precision: Option<u8> },
}
```

Formatting is presentation, not arithmetic — keep `| fmt` **out of** `Value`
evaluation. The parser returns `(Node, Option<FormatSpec>)`; the result `Value`
is rendered with the spec applied over the config defaults. Applies in both
one-shot and REPL paths.

Applying a `FormatSpec` to `FormatOptions` means cloning the defaults and
overriding `repr` (and the relevant precision field if `N` was given).

## Steps

1. Lexer: add `|` → `Token::Pipe`.
2. Parser: trailing format clause (lowest precedence, top-level only — not inside
   parens); parse `FormatSpec` = variant + optional integer.
3. Plumb `Option<FormatSpec>` from parse → render in `main.rs` (both modes).
4. Apply spec: clone `FormatOptions` from config, override `repr` and precision.
5. Tests (CLI): the rows in the table above + an unknown-formatter error.

## Open questions

- `fin` for negative values: `-1234567 | fin` → `-1.23…m`? (lean: yes, sign preserved)
- Error vs. ignore for an unknown formatter name (lean: error).

## Progress

- [ ] Step 1: `|` token
- [ ] Step 2: trailing format-clause grammar + `FormatSpec`
- [ ] Step 3: plumb spec to render
- [ ] Step 4: apply spec over config defaults
- [ ] Step 5: tests
