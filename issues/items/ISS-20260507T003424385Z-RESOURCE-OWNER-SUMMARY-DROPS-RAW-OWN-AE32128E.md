---
id: ISS-20260507T003424385Z-RESOURCE-OWNER-SUMMARY-DROPS-RAW-OWN-AE32128E
title: "Resource owner summary drops raw owner carried by Result scalar payload"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/owner_summary_leaf.rs, nepl-core/tests/resource_ir.rs, nepl-core/tests/kp.rs"
---

# ISS-20260507T003424385Z-RESOURCE-OWNER-SUMMARY-DROPS-RAW-OWN-AE32128E: Resource owner summary drops raw owner carried by Result scalar payload

## 概要

After scalar owner leaf tightening, bare i32 parameters are no longer seeded as owners. That fixed false positives for ordinary spans, but it also removed owner summary seeds for Result::Ok<i32> payloads, so unwrap_ok alloc leaves the Ok payload owner live and KP prefixsum fails with resource.owner.leak.

## 対象

- `nepl-core/src/resource/owner_summary_leaf.rs, nepl-core/tests/resource_ir.rs, nepl-core/tests/kp.rs`

## 根拠

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_resolves_unwrap_ok_raw_dealloc_consumption -- --nocapture` が、`alloc` の `Result::Ok<i32>` payload に残った `resource.owner.leak` で失敗した。
- `cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 -- --nocapture` も、同じ `unwrap_ok alloc` 経路の owner leak で失敗した。
- 直前の scalar owner leaf 修正で root の bare `i32` を owner seed しない設計にしたこと自体は正しいが、enum payload に格納された raw owner scalar まで summary source から落ちていた。

## 問題

After scalar owner leaf tightening, bare i32 parameters are no longer seeded as owners. That fixed false positives for ordinary spans, but it also removed owner summary seeds for Result::Ok<i32> payloads, so unwrap_ok alloc leaves the Ok payload owner live and KP prefixsum fails with resource.owner.leak.

## 影響

Checked allocation/deallocation through Result helpers fails even when source code uses unwrap_ok dealloc correctly. Restoring broad i32 ownership would reintroduce false owner diagnostics, so the fix must distinguish root scalar values from enum payloads that can carry an actual owner at the caller.

## 修正方針

Keep root plain i32 non-owning, but allow enum payload scalar leaves to participate in owner summary projection returns/consumption. Apply transfer only when the caller has an actual owner state for that payload.

## 検証

Run the unwrap_ok raw dealloc Resource IR regression, scalar false-positive regressions, raw/MemPtr dealloc regressions, and kp prefixsum.

## 2026-05-07 修正

`owner_leaf_places` は root の bare `i32` と struct field の ordinary scalar を owner leaf に戻さない方針を維持した。その上で、enum payload の `i32` だけは owner summary source として扱えるようにした。

これにより `Result::Ok<i32>` の payload に実際の raw owner state がある場合だけ、`unwrap_ok` のような enum payload projection helper が owner を caller output へ移せる。caller に owner state がない通常の `Result<i32, E>` では transfer は発火しないため、span / flag / ordinary scalar helper を owner と誤認する false positive は戻らない。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_resolves_unwrap_ok_raw_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_does_not_treat_plain_i32_identity_as_owner_return -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_summary_does_not_treat -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_result_ok_ -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 -- --nocapture`: passed
- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/issues.js check`: passed
