---
id: ISS-20260517T123624102Z-SOURCECAPABILITIES-INSERT-USE-SITE-L-7F2B23E5
title: "SourceCapabilities insert_use_site lacks a production caller policy"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/source_map.rs, nepl-core/src/source_capability/proof_builder.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T123624102Z-SOURCECAPABILITIES-INSERT-USE-SITE-L-7F2B23E5: SourceCapabilities insert_use_site lacks a production caller policy

## 概要

SourceCapabilities::insert_use_site is pub(crate) because source_map and source_capability are sibling modules. The current source policy checks the typed proof fact path but does not explicitly reject future production code that mutates SourceCapabilities directly outside proof_builder.

## 対象

- `nepl-core/src/source_map.rs, nepl-core/src/source_capability/proof_builder.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- 現在の source capability proof は `SourceCapabilityProofEvent` / `SourceCapabilityProofFact` / `SourceCapabilityProof::insert_fact` へ集約済みである。
- ただし `SourceCapabilities::insert_use_site` は `source_map.rs` と `source_capability/proof_builder.rs` が sibling module であるため `pub(crate)` になっており、Rust visibility だけでは production caller を `proof_builder` に限定できない。
- 既存の `nodesrc/test_static_check_boundary_responsibility.js` は typed proof fact path の存在を検査していたが、将来の production code が `.insert_use_site(...)` を直接呼ぶ退行までは列挙していなかった。

## 問題

SourceCapabilities::insert_use_site is pub(crate) because source_map and source_capability are sibling modules. The current source policy checks the typed proof fact path but does not explicitly reject future production code that mutates SourceCapabilities directly outside proof_builder.

## 影響

A later static-check change could bypass SourceCapabilityProofEvent/SourceCapabilityProofFact and insert exact use-site authority by hand. That would weaken the generic proof pipeline and make checker wiring errors harder to catch statically.

## 修正方針

Add a source policy that allows direct insert_use_site calls only in SourceCapabilities internals, SourceCapabilityProofBuilder, and cfg(test) test helpers. Production source capability construction must keep routing through typed proof facts.

## 検証

Run node nodesrc/test_static_check_boundary_responsibility.js and focused issue consistency checks.

## 関連計画

- [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 対応内容

- `nodesrc/test_static_check_boundary_responsibility.js` に direct `insert_use_site` caller policy を追加した。
- production code では `nepl-core/src/source_capability/proof_builder.rs` だけが `.insert_use_site(...)` を呼べるようにし、`#[cfg(test)] mod tests` 内の test helper は許可した。
- `SourceCapabilityProofBuilder` が `self.capabilities.insert_use_site(use_site);` を持つことも明示的に確認し、typed proof fact pipeline の唯一の production bridge として固定した。
- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 6 関連 issue にこの policy issue を追記した。

## 検証結果

- `node nodesrc/test_static_check_boundary_responsibility.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
