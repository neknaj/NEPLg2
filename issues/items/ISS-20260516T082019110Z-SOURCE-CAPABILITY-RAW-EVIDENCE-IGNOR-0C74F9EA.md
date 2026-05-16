---
id: ISS-20260516T082019110Z-SOURCE-CAPABILITY-RAW-EVIDENCE-IGNOR-0C74F9EA
title: "source capability raw evidence ignores qualified shadowing"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: nepl-core/src/source_capability/proof.rs
---

# ISS-20260516T082019110Z-SOURCE-CAPABILITY-RAW-EVIDENCE-IGNOR-0C74F9EA: source capability raw evidence ignores qualified shadowing

## 概要

Compiler-owned source can receive raw memory operation or structural capability from a qualified helper-looking symbol whose qualifier is a value binding, not an imported raw helper namespace. This is inconsistent with owner aggregate evidence, which checks split_leading_qualifier.

## 対象

- `nepl-core/src/source_capability/proof.rs`

## 根拠

SourceCapabilityProofCollector::collect_raw_symbol_evidence checks scope.shadows(symbol) directly, while raw_memory_op_from_name and MemoryHelperPrimitive::from_symbol classify by helper_base_name/member_tail. For a call-head like raw::load_i32, a local or parameter named raw is not treated as shadowing the qualified symbol.

## 問題

Compiler-owned source can receive raw memory operation or structural capability from a qualified helper-looking symbol whose qualifier is a value binding, not an imported raw helper namespace. This is inconsistent with owner aggregate evidence, which checks split_leading_qualifier.

## 影響

The source capability proof can over-grant raw memory authority from source syntax that does not prove a raw helper call. That weakens the static-check boundary and can hide source proof mistakes behind qualified helper-looking names.

## 修正方針

Make raw source evidence use the same qualifier-aware shadow rule as owner aggregate evidence before classifying helper_base_name/member_tail. Add loader regression coverage for a shadowed qualified raw operation and static boundary policy coverage.

## 検証

Run the new loader regression before/after the fix, loader raw memory boundary tests, static check boundary policy, cargo fmt/check, issues check, and git diff --check.

## 対応

2026-05-16 Agent 1:

- `SourceCapabilityScope::shadows_symbol_or_qualifier` を追加し、qualified symbol の先頭 qualifier が value binding で shadow されている場合も source evidence から除外する共通 rule にした。
- raw memory evidence collection は `scope.shadows(symbol)` ではなく `scope.shadows_symbol_or_qualifier(symbol)` を使うようにし、`raw::load_i32` の `raw` が引数や local の場合に `load_i32` capability を付与しないようにした。
- owner aggregate evidence 側も同じ共通 API を使うようにして、raw / owner aggregate source proof の qualifier shadowing semantics を揃えた。
- loader regression と static boundary policy を追加し、qualified raw helper-looking symbol の過大 capability 付与を退行検出できるようにした。

この修正は stdlib path や helper 名の allowlist 追加ではなく、source scope と qualified name 構文から「raw helper call として信頼できるか」を証明する gate の修正である。
