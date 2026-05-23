---
id: ISS-20260523T032117760Z-VEC-QUERY-NEEDS-SCOPED-BORROWED-PRED-C29C915F
title: "Vec query needs scoped borrowed predicate observers for non-Copy payloads"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-23
updated: 2026-05-23
target: "stdlib/alloc/collections/vec/query/**, nepl-core/src/resource/**, nepl-core/tests/resource_ir.rs, nodesrc/test_stdlib_vec_borrowed_observers.js"
---

# ISS-20260523T032117760Z-VEC-QUERY-NEEDS-SCOPED-BORROWED-PRED-C29C915F: Vec query needs scoped borrowed predicate observers for non-Copy payloads

## 概要

Vec has a compiler-owned BorrowRead-backed borrow_at_predicate_or boundary, but count/any/all/find-style query surfaces still require Copy and by-value predicates even when the result is scalar metadata. This forces non-Copy payload observers back to Copy raw access or transform-specific workarounds.

## 対象

- `stdlib/alloc/collections/vec/query/**, stdlib/alloc/collections/vec/access/borrow.nepl, nodesrc/test_stdlib_vec_borrowed_observers.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) の Stage 6 は、collection slot の initialized / moved / borrow / drop state を generic Resource IR proof として扱い、stdlib 関数名 allowlist や個別 module ごとの証明器へ分岐しない方針である。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、non-Copy payload observer を `get<T: Copy>` や raw `MemPtr<T>` ではなく `BorrowRead` backed borrowed observer に載せる方針を定めている。
- 既存の `borrow_at_predicate_or<T>(&Vec<T>, i32, (&T)->bool, bool)->bool` は `BorrowRead` と scoped callback borrow の境界を持つが、`count` / `any` / `all` / `find` はまだ `(.T)->bool` と payload copy-out に寄っていた。
- 実装中に、`Vec<DropPayload>` の `push -> borrowed query -> free` regression が `resource.collection_slot.borrow_read_uninitialized` を出すことを確認した。原因は stdlib API ではなく、`vec_push_storage_checked` / realloc / return summary で collection slot initialized state を caller 側の returned storage projection へ再接続できていなかったことである。

## 問題

Vec has a compiler-owned BorrowRead-backed borrow_at_predicate_or boundary, but count/any/all/find-style query surfaces still require Copy and by-value predicates even when the result is scalar metadata. This forces non-Copy payload observers back to Copy raw access or transform-specific workarounds.

## 影響

Self-host code that stores owning AST/HIR/diagnostic payloads in Vec cannot perform simple predicate scans without either copying payloads or writing bespoke collection logic. That conflicts with the generic Resource IR proof boundary policy.

## 修正方針

Add explicit scoped borrowed predicate query APIs that take (&T)->bool, return scalar or index results, use VecStorageInvariant plus borrow_at_predicate_or, and keep payload copy-out APIs Copy-only.

## 解決内容

- `count_ref<T>`, `find_index_ref<T>`, `any_ref<T>`, `all_ref<T>` を追加した。いずれも `(&T)->bool` predicate を callback scope 内だけで呼び、返すのは count / bool / index だけに限定する。`find<T: Copy>` のように payload を `Option<T>` として返す API は Copy-only のまま残した。
- query entry は `VecStorageInvariant` を先に確認し、invalid metadata では neutral result を返す。slot 読み取りは `borrow_at_predicate_or` 経由に統一し、raw `MemPtr<T>` や `VecDataView<T>` を non-Copy observer の public surface へ出していない。
- Resource IR 側は、return summary と path summary が collection storage relocate / returned storage projection を alias-aware に扱うようにした。これにより `Vec.push` や grow 後に returned `Vec<T>` の storage へ initialized slot state が移る。
- ambiguous scalar alias のために summary offset proof が落ちていた根本原因を修正した。通常の strict API は維持しつつ、summary generation では `i32_scaled_source_candidates` / `i32_type_size_scaled_source_candidates` を列挙し、function parameter 由来へ要約できる候補だけを比較・統合する。
- source policy は、borrowed query API が Copy 制約なしで `borrow_at_predicate_or` を使うこと、既存 copy-out query が Copy-only のまま残ること、collection cleanup contract が non-Copy observer を raw bypass として扱わないことを監視するよう更新した。

## 検証

Add source-policy regressions and focused doctests/Rust resource_ir tests proving Vec<DropPayload> predicate scans borrow slots without moving payload owners.

Focused regression:

- `cargo fmt --check`
- `git diff --check`
- `node nodesrc/issues.js check --dir issues`
- `node --check nodesrc/test_stdlib_vec_borrowed_observers.js`
- `node --check nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node --check nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `cargo test -q -p nepl-core resource::collection_slot_summary_target_tests::summary_target_rewrites_ambiguous_scaled_scalar_alias_to_parameter_projection -- --test-threads=1 --exact --nocapture`
- `cargo test -q -p nepl-core --test resource_ir resource_ir_vec_borrowed_predicate_queries_observe_drop_payload_without_move -- --test-threads=1 --exact --nocapture`
- `cargo test -q -p nepl-core --test resource_ir resource_ir_initialized_check_vec_drop_push_grow_relocates_stdlib_lifecycle -- --test-threads=1 --exact --nocapture`
- `trunk build`
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/query/aggregate.nepl -n 2 --dist web/dist`
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/query/predicate.nepl -n 2 --dist web/dist`
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/query/predicate.nepl -n 4 --dist web/dist`
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/query/predicate.nepl -n 6 --dist web/dist`

補足: `nodesrc/tests.js` の file-level default 60000ms では、新規 query doctest が 60 秒境界に近く、局所実行では `run_doctest.js` に絞って確認した。これは静的検査の正確性とは別の compile-time budget 監視対象であり、timeout 延長だけではなく Resource IR summary / doctest fixture 分割の観点で継続監視する。
