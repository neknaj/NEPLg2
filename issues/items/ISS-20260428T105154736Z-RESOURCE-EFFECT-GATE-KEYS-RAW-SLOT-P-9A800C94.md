---
id: ISS-20260428T105154736Z-RESOURCE-EFFECT-GATE-KEYS-RAW-SLOT-P-9A800C94
title: "Resource effect gate keys raw slot payloads only by internal allocation identity"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/effect.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T105154736Z-RESOURCE-EFFECT-GATE-KEYS-RAW-SLOT-P-9A800C94: Resource effect gate keys raw slot payloads only by internal allocation identity

## 概要

Stage 5 raw memory payload tracking only keys slots through RawIdentityTable groups. A function parameter or copied raw pointer that is not itself an internal allocation identity can receive an alloc_raw-derived value by store_i32 and later return it by load_i32 without D3025.

## 対象

- `nepl-core/src/resource/effect.rs, tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 5 は、internal allocation identity が public surface へ漏れない場合だけ `Pure` へ fold できることを求めている。
- `ISS-20260428T103216940Z-RESOURCE-EFFECT-GATE-LOSES-RAW-ALLOC-DC80BAD0` では raw memory slot 内の identity payload を追跡するようにしたが、slot key を `RawIdentityTable` の group に依存させていた。
- function parameter や caller-provided slot は、それ自体が internal allocation identity ではない。そのため `store_i32 slot p` / `load_i32 slot` のように同じ pointer value を使っても payload table の key を作れなかった。
- raw memory slot の key は「漏れてはいけない raw allocation identity」ではなく「同じ raw pointer value を指す alias group」で管理する必要がある。

## 問題

Stage 5 raw memory payload tracking only keys slots through RawIdentityTable groups. A function parameter or copied raw pointer that is not itself an internal allocation identity can receive an alloc_raw-derived value by store_i32 and later return it by load_i32 without D3025.

## 影響

Pure helper APIs can launder compiler-internal raw allocation identity through caller-provided or parameter-derived raw slots while UnsafeMemoryInPureFunction remains shadow-only. This leaves a public escape route for internal allocation addresses across function boundaries.

## 修正方針

Separate raw pointer alias groups from tracked raw allocation identity groups in ResourceEffectBoundaryEngine. Use pointer alias groups to key raw memory slots, while keeping RawIdentityTable only for values that carry internal allocation identity. Add regressions for parameter slots and copied pointer aliases.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/stage5-raw-slot-pointer-alias.json -j 1; node nodesrc/issues.js check

## 2026-04-28 Stage 5 raw slot pointer alias 対応

`ResourceEffectBoundaryEngine` に `RawPointerAliasTable` を追加し、raw allocation identity の伝播 (`RawIdentityTable`) と raw memory slot key の同一性 (`RawPointerAliasTable`) を分離した。`DeclareLocal` / `Read` / `Move` / `Assign` と branch / loop / match merge では pointer alias を伝播し、`RawMemoryIdentityTable` はこの alias group で slot payload を管理する。

これにより、function parameter や copied pointer alias が internal allocation identity でなくても、そこへ `alloc_raw` 由来の raw identity を store してから load / return する経路が D3025 になる。`tests/compiler/move_effect.n.md` には parameter slot、copied parameter slot、helper に渡した raw identity の slot laundering 回帰を追加した。`nepl-core/tests/resource_ir.rs` には Resource IR checker 単体で parameter slot alias を検出する回帰を追加した。
