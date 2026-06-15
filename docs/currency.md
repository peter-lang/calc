# Currency conversion

Files: [`src/currency.rs`](../src/currency.rs),
currency tokens in [`src/parser/token.rs`](../src/parser/token.rs),
cache paths in [`src/files.rs`](../src/files.rs)

## Source of rates

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

## Base currency and the conversion math

All MNB rates are **against HUF** (`BASE_CURRENCY = "HUF"`). A map entry
`code → rate` means "1 `code` = `rate` HUF". `convert(from, to)` returns the
`Rational` multiplier such that `value_in_from * rate = value_in_to`:

- `from == HUF` → `1 / rate(to)`
- `to == HUF` → `rate(from)`
- neither is HUF → `rate(from) / rate(to)` (cross rate, computed as
  `rate(to)⁻¹ * rate(from)`)

Rates stay rational throughout, so currency conversions are exact w.r.t. the
published rate (the final display still rounds — see [numbers.md](numbers.md)).

`unit.rs::convert` is the caller: for `UnitType::Currency` it multiplies the
value by `Number::Rational(currency::convert(from, to)?)`.

## Caching

Rates are cached to avoid a network call on every conversion:

- **In-process:** a `OnceLock` holds the parsed rate map for the lifetime of the
  process, so only the first currency conversion can trigger work.
- **On disk:** the raw inner XML is written to `rates.xml` in the platform cache
  directory (`files::cache`, via the `directories` crate). On startup the cached
  file is reused **only if its `Day` date is today or yesterday**; otherwise a
  fresh fetch is made and the file overwritten.

The same cache directory also holds the REPL's `history.txt`.

> Implication: currency conversion needs network access **the first time per
> day**. With no network and no fresh cache, the conversion fails with a wrapped
> `ureq` error surfaced as `CalcError::RequestError`.

## Adding / changing supported currencies

Currency **codes accepted as input** are defined in
[`token.rs`](../src/parser/token.rs) by two hand-maintained, coupled constants
(there's a `// TODO: this should come from a macro`):

- `CURRENCIES: [&str; 34]` — **must stay sorted** (the parser does a
  `binary_search` to validate `Token::Curr`), and the length must be updated.
- `CURRENCIES_PATTERN` — the lexer regex listing each code in upper- and
  lower-case.

To add a currency: insert it into both, keep `CURRENCIES` sorted, bump the array
length. It also has to be present in the MNB feed, or `convert` returns
`CalcError::ConversionError`. No changes are needed in `unit.rs` — currencies
flow through the generic `Unit::Curr(&str)` variant.

## Failure modes

| Situation | Result |
|-----------|--------|
| Network down, stale/no cache | `CalcError::RequestError` (wrapped `ureq`) |
| Malformed XML response | `CalcError::DeError` (wrapped `quick_xml`) |
| Code not in MNB feed | `CalcError::ConversionError` |
| Converting a currency to a non-currency unit | `DifferentUnitTypes` |
