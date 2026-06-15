# Plan 6 — Currency provider interface & static provider

**Goal:** turn the hard-coded MNB rate source into a proper provider interface,
add a config-driven **static** provider (for offline/deterministic tests), and
leave room for a real open API (e.g. ECB) later.

**Depends on:** plan 2 (provider selection + static rates from config).

## Design

- Refactor `src/currency.rs` into a module: `currency/mod.rs` (generic conversion
  math + disk/`OnceLock` caching + provider selection), `currency/providers/mnb.rs`,
  `currency/providers/static.rs`.
- **Trait:**
  ```rust
  trait RateProvider {
      fn id(&self) -> &str;            // cache key / config name
      fn base(&self) -> &str;          // e.g. "HUF", "EUR"
      fn rates(&self) -> Result<HashMap<String, Rational>, CalcError>; // 1 <code> = rate <base>
  }
  ```
  The cross-rate math (convert via base) and per-provider caching stay generic in
  `mod.rs`; disk cache is keyed by `id()` so providers don't collide.
- **`MnbProvider`:** the existing SOAP/XML logic, base `HUF`.
- **`StaticProvider`:** user-configured fixed rates from `[currency.static]`.
  **Direct lookup only** — no triangulation / graph search / inverse-filling; an
  undefined pair returns `CalcError::ConversionError`. Primary purpose: drive
  **network-free integration tests** for the currency path.
- **Selection:** `[currency].provider = "mnb" | "static"`.

```toml
[currency]
provider = "static"

[currency.static]
"EUR/USD" = 1.08
"USD/HUF" = 360
```

## Steps

1. Define `RateProvider`; extract generic conversion + caching into `currency/mod.rs`.
2. Move MNB behind the trait (`providers/mnb.rs`).
3. `providers/static.rs`: read config, direct-lookup convert.
4. Provider selection from config.
5. Network-free currency integration tests via the static provider (lets us drop
   the `#[ignore]` on the live currency test, or keep it separate).
6. **Follow-up (deferred):** a real open provider — research ECB (EUR base, free
   daily XML, no key) or alternatives. Not blocking; pick the API later.

## Open questions

- Static config shape: flat `"FROM/TO" = rate` table (assumed) vs base + rate map.
- Auto-derive inverse pairs, or require each direction explicitly? (Lean: pure
  direct lookup, as requested — no auto-fill.)
- Which open API for step 6 (ECB vs a multi-currency aggregator).

## Progress

- [ ] Step 1: `RateProvider` + generic `currency/mod.rs`
- [ ] Step 2: `MnbProvider`
- [ ] Step 3: `StaticProvider`
- [ ] Step 4: provider selection
- [ ] Step 5: offline integration tests
- [ ] Step 6: open-API provider (deferred)
