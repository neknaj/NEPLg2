---
id: ISS-20260428T103216940Z-RESOURCE-EFFECT-GATE-LOSES-RAW-ALLOC-DC80BAD0
title: "Resource effect gate loses raw allocation identity stored through raw memory slots"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/effect.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T103216940Z-RESOURCE-EFFECT-GATE-LOSES-RAW-ALLOC-DC80BAD0: Resource effect gate loses raw allocation identity stored through raw memory slots

## 概要

Stage 5 raw identity escape detection tracks copies, aggregates, calls, and callbacks, but not identity values written to raw memory and later loaded back. A pure function can alloc_raw p, store_i32 slot p, then return load_i32 slot without RawAddressEscapeFromInternalAlloc.

## 対象

- `nepl-core/src/resource/effect.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` は Stage 5 で `InternalAlloc` が public raw identity を漏らさない場合だけ `Pure` へ fold できることを求めている。
- `nepl-core/src/resource/effect.rs` は raw identity を local copy、aggregate、direct call、known function value、higher-order callback では追跡するが、raw memory cell に保存された identity payload を追跡していなかった。
- `UnsafeMemoryInPureFunction` は Stage 6 の stdlib migration 前なのでまだ通常 compiler error にしていない。そのため `store_i32 slot p` / `load_i32 slot` のような raw memory detour は、戻り値が raw identity かどうかを Stage 5 側で判定する必要がある。
- 既存の `pure からメモリ操作を呼べる` 回帰は、通常の数値を raw slot に store/load する内部処理は許可する意図を示している。必要なのは raw memory operation 全体の禁止ではなく、raw identity payload の public escape 検出である。

## 問題

Stage 5 raw identity escape detection tracks copies, aggregates, calls, and callbacks, but not identity values written to raw memory and later loaded back. A pure function can alloc_raw p, store_i32 slot p, then return load_i32 slot without RawAddressEscapeFromInternalAlloc.

## 影響

User code can launder an internal allocation raw address through a raw i32 slot while UnsafeMemoryInPureFunction remains shadow-only during stdlib migration. This bypasses the intended public escape diagnostics without requiring direct return or function call propagation.

## 修正方針

Track raw identity payloads stored into ResourceOp::RawMemory Store operations and mark ResourceOp::RawMemory Load outputs when loading from a slot that currently carries a tracked raw identity. Preserve payload state through realloc and bulk copy/move where Resource IR can see the source and destination. Keep ordinary numeric store/load accepted and add focused regressions.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/stage5-raw-slot-identity-escape.json -j 1; node nodesrc/issues.js check

## 2026-04-28 Stage 5 raw slot identity payload 対応

Resource IR effect boundary checker に raw memory identity payload table を追加した。`ResourceOp::RawMemory { operation: Store }` が tracked raw identity value を raw slot に書き込んだ場合、その slot を identity payload holder として記録する。`Load` が同じ slot から値を読む場合は load output へ identity を伝播する。

通常の数値 store は identity payload holder を clear するため、既存の pure internal numeric store/load は許可される。`Realloc` は旧 slot が identity payload を持つ場合に新 output へ payload を移し、`BulkCopy` / `BulkMove` は source slot の payload を destination slot へ伝播する。`Dealloc` では slot の identity payload を clear し、branch / loop / match merge ではいずれかの経路で identity payload を持ち得る slot を保守的に保持する。

`tests/compiler/move_effect.n.md` に `alloc_raw` address を `store_i32` / `load_i32`、`realloc_raw`、`mem_copy` で laundering する compile_fail を追加し、`nepl-core/tests/resource_ir.rs` に Resource IR checker 単体の回帰を追加した。
