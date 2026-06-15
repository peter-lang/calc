# Units & conversions

File: [`src/unit.rs`](../src/unit.rs)
(currency conversion is in [currency.md](currency.md))

## Model

A [`Value`](evaluation.md) carries an optional `Unit`. Units are grouped into
`UnitType`s:

```
Length · Area · Volume · Mass · Temperature · Time · Currency
```

Two units can be combined or converted only when they share a `UnitType`
(`common_type` returns `Some(type)` or `None`).

The `Unit` enum lists every concrete unit. Each unit needs four things wired up,
in four parallel `match` statements — **keep them in sync**:

| Function | Purpose |
|----------|---------|
| `get_default_factor` | numeric factor to the type's **base unit** (drives conversion) |
| `get_unit_name` | the string printed after a value |
| `get_unit_type` | which `UnitType` the unit belongs to |
| `expect_unit` (in `parser.rs`) | maps the lexer `Token` to this `Unit` |

The lexer also needs a token + regex for the unit's spellings — see
[lexer.md](lexer.md).

## Base units and factors

Conversion (for everything except temperature and currency) is **factor-based**:
each unit declares how many base units it equals, and

```
convert(value, from, to) = value * factor(from) / factor(to)
```

Base unit per type:

| Type | Base | Examples of factors |
|------|------|---------------------|
| Length | metre (`m` = 1) | `km`=1000, `cm`=1/100, `in`=254/10000 |
| Area | `m2` = 1 | `km2`=1000², `in2`=(254/10000)² |
| Volume | litre (`l` = 1) | `ml`=1/1000, `m3`=1000, `gallon`=3785411784/1e9 |
| Mass | kilogram (`kg` = 1) | `g`=1/1000, `lb`=45359237/1e8 |
| Time | second (`s` = 1) | `min`=60, `h`=3600 |

Factors are `Rational`s built with `const` expressions (e.g.
`(FEET2INCH * INCH2M).pow(2)`), so they are **exact and computed at compile
time**. Imperial units derive from the exact inch definition
`INCH2M = 254/10000` (i.e. 2.54 cm). The `wrap` helper demotes a factor whose
denominator is 1 to `Number::Int`.

Because factors are rational, conversions like `1 pint to gallon` come out as the
exact `1/8` rather than a rounded float (see the tests in `unit.rs`).

## Temperature is special (affine, not scaling)

Celsius/Fahrenheit differ by both a scale and an offset, so they bypass the
factor path. `convert_temp` applies:

- C→F: `val * 9/5 + 32`
- F→C: `(val - 32) / (9/5)`

(`9/5` is encoded as the rational `18/10`, `32` as an int, keeping integer inputs
exact — `95 F to C` → `35 C`.) There is **no Kelvin** and temperatures cannot be
added/subtracted as if linear; only `to`-conversion between C and F is meaningful.

## Currency is special (live rates)

`UnitType::Currency` units hold the code as `Unit::Curr(&'static str)`.
Conversion defers to [`currency::convert`](currency.md), which returns a
`Rational` exchange rate that is then multiplied in. See [currency.md](currency.md).

## Combining units under arithmetic (`single`)

`value_op::mul`/`div` call `unit::single(a, b)` to decide the result unit. The
current rule is intentionally simple:

- one side unitless → keep the other side's unit (`3k * 2 m` → `… m`),
- **both** sides have units → `CalcError::OperateWithUnits`.

So the calculator does **not** synthesize compound/derived units: `100 km / 2 h`
is an error, not `50 km/h`. This is a known limitation; deriving units would
require a richer `Unit` representation (e.g. dimension vectors).

Addition/subtraction (`value_op::add`/`sub`) instead **convert the right operand
into the left's unit** when both are present and same-type, erroring with
`DifferentUnitTypes` otherwise.

## Adding a new unit — checklist

1. **Token** — add a `Token` variant and a lexer `PATTERNS` row with its
   spellings, correctly ordered for longest-match ([lexer.md](lexer.md)).
2. **Unit variant** — add it to the `Unit` enum.
3. **Four matches** — add arms to `get_default_factor` (factor to base),
   `get_unit_name`, `get_unit_type`, and `parser.rs::expect_unit`.
4. If introducing a **whole new `UnitType`**, also add the variant to `UnitType`
   and decide its base unit / conversion path (factor-based, or special-cased
   like temperature).
5. Add a conversion test in the `#[cfg(test)] mod tests` block.

A missing factor arm falls through to `CalcError::ConversionError`, so a
half-wired unit will parse but fail to convert — add all four arms.
