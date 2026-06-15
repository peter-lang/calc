# Lexer

Files: [`src/parser/lexer.rs`](../src/parser/lexer.rs),
[`src/parser/token.rs`](../src/parser/token.rs)

## What it does

`Lexer::parse(text)` returns an iterator of [`Token`](../src/parser/token.rs)s.
The lexer is built from a single static table, `PATTERNS`, where each entry is a
pair of:

- a **regex fragment** (a string), and
- an **extractor** `fn(&str) -> Token` that turns the matched text into a token.

At construction time (`Lexer::new`) all fragments are wrapped in capture groups
and joined with `|` into one big alternation regex. At parse time,
`captures_iter` walks the input; `map_captures` finds which group matched (by
index) and calls that group's extractor.

```
PATTERNS = [ (r"...", |x| Token::LitFloat(...)),
             (r"...", |x| Token::LitInt(...)),
             ... ]
              │
   Lexer::new joins as  (frag1)|(frag2)|...  ──▶ one Regex
              │
   map_captures: first matching group index → PATTERNS[index-1].extractor
```

`Token::COUNT` (from `strum::EnumCount`) is used to size and iterate the table,
so the table length **must** match the number of `Token` variants.

## Ordering matters — longest match wins

The regex alternation is **left-biased**: the first fragment that matches at a
position wins. The table is therefore ordered deliberately:

- floats before ints (so `1.5` isn't read as `1`),
- multi-character unit names before shorter ones — the table is grouped
  `// 3 char`, `// 2 char`, `// 1 char` (e.g. `cm3` before `cm`, `cm` before
  `m`),
- **formatter keywords** (`fixed`, `float`, `sci`, `fin`/`financial`,
  `rat`/`rational`) appear before any unit token. `fixed`, `float`,
  `fin`/`financial` start with `f` (which is `TempF`) and `sci` starts with
  `s` (which is `TimeSec`), so without dedicated keywords those names would
  tokenize incorrectly (`fin` → `[TempF, LenInch]`). `rat`/`rational` have no
  conflict but are also keywords for consistency.
- the generic identifier rule `[A-Za-z_]...` and the catch-all `\S+`
  (`Token::INVALID`) come last.

When adding a token, **place it so that longer/more-specific spellings are tried
before shorter prefixes of them**, or they will never match.

## Number literals and suffixes

Number fragments accept human-friendly magnitude suffixes, handled in the
extractor:

| Suffix | Meaning | Example |
|--------|---------|---------|
| `k`    | ×1 000  | `3k` → 3000 |
| `m`    | ×1 000 000 | `2m` → 2 000 000 |
| `kk`   | ×1 000 000 | `2kk` → 2 000 000 |

Floats also accept scientific notation (`1e9`, `1.5E-3`). Integer literals
produce `Token::LitInt(i64)`; anything with a decimal point or exponent produces
`Token::LitFloat(f64)`.

> Note: `m` is overloaded — as a number suffix it means "million", but as a unit
> token it means "metre". The lexer resolves this by context: the suffix only
> applies when `m` immediately follows digits inside a number match.

## Currencies

Currency codes are matched by `CURRENCIES_PATTERN` in
[`token.rs`](../src/parser/token.rs) and produce `Token::Curr(String)`
(upper-cased). The matched code is validated later by the parser against the
sorted `CURRENCIES` array.

There are **two coupled constants** in `token.rs`:

- `CURRENCIES: [&str; 34]` — the sorted list (used for `binary_search`), and
- `CURRENCIES_PATTERN` — a hand-written `upper|lower|...` regex alternation.

A `// TODO: this should come from a macro` marks that these are maintained by
hand. **If you add a currency you must update both**, keep `CURRENCIES` sorted,
and bump the array length. See [currency.md](currency.md).

## Adding a token / unit spelling

1. Add the variant to the `Token` enum in `token.rs`.
2. Add a `(regex, extractor)` row to `PATTERNS` in `lexer.rs`, placed correctly
   for longest-match ordering.
3. If it is a unit, also wire it through the parser's `expect_unit` and the
   `Unit` machinery — see [units.md](units.md).

The table length is checked against `Token::COUNT` implicitly (the array type is
`[_; Token::COUNT]`), so a mismatch is a compile error.
