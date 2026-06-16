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

## Output formatting (`number.rs`, `value.rs`, `config/mod.rs`)

Two entry points:

- `format_number(&Number, &FormatOptions) -> String` — formats a bare number.
- `format_value(&Value, &FormatOptions) -> String` in `value.rs` — wraps
  `format_number` and appends the unit name when present.

`Display for Number` calls `format_number` with `config::current().format`.
`repl.rs` uses `format_value` so it can supply a per-expression `FormatOptions`
built from the config + any `| formatter` clause override.

### `NumberRepr` and `FormatOptions`

`FormatOptions` (under the `[format]` TOML key) controls all formatting:

```rust
pub enum NumberRepr { Fixed, Float, Sci, Rational, Financial }

pub struct FormatOptions {
    pub repr: NumberRepr,         // default: Float
    pub float: FloatConfig,
    pub sci:   SciConfig,
    pub fin:   FinConfig,
    pub int:   IntConfig,
}
```

| `repr` | Effect |
|--------|--------|
| `Float` | fixed-point; auto-upgrades to sci outside `[float.sci_upgrade_lower, float.sci_upgrade_upper]` |
| `Fixed` | always fixed-point, no auto-upgrade |
| `Sci` | always scientific notation |
| `Rational` | exact fraction (`1/3`) for `Number::Rational`; float path otherwise |
| `Financial` | fixed-point with `fin.precision` decimal places; `m` suffix for `|x| ≥ 1e6` |

**Integers** (`Number::Int`) respect `repr`:
- `Sci` → always scientific notation.
- `Financial` → always financial format (decimal places preserved, no trailing-zero trim).
- Others → plain integer; sci only when `[format.int] sci_upgrade = true` and
  `|x| ≥ sci_upgrade_upper` (default 1e15).

**Whole-valued floats** (`Float(x)` where `x.fract() == 0`) go through the
integer path, not the float path.

**Floats and rationals** use `repr` as above. `Rational` is converted to `f64`
before formatting unless `repr = Rational`.

### Precision and the `…` marker

- **Fixed-point / `Fixed` / `Float`**: `float.precision` decimal places (default 4).
  Rounded value equals original → trim trailing zeros (`0.5`, `3`).
  Rounded ≠ original → append `…` (`0.3333…`, `1.4142…`).
- **Scientific**: `sci.precision` mantissa decimals (default 4). Exact/approx
  decided by round-trip parse (`1.5e-8` → exact; `1.5000005e6` → `1.5000…e6`).
- **Financial**: `fin.precision` decimal places (default 2). Trailing zeros are
  **kept** (`42.00`); `…` appended only when rounding loses information (`1.23…m`).
  The `m` suffix is placed after the `…` when present (`1.23…m`, not `1.23m…`).

### `| formatter [N]` — per-expression override

A trailing `| formatter` clause overrides `repr` (and optionally precision) for
that one result. `FormatSpec` is defined in `config/mod.rs`; `apply_spec` clones the
config `FormatOptions` and patches it:

| Clause | `repr` set | Precision field |
|--------|------------|-----------------|
| `\| fixed [N]` | `Fixed` | `float.precision` |
| `\| float` | `Float` | — |
| `\| sci [N]` | `Sci` | `sci.precision` |
| `\| rat` / `\| rational` | `Rational` | — |
| `\| fin [N]` / `\| financial [N]` | `Financial` | `fin.precision` |

See [parser.md](parser.md) for the grammar and how `repl.rs` applies the spec.

`debug.rs` defines `Debug for Number` delegating to `Display` so AST dumps use
the same rendering.
