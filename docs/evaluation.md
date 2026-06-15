# Evaluation: Node, Value, and operators

Files: [`src/node.rs`](../src/node.rs), [`src/value.rs`](../src/value.rs),
[`src/value_op.rs`](../src/value_op.rs), [`src/debug.rs`](../src/debug.rs)

## The AST: `Node`

```rust
pub enum Node {
    Value(Value),
    UnaryExpr  { op: UnaryOp,  val: Box<Node> },
    BinaryExpr { op: BinaryOp, lhs: Box<Node>, rhs: Box<Node> },
}
```

Operators are **enum variants** (`BinaryOp` / `UnaryOp`, defined in `value_op`),
each with an `apply()` method (the operation) and a `symbol()` method (for debug
output). `Node::eval` is therefore a three-line recursion: evaluate the children,
then `apply` the operator.

```rust
Node::Value(v)            => Ok(v),
Node::UnaryExpr { op, v } => op.apply(v.eval()?),
Node::BinaryExpr {op,l,r} => op.apply(l.eval()?, r.eval()?),
```

Evaluation is eager and bottom-up; errors short-circuit via `?`.

## `Value`

```rust
pub struct Value { pub num: Number, pub unit: Option<Unit> }
```

A number with an optional unit. `From<i64>`/`From<f64>` make unitless values;
`From<Unit>` makes the quantity `1 <unit>` (used when a bare unit is parsed).
`Display` prints `"<num> <unit>"` or just `"<num>"`, delegating to
[`Number`](numbers.md)'s formatting and `unit::get_unit_name`.

## Operators: `value_op`

`value_op` is the bridge between pure-number arithmetic ([`number_op`](numbers.md))
and unit handling ([`unit`](units.md)). The `BinaryOp`/`UnaryOp` enums dispatch
(via `apply`) to one function per operator; each function combines the **numbers**
and the **units** and decides the result's unit:

| Function | Number step | Unit step / rule |
|----------|-------------|------------------|
| `add` / `sub` | `number_op::add/sub` | same unit → keep; else convert rhs into lhs's unit; mismatched types → `DifferentUnitTypes` |
| `mul` / `div` | `number_op::mul/div` | `unit::single`: one side unitless keeps the other; both united → `OperateWithUnits` |
| `pow` | `number_op::pow` | exponent must be unitless (`ExpByUnit`); result keeps base's unit |
| `conversion` | `unit::convert` | requires units on both sides; result takes the target unit |
| `sub_unary` | `number_op::sub_unary` | unit unchanged |

This is the layer to edit when changing **how an operator treats units** (the
numeric behavior lives one level down in `number_op`).

## Why operators are an enum

Operators are modelled as the `BinaryOp`/`UnaryOp` enums rather than bare `fn`
pointers. The enum carries the operator's identity directly, so `debug.rs` can
ask for `op.symbol()` to pretty-print the parsed tree (e.g. `((5+3)*2)`) — no
fragile pointer comparison. (The earlier `fn`-pointer design relied on comparing
function addresses, which rustc now warns is unreliable.) To add an operator: add
a variant and extend the `apply`/`symbol` `match` arms in `value_op.rs` — the
compiler's exhaustiveness checking guarantees you don't miss one.

## Worked example

Input `5 m 10 cm to cm`:

```
conversion(                      ← term "to" unit
  add(                           ← num_unit folds the compound quantity
    Value{5, m},
    Value{10, cm}),
  Value{1, cm})
```

`add` converts `10 cm` into metres (`0.1 m`) → `5.1 m`; `conversion` converts
`5.1 m` into cm → `510 cm`.
