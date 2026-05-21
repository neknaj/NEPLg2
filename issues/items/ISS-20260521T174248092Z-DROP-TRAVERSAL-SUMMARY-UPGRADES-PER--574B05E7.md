---
id: ISS-20260521T174248092Z-DROP-TRAVERSAL-SUMMARY-UPGRADES-PER--574B05E7
title: "Drop traversal summary upgrades per-slot range witness into forall coverage"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/collection_slot_summary_build_drop_traversal.rs
---

# ISS-20260521T174248092Z-DROP-TRAVERSAL-SUMMARY-UPGRADES-PER--574B05E7: Drop traversal summary upgrades per-slot range witness into forall coverage

## 概要

Collection slot summary build treats any symbolic slot that has an inside-initialized-count proof as CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange. A guarded branch or one loop iteration can prove only that a particular symbolic slot is inside the range, not that every initialized slot in caller storage was traversed and dropped. Caller summary replay can then drop all caller initialized slots from a callee-local per-slot witness.

## 対象

- `nepl-core/src/resource/collection_slot_summary_build_drop_traversal.rs`

## 根拠

- `collect_summary_drop_traversal_op` は、`slot_requires_range_proof` が必要な symbolic / scaled-symbolic slot を 1 つでも含むと `ForallInitializedRange` summary を生成していた。
- `slot_requires_range_proof` と `collection_slot_offset_is_inside_initialized_count` は「その slot offset が `initialized_count` 内にある」ことだけを証明する。これは loop / traversal が `0..initialized_count` の全 slot を訪問したことの証明ではない。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、collection slot lifecycle / drop traversal を stdlib 個別許可ではなく generic typed proof boundary に載せる方針を定めている。

## 問題

Collection slot summary build treats any symbolic slot that has an inside-initialized-count proof as CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange. A guarded branch or one loop iteration can prove only that a particular symbolic slot is inside the range, not that every initialized slot in caller storage was traversed and dropped. Caller summary replay can then drop all caller initialized slots from a callee-local per-slot witness.

## 影響

This is a Resource IR soundness bug for non-Copy collection cleanup. It can hide live owner payloads during storage dealloc and violates the policy that static checks must derive generic proof from source semantics rather than trusting a broad marker or module-specific allowance.

## 修正方針

Separate per-slot range certification from full-range traversal certification. Do not emit forall drop traversal summary unless a typed full initialized-range coverage certificate exists; finite slots may still be replayed as CertifiedSlots. Add regression coverage that branch-only symbolic proof does not build a forall summary.

## 修正内容

- `collect_summary_drop_traversal_op` は、range proof が必要な symbolic slot を見つけた場合に summary を生成しないようにした。
- finite slot list だけは `CertifiedSlots` として従来どおり summary replay できる。
- 未使用の `ForallInitializedRange` replay variant と専用 test/module は削除した。full initialized range coverage は、別途 typed traversal coverage certificate が導入された場合に改めて追加する。
- 回帰として、branch-local な symbolic slot proof が `DropTraversal` summary に昇格しないことを追加した。

## 検証

- `cargo test -p nepl-core --lib collection_slot_summary_build_ops -- --test-threads=1`: pass
- dormant replay variant and its dedicated test module were deleted instead of leaving dead code
- `cargo check -p nepl-core`: pass
- `cargo fmt --check -p nepl-core`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass（CRLF warning のみ）
