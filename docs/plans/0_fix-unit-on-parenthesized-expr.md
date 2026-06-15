# Plan 0 — Unit attachment & juxtaposition semantics

**Goal:** make units attach to values correctly. Concretely:

1. `(2*3) eur to huf` works as `6 eur to huf` (the reported bug — today it fails
   to parse).
2. The "adjacent quantities are **added**" notation (`5 m 10 cm`, `5' 11"`) is
   **restricted to same-system length** only — it's a human notation for length,
   not a general feature.
3. Any other value-then-unit juxtaposition is **implicit multiplication**.
4. **Exponentiating a value that has a unit is an error** (no unit algebra yet).

**Type:** immediate (`0_`). Grew beyond the original one-liner because the
juxtaposition rules all interact.

## The reported bug & root cause

`(2*3) eur to huf` produces no output. Units only attach to **number literals**:
`single_num_unit = expect_number expect_unit`, chained by `num_unit`; `atom` has
no rule to attach a unit to a parenthesized (non-literal) value. So `(2*3)` parses
and the leftover `eur` makes `parse()` return `None`.

## Decisions

### 1. Attach a unit to a parenthesized value
`( expr ) unit` → `expr × 1·unit` (`value_op::mul` turns `6 × 1 eur` into
`6 eur`). Scalar inner value only; if the inner value already has a unit it falls
to rule 3 (and errors, as desired).

### 2. Compound notation = fixed length pairs (for now)
`N₁ u₁ N₂ u₂` summing adjacent quantities is valid **only** for these length
pairs, and binds as a tight **atom** (so `5 m 10 cm to cm` = `(5 m + 10 cm) to
cm`):

- **m + cm**
- **ft + in** (`5' 11"`)

That's the "feet-and-inches" / "metres-and-centimetres" notation. Everything else —
other units (`km`, `mm`, `yd`, `mi`), mixed systems (`5 m 10 in`), non-length
(`5 kg 10 g`) — **no longer** compounds. Suggested implementation: a dedicated
compound rule keyed on these specific pairs, replacing the generic "same
`UnitType`" chain in `num_unit`.

> **Future (optional, config-driven):** let users define their own compound groups
> — see [plan 9](9_configurable-compound-groups.md).

### 3. Everything else = implicit multiplication
A value followed by a unit multiplies by `1·unit`. Two **united** values therefore
multiply and error if incompatible — e.g. `(2 m) eur` → `Cannot operate with
units`. Correct until unit algebra exists.

### 4. No unit exponentiation yet
Raising a value that has a unit to a power **errors** (e.g. `(2 m)^2`, `2 m ^ 2`).
It would yield a derived unit (`m²`) we don't support — deferred to
[plan 8](8_unit-algebra-derived-units.md). `value_op::pow` must reject a base that
carries a unit, **reusing the existing `CalcError::OperateWithUnits`** (no new
error variant). **This changes today's behaviour** (`(2 m)^2` currently → `4 m`).

## Behaviour summary

| Input | After this plan |
|-------|-----------------|
| `(2*3) eur to huf` | `6 eur → huf` |
| `5 m 10 cm to cm` | `510 cm` (unchanged) |
| `5' 11" to cm` | sum then convert (same-system imperial) |
| `5 kg 10 g` | no longer compound → implicit mul → error |
| `(2 m)^2` | **error** (was `4 m`) |
| `(2 m) eur` | error (unchanged) |

## Must stay unchanged

- `5 m 10 cm to cm` → `510 cm` (the `m+cm` and `ft+in` compounds).
- bare unit `m` → `1 m`; `2 m` → `2 m`; `3 m * 2` → `6 m`.

## Intended changes (update tests)

- `(2 m)^2`: `4 m` → error. Update the `units_with_arithmetic` CLI test.
- Non-length same-type compounds (e.g. `5 kg 10 g`) stop compounding.

## Resolved

- Compound pairs limited to **m+cm** and **ft+in** for now; user-defined groups
  deferred to [plan 9](9_configurable-compound-groups.md).
- `5 kg 10 g` (non-length adjacency): leave unsupported (errors via implicit mul /
  parse-fail) — no general quantity×quantity multiplication.
- **No new error type** for unit exponentiation — reuse `CalcError::OperateWithUnits`.
  New variants only when something is handled differently.

## Steps

1. Confirm current outputs for the cases above.
2. `parser.rs`: parenthesized-value + unit attaches via `Mul`.
3. Restrict compound to the fixed pairs (`m+cm`, `ft+in`): dedicated rule; drop
   generic same-`UnitType` chaining.
4. `value_op::pow`: error (`OperateWithUnits`) when the base has a unit.
5. Update/extend CLI tests (new behaviour + the two intended changes; add an
   imperial `5' 11"` compound case).
6. Manually verify `(2*3) eur to huf` against the live feed.
7. Update [docs/parser.md](../parser.md) / [docs/units.md](../units.md) (compound
   rule + unit powers).

## Progress

- [ ] Step 1: confirm repro
- [ ] Step 2: paren value + unit
- [ ] Step 3: compound restricted to same-system length
- [ ] Step 4: error on unit exponentiation
- [ ] Step 5: tests
- [ ] Step 6: manual currency check
- [ ] Step 7: docs
