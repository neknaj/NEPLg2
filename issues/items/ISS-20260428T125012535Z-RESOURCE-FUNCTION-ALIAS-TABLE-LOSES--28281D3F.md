---
id: ISS-20260428T125012535Z-RESOURCE-FUNCTION-ALIAS-TABLE-LOSES--28281D3F
title: "Resource function alias table loses aliases stored in aggregate fields"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs
---

# ISS-20260428T125012535Z-RESOURCE-FUNCTION-ALIAS-TABLE-LOSES--28281D3F: Resource function alias table loses aliases stored in aggregate fields

## 概要

ResourceBorrowCheckEngine and ResourceOwnerCheckEngine track function aliases across locals and branch merges, but ResourceOp::Construct does not move a function value alias from an input into the corresponding aggregate field projection.

## 対象

- `nepl-core/src/resource/check.rs`
- `nepl-core/tests/resource_ir.rs`
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4: resource check への移行

## 根拠

- `FunctionAliasTable` は `DeclareLocal` / `Read` / `Assign` / `FunctionValue` / branch merge で known callee を追跡していた。
- 一方で `ResourceBorrowCheckEngine` と `ResourceOwnerCheckEngine` の `ResourceOp::Construct` は function alias を aggregate field projection へ移していなかった。
- `OwnerState` は aggregate field owner を扱えるため、function value だけが aggregate construction で消える非対称な状態だった。

## 問題

ResourceBorrowCheckEngine and ResourceOwnerCheckEngine track function aliases across locals and branch merges, but ResourceOp::Construct does not move a function value alias from an input into the corresponding aggregate field projection.

## 影響

Indirect calls through struct fields, tuple fields, or enum payloads can lose known callee summaries. For owner checking, a field-stored function that returns a fresh owner can be treated as an unknown callback and the returned free obligation is missed.

## 修正方針

Propagate function aliases during aggregate construction using the same deterministic field projection mapping as owner transfer, so known function values stored in aggregates remain available at projected callee places.

## 修正内容

- `ResourceOp::Construct` で input の known function alias を output の struct / tuple / enum payload field projection へ伝播するようにした。
- borrow checker と owner checker の双方で同じ伝播処理を使い、callback boundary の known callee summary を aggregate field 経由でも維持するようにした。
- helper は owner transfer と同じ `construct_owner_field_place` の projection mapping を使うため、aggregate field の resource state と function alias state が同じ place 表現に揃う。

## 検証

- `resource_ir_owner_check_reports_function_value_stored_in_aggregate_field_alloc_return_leak` を追加した。
- function value を struct field に格納し、その field projection を callee とする `IndirectCall` が fresh owner を返す場合に、caller 側で戻り値 leak が検出されることを確認する。
- 実行済み: `cargo test -p nepl-core --test resource_ir -- --nocapture`
