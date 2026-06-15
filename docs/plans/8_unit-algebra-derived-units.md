# Plan 8 — Unit algebra & derived units

**Goal:** let units combine under arithmetic to form derived units, e.g.
`(2 m)^2` → `4 m²` (area), `m * m` → area, `100 km / 2 h` → `50 km/h`, with
automatic normalization to sensible/common unit forms.

**Status:** large, future, exploratory. This is the big enabler behind several
"no derived units" limitations and is explicitly **not** in scope for the small
parenthesized-unit fix ([plan 0](0_fix-unit-on-parenthesized-expr.md)).

**Depends on:** ideally after the formatting work (3) so derived units render
well; otherwise standalone.

## Why it's big

The current model is a **flat `Unit` enum** (`LenM`, `AreaM`, `VolM`, …) with
hard-coded pairwise conversion factors, and `value_op`/`unit::single` explicitly
*reject* multiplying/dividing two united values (`OperateWithUnits`). Real unit
algebra needs a different representation. Sub-problems:

1. **Dimensional representation.** Model a unit as a dimension vector / product of
   base units with exponents (e.g. length¹, length², length·time⁻¹) instead of a
   flat enum — or a hybrid that maps known products back to named units (`m·m` →
   `m²`, `m³`, `km/h`).
2. **Arithmetic.** `mul`/`div`/`pow` on units: add/subtract/scale exponents;
   `pow` on a value carries the unit to that power (`(2 m)^2` → `4 m²`).
3. **Normalization / canonical form.** Decide a canonical display (`m²` vs `m*m`),
   and auto-convert to "common formats" (the user's phrase) — e.g. simplify and
   pick a conventional unit.
4. **Conversion.** Generalize `unit::convert` to dimensioned units (factors compose
   from base units), replacing the per-pair factor table.
5. **Display.** Render derived units (`m²`, `km/h`), and parse them back
   (lexer/parser already has `m2`, `m3`, etc. as distinct tokens — reconcile).
6. **Migration.** Reconcile the existing flat `Area*`/`Vol*` units and their tests
   with the new model without breaking current behaviour.

## Examples to support (eventually)

```
(2 m)^2        -> 4 m²        (area)
m * m          -> m²
100 km / 2 h   -> 50 km/h     (currently errors)
(2 m)^3        -> 8 m³
```

## Open questions

- Representation: full dimension vector vs. named-product lookup vs. hybrid?
- How much to unify with the existing `Area`/`Volume` enum variants.
- How aggressive normalization should be (when to simplify / which unit to prefer).
- Scope creep: this borders on a small computer-algebra/units library — decide how
  far to go (just powers of length? full dimensional analysis?).

## Steps (rough, to refine when prioritized)

1. Spike the unit representation (dimension vector or product-of-base) and how it
   maps to/from named units.
2. Implement unit `mul`/`div`/`pow`; lift the `OperateWithUnits` restriction.
3. Generalize conversion + display for derived units.
4. Migrate existing area/volume units and tests.

## Progress

- [ ] Step 1: representation spike
- [ ] Step 2: unit arithmetic
- [ ] Step 3: conversion + display
- [ ] Step 4: migrate existing units/tests
