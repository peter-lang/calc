# Currency conversion

Files: [`src/currency/mod.rs`](../src/currency/mod.rs),
[`src/currency/providers/mnb.rs`](../src/currency/providers/mnb.rs),
[`src/currency/providers/static_provider.rs`](../src/currency/providers/static_provider.rs),
currency tokens in [`src/parser/token.rs`](../src/parser/token.rs),
cache paths in [`src/files.rs`](../src/files.rs)

## Provider architecture

Rate sourcing is abstracted behind a `RateProvider` trait in `currency/mod.rs`:

```rust
pub trait RateProvider: Sync {
    fn id(&self) -> &str;   // cache key / config name
    fn convert(&self, from: &str, to: &str) -> Result<Rational, CalcError>;
}
```

`currency::convert(from, to)` delegates to whichever provider is active.
Provider selection is driven by `[currency].provider` in the config (default:
`mnb`). The active provider is resolved once per process via `active_provider()`.

## MNB provider (`providers/mnb.rs`)

Exchange rates come from the **Hungarian National Bank (MNB)** SOAP web service
`http://www.mnb.hu/arfolyamok.asmx`, operation `GetCurrentExchangeRates`. The
request body is a fixed SOAP envelope (`MNB_BODY`); the response wraps an XML
document **as a string**, which is deserialized in two stages with
`quick-xml` + `serde`:

1. `Envelope → Body → GetCurrentExchangeRatesResponse` to pull out the inner XML
   string.
2. That string → `MNBCurrentExchangeRates` (`Day` with a date and a list of
   `Rate`s).

Each `Rate` has a currency code, a `unit` (how many units the quoted rate is
for), and the rate value. `deserialize_rate` parses the MNB's comma-decimal
formatting (e.g. `387,45`) into an exact `Rational` by counting decimal places;
`to_map` normalizes any `unit != 1` rate down to a per-1-unit rate.

### Base currency and the conversion math

All MNB rates are **against HUF** (`BASE = "HUF"`). A map entry `code → rate`
means "1 `code` = `rate` HUF". `MnbProvider::convert(from, to)` returns the
`Rational` multiplier such that `value_in_from * rate = value_in_to`:

- `from == HUF` → `1 / rate(to)`
- `to == HUF` → `rate(from)`
- neither is HUF → `rate(from) / rate(to)` (cross rate via HUF)

### Caching

- **In-process:** a `static OnceLock` inside `MnbProvider::convert` holds the
  parsed rate map for the lifetime of the process.
- **On disk:** the raw inner XML is written to `rates.xml` in the platform cache
  directory (`files::cache`). On startup the cached file is reused **only if its
  `Day` date is today or yesterday**; otherwise a fresh fetch is made.

> Implication: currency conversion needs network access **the first time per
> day**. With no network and no fresh cache, it fails with `CalcError::RequestError`.

## Static provider (`providers/static_provider.rs`)

User-configured fixed rates for offline use and deterministic tests.

```toml
[currency]
provider = "static"

[currency.static]
"EUR/USD" = 1.08
"USD/HUF" = 360.0
```

**Direct lookup only** — `"EUR/USD" = 1.08` means 1 EUR = 1.08 USD when
converting EUR → USD. The inverse (`USD → EUR`) must be configured separately;
unconfigured pairs return `CalcError::ConversionError`. No triangulation,
no auto-fill.

TOML float values are converted to `Rational` via string-decimal parsing (same
technique as `deserialize_rate`), so `1.08` → `Rational { num: 27, den: 25 }`.

## Adding / changing supported currencies

Currency **codes accepted as input** are defined in
[`token.rs`](../src/parser/token.rs) by two hand-maintained, coupled constants:

- `CURRENCIES: [&str; 34]` — **must stay sorted** (the parser does a
  `binary_search` to validate `Token::Curr`), and the length must be updated.
- `CURRENCIES_PATTERN` — the lexer regex listing each code in upper- and
  lower-case.

To add a currency: insert it into both, keep `CURRENCIES` sorted, bump the array
length. For the MNB provider it also has to be present in the MNB feed, or
`convert` returns `CalcError::ConversionError`. No changes are needed in
`unit.rs` — currencies flow through the generic `Unit::Curr(&str)` variant.

## Failure modes

| Situation | Result |
|-----------|--------|
| MNB: network down, stale/no cache | `CalcError::RequestError` (wrapped `ureq`) |
| MNB: malformed XML response | `CalcError::DeError` (wrapped `quick_xml`) |
| MNB: code not in feed | `CalcError::ConversionError` |
| Static: pair not configured | `CalcError::ConversionError` |
| Converting a currency to a non-currency unit | `DifferentUnitTypes` |
