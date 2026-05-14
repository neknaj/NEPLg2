---
id: ISS-20260514T211956079Z-OWNER-AGGREGATE-BOUNDARY-TREATS-QUAL-8D858CD3
title: "Owner aggregate boundary treats qualified enum variants as aggregate constructor evidence"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "nepl-core/src/source_capability/owner_aggregate.rs, nepl-core/src/loader.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260514T211956079Z-OWNER-AGGREGATE-BOUNDARY-TREATS-QUAL-8D858CD3: Owner aggregate boundary treats qualified enum variants as aggregate constructor evidence

## 概要

The source-capability evidence walker grants OwnerAggregateBoundary when a configured stdlib source merely mentions a qualified enum variant such as Result::Ok. helper_base_name strips the qualifier, constructor_like_symbol sees Ok as uppercase, and the file receives owner aggregate authority even though no owner-backed aggregate constructor or field access is present.

## 対象

- `nepl-core/src/source_capability/owner_aggregate.rs, nepl-core/src/loader.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6。
- source capability は configured stdlib path だけでなく、source 内にある構造化された証拠に基づいて付与する方針である。
- `Result::Ok` / `Option::Some` のような qualified enum variant は owner-backed aggregate の constructor ではなく、owner token field projection や owner aggregate construction の証拠にはならない。

## 問題

The source-capability evidence walker grants OwnerAggregateBoundary when a configured stdlib source merely mentions a qualified enum variant such as Result::Ok. helper_base_name strips the qualifier, constructor_like_symbol sees Ok as uppercase, and the file receives owner aggregate authority even though no owner-backed aggregate constructor or field access is present.

## 影響

Owner-backed aggregate constructor/projection authority is wider than the source proof. This weakens the intended static-check boundary because ordinary result/option construction can mark a file as privileged.

## 修正方針

Classify constructor evidence only for unqualified constructor-like symbols, keep field accessor helper evidence qualified-aware, and add regression tests so qualified enum variants do not grant OwnerAggregateBoundary while genuine unqualified owner aggregate constructors still do.

## 検証

Run focused loader source capability tests, static-check responsibility policy, issue validation, and diff whitespace checks.

## 解決内容

`OwnerAggregateBoundaryEvidence` の constructor evidence を、unqualified constructor-like symbol だけに限定した。`helper_base_name` は `Result::Ok` の tail を `Ok` として返すため、修正前は qualified enum variant が uppercase constructor と誤分類されていた。修正後は `member_tail(symbol) == symbol` の場合だけ aggregate constructor evidence とし、`field::get` / `field::get_ref` などの field accessor evidence は従来どおり qualified helper として扱う。

`loader` の source capability regression に、configured stdlib source が `Result::Ok` を使うだけでは `OwnerAggregateBoundary` を得ないケースを追加した。既存の unqualified `Vec` constructor evidence と field accessor evidence は維持するため、owner aggregate implementation module の正当な操作は引き続き source evidence によって capability を得る。

## 関連

- Parent: `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`
- Doc: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6
