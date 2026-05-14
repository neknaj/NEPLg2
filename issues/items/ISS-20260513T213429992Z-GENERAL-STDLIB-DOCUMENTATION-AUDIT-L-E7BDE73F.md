---
id: ISS-20260513T213429992Z-GENERAL-STDLIB-DOCUMENTATION-AUDIT-L-E7BDE73F
title: "general stdlib documentation audit lacks non-kp guidance"
area: docs
status: fixed
resolved: true
priority: P1
type: doc
created: 2026-05-13
updated: 2026-05-13
target: "doc/neplg2/stdlib_documentation_style_guide.md, doc/neplg2/stdlib_documentation_contract_plan.md"
---

# ISS-20260513T213429992Z-GENERAL-STDLIB-DOCUMENTATION-AUDIT-L-E7BDE73F: general stdlib documentation audit lacks non-kp guidance

## 概要

The current stdlib documentation style guide uses kpgraph as the concrete audit example, but kp is a performance-oriented special layer. General stdlib modules such as alloc/hash/sha256, core/result, alloc/string/storage, alloc/io/bytebuf, std/test/types, and std/streamio/scanner/state need separate guidance.

## 対象

- `doc/neplg2/stdlib_documentation_style_guide.md, doc/neplg2/stdlib_documentation_contract_plan.md`

## 根拠

- `stdlib/alloc/hash/sha256.nepl` は facade / submodule 責務、incremental API、finalize の state 消費、内部 buffer 解放、計算量の説明がある一方、facade doctest が不足している。
- `stdlib/core/result.nepl` は `Copy` 制約、`should_panic`、`compile_fail` が良いが、旧見出しと `ret:` 中心 doctest が残る。
- `stdlib/alloc/string/storage.nepl` は raw storage layout と owner 境界の説明が厚いが、内部 raw helper の safety contract は継続して厚く保つ必要がある。
- `stdlib/alloc/io/bytebuf.nepl` は owner doc と module doctest が良いが、小さい public helper の declaration doc が不足している。
- `stdlib/alloc/collections/vec.nepl` と `vec/types.nepl` は facade、storage state enum、`Copy` 制約、再確保、move-after-use `compile_fail` を説明しており基準例になる。一方で bitset / adjacency_matrix / fenwick / binary_heap などの layout/storage/order helper には declaration doc 欠落が多く、collection 固有の owner flow、slot invariant、index formula、algorithm complexity を整理する必要がある。
- `stdlib/std/test/types.nepl` は enum / struct による test report model が良いが、stable renderer output と owner-consuming helper の contract が薄い。
- `stdlib/std/streamio/scanner/state.nepl` は scanner copy / close 規則と typed cursor storage の方向が良いが、`ByteBuf` owner と cursor storage を分ける理由、constructor の owner 消費、token slice helper の memory-safety contract が不足している。

## 問題

The current stdlib documentation style guide uses kpgraph as the concrete audit example, but kp is a performance-oriented special layer. General stdlib modules such as alloc/hash/sha256, core/result, alloc/string/storage, alloc/io/bytebuf, std/test/types, and std/streamio/scanner/state need separate guidance.

## 影響

Documentation policy can overfit kp-style raw/performance examples and miss normal stdlib requirements: facade usage doctests, public helper documentation, stable enum/string output contracts, ownership and effect notes, and stdout/assertion-oriented doctests.

## 修正方針

Audit representative general stdlib files and update the style guide and contract plan with non-kp findings and migration guidance.

## 検証

Run the stdlib documentation contract policy and issue metadata checks.

## 対応結果

- `doc/neplg2/stdlib_documentation_style_guide.md` に一般 stdlib 追加監査を追加し、`sha256` / `result` / `string storage` / `ByteBuf` / `alloc/collections` / `test types` / `streamio scanner state` の良い点と不足を整理した。
- `doc/neplg2/stdlib_documentation_contract_plan.md` に同監査を反映し、Stage 1 から Stage 3 で通常利用される `core` / `alloc` / `std` API を優先する方針を追記した。
- これは実装本体の stdlib 修正ではなく、今後の doc 整備で `kp` に偏らない基準を使うための issue として解決した。
