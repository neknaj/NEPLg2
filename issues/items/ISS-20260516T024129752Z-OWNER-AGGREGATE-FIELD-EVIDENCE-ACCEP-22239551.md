---
id: ISS-20260516T024129752Z-OWNER-AGGREGATE-FIELD-EVIDENCE-ACCEP-22239551
title: "owner aggregate field evidence accepts unrelated get names"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: nepl-core/src/source_capability/owner_aggregate/evidence.rs
---

# ISS-20260516T024129752Z-OWNER-AGGREGATE-FIELD-EVIDENCE-ACCEP-22239551: owner aggregate field evidence accepts unrelated get names

## 概要

Owner aggregate source evidence treats any unshadowed call-head named get, get_ref, or put as field accessor evidence. A compiler-owned stdlib file that imports or defines an unrelated collection/query get can acquire OwnerAggregateFieldBoundary without using core field access or field intrinsics.

## 対象

- `nepl-core/src/source_capability/owner_aggregate/evidence.rs`

## 根拠

- `nepl-core/src/source_capability/owner_aggregate/evidence.rs` は修正前、call-head symbol の `helper_base_name` が `get` / `get_ref` / `put` / `get_field` / `get_field_ref` なら field accessor evidence として扱っていた。
- この判定は `core/field` 由来かどうかを見ず、`alloc/collections/vec/query/get` のような通常 helper の open import でも owner aggregate field boundary を得られる構造だった。
- `field::get` / `f::get` / selective import / open import は source に現れる import provenance で区別できるため、単なる名前 allowlist ではなく import clause の AST を証拠として使うべきだった。

## 問題

Owner aggregate source evidence treats any unshadowed call-head named get, get_ref, or put as field accessor evidence. A compiler-owned stdlib file that imports or defines an unrelated collection/query get can acquire OwnerAggregateFieldBoundary without using core field access or field intrinsics.

## 影響

Stage 6 owner aggregate field capability can be over-granted from ordinary helper names. This weakens the source proof gate for owner-token field projection and conflicts with the requirement that static-check authority be derived from source properties rather than broad name allowlists.

## 修正方針

Make owner aggregate field evidence import-aware: accept core/field imported aliases, open/merge/selective imports, and explicit get_field/get_field_ref intrinsics, but reject unrelated get/get_ref/put names. Keep constructor evidence and enum-variant filtering unchanged.

## 解決内容

- `owner_aggregate/context.rs` を追加し、same-module enum variant と `core/field` import provenance を `OwnerAggregateEvidenceContext` に集約した。
- `owner_aggregate/field_imports.rs` を追加し、`ImportClause` を網羅 `match` で処理して、`core/field` の default alias / explicit alias / open import / merge import / selective import からだけ field accessor import proof を作るようにした。
- `owner_aggregate/evidence.rs` は call-head symbol を受け取り、scope shadowing、import provenance、constructor evidence の順で分類するだけに縮小した。
- loader regression に、`core/field` open import と alias import の positive case、および `alloc/collections/vec/query/get` 由来の unrelated `get` negative case を追加した。
- 調査中、Windows で既存 stdlib root と仮想 stdlib child path の canonical prefix が不一致になり capability が落ちる別問題を発見し、`ISS-20260516T025931471Z-WINDOWS-STDLIB-PATH-CANONICALIZATION-5C6E2D4E` として分離した。

## 検証

Add loader regressions for unrelated get rejection and core/field open/alias imports; run cargo test -p nepl-core owner_aggregate_boundary -- --nocapture and source policy checks.

- `cargo test -p nepl-core owner_aggregate_boundary -- --nocapture`
- `cargo test -p nepl-core owner_aggregate_boundaries -- --nocapture`
- `cargo test -p nepl-core raw_memory_boundary -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `cargo fmt -p nepl-core -- --check`
- `git diff --check`
- `trunk build`
