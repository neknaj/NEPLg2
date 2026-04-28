---
id: ISS-20260428T000058805Z-MOVE-CHECK-DROPS-RAW-EFFECTS-THROUGH-F9FD5C67
title: "move_check drops raw effects through aggregate function fields"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260428T000058805Z-MOVE-CHECK-DROPS-RAW-EFFECTS-THROUGH-F9FD5C67: move_check drops raw effects through aggregate function fields

## 概要

Function values stored in struct or tuple fields are not tracked as aggregate field aliases. field::get can recover the callback value, but CallIndirect then has no callee alias and skips raw memory effect checks.

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `CallbackHolder` の field に `@clobber_i32` を保存し、`field::get holder "cb"` で取り出して `f pi` を呼ぶ repro が `compile_fail` 期待にもかかわらず compiled successfully になった。
- `field::get` は typecheck で `#intrinsic "load"` に lower されるため、function field load から aggregate field の function alias を復元する経路が必要だった。
- 修正前の `move_check` は enum payload function alias は保持していたが、struct/tuple field の function alias と、関数引数 aggregate field の `$fnparam` placeholder を持っていなかった。

## 問題

Function values stored in struct or tuple fields are not tracked as aggregate field aliases. field::get can recover the callback value, but CallIndirect then has no callee alias and skips raw memory effect checks.

## 影響

A callback stored inside an aggregate can overwrite a live non-Copy raw place through MemPtr without D3100, bypassing compiler raw ownership checks.

## 修正方針

Track function-value aliases in aggregate fields and propagate them through aggregate construction, field projection, function summaries, and indirect raw effect application.

## 対応

- aggregate field function alias state を `MoveCheckContext` / `ValueAliasSummary` / `FunctionRawAliasSummary` に追加した。
- struct/tuple construction、`let` / `set`、branch merge、function summary instantiation で aggregate field function alias を伝播するようにした。
- enum payload 内 aggregate field の function alias も保持し、match bind で復元できるようにした。
- function-typed aggregate field parameter には `$fnparam_field:*` / `$fnparam_enum_payload_field:*` placeholder を seed し、call site で concrete callback 候補へ展開するようにした。
- `field::get` が lower された `#intrinsic "load"` から function field alias を復元し、`CallIndirect` の raw memory effect summary に接続した。
- `tests/compiler/move_effect.n.md` に aggregate field callback が D3100 になる回帰テストを追加した。

## 検証

- `cargo fmt --check`: pass
- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --test move_check -- --nocapture`: 51/51 passed
- `cargo test -p nepl-core --test check_pipeline move_check_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/aggregate-function-raw-effects.json -j 1`: 97/97 passed
- `node nodesrc/issues.js check`: pass
