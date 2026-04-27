---
id: ISS-20260427T192528620Z-MOVE-CHECK-LOSES-PROVENANCE-FOR-MEM--A1AE98CC
title: "move_check loses provenance for pointer add with non-literal offset"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260427T192528620Z-MOVE-CHECK-LOSES-PROVENANCE-FOR-MEM--A1AE98CC: move_check loses provenance for pointer add with non-literal offset

## 概要

mem_ptr_add や raw address add の offset が literal でない場合、move_check の raw place key 生成が None になり、同じ base 由来の alias として扱われない。実行時に offset が 0 や既存 payload offset になれるため、non-Copy raw load/store/dealloc の検査を迂回できる。

## 対象

- `nepl-core/src/passes/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- `nepl-core/src/passes/move_check.rs` の `raw_memory_place_key_from_mem_ptr` は、`mem_ptr_add` の第2引数が `LiteralI32` のときだけ `base+offset` を返し、それ以外では `None` を返していた。
- `raw_memory_place_key` の raw `i32` address `add base off` も、`off` が literal でない場合は同様に `None` を返していた。
- 修正前再現では `let off choose_offset true; let q mem_ptr_add p off` とした後、`p` と `q` から同じ `LocalToken` を二重 `load` しても compiler が exit 0 で受理した。
- 既知 offset 0 の alias は `ISS-20260427T191722304Z-MOVE-CHECK-DOES-NOT-CANONICALIZE-MEM-FEAEF49B` で塞いだが、non-literal offset では base provenance まで消えていた。

## 問題

mem_ptr_add や raw address add の offset が literal でない場合、move_check の raw place key 生成が None になり、同じ base 由来の alias として扱われない。実行時に offset が 0 や既存 payload offset になれるため、non-Copy raw load/store/dealloc の検査を迂回できる。

## 影響

MemPtr pointer arithmetic 経由で同じ storage から non-Copy owner を二重に作ったり、live payload を raw operation で破壊できる可能性があり、compiler 側のメモリ安全検査が不健全になる。

## 修正方針

non-literal offset の pointer add は base provenance を保持した unknown-offset raw place として表現し、同じ base の既存 raw place と保守的に overlap させる。

## 対応結果

- `move_check` の raw place key を known offset / unknown offset に分け、unknown offset を `base+?` として保持するようにした。
- `raw_place_ranges_overlap` は同じ base で片側の offset が unknown の場合、保守的に overlap とみなすようにした。
- `mem_ptr_add base offset` と raw `i32` address `add base offset` は、offset が literal なら従来どおり `base+literal`、non-literal なら base provenance を保持した unknown-offset key に正規化する。
- unknown-offset key からの non-Copy load は、同じ base の既存 raw place を `PossiblyMoved` にし、後続の既知 offset load を D3100 にする。
- unknown-offset key への non-Copy store / dealloc / byte write / bulk copy は、同じ base の live non-Copy raw place と衝突する場合に D3100 にする。

## 検証

non-literal pointer add offset の二重 non-Copy load / live payload store/dealloc が D3100 で拒否される回帰テストを追加する。

2026-04-28 実施:

- `cargo fmt --check`: pass
- `cargo check -p nepl-core`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/memptr-add-unknown-offset-node.json -j 1`: `total=71`, `passed=71`
- 修正前再現 `tmp/memptr-add-unknown-offset-double-load.nepl` は修正後 `D3100` で拒否されることを確認した。
