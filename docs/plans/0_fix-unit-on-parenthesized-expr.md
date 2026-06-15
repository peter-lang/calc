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

### 1. Multiply by a bare unit (general)
A value followed by a **bare unit** is multiplication by `1·unit`, at
multiplication precedence (a `term := term unit` rule) — not a special case for
parentheses. Covers the reported `(2*3) eur` → `6 eur`, and also `(1+2) m`,
`1/2 eur`, etc. `value_op::mul` turns `6 × 1 eur` into `6 eur`. (`5 m` literals are
still parsed by `num_unit`; both routes give the same result.)

### 2. Compound notation = fixed groups (for now)
Adjacent quantities are summed when their units share a **compound group**,
binding as a tight **atom** (so `5 m 10 cm to cm` = `(5 m + 10 cm) to cm`), and it
chains N-way within a group:

- **{m, cm}**
- **{ft, in}** (`5' 11"`)
- **{h, min, s}** (`1 h 30 min 15 s`)

The "feet-and-inches" / "hours-minutes-seconds" notation. Everything else — other
units (`km`, `mm`, `yd`, `mi`), mixed groups (`5 m 10 in`), non-grouped
(`5 kg 10 g`) — does **not** compound. Implementation: `unit::compound_group`
plus a greedy same-group chain in `num_unit` (replacing the generic same-`UnitType`
chain).

> **Future (optional, config-driven):** let users define their own compound groups
> — see [plan 9](9_configurable-compound-groups.md).

### 3. Incompatible juxtapositions error
Because rule 1 is multiplication, two **united** values written together multiply
and error if incompatible — e.g. `(2 m) eur` → `Cannot operate with units`.
Correct until unit algebra exists.

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
2. `parser.rs`: `term unit` rule — bare-unit multiplication via `Mul`.
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
