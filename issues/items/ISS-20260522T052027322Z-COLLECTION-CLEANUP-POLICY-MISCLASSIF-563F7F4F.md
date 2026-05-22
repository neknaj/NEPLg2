---
id: ISS-20260522T052027322Z-COLLECTION-CLEANUP-POLICY-MISCLASSIF-563F7F4F
title: "Collection cleanup policy misclassifies private lifecycle proof helpers"
area: tooling
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-22
updated: 2026-05-22
target: "nodesrc/test_stdlib_collection_cleanup_contract.js, stdlib/alloc/collections/vec/mutation/*.nepl"
---

# ISS-20260522T052027322Z-COLLECTION-CLEANUP-POLICY-MISCLASSIF-563F7F4F: Collection cleanup policy misclassifies private lifecycle proof helpers

## 概要

The collection cleanup contract scans every generic owner surface and treats private same-file lifecycle proof helpers such as vec_push_slot_store_initialized as public owner-producing/updating APIs that must remain Copy-only. This blocks the intended Resource IR marker authority design even though the helper is non-public and pairs raw store with collection_slot_initialize_empty in the implementation boundary.

## 対象

- `nodesrc/test_stdlib_collection_cleanup_contract.js, stdlib/alloc/collections/vec/mutation/*.nepl`

## 根拠

- 親 issue: [Non-Copy collection payload support needs compiler-issued owner and drop traversal](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md)
- 関連 doc: [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-resource-ir-と-stdlib-境界の完成)
- 関連方針: collection lifecycle marker authority は public wrapper や stdlib allowlist ではなく、source-derived raw operation と同一 implementation boundary に閉じる。

## 問題

The collection cleanup contract scans every generic owner surface and treats private same-file lifecycle proof helpers such as vec_push_slot_store_initialized as public owner-producing/updating APIs that must remain Copy-only. This blocks the intended Resource IR marker authority design even though the helper is non-public and pairs raw store with collection_slot_initialize_empty in the implementation boundary.

## 影響

Static-check source policy can force incorrect Copy bounds or discourage private compiler-owned proof helpers, pushing the implementation toward public wrappers or allowlists. That weakens the planned generic Resource IR lifecycle proof model for non-Copy collection payloads.

## 修正方針

Classify private lifecycle proof helpers structurally: only non-pub functions whose own body pairs collection_slot lifecycle markers with the corresponding raw operation may bypass the public owner surface Copy-only rule. Keep public APIs and ordinary owner surfaces under the existing Copy requirement.

## 検証

Run node --check nodesrc/test_stdlib_collection_cleanup_contract.js, node nodesrc/test_stdlib_collection_cleanup_contract.js, node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, node nodesrc/issues.js check --dir issues, and git diff --check.

## 解決内容

2026-05-22 に Agent 1 が修正した。

- `nodesrc/test_stdlib_collection_cleanup_contract.js` の関数 signature parser が `pub` の有無を保持するようにした。
- owner-producing / owner-updating generic collection surface の Copy-only 検査で、non-`pub` helper だけを private lifecycle proof helper 候補として分離した。
- private lifecycle proof helper は関数名 allowlist ではなく、その関数 body が `collection_slot_*` marker と対応する raw operation を同時に持つ場合だけ分類する。例えば `collection_slot_initialize_empty` は `store<T>`、`collection_slot_drop_traversal` は `while` + `load<T>` + `Drop::drop`、`collection_slot_storage_dealloc` は raw/typed dealloc evidence を要求する。
- public API や通常の owner surface は従来どおり Copy-only 検査を受ける。private helper を public 化した場合はこの分類に入らない。

## 回帰テスト

- `node --check nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
