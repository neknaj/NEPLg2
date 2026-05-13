---
id: ISS-20260513T054043692Z-RESOURCE-CELL-GATE-NEEDS-STATIC-RAW--EDAE05BC
title: "Resource cell gate needs static raw-memory initialization proof"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-13
updated: 2026-05-13
target: nepl-core/src/resource/initialized_raw_memory.rs
---

# ISS-20260513T054043692Z-RESOURCE-CELL-GATE-NEEDS-STATIC-RAW--EDAE05BC: Resource cell gate needs static raw-memory initialization proof

## 概要

raw-memory-boundary の診断抑制を外した結果、stdlib/core/mem/allocator.nepl の load_i32 0/4 など、runtime のゼロ初期化済み線形メモリからの読み出しを Resource IR が証明できず resource.cell.uninit を出す。これは suppress で隠すのではなく、ソース上の raw memory operation と scalar address fact から証明する必要がある。

## 対象

- `nepl-core/src/resource/initialized_raw_memory.rs`

## 根拠

- `trunk build` 後の `tests/compiler/move_effect.n.md` で、`allocator.nepl` の `load_i32 0` / `load_i32 4` が `resource.cell.uninit` を出していた。
- `alloc` 由来の owned raw storage では未 store load を拒否する既存テストがあるため、単純に raw load 全体を許可するのは不適切。
- runtime の線形メモリはゼロ初期化される一方、Resource IR は scalar address fact と tracked raw storage の区別を見ていなかった。

## 問題

raw-memory-boundary の診断抑制を外した結果、stdlib/core/mem/allocator.nepl の load_i32 0/4 など、runtime のゼロ初期化済み線形メモリからの読み出しを Resource IR が証明できず resource.cell.uninit を出す。これは suppress で隠すのではなく、ソース上の raw memory operation と scalar address fact から証明する必要がある。

## 影響

fresh trunk build 後の tests/compiler/move_effect.n.md doctest が resource.cell.uninit で先に落ち、effect/type の期待診断まで到達できない。静的検査の正確性を損なう。

## 修正方針

RawMemory::Load の初期化判定を、所有 raw storage / 明示 store / byte-range 初期化 / 外部 raw storage に加えて、非負の既知スカラーアドレスが tracked raw storage に属していない場合は runtime zero-initialized linear memory として扱う形に再設計する。alloc 由来の owned raw storage は引き続き未 store 読み出しを拒否する。

## 検証

- `cargo test -p nepl-core resource_ir_cell_check_reports_raw_load_before_store -- --nocapture` 成功。
- `cargo test -p nepl-core resource_ir_cell_check_allows_zero_initialized_runtime_literal_raw_load -- --nocapture` 成功。
- `trunk build` 成功。
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/agent1-resource-raw-memory-proof-move-effect.json -j 1 --dist web/dist` は `resource.cell.uninit` の allocator 起点失敗が消えたことを確認。ただし別 Issue `ISS-20260513T054944970Z-ALLOCATOR-RUNTIME-ABI-IS-DECLARED-PU-4C2B8794` の allocator ABI / effect 設計不整合により 40 件が継続失敗。
