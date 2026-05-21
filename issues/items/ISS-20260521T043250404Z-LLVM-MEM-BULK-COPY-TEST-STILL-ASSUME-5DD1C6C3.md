---
id: ISS-20260521T043250404Z-LLVM-MEM-BULK-COPY-TEST-STILL-ASSUME-5DD1C6C3
title: "LLVM mem bulk copy test still assumes user raw boundary access"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/tests/neplg2.rs
---

# ISS-20260521T043250404Z-LLVM-MEM-BULK-COPY-TEST-STILL-ASSUME-5DD1C6C3: LLVM mem bulk copy test still assumes user raw boundary access

## 概要

nepl-core/tests/neplg2.rs::llvm_mem_bulk_copy_stdlib_lowers_to_intrinsics calls mem_copy and mem_move from ordinary user source while current Stage 6 raw-memory boundary correctly rejects direct raw operations without compiler-owned source evidence.

## 対象

- `nepl-core/tests/neplg2.rs`

## 根拠

- `cargo test -p nepl-core --test neplg2 llvm_mem_bulk_copy_stdlib_lowers_to_intrinsics -- --test-threads=1 --exact` が current worktree で `effect.pure.calls_impure` と `resource.raw.memory_outside_boundary` により失敗した。
- 同じ exact test は clean worktree の `HEAD=89fae3cb` でも同じ診断で失敗したため、collection slot lifecycle capability span 修正による新規 regression ではない。
- 既に修正済みの [ISS-20260518T012520895Z-COMPILER-INTRINSIC-DOCTESTS-STILL-AS-4AD0DA0D](./ISS-20260518T012520895Z-COMPILER-INTRINSIC-DOCTESTS-STILL-AS-4AD0DA0D.md) と同じく、ordinary user source から direct raw memory operation を runtime positive fixture として実行しようとしている古い前提が残っている。

## 問題

nepl-core/tests/neplg2.rs::llvm_mem_bulk_copy_stdlib_lowers_to_intrinsics calls mem_copy and mem_move from ordinary user source while current Stage 6 raw-memory boundary correctly rejects direct raw operations without compiler-owned source evidence.

## 影響

The broad intrinsic Rust test remains red on current main and can pressure future work to weaken resource.raw.memory_outside_boundary or effect.pure.calls_impure instead of moving codegen coverage to a compiler-owned raw boundary harness.

## 修正方針

Move LLVM bulk copy lowering coverage to a harness that supplies compiler-owned core/mem raw boundary provenance, or change the user-source fixture to a compile-fail boundary regression while keeping lowering assertions in a proper compiler/codegen test.

## 修正内容

- `llvm_mem_bulk_copy_stdlib_lowers_to_intrinsics` を ordinary user source から raw memory helper を呼ぶ fixture ではなく、configured stdlib root 配下の `core/mem/raw.nepl` として inline load する codegen boundary fixture に変更した。
- fixture は `mem_copy` / `mem_move` の LLVM lowering だけを検査する最小の compiler-owned raw boundary source とし、user source から `mem_copy` / `mem_move` を直接呼んで `effect.pure.calls_impure` / `resource.raw.memory_outside_boundary` を踏む古い前提を取り除いた。
- raw memory boundary は緩めていない。positive codegen coverage は compiler-owned stdlib raw boundary provenance で実行し、ordinary user source の raw operation は引き続き静的検査で拒否される。
- actual `stdlib/core/mem/raw.nepl` の import graph をこの unit test に直接取り込むと circular import の既存問題に当たるため、今回の regression は LLVM intrinsic lowering と source provenance 境界だけを分離して固定した。

## 検証

- `cargo test -p nepl-core --test neplg2 llvm_mem_bulk_copy_stdlib_lowers_to_intrinsics -- --test-threads=1 --exact`
- `cargo test -p nepl-core intrinsic -- --test-threads=1`
