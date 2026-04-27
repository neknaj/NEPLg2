---
id: ISS-20260427T152951013Z-RUNTIME-ALLOCATOR-HELPER-LOOKUP-DEPE-D070168E
title: "runtime allocator helper lookup depends on public core mem names"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/runtime_helpers.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, stdlib/core/mem.nepl"
---

# ISS-20260427T152951013Z-RUNTIME-ALLOCATOR-HELPER-LOOKUP-DEPE-D070168E: runtime allocator helper lookup depends on public core mem names

## 概要

compiler の codegen が allocator / deallocator / reallocator を探すとき、stable な compiler runtime ABI ではなく public stdlib function 名候補に依存している。`core/mem.nepl` の safe API 隔離や raw helper の非公開化を進めると、compiler 内部 helper discovery と stdlib 公開面が同時に壊れる。

## 対象

- `nepl-core/src/runtime_helpers.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, stdlib/core/mem.nepl`

## 根拠

- `nepl-core/src/runtime_helpers.rs:8` は alloc helper 候補を `["alloc_raw", "alloc"]` としている。
- `nepl-core/src/runtime_helpers.rs:9` は dealloc helper 候補を `["dealloc", "dealloc_raw"]` としている。
- `nepl-core/src/runtime_helpers.rs:10` は realloc helper 候補を `["realloc", "realloc_raw"]` としている。
- `nepl-core/src/runtime_helpers.rs:94` の regression test も internal codegen が `alloc_raw` を優先することを仕様化している。
- `stdlib/core/mem.nepl:278` / `386` / `450` の `alloc_raw` / `dealloc_raw` / `realloc_raw` は public source API として存在しており、compiler 内部 ABI と利用者向け API が同じ名前に乗っている。

## 問題

allocator helper lookup が public symbol name に依存しているため、`core/mem.nepl` 側で raw helper を non-public 化したり、compiler-owned boundary へ移したりする設計変更が codegen の暗黙仕様と衝突する。責務としては compiler が必要とする heap primitive は runtime ABI であり、stdlib safe API の名前探索で決まるべきではない。

## 影響

raw memory API の公開面を閉じる作業で codegen が helper を見失う、または互換名を残すために unsafe public API を温存する圧力が生まれる。self-host compiler では allocator boundary が固定されないまま stdlib と compiler が相互依存し、NEPLg2 self-host と NEPLg3 実装計画の両方で移行リスクになる。

## 修正方針

compiler runtime helper は public function name discovery ではなく、DefId / module-private symbol / reserved intrinsic のいずれかで解決する。`core/mem.nepl` には safe wrapper だけを残し、codegen が必要な alloc/dealloc/realloc は runtime ABI table または compiler-internal import として管理する。移行中は compatibility alias を残す場合でも public API と runtime ABI を別 issue / 別 test で区別する。

## 検証

`runtime_helpers` の unit test を、public `alloc_raw` 名優先ではなく compiler-owned ABI 優先へ更新する。`core/mem.nepl` から raw helper を非公開化しても wasm / llvm codegen が allocator を解決できる regression を追加する。stdlib safe API の rename が codegen helper lookup に影響しないことも確認する。

## 対応結果

- `nepl-core/src/runtime_helpers.rs` に compiler runtime ABI 名 `__nepl_rt_alloc` / `__nepl_rt_dealloc` / `__nepl_rt_realloc` を定義し、helper lookup は ABI 名を最優先、旧 public raw/safe 名は移行用 fallback とした。
- `helper_base_name` は予約 ABI 名の先頭 `__` と monomorphize suffix の `__...` を区別するようにした。これにより `__nepl_rt_alloc__i32__i32__pure` のような lowered symbol も runtime ABI helper として解決できる。
- `stdlib/core/mem.nepl` に compiler 予約 ABI wrapper を追加し、codegen が公開 `alloc_raw` 名を直接探索しなくても runtime allocator を保持・解決できるようにした。
- LLVM の aggregate allocation regression は `__nepl_rt_alloc` の呼び出しと定義を確認するよう更新した。`core/mem.nepl` raw body の許可境界は SourceMap path に依存するため、stdlib を読み込む LLVM tests は source-map aware API を使うようにした。

## 実施した検証

- `cargo test -p nepl-core runtime_helpers`: pass
- `cargo test -p nepl-core --test neplg2 llvm_allocator_helper_is_emitted_for_codegen_inserted_alloc`: pass
- `cargo test -p nepl-core --test neplg2 llvm_mem_bulk_copy_stdlib_lowers_to_intrinsics`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/intrinsic.n.md --no-tree -o tmp/runtime-helper-abi-intrinsic.json -j 1`: `total=8`, `passed=8`
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
