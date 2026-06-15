# Plan 7 — `ans` keyword (previous result)

**Goal:** a special `ans` keyword that refers to the result of the previous
evaluation, usable in a new expression and especially handy with the formatting
operator — e.g. re-display the last answer differently: `ans | rat` → `5/6`.

**Depends on:** nothing for the basic recall; the `ans | fmt` use case needs
[plan 4](4_format-operator-and-named-formatters.md). Mostly a REPL feature.

## Behaviour

```
>> 1/2 + 1/3
0.8333…
>> ans | rat        # reformat the previous answer
5/6
>> ans * 2
1.6667…
```

- `ans` evaluates to the previous result `Value` (number **and** unit, e.g. if the
  last answer was `510 cm`, `ans to m` → `5.1 m`).
- One-shot CLI mode has no history → `ans` is an error there (see open Qs).

## Design

- **Lexer:** recognise `ans` as a keyword (today it would lex as `Ident("ans")`;
  either add `Token::KwAns` or special-case the ident).
- **REPL state:** `main.rs` keeps the last successful `Value`.
- **Resolution:** simplest is for the REPL to supply the stored answer to the
  parser, so `ans` becomes a `Value` node at parse time — keeps `Node::eval`
  context-free. (Alternative: an `Ans` node resolved at eval with injected state.)
- Pairs naturally with plan 4: `ans | <fmt>` just reformats the stored value.

## Open questions

- First operation (no previous answer): error message wording.
- One-shot mode: error, or treat as unsupported?
- Should `ans` be assignable / chainable beyond the previous single result (history
  like `ans1`, `ans2`)? Probably out of scope.

## Steps

1. Lexer: `ans` keyword.
2. REPL: store last `Value`; feed it to the parser for `ans` resolution.
3. Parser: `ans` → value node (atom level).
4. Tests: REPL stdin sequence (`1/2 + 1/3` then `ans * 2`, `ans | rat` once plan 4
   lands); error when no previous answer.

## Progress

- [ ] Step 1: lexer keyword
- [ ] Step 2: REPL last-value state
- [ ] Step 3: parser resolution
- [ ] Step 4: tests
