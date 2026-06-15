# Plan 9 — User-configurable compound groups (optional)

**Goal:** let users define their own **compound-addition groups** — the
adjacent-quantity notation like `5 m 10 cm` / `5' 11"` — beyond the built-in
`m+cm` and `ft+in` pairs.

**Status:** optional, low priority. Captures a future extension noted while
scoping [plan 0](0_fix-unit-on-parenthesized-expr.md) (which ships the fixed
built-in pairs).

**Depends on:** plan 2 (config) + plan 0 (the compound mechanism).

## Idea

[Plan 0](0_fix-unit-on-parenthesized-expr.md) hard-codes the compound pairs. Make
them config-driven instead, so a user can add e.g. `km+m`, `h+min`, `lb+oz`:

```toml
[compound]
groups = [
  ["m", "cm"],
  ["ft", "in"],
  ["h", "min", "s"],   # user-added
]
```

Adjacent `N u N u …` is summed when all units belong to one configured group.

## Open questions

- Validate that a group's units share a dimension/type (reject `["m","kg"]`)?
- Ordering / direction — require descending magnitude, or allow any order?
- Interaction with rule 3 (implicit multiplication) when a unit isn't in any
  group — unchanged: it stays implicit multiplication.
- Built-ins (`m+cm`, `ft+in`) as defaults written into the first-run `conf.toml`,
  or implicit and only extended by config?

## Steps (rough)

1. `[compound]` config schema + defaults (the built-in pairs).
2. Parser consults configured groups instead of the hard-coded pairs.
3. Validation + clear errors for bad groups.
4. Tests: a custom group enables a new compound; non-grouped units don't compound.

## Progress

- [ ] Step 1: `[compound]` schema
- [ ] Step 2: parser uses config
- [ ] Step 3: validation
- [ ] Step 4: tests
