---
id: ISS-20260520T045937560Z-SELF-HOST-DIAGNOSTIC-INFRASTRUCTURE--D61FA83C
title: "self-host diagnostic infrastructure remains a flat implementation file"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/infra/diag.nepl; stdlib/neplg2/core/infra/diag/**; doc/neplg2/self_host_source_tree_layout_review_20260518.md"
---

# ISS-20260520T045937560Z-SELF-HOST-DIAGNOSTIC-INFRASTRUCTURE--D61FA83C: self-host diagnostic infrastructure remains a flat implementation file

## 概要

The self-host diagnostic infrastructure still stores diagnostic code enums, rendering, diagnostic values, collection helpers, and stage smoke tests in one flat file. This keeps a central compiler-facing support module from matching the staged self-host source tree plan.

## 対象

- `stdlib/neplg2/core/infra/diag.nepl; stdlib/neplg2/core/infra/diag/**; doc/neplg2/self_host_source_tree_layout_review_20260518.md`

## 根拠

- `core/infra/diag.nepl` は diagnostic code enum、diagnostic payload、collection owner operation、stage0 smoke を 1 ファイルに保持していた。
- `doc/neplg2/self_host_source_tree_layout_review_20260518.md` は root module を orchestration / public facade に限定し、Rust 実装の flat root file 構造を self-host 側へ移植しない方針を定めている。
- diagnostic code は階層 enum と exhaustive match で管理する必要があるため、code mapping を collection や payload constructor と同じファイルで膨らませ続けると監査境界が曖昧になる。

## 問題

The self-host diagnostic infrastructure still stores diagnostic code enums, rendering, diagnostic values, collection helpers, and stage smoke tests in one flat file. This keeps a central compiler-facing support module from matching the staged self-host source tree plan.

## 影響

Continuing to grow diag.nepl as a flat file makes diagnostics harder to audit for enum exhaustiveness, hides ownership boundaries for diagnostic collections, and would carry the current Rust implementation's flat layout problem into the self-host compiler.

## 修正方針

Split diag.nepl into a facade plus responsibility-specific diagnostic code, value, collection, and stage0 modules. Keep the public API through the facade and add source-policy regressions so the file does not collapse back into a flat implementation.

## 修正内容

- `core/infra/diag.nepl` を doctest と public re-export だけを持つ implementation-free facade にした。
- `core/infra/diag/code.nepl` に severity、階層 diagnostic code enum、stable string mapping を集約した。
- `core/infra/diag/value.nepl` に label / diagnostic payload と constructor / label-note helper を分離した。
- `core/infra/diag/collection.nepl` に `SelfhostDiagnostics` と owning collection operation を分離した。
- `core/infra/diag/stage0.nepl` に smoke API を分離した。
- `nodesrc/selfhost_diag_sources.js` と `nodesrc/test_selfhost_diag_split_contract.js` を追加し、facade への実装再導入と split file の巨大化を source policy で監視する。
- `nodesrc/test_selfhost_diag_code_enum.js` は split 後の diagnostic source 全体を読むように変更し、階層 enum / exhaustive mapping 方針を維持した。

## 検証

Run the diag split contract, diagnostic code enum policy, diagnostic outcome report contract, focused diag doctests, issues check, and diff check.

- `node nodesrc/test_selfhost_diag_split_contract.js`
- `node nodesrc/test_selfhost_diag_code_enum.js`
- `node nodesrc/test_selfhost_cli_reporter_boundary.js`
- `node nodesrc/test_selfhost_diag_outcome_report_contract.js`
- `node nodesrc/tests.js -i stdlib/neplg2/core/infra/diag.nepl --no-tree -o tmp/agent1-diag-split-core.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_diag_outcome.n.md --no-tree -o tmp/agent1-diag-split-outcome.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
