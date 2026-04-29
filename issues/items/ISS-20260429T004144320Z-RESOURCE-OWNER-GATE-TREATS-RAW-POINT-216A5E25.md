---
id: ISS-20260429T004144320Z-RESOURCE-OWNER-GATE-TREATS-RAW-POINT-216A5E25
title: "Resource owner gate treats raw pointer reads as owner transfers"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/owner_check.rs, nepl-core/src/compiler.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260429T004144320Z-RESOURCE-OWNER-GATE-TREATS-RAW-POINT-216A5E25: Resource owner gate treats raw pointer reads as owner transfers

## 概要

Resource IR owner checker currently transfers free obligation on ordinary ResourceOp::Read. Enabling owner diagnostics as compiler errors therefore makes raw pointer reads such as load_i32 p or dealloc_raw p see p as Moved/NoFreeObligation, and it preempts the intended D3025 raw identity escape diagnostics with D3100.

## 対象

- `nepl-core/src/resource/owner_check.rs, nepl-core/src/compiler.rs, tests/compiler/move_effect.n.md`

## 根拠

- Resource owner gate を試験的に compiler pipeline へ接続して `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\agent1-resource-owner-borrow-gate-move-effect.json -j 1` を実行すると、110 件中 97 件 pass / 13 件 failed になった。
- positive case の `doctest#1` では `load_i32 p` の後に `dealloc_raw p` するだけで `Read ... found Moved` と `Dealloc ... found NoFreeObligation` の D3100 が出た。これは raw pointer value の read/copy を owning transfer と見なしている false positive である。
- 既存の D3025 raw identity escape compile_fail 群では、`doctest#3` / `#7` / `#8` / `#9` / `#10` / `#11` / `#12` / `#13` / `#14` が Resource owner D3100 に先取りされ、期待される effect / raw identity diagnostic が見えなくなった。
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は `MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell/Resource IR = initialized/moved/drop state` の責務分離を前提にしているが、現行 owner checker は read を owner state transition として扱っており、この分離に達していない。

## 問題

Resource IR owner checker currently transfers free obligation on ordinary ResourceOp::Read. Enabling owner diagnostics as compiler errors therefore makes raw pointer reads such as load_i32 p or dealloc_raw p see p as Moved/NoFreeObligation, and it preempts the intended D3025 raw identity escape diagnostics with D3100.

## 影響

RV-CORE-009 cannot safely make owner obligation diagnostics authoritative until raw pointer aliases and owner transfer are separated. Otherwise correct raw storage use and existing move_effect compile_fail expectations regress.

## 修正方針

Separate non-owning raw pointer alias reads from storage-owner transfer. Reads/copies of raw address or MemPtr should preserve an alias to the same owner storage, while dealloc/realloc/return/construct owner transfer should consume the free obligation exactly once. After that, enable the Resource owner gate in compiler.rs.

## 検証

Temporarily enable the Resource owner gate and run trunk build, tests/compiler/move_effect.n.md, tests/compiler/move_check.n.md, and resource_ir owner tests. The D3025 raw identity escape cases must remain D3025, and owner leak/double-free cases must become D3100 without false positives.

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check -- --nocapture`: 16 passed
- `cargo test -p nepl-core compiler::tests::resource_owner_gate -- --nocapture`: 3 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\agent1-resource-owner-pointer-read-move-effect-2.json -j 1`: total=110, passed=110, failed=0
- `node nodesrc\tests.js -i tests\compiler\move_check.n.md --no-tree -o tmp\agent1-resource-owner-pointer-read-move-check.json -j 1`: total=52, passed=52, failed=0

## 対応結果

Resource owner checker で `ResourceOp::Read` を free obligation transfer ではなく raw pointer alias propagation として扱うようにした。owner を消費する `DeclareLocal` / `Assign` / `Move` / `Construct` / `Return` / `Dealloc` / `Realloc` は alias から実 owner place を解決し、その place の free obligation を 1 回だけ移動または解放する。

compiler pipeline では Resource owner gate を D3100 に接続した。ただし D3025 raw identity escape を owner leak が先取りしないよう、effect boundary gate を owner gate より先に実行する。固定 raw address など storage origin を持たない unmanaged address の `NoFreeObligation` は、`ISS-20260429T012328323Z-RESOURCE-IR-LACKS-STORAGE-ORIGIN-FOR-549F82A4` に分離し、owned storage と unmanaged storage の分類を追加するまで shadow-only にする。

## 関連

- 親 issue: `ISS-20260425T000000Z-RV-CORE-009-58589A3F`
- 後続 issue: `ISS-20260429T012328323Z-RESOURCE-IR-LACKS-STORAGE-ORIGIN-FOR-549F82A4`
- 関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
