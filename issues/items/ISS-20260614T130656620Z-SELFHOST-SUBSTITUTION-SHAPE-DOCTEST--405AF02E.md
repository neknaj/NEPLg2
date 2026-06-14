---
id: ISS-20260614T130656620Z-SELFHOST-SUBSTITUTION-SHAPE-DOCTEST--405AF02E
title: "Selfhost substitution shape doctest compile time exceeds default timeout"
area: selfhost
status: open
resolved: false
priority: P1
type: performance
created: 2026-06-14
updated: 2026-06-14
target: "stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_substitution_shape.nepl; stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_instantiation.nepl; nepl-core Resource static check; nodesrc/run_doctest.js"
---

# ISS-20260614T130656620Z-SELFHOST-SUBSTITUTION-SHAPE-DOCTEST--405AF02E: Selfhost substitution shape doctest compile time exceeds default timeout

## 概要

The selfhost generic substitution shape and generic instantiation doctests pass semantically but need an extended timeout. The current measurement is dominated by resource_static_initialized_moves and resource_static_check, so the default doctest budget can hide whether the semantic boundary is correct.

## 対象

- `stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_substitution_shape.nepl; stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_instantiation.nepl; nepl-core Resource static check; nodesrc/run_doctest.js`

## 根拠

- 2026-06-14 の `work/selfhost-substitution-shape-evidence-connector` checkpoint で、`node nodesrc/test_selfhost_memo_trait_public_impl_generic_substitution_shape_contract.js` は pass した。
- 同 checkpoint で、`node nodesrc/test_selfhost_memo_trait_public_impl_generic_instantiation_contract.js` は pass した。
- 同 checkpoint で、`NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/run_doctest.js -i stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_substitution_shape.nepl -n 1` は pass したが、`compile_ms` は約 `189.6s` だった。
- stage timing では `resource_static_initialized_moves` が約 `184.0s`、`resource_static_check` が約 `188.0s` であり、type error や parser error ではなく Resource checker の initialized-state 探索が支配的である。
- 同 checkpoint の follow-up で、`node nodesrc/run_doctest.js -i stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_instantiation.nepl -n 1` は pass したが、`compile_ms` は約 `171.4s` だった。
- instantiation 側の stage timing でも `resource_static_initialized_moves` が約 `160.9s`、`resource_static_check` が約 `167.7s` であり、substitution traversal evidence を保持する selfhost fixture が同じ Resource initialized-state 探索問題を踏んでいる。
- terminal step endpoint verification、substitution shape hash recheck、schema mismatch check、error code uniqueness contract を入れた後の focused test でも、`NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/tests.js -i stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_substitution_shape.nepl --no-tree -j 1 --dist web/dist -o tmp/selfhost-substitution-shape-long.json` は pass した。ただし `compile_ms` は約 `318.8s` だった。
- 同 follow-up の instantiation focused test でも、`NEPL_TEST_CASE_TIMEOUT_MS=600000 node nodesrc/tests.js -i stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_instantiation.nepl --no-tree -j 1 --dist web/dist -o tmp/selfhost-instantiation-long.json` は pass した。ただし `compile_ms` は約 `174.4s` だった。
- `nodesrc/tests.js` の latest JSON は stage breakdown を含まないため、細分化された支配要因としては上記の `run_doctest` stage timing を根拠に残す。latest focused test は、受理境界を固めても compile-time 問題が semantic failure ではなく Resource checker 探索問題として残ることを確認する補助証拠である。
- この doctest は `SelfhostTypeSubstitutionStepTable` の owner を stage0 fixture 内で作り、target type と trait application の両方について substitution evidence と step table hash を再検査する。semantic contract としては必要な境界を通しているが、単一 doctest に owner-bearing setup と producer contract の両方が集中している。

## 問題

The selfhost generic substitution shape and instantiation doctests pass semantically but need an extended timeout. The current measurement is dominated by resource_static_initialized_moves and resource_static_check, so the default doctest budget can hide whether the semantic boundary is correct.

## 影響

CI and local focused checks may report timeout even when the substitution evidence boundary is sound. This also makes future selfhost stages harder to review because semantic regressions and Resource checker exploration cost are mixed together.

## 修正方針

Split the stage0 fixture and Resource checker cost at the root: keep the production producer contract checked, move heavyweight owner-bearing setup out of the single doctest where possible, profile initialized-moves for this fixture, and reduce repeated exploration without weakening typed evidence, owner, borrow, drop, or fail-closed proof boundaries.

## 検証

Run the substitution shape contract test, the instantiation contract test, the focused substitution shape and instantiation doctests with normal and extended timeout measurements, issues check, and diff whitespace check.
