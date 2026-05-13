---
id: ISS-20260513T100047236Z-REGION-PTR-AT-RETURNS-TYPED-MEMPTR-W-39BD1C91
title: "region_ptr_at returns typed MemPtr without alignment proof"
area: stdlib
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "stdlib/core/mem/pointer/region.nepl, tests/stdlib/memory_safety.n.md, nodesrc/test_stdlib_core_mem_boundary.js"
---

# ISS-20260513T100047236Z-REGION-PTR-AT-RETURNS-TYPED-MEMPTR-W-39BD1C91: region_ptr_at returns typed MemPtr without alignment proof

## 概要

region_ptr_at checks that off..off+size_of<U> fits in RegionToken, but it returns MemPtr<U> without proving the actual address base+off is aligned to align_of<U>. The comment explicitly delegated alignment to later typed wrappers, leaving a typed pointer projection whose safety precondition was not enforced at the owner boundary.

## 対象

- `stdlib/core/mem/pointer/region.nepl, tests/stdlib/memory_safety.n.md, nodesrc/test_stdlib_core_mem_boundary.js`

## 根拠

- `stdlib/core/mem/pointer/region.nepl` の `region_ptr_at` は `off < 0`、`size_of<U> < 0`、`off + size_of<U> <= region_size`、加算 overflow の検査だけを行っていた。
- 同じ箇所のドキュメントは「alignment は現時点では検査しません」と明記しており、`MemPtr<U>` を返す API なのに `align_of<U>` を確認していなかった。
- `tests/stdlib/memory_safety.n.md` には範囲外 offset の regression はあったが、`RegionToken<u8>` から offset 1 で `MemPtr<i32>` を得るような unaligned typed projection の regression がなかった。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 親 issue: [ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84](./ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md)

## 問題

region_ptr_at checks that off..off+size_of<U> fits in RegionToken, but it returns MemPtr<U> without proving the actual address base+off is aligned to align_of<U>. The comment explicitly delegated alignment to later typed wrappers, leaving a typed pointer projection whose safety precondition was not enforced at the owner boundary.

## 影響

A caller can obtain MemPtr<i32> from a byte-aligned RegionToken<u8> at offset 1. Resource IR can then reason about initialized/moved cells through a typed pointer whose address does not satisfy the target type layout, so memory/type safety depends on caller discipline instead of compiler/library proof.

## 修正方針

Make region_ptr_at reject projections whose actual address has a non-zero signed remainder by align_of<U>, update the docs to state that bounds and alignment are both checked, and add regression coverage for unaligned typed projection.

## 検証

node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/region-ptr-at-alignment.json -j 1 --dist web/dist; node nodesrc/test_stdlib_core_mem_boundary.js; node nodesrc/issues.js check --dir issues

## 修正結果

- `region_ptr_at` が `align_of<U>` を取得し、実 address `base + off` が target type alignment を満たす場合だけ `MemPtr<U>` を返すようにした。
- bounds check は従来の `off..off+size_of<U>` と overflow guard を維持し、さらに raw address 加算が wrap して base より小さくなる projection も拒否する。
- ドキュメントから「alignment は呼び出し側で検査する」という責務漏れを削除し、owner boundary で bounds と alignment の両方を検査する契約に更新した。
- `tests/stdlib/memory_safety.n.md` と `stdlib/core/mem/pointer/region.nepl` の doctest に、`RegionToken<u8>` の offset 1 から `MemPtr<i32>` を得ようとすると `Err` になる regression を追加した。
- `nodesrc/test_stdlib_core_mem_boundary.js` に `align_of<U>` / `rem_s addr align` / 旧コメント禁止の source policy を追加した。

## 検証結果

- `node nodesrc/test_stdlib_core_mem_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/core/mem/pointer/region.nepl --no-tree -o tmp/region-ptr-at-alignment-stdlib-doc.json -j 1 --dist web/dist`: total=6, passed=6
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/region-ptr-at-alignment-memory-safety.json -j 1 --dist web/dist`: total=27, passed=27
