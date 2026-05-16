---
id: ISS-20260516T061152439Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-B1897B0F
title: "Stdlib documentation contract declaration doctest baseline regressed to 1039 on main"
area: stdlib
status: fixed
resolved: true
priority: P1
type: doc
created: 2026-05-16
updated: 2026-05-16
target: "stdlib/alloc/collections/**/*.nepl, stdlib/core/mem/types.nepl"
---

# ISS-20260516T061152439Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-B1897B0F: Stdlib documentation contract declaration doctest baseline regressed to 1039 on main

## 概要

origin/main currently reports declarationNoDoctest=1039 while nodesrc/test_stdlib_documentation_contract.js freezes the baseline at 1032. This is not caused by the BTreeMap report fixture branch; current, HEAD, and origin/main have the same count. The new gaps are concentrated in owner-bearing collection/memory APIs such as BTreeMapInsertError, BTreeSetInsertError, StackPop accessors, VecPartition helpers, VecStorage/OwnedBuffer, VecPop accessors, and region_token_raw_ref.

## 対象

- `stdlib/alloc/collections/**/*.nepl, stdlib/core/mem/types.nepl`

## 根拠

- `node nodesrc/test_stdlib_documentation_contract.js` が main baseline `declarationNoDoctest <= 1032` を強制しているが、origin/main 由来の状態では `declarationNoDoctest=1039` となり source policy が失敗していた。
- gap は owner を含む API に集中していたため、baseline を上げるのではなく、owner を直接 field projection しない public accessor / safe constructor 経由の利用例を追加する必要があった。
- direct constructor が禁止される owner aggregate については、その制約自体を executable documentation として `compile_fail` doctest で固定する必要があった。

## 問題

origin/main currently reports declarationNoDoctest=1039 while nodesrc/test_stdlib_documentation_contract.js freezes the baseline at 1032. This is not caused by the BTreeMap report fixture branch; current, HEAD, and origin/main have the same count. The new gaps are concentrated in owner-bearing collection/memory APIs such as BTreeMapInsertError, BTreeSetInsertError, StackPop accessors, VecPartition helpers, VecStorage/OwnedBuffer, VecPop accessors, and region_token_raw_ref.

## 影響

The global source-policy runner fails before unrelated static-check work can be reported cleanly, and owner-bearing stdlib APIs lack executable documentation examples even though docs/doctests are part of the API contract.

## 修正方針

Add meaningful n.md-style declaration doctests for the listed public APIs instead of raising the baseline. Keep the policy baseline at 1032 or lower. Runtime examples must validate the observed value before returning success, and `compile_fail` examples must pin the expected diagnostic code where the API contract is a static rejection.

## 対応

- `BTreeMapInsertError` / `BTreeSetInsertError` / `OwnedBuffer` には、owner aggregate direct constructor が `type.owner_aggregate.constructor_restricted` で拒否されることを示す `compile_fail` doctest を追加した。
- `StackPop` / `VecPop` / `VecPartition` / `VecStorage` / `region_token_raw_ref` とそれらの public accessor には、safe API で owner を生成し、accessor で観測し、最後に owner を明示的に解放する実行 doctest を追加した。
- `VecPartition` helper は direct constructor ではなく `partition` から値を得る形に揃え、stdlib の owner aggregate field を caller 側で直接 projection しない設計を doctest でも固定した。
- documentation contract count は `declarationNoDoctest=1022` まで改善し、baseline `1032` 以下へ戻した。

## 検証

- `node nodesrc/test_stdlib_documentation_contract.js`: passed (`declarationNoDoctest=1022`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/btreemap/types.nepl -i stdlib/alloc/collections/btreeset/types.nepl -i stdlib/alloc/collections/stack/types.nepl -i stdlib/alloc/collections/stack/api.nepl -i stdlib/alloc/collections/vec/types.nepl -i stdlib/alloc/collections/vec/transform/filter.nepl -i stdlib/core/mem/types.nepl --no-tree -o tmp/agent1-stdlib-doc-contract.json -j 1 --dist web/dist --assert-io`: total=21, passed=21
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
