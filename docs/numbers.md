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

## Output formatting (`number.rs`, `Display`)

`Display for Number` is where results get their human-friendly shape. `Int`s and
the float path use different rules:

- **Large magnitudes** get `k`/`m` suffixes: ≥1e6 → `…m`, ≥1e3 → `…k` (integers
  only collapse to `k` when exactly divisible by 1000).
- **Very large/small floats** (`≥1e9` or `<1e-3`) use scientific notation
  (`{:.6e}`).
- **Rounding** for display: floats are rounded to ~3 significant decimals (6 when
  `<1`). If rounding lost information, a trailing **`…`** is appended to signal
  the displayed value is approximate (e.g. `1/3` prints `0.833333…`).
- `Rational` is displayed by first converting to `f64` and using the float path
  — so the REPL shows decimals, not `5/6`. (The raw `a/b` form is only used by
  `Rational`'s own `Display`, e.g. in debug output.)

If you want to change **how answers look** (precision, suffixes, sci-notation
thresholds, the `…` marker), this `Display` impl is the single place to edit.

`debug.rs` also defines `Debug for Number` (delegating to `Display`) so AST dumps
read naturally.
