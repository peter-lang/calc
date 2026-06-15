# Plan 3 — Output formatting engine & new defaults

**Goal:** replace the current clunky `Display for Number` with a configurable
formatter, and adopt sensible new defaults. Today's formatting (auto `k`/`m`
suffixes, fixed thresholds, `…` marker) is hard to control.

**Depends on:** plan 2 (reads `[format]` from config). **Blocks:** plan 4.

## New default behaviour (no overrides)

Behaviour differs by number kind, because integers are exact and floats/rationals
may be rounded:

- **Integers** (`Number::Int`): printed in **full precision, plain** (e.g.
  `2000000`), never `…`. Scientific notation is **opt-in** per config
  (`int_scientific`, off by default) above a configurable magnitude.
- **Floats & rationals**:
  - `|x| > sci_upper` (1e6) or `0 < |x| < sci_lower` (1e-6) → **scientific**,
    `sci_precision` (4) mantissa decimals, e.g. `1.4452e12`, `4.2112e-6`.
  - otherwise → **fixed-point**, `precision` (4) decimals.
- **Whole-valued results print as integers.** The arithmetic engine already
  demotes a `Rational` with denominator 1 to `Int` (`number_op.rs`), so e.g.
  `1/2 + 1/2` → `1`. Keep that; ensure a whole-valued float renders the same way.
- **Trailing zeros & the `…` marker** (the precise rule):
  - if the value is **exact** at the chosen precision → **trim** trailing zeros
    (`0.5`, `1`).
  - if it was **rounded/truncated** → **pad to full precision and append `…`**
    (`0.5000…`, `0.3333…`). Both the padding and the marker signal "not exact".
- **Drop the implicit `k`/`m` suffixing** from the default — that becomes the
  `fin` formatter's job (plan 4).

> This deliberately changes observable output, so the golden values in
> `tests/cli.rs` (e.g. `3k`, `1m`, `1.235m`, `2k g`, `1.000000e9`) must be updated
> in this plan. Capture old → new in the commit.

## Config: `[format]`

```toml
[format]
precision      = 4      # fixed-point decimals for floats/rationals (default 4)
scientific     = true   # allow scientific notation for floats/rationals
sci_lower      = 1e-6   # |x| below this → scientific
sci_upper      = 1e6    # |x| above this → scientific
sci_precision  = 4      # mantissa decimals in scientific mode
rational       = false  # print exact fractions as a/b instead of decimals
int_scientific = false  # integers: opt in to scientific above int_sci_upper
int_sci_upper  = 1e15   # threshold used only when int_scientific = true
```

`scientific = false` disables sci for floats/rationals (always fixed-point);
`rational = true` prints `Number::Rational` as `5/6`; integers stay full-precision
unless `int_scientific = true`.

## Steps

1. Add `FormatOptions` to config (`[format]`) with the defaults above.
2. `format_number(&Number, &FormatOptions) -> String`; refactor `Display for
   Number`/`Value` to call it with the current config's options.
3. Implement the three modes: scientific, fixed-point (with `…`), rational.
4. Update `tests/cli.rs` golden values to the new defaults; add cases for
   sci/fixed/rational boundaries and `scientific=false`.
5. Update [docs/numbers.md](../numbers.md) (and the formatting note in CLAUDE.md).

## Decisions

- Default fixed-point precision: **4 decimals** (floats/rationals).
- **Integers**: full precision by default; scientific is separately opt-in
  (`int_scientific`, off) — integers never use sci or `…` by default.
- **Trailing zeros**: trim when the value is exact (`0.5`); when rounded, pad to
  precision and append `…` (`0.5000…`).
- **Whole-valued rationals** demote to `Int` in the arithmetic engine (already
  done in `number_op.rs`) and format as integers; whole-valued floats format the
  same way.
- `rational` (a/b output) default **off**.

## Open questions

_None blocking._ Remaining details (e.g. negative-zero, exact threshold edge cases)
to settle during implementation against tests.

## Progress

- [ ] Step 1: `[format]` options
- [ ] Step 2: `format_number` + Display refactor
- [ ] Step 3: scientific / fixed / rational modes
- [ ] Step 4: update + extend tests
- [ ] Step 5: docs
