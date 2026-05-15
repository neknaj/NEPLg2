---
id: ISS-20260515T130627053Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-3CDEFF1A
title: "Stdlib documentation contract declaration doctest baseline regressed to 1038"
area: stdlib
status: fixed
resolved: true
priority: P1
type: doc
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/**/*.nepl, nodesrc/test_stdlib_documentation_contract.js"
---

# ISS-20260515T130627053Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-3CDEFF1A: Stdlib documentation contract declaration doctest baseline regressed to 1038

## 概要

node nodesrc/test_stdlib_documentation_contract.js now reports declaration doctest gaps increased to 1038 while the frozen baseline is 1032. This reopens the documentation contract signal after previous fixes brought the count back to 1032.

## 対象

- `stdlib/**/*.nepl, nodesrc/test_stdlib_documentation_contract.js`

## 根拠

- `node nodesrc/test_stdlib_documentation_contract.js` が `declarationNoDoctest: 1038` を報告し、固定 baseline `1032` を 6 件超過していた。
- `node nodesrc/run_source_policy_regressions.js --warn-only` でも同じ documentation contract warning が残り、Resource checker responsibility policy が clean になった後の唯一の source-policy warning になっていた。

## 問題

node nodesrc/test_stdlib_documentation_contract.js now reports declaration doctest gaps increased to 1038 while the frozen baseline is 1032. This reopens the documentation contract signal after previous fixes brought the count back to 1032.

## 影響

Source policy warn-only hides an executable documentation coverage regression. New stdlib APIs may lack typical-use doctests despite the project rule that docs and doctests are part of the API contract.

## 修正方針

Audit the six-gap regression, add meaningful declaration doctests for the changed public APIs instead of raising the baseline, and keep the policy baseline at or below 1032.

## 対応

- `stdlib/alloc/collections/deque/index.nepl` に `deque_normalize_capacity` / `deque_prev_index` / `deque_tail_index` / `deque_back_index` の canonical stdout report doctest を追加した。
- `stdlib/alloc/collections/hashmap/probe.nepl` に `hashmap_next_slot` の wrap-around を含む canonical stdout report doctest を追加した。
- `stdlib/alloc/collections/hashset/probe.nepl` に `hashset_next_slot` の wrap-around を含む canonical stdout report doctest を追加した。
- baseline は緩和せず、公開 helper の典型例と境界例を `TestReport` stdout に固定した。

## 検証

Run node nodesrc/test_stdlib_documentation_contract.js, focused doctests for updated files, source policy warn-only, issues check, and diff whitespace check.

- `node nodesrc/test_stdlib_documentation_contract.js`: pass, `declarationNoDoctest: 1032`
- `node nodesrc/tests.js -i stdlib/alloc/collections/deque/index.nepl -i stdlib/alloc/collections/hashmap/probe.nepl -i stdlib/alloc/collections/hashset/probe.nepl --no-tree -o tmp/agent1-doc-contract-index-probe.json -j 1 --dist web/dist --assert-io`: 6 passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: source-policy warning なし
