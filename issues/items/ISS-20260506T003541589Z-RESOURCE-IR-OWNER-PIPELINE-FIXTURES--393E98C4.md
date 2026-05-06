---
id: ISS-20260506T003541589Z-RESOURCE-IR-OWNER-PIPELINE-FIXTURES--393E98C4
title: "Resource IR owner pipeline fixtures call unsafe memory from pure main"
area: core
status: open
resolved: false
priority: P1
type: test
created: 2026-05-06
updated: 2026-05-06
target: nepl-core/tests/resource_ir.rs
---

# ISS-20260506T003541589Z-RESOURCE-IR-OWNER-PIPELINE-FIXTURES--393E98C4: Resource IR owner pipeline fixtures call unsafe memory from pure main

## 概要

Three Resource IR owner integration tests still compile owner-acceptance sources whose main function is pure while the fixture calls store/load/fill raw memory helpers. After the Resource IR unsafe-memory effect gate became authoritative, those fixtures fail with effect.pure.calls_impure before reaching the owner assertion they were meant to cover.

## 対象

- `nepl-core/tests/resource_ir.rs`

## 根拠

- `cargo test -p nepl-core --test resource_ir -- --nocapture` で次の 3 件が `effect.pure.calls_impure` により失敗した。
  - `resource_ir_owner_check_applies_result_ok_raw_dealloc_consumption`
  - `resource_ir_owner_check_accepts_borrowed_region_ptr_at_then_region_dealloc`
  - `resource_ir_owner_check_accepts_borrowed_region_ptr_retag_then_region_dealloc`
- いずれも owner checker の妥当性を見る test だが、source fixture の `fn main <()->i32> ():` が pure のまま `store_i32` / `load_i32` / `fill_i32` を呼ぶ。
- Stage 5 で unsafe memory effect gate が hard error 化された現在、この fixture は owner check に到達する前に effect gate で落ちる。

## 問題

Three Resource IR owner integration tests still compile owner-acceptance sources whose main function is pure while the fixture calls store/load/fill raw memory helpers. After the Resource IR unsafe-memory effect gate became authoritative, those fixtures fail with effect.pure.calls_impure before reaching the owner assertion they were meant to cover.

## 影響

The full Resource IR test suite reports failures unrelated to owner logic, making it harder to distinguish real owner regressions from stale effect annotations. The fixtures also document an invalid post-Stage-5 static-check pattern.

## 修正方針

Update the affected owner pipeline fixtures to use the current effect model, either by marking the test function as impure when raw memory effects are intentional or by moving raw memory calls behind a compiler-owned boundary fixture if the test is meant to exercise internal stdlib behavior.

## 検証

Run the three affected Resource IR owner tests and then cargo test -p nepl-core --test resource_ir.
