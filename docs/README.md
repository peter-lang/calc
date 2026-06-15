# calc — documentation

`calc` is a command-line calculator written in Rust. Beyond ordinary
arithmetic it understands **exact rational arithmetic**, **physical units**
(length, area, volume, mass, temperature, time) with conversions, and **live
currency conversion** using exchange rates from the Hungarian National Bank
(MNB).

It runs as an interactive REPL (no arguments) or evaluates a single
expression passed on the command line.

```console
$ cargo run
>> 1/2 + 1/3
0.833333…
>> 5 m 10 cm to cm
510 cm
>> 95 f to c
35 C
>> 100 EUR to USD
108.42…  USD

$ cargo run -- 2^10
1024
```

## How it fits together

Input flows through a classic interpreter pipeline:

```
text ──▶ Lexer ──▶ [Token] ──▶ Parser ──▶ Node (AST) ──▶ eval() ──▶ Value ──▶ Display
        (regex)              (packrat)                  (fn ptrs)
```

A `Value` is a [`Number`](numbers.md) paired with an optional
[`Unit`](units.md). Numbers are kept exact (`Int` / `Rational`) for as long as
possible and only fall back to `Float` when an operation forces it.

## Documentation map

| Topic | File | What it covers |
|-------|------|----------------|
| Architecture & data flow | [architecture.md](architecture.md) | Module layout, the pipeline, the REPL loop, error handling |
| Lexer | [lexer.md](lexer.md) | Tokenizing, the regex table, number suffixes, adding tokens |
| Parser & grammar | [parser.md](parser.md) | The packrat/left-recursive parser, operator precedence, the grammar |
| Numbers | [numbers.md](numbers.md) | `Number`, `Rational`, type promotion, output formatting |
| Units & conversions | [units.md](units.md) | Unit catalog, base-unit factors, temperature, adding a unit |
| Currency | [currency.md](currency.md) | MNB SOAP fetch, caching, HUF base conversion |
| Evaluation | [evaluation.md](evaluation.md) | `Node`, `Value`, the `value_op` operators, unit combining rules |

## Building and running

```bash
cargo build            # debug build
cargo run              # REPL
cargo run -- 2 + 2     # one-shot evaluation
cargo test             # unit tests (rational.rs, unit.rs)
```

Cached data (REPL history, currency rates) lives in the platform cache
directory resolved by [`files::cache`](../src/files.rs); see
[currency.md](currency.md).
