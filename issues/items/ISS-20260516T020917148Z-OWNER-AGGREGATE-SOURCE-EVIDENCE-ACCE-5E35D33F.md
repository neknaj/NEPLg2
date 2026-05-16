---
id: ISS-20260516T020917148Z-OWNER-AGGREGATE-SOURCE-EVIDENCE-ACCE-5E35D33F
title: "owner aggregate source evidence accepts non-call uppercase symbols"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: nepl-core/src/source_capability/owner_aggregate.rs
---

# ISS-20260516T020917148Z-OWNER-AGGREGATE-SOURCE-EVIDENCE-ACCE-5E35D33F: owner aggregate source evidence accepts non-call uppercase symbols

## 概要

Owner aggregate source capability currently treats every unqualified uppercase symbol in an expression as constructor boundary evidence. That is weaker than the intended source proof: an uppercase value used as an argument or a same-module enum variant can grant constructor authority even though no owner-backed aggregate constructor call was observed.

## 対象

- `nepl-core/src/source_capability/owner_aggregate.rs`

## 根拠

- `nepl-core/src/source_capability/owner_aggregate.rs` は修正前、prefix expression 内の全 `PrefixItem::Symbol` を走査し、未修飾かつ先頭が大文字の symbol を constructor evidence としていた。
- そのため `consume Diag` のように大文字値が引数位置にあるだけでも `OwnerAggregateConstructorBoundary("Diag")` が付与され得た。
- `Result::Ok` のような qualified enum variant は除外していたが、同一 module の unqualified enum variant `Ok 1` は constructor evidence と区別できていなかった。
- これは stdlib module allowlist ではないものの、source proof としては「構築子呼び出しを観測した」ことを証明しておらず、静的検査自身の誤りを policy / enum / match で発見しにくい形だった。

## 問題

Owner aggregate source capability currently treats every unqualified uppercase symbol in an expression as constructor boundary evidence. That is weaker than the intended source proof: an uppercase value used as an argument or a same-module enum variant can grant constructor authority even though no owner-backed aggregate constructor call was observed.

## 影響

The static-check boundary remains name-driven in a place where it should be syntax/proof-driven. This makes false evidence harder to audit and conflicts with the generic proof direction for ResourceIR and static checks.

## 修正方針

Restrict constructor evidence to call-head syntax, keep field-access evidence explicit, exclude same-module enum variants, and add focused regressions for non-call uppercase symbols and unqualified enum variants.

## 解決

2026-05-16 に修正した。

- owner aggregate evidence 判定を `source_capability/owner_aggregate/evidence.rs` に分離し、AST traversal と evidence classification の責務を分けた。
- constructor evidence は prefix expression の call-head に現れた symbol からだけ導出するようにした。大文字 symbol が引数や後続 item として現れるだけでは constructor boundary authority を得られない。
- 同一 module で定義された enum variant 名を `OwnerAggregateEvidenceContext` に集め、unqualified enum variant call を owner-backed aggregate constructor evidence から除外した。
- field projection evidence は `field::get` 系 symbol と `get_field` / `get_field_ref` intrinsic の explicit evidence として維持した。
- loader regression を追加し、call-head 以外の大文字値、same-module enum variant、qualified enum variant、intrinsic field evidence、constructor-name isolation を固定した。
- `nodesrc/test_static_check_boundary_responsibility.js` に evidence module、call-head evidence、enum variant exclusion、loader regression の監視を追加した。

この修正は `Vec` や特定 stdlib module を登録して許可する設計ではない。source capability は「compiler-owned source に privilege が必要な構文証拠があるか」を見る authority gate に留め、owner-backed aggregate かどうかの semantic proof は typecheck の構造的 owner token 判定と Resource IR 側へ残す。

## 検証

cargo test -p nepl-core owner_aggregate_boundary -- --nocapture; node nodesrc/test_static_check_boundary_responsibility.js; node nodesrc/issues.js check --dir issues

実施済み:

- `cargo test -p nepl-core owner_aggregate_boundary -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
