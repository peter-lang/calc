# Numbers

Files: [`src/number.rs`](../src/number.rs),
[`src/number_op.rs`](../src/number_op.rs),
[`src/rational.rs`](../src/rational.rs)

## The `Number` type

```rust
pub enum Number {
    Int(i64),
    Rational(Rational),
    Float(f64),
}
```

The guiding principle is **stay exact as long as possible**. Integers and
rationals are exact; `Float` is the lossy fallback used only when an operation
can't be represented exactly.

`Number` converts from `i64`/`f64` via `From`, and exposes `to_float()` and
`to_rational()` (the latter **panics** on a `Float` — it's only called on paths
already known to be non-float).

## `Rational`

```rust
pub struct Rational { pub num: i64, pub den: u64 }
```

- Always normalized: `Rational::new` divides out the `gcd`; the sign lives in
  `num` (the denominator is unsigned). `den == 0` panics.
- Implements `Add`/`Sub`/`Mul`/`Div`/`Neg` (all re-normalizing), plus `invert`,
  `inverse` (reciprocal of an `i64`), `checked_pow` (overflow-aware) and a
  `const fn pow` used to precompute unit factors at compile time.
- `gcd`/`lcm` are free functions; `gcd` is `const` so factors in
  [`unit.rs`](../src/unit.rs) can be `const`-evaluated.
- Unit tests live at the bottom of the file (`cargo test`).

## Arithmetic & type promotion (`number_op.rs`)

`number_op` implements `add`, `sub`, `mul`, `div`, `pow`, `sub_unary` on
`Number`. The promotion lattice is:

```
Int  ⊆  Rational  ⊆  Float
```

Rules applied per operation:

- **Int ∘ Int** stays `Int` (using checked/native ops). `pow` falls back to
  `Float` on overflow or a negative exponent that isn't a clean reciprocal.
- **anything ∘ Float** (or Float ∘ anything) → `Float`.
- **mixed Int/Rational** → compute in `Rational`, then **demote back to `Int`**
  if the result's denominator is 1. This keeps `1/2 + 1/2` as `Int(1)`.
- **div / pow** can fail: division by zero → `CalcError::DivByZero`; these return
  `Result`, while `add`/`sub`/`mul`/`sub_unary` are infallible.

This is the layer to touch when changing **how exactness is preserved or when a
fallback to float happens**.

## Output formatting (`number.rs`, `config.rs`)

Formatting is driven by `format_number(&Number, &FormatOptions)` in `number.rs`,
called from `Display for Number` with the current process config. `FormatOptions`
lives under the `[format]` TOML key (see `config.rs`).

**Integers** (`Number::Int`) print in full precision with no suffix by default:
`3000 → 3000`, `2000000 → 2000000`. Scientific notation is opt-in via
`int_scientific = true` (threshold: `int_sci_upper`, default 1e15).

**Whole-valued floats** (`Float(x)` where `x.fract() == 0`) are formatted
identically to integers — they go through the integer path, not the float path.

**Floats and rationals** use one of three modes depending on magnitude and config:

| Condition | Mode | Example |
|-----------|------|---------|
| `abs >= sci_upper` (1e6) or `0 < abs < sci_lower` (1e-6), `scientific = true` | scientific | `1.5e-8`, `1.5000…e6` |
| otherwise | fixed-point | `3.3333…`, `0.5` |
| `rational = true` + `Number::Rational` | fraction | `1/3`, `5/6` |

**Precision and the `…` marker:**
- Fixed-point: `precision` decimal places (default 4). If the rounded value equals
  the original → trim trailing zeros (`3.5`, `0.5`). If rounded → pad to full
  precision and append `…` (`3.3333…`, `1.4142…`).
- Scientific: `sci_precision` mantissa decimals (default 4), same exact/approx
  check via round-trip parse (`1.5e-8` exact → `1.5e-8`; `1.5000005e6` → `1.5000…e6`).

`Rational` is formatted by converting to `f64` then applying the float rules
(unless `rational = true`). `debug.rs` defines `Debug for Number` delegating to
`Display` so AST dumps use the same rendering.
