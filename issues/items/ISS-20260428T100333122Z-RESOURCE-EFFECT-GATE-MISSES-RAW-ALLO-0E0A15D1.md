---
id: ISS-20260428T100333122Z-RESOURCE-EFFECT-GATE-MISSES-RAW-ALLO-0E0A15D1
title: "Resource effect gate misses raw allocation identity wrapped in aggregates"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/effect.rs, nepl-core/src/compiler.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T100333122Z-RESOURCE-EFFECT-GATE-MISSES-RAW-ALLO-0E0A15D1: Resource effect gate misses raw allocation identity wrapped in aggregates

## 概要

Stage 5 raw identity escape detection only tracks direct place copies. If alloc_raw or realloc result is wrapped in a tuple, struct, or enum constructor and returned from a pure function, the public surface can still carry compiler-internal raw address identity without triggering D3025.

## 対象

- `nepl-core/src/resource/effect.rs, nepl-core/src/compiler.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 5 は、internal allocation が `Pure` へ fold できる条件を「raw identity / owner token が public surface へ漏れないこと」としている。
- `nepl-core/src/resource/effect.rs` の `RawIdentityTable` は `RawMemoryOp::Alloc` / `Realloc` 由来 place と、`let` / `read` / `move` / `assign` / branch / match の value copy だけを追跡していた。
- `ResourceOp::Construct` は tuple / struct / enum payload を表すが、入力 place の raw identity を output place へ伝播していなかった。
- そのため `alloc_raw` の戻り値を `RawBox p` のような aggregate に包むと、pure function の戻り値は raw identity を含んでいても D3025 に昇格されなかった。

## 問題

Stage 5 raw identity escape detection only tracks direct place copies. If alloc_raw or realloc result is wrapped in a tuple, struct, or enum constructor and returned from a pure function, the public surface can still carry compiler-internal raw address identity without triggering D3025.

## 影響

Safe source code can hide an internal allocation raw address in an aggregate, then unwrap it later and use raw memory outside the intended compiler-owned boundary. This weakens the Stage 5 guarantee that internal allocation may fold to pure only when raw identity does not escape.

## 修正方針

Propagate raw identity through ResourceOp::Construct in the Resource IR effect boundary checker, keep branch/match merge conservative, and add regression tests for aggregate-wrapped allocation escape.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/stage5-aggregate-raw-identity-escape.json -j 1; node nodesrc/issues.js check

## 2026-04-28 Stage 5 aggregate raw identity escape 対応

`ResourceOp::Construct` の inputs に internal allocation identity が含まれる場合、construct output も同じ raw identity を運ぶものとして `RawIdentityTable` に登録するようにした。これにより、`alloc_raw` / `realloc` の戻り値を struct / tuple / enum に包んで pure function から返す経路も `RawAddressEscapeFromInternalAlloc` として検出できる。

この修正は `UnsafeMemoryInPureFunction` の強制範囲を広げるものではなく、Stage 5 commit 単位 4 の public escape diagnostics を aggregate construction に拡張する。`tests/compiler/move_effect.n.md` には `RawBox` に包んだ allocation address が D3025 になる compile_fail を追加し、`nepl-core/tests/resource_ir.rs` には Resource IR checker 単体の回帰を追加した。
