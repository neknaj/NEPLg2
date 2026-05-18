---
id: ISS-20260518T051123451Z-COLLECTION-CLEANUP-CONTRACT-LACKS-GE-63D5D8EE
title: "collection cleanup contract lacks generic source policy"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-18
updated: 2026-05-18
target: "nodesrc/test_stdlib_collection_cleanup_contract.js, nodesrc/run_source_policy_regressions.js, issues/items/ISS-20260425T000000Z-RV-STDLIB-004-91534828.md"
---

# ISS-20260518T051123451Z-COLLECTION-CLEANUP-CONTRACT-LACKS-GE-63D5D8EE: collection cleanup contract lacks generic source policy

## 概要

RV-STDLIB-004 is currently mitigated by per-collection Copy-only cleanup contracts, but the source-policy coverage is spread across individual collection tests. A newly added generic free/clear/storage cleanup function under stdlib/alloc/collections can reintroduce unconstrained non-Copy payload cleanup without a single cross-collection policy failing.

## 対象

- `nodesrc/test_stdlib_collection_cleanup_contract.js, nodesrc/run_source_policy_regressions.js, issues/items/ISS-20260425T000000Z-RV-STDLIB-004-91534828.md`

## 根拠

- `stdlib/alloc/collections/**` には `Vec` / `Queue` / `Deque` / `HashMap` など複数 family の `free` / `clear` / storage cleanup が分散している。
- 既存の source policy は各 family の代表的な signature を固定していたが、新しい generic cleanup 関数を横断的に検出する検査がなかった。
- `RV-STDLIB-004` の最終修正前は、generic cleanup が non-Copy payload を受け入れないことが memory-safety boundary になる。

## 問題

RV-STDLIB-004 is currently mitigated by per-collection Copy-only cleanup contracts, but the source-policy coverage is spread across individual collection tests. A newly added generic free/clear/storage cleanup function under stdlib/alloc/collections can reintroduce unconstrained non-Copy payload cleanup without a single cross-collection policy failing.

## 影響

Unsupported non-Copy payload collections could regain a storage-only free path that neither drops elements nor rejects the API at typecheck time, hiding the parent memory-safety issue.

## 修正方針

Add a cross-collection source policy that scans stdlib/alloc/collections cleanup/free/clear functions and requires every generic parameter in such cleanup signatures to carry Copy until OwnedBuffer initialized-prefix drop traversal exists. Register the policy in run_source_policy_regressions and record the parent issue progress.

## 検証

Run the new policy, existing collection cleanup contract doctest, stdlib Vec policy, source policy regression subset, and issues index/check.

## 解決内容

`nodesrc/test_stdlib_collection_cleanup_contract.js` を追加し、`stdlib/alloc/collections/**/*.nepl` の generic `free` / `clear` / cleanup signature を横断的に走査する source policy にした。現行の collection は non-Copy payload drop traversal が完成していないため、cleanup API の全 generic parameter が `Copy` bound を持つことを要求する。

この policy は個別 collection 名の allowlist ではなく、source 上の cleanup API surface を構文的に検出する。`Vec.free` / `Vec.clear` / `vec_free_storage` / `HashMap.free` / `HashSet.free` / `Queue` / `Deque` など既存の主要 cleanup signature を実際に検査していることも確認し、検査の空振りを防いだ。

`nodesrc/run_source_policy_regressions.js` に登録したため、今後 `stdlib/alloc/collections` に generic cleanup API を追加するとき、`OwnedBuffer<T>` / initialized prefix / drop traversal が完成するまでは Copy-only contract を外せない。
