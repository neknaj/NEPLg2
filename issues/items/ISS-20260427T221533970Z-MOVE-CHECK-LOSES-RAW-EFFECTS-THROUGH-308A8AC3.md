---
id: ISS-20260427T221533970Z-MOVE-CHECK-LOSES-RAW-EFFECTS-THROUGH-308A8AC3
title: "move_check loses raw effects through enum-wrapped function values"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T221533970Z-MOVE-CHECK-LOSES-RAW-EFFECTS-THROUGH-308A8AC3: move_check loses raw effects through enum-wrapped function values

## 概要

Function value aliases are tracked for variables and function parameters, but not for enum payloads. A callback stored in Option::Some can be match-bound and called indirectly without propagating the callback's raw memory effects to the caller.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `move_check` は `@fn` や function-typed parameter の alias を `function_value_alias_stacks` と関数サマリに保持していたが、`Option::Some @fn` のような enum payload には保持していなかった。
- `match opt: Option::Some f:` で payload を bind しても `f` に function value alias が復元されず、`CallIndirect` が concrete callee の raw memory effect summary を参照できなかった。
- 修正前の inline compile_fail probe では、`Option::Some @clobber_i32` を helper に渡して match-bind 後に呼ぶと、caller の live `LocalToken` raw place が `store_i32` で上書きされても compile が成功した。

## 問題

Function value aliases are tracked for variables and function parameters, but not for enum payloads. A callback stored in Option::Some can be match-bound and called indirectly without propagating the callback's raw memory effects to the caller.

## 影響

Higher-order stdlib/self-host code can hide raw memory writes inside Option/Result payloads and bypass D3100 memory-safety checks on live non-Copy raw places.

## 修正方針

Extend move_check summaries and context stacks to preserve function value aliases in enum payloads, instantiate them across function calls, and seed/match-bind function-typed payload placeholders.

## 解決内容

`MoveCheckContext` と `FunctionRawAliasSummary` に enum payload function alias を追加し、`let` / `set`、snapshot / restore、branch merge、function summary instantiation へ接続した。function-typed enum payload を持つ parameter には `$fnparam_enum_payload:N:Variant` placeholder を seed し、outer call で concrete callback へ展開する。

`match` の payload bind では、scrutinee に保存されている enum payload function alias を bind local の function value alias として復元するようにした。これにより、`Option::Some @clobber_i32` などに包まれた callback を helper 内で `f p` と呼んだ場合も、callback 内の raw memory write が caller の raw ownership state に伝播する。

variant 名は `Some` / `Option::Some` の表記差で alias を落とさないように、保存済み map の key を short variant name でも照合するようにした。

## 検証

- `tests/compiler/move_effect.n.md` に、`Option::Some @clobber_i32` を helper へ渡し、match-bind した callback が live non-Copy raw place を `store_i32` で上書きする compile_fail regression を追加した。
- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test move_check`: 51/51 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/enum-function-raw-effect-summary-after.json -j 1`: 92/92 passed
- `cargo check -p nepl-core --tests`: pass
