---
id: ISS-20260519T124203801Z-VEC-RAW-ELEMENT-HELPERS-ARE-DIRECTLY-85EBD72F
title: "Vec raw element helpers are directly callable from ordinary source"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/alloc/collections/vec/raw.nepl; stdlib/alloc/collections/vec/raw/element.nepl; stdlib/alloc/collections/vec/{query,mutation,transform}"
---

# ISS-20260519T124203801Z-VEC-RAW-ELEMENT-HELPERS-ARE-DIRECTLY-85EBD72F: Vec raw element helpers are directly callable from ordinary source

## 概要

alloc/collections/vec/raw publicly re-exports unchecked vec_read_at and vec_write_at. Ordinary user source can explicitly import alloc/collections/vec/raw and call these helpers with data_mem_ptr, bypassing Vec len/storage checks and initialized-cell discipline. The Resource IR sees the raw load/store at the compiler-owned stdlib callee span, so the caller-side API boundary is not enforced.

## 対象

- `stdlib/alloc/collections/vec/raw.nepl; stdlib/alloc/collections/vec/raw/element.nepl; stdlib/alloc/collections/vec/{query,mutation,transform}`

## 根拠

- 2026-05-19 Agent 1 調査で、通常 source が `#import "alloc/collections/vec/raw" as raw` を明示し、`data_mem_ptr(&v)` と `raw::vec_write_at` / `raw::vec_read_at` を組み合わせると compile に成功することを確認した。
- この経路では `Vec.len` の更新や `get` / `replace` の範囲検査を通らず、capacity 内の未初期化 slot を読み書きできる。
- raw operation の span は compiler-owned stdlib callee 側にあるため、caller が任意の `MemPtr<T>` を渡している事実が API 境界で型として表現されていなかった。

## 問題

alloc/collections/vec/raw publicly re-exports unchecked vec_read_at and vec_write_at. Ordinary user source can explicitly import alloc/collections/vec/raw and call these helpers with data_mem_ptr, bypassing Vec len/storage checks and initialized-cell discipline. The Resource IR sees the raw load/store at the compiler-owned stdlib callee span, so the caller-side API boundary is not enforced.

## 影響

A program can write past Vec.len within allocated capacity or read uninitialized slots through public raw helpers. This violates the planned OwnedBuffer/InitializedCell separation and weakens memory/type-safety guarantees before self-host work.

## 修正方針

Remove the public Vec raw element helper facade and keep element load/store as private, module-local raw operations inside checked Vec operations only. Update source-policy regression coverage so vec/raw cannot reappear as an explicit public bypass.

## 対応

- `stdlib/alloc/collections/vec/raw.nepl` と `stdlib/alloc/collections/vec/raw/element.nepl` を削除し、`alloc/collections/vec/raw` を直接 import できる public bypass をなくした。
- `get` / `pop` の typed `load` と、`push` / `replace` / `map` / `filter` / `partition` / `take_while` / `drop_while` の typed `store` は、それぞれ len・storage variant・capacity・output index を検査する同じ source file 内に置いた。
- 新しい helper 関数を増やして doctest policy gap を増やさないよう、検査済み分岐内で raw operation を直接実行する形にした。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` と `nodesrc/test_stdlib_vec_borrowed_observers.js` を更新し、`vec/raw` facade / element helper が戻らないこと、Vec 実装が `../raw` に依存しないこと、callback-facing query/transform が raw load を callback へ直接渡さないことを監視する。
- `doc/neplg2/static_check_complexity_reduction_plan.md` と `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` を更新し、Stage 6 の Vec raw boundary 方針を direct-import helper ではなく検査済み source file 内 raw operation に改めた。

## 検証

Add/adjust source-policy regressions and compile-focused checks proving alloc/collections/vec/raw is no longer a public bypass; run focused Vec/collection policy tests and issue validation.

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: pass
- `node nodesrc/test_stdlib_documentation_contract.js`: pass
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`: pass
- `node nodesrc/test_stdlib_memptr_owner_field_policy.js`: pass
- `node nodesrc/test_stdlib_core_mem_boundary.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec -o tmp\agent1-vec-raw-boundary.json --no-tree -j 4`: total=52, passed=52
- `cargo run -p nepl-cli -- --target wasi --profile debug --input tmp\agent1-vec-raw-direct-import.nepl --output tmp\agent1-vec-raw-direct-import.wasm`: expected failure after deletion of `alloc/collections/vec/raw` direct import path
- `node nodesrc/run_source_policy_regressions.js --warn-only`: pass
