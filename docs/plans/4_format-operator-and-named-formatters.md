# Plan 4 — `|` format operator & named formatters

**Goal:** per-calculation output formatting via a trailing `|` clause, plus named
(user-definable) formatters, shipping `fin` (financial) as a default one.

**Depends on:** plan 3 (formatting engine), plan 2 (config + default template).

## Behaviour

A trailing `| <formatter>` overrides formatting for that one result:

| Input | Output |
|-------|--------|
| `1/2 + 1/3 \| rat` (or `rational`) | `5/6` |
| `1/3 \| prec 6` | `0.333333…` |
| `1/1000000 \| sci` | `1e-6` |
| `1234567 \| fin` | `1.23m` |

- **`|` is a new token** — confirmed unused in the lexer today (`/` is `Div`; the
  `|` in lexer.rs is only the regex-join char).
- Built-in directives: `rat`/`rational`, `sci`, `dec`/`prec N`, `fin`, or any
  **named formatter** from config. **`prec N` = fixed-point with N decimals**
  (forces scientific off for that result), e.g. `1/3 | prec 6` → `0.333333…`.
- **`fin` (financial):** 2 decimals; for `|x| ≥ 1e6`, divide by `1e6` and suffix
  `m` (e.g. `1.23m`). Shipped as a default *user-defined* formatter written into
  the first-run `conf.toml` (plan 2 template) — i.e. `fin` is config, not
  hard-coded, proving the user-formatter mechanism.

## Design

Formatting is presentation, not arithmetic — keep `| fmt` **out** of `Value`
evaluation. The parser returns `(Node, Option<FormatSpec>)`; the result `Value`
is rendered with the spec merged over the config defaults. Applies in both
one-shot and REPL paths.

`[format.formatters.<name>]` config defines named formatters as a set of
`FormatOptions` overrides (+ a `financial` flag for the `m`/precision rule):

```toml
[format.formatters.fin]
financial = true
precision = 2
```

## Steps

1. Lexer: add `|` → `Token::Pipe`.
2. Parser: trailing format clause (lowest precedence, top-level only — not inside
   parens); parse `FormatSpec` = name + optional integer arg.
3. Plumb `Option<FormatSpec>` from parse → render in `main.rs` (both modes).
4. Built-in directives: `rat`, `sci`, `dec`/`prec N`.
5. `financial` render mode + resolve named formatters from config by name; add
   `fin` to the default template.
6. Tests (CLI): the four rows above + an unknown-formatter error.

## Open questions

- Exact `fin` rules: thousands separators? negative handling? sub-1 values?
- Error vs. ignore for an unknown formatter name (lean: error).

## Progress

- [ ] Step 1: `|` token
- [ ] Step 2: trailing format-clause grammar + `FormatSpec`
- [ ] Step 3: plumb spec to render
- [ ] Step 4: built-in directives
- [ ] Step 5: financial mode + named formatters + default `fin`
- [ ] Step 6: tests
