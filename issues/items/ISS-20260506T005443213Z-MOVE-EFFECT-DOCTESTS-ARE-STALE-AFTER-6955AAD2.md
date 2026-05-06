---
id: ISS-20260506T005443213Z-MOVE-EFFECT-DOCTESTS-ARE-STALE-AFTER-6955AAD2
title: "move_effect doctests are stale after Resource IR and effect gates"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-06
updated: 2026-05-06
target: "tests/compiler/move_effect.n.md, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260506T005443213Z-MOVE-EFFECT-DOCTESTS-ARE-STALE-AFTER-6955AAD2: move_effect doctests are stale after Resource IR and effect gates

## 概要

A focused run of tests/compiler/move_effect.n.md after the Resource IR/effect gate migration reports 94 passed and 36 failed. Several fixtures still use raw memory operations in pure functions or load non-Copy raw cells from uninitialized fixed addresses while expecting older resource.cell/resource.move diagnostics.

## 対象

- `tests/compiler/move_effect.n.md, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- 2026-05-06 の focused run で `tests/compiler/move_effect.n.md` は 130 件中 94 passed / 36 failed だった。
- 主な失敗は `effect.pure.calls_impure` が先に出る raw memory fixture、`resource.cell.uninit` が先に出る未初期化 raw load fixture、Resource IR の `resource.cell.*` が legacy `resource.move.*` より先に出る diagnostic taxonomy drift である。
- fixture 更新後に残った direct `Result::Ok` payload match の失敗は、raw alias summary 自体は存在していたが、`RawCellAddressAliases::union_group` が既存 alias group へ新しい束縛名を足す際に同順位の新規 local を canonical に押し出し、CellTable の raw cell state と後続 load の canonical address が分裂していた。
- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 4/5 方針では compiler を弱めず、fixture 側を現在の Resource IR / effect gate authority に合わせる必要がある。

## 問題

A focused run of tests/compiler/move_effect.n.md after the Resource IR/effect gate migration reports 94 passed and 36 failed. Several fixtures still use raw memory operations in pure functions or load non-Copy raw cells from uninitialized fixed addresses while expecting older resource.cell/resource.move diagnostics.

## 影響

The suite no longer cleanly isolates effect boundary, raw cell initialization, moved-cell, and legacy move diagnostics. CI failures can be misread as compiler regressions, or stale expectations can pressure the compiler to weaken static safety.

## 修正方針

Split the fixtures by invariant: keep pure raw operation tests expecting effect.pure.calls_impure, mark raw cell state fixtures impure, initialize raw storage before moved-cell assertions, and update diag_code expectations to Resource IR cell/owner/effect taxonomy without restoring legacy buckets. Fix the Resource IR raw alias canonicalization root cause so direct enum payload aliases preserve the existing raw cell address canonical instead of masking moved-cell diagnostics as uninit.

## 検証

trunk build; cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_direct_result_payload_raw_address_alias -- --nocapture; cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves -- --nocapture; cargo check -p nepl-core --tests; node nodesrc/tests.js -i tests/compiler/move_effect.n.md -o output/move_effect_after_fix.json --runner wasm --no-tree -j 1

## 対応内容

- `tests/compiler/move_effect.n.md` を現在の Resource IR / effect gate authority に合わせ、pure raw operation、impure raw memory fixture、raw cell initialization、Resource IR cell diagnostic の期待値を分離した。
- enum payload の raw address alias が fully-qualified variant と match arm variant で分裂しないよう、aggregate field place の enum payload variant 名を canonical 化した。
- raw alias group の合流時に既存 group の canonical を同順位の新規 alias が押し出さないよう、`RawCellAddressAliases::union_group` の merge 順序を修正した。
- direct `Result::Ok` payload match を介した `MemPtr` alias で、2 回目の non-Copy raw load が `resource.cell.moved` として検出される Resource IR regression を追加した。
