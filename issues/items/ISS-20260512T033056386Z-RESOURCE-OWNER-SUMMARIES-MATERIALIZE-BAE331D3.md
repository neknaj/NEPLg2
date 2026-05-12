---
id: ISS-20260512T033056386Z-RESOURCE-OWNER-SUMMARIES-MATERIALIZE-BAE331D3
title: "Resource owner summaries materialize Result payload owners unconditionally"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: nepl-core/src/resource
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行
---

# ISS-20260512T033056386Z-RESOURCE-OWNER-SUMMARIES-MATERIALIZE-BAE331D3: Resource owner summaries materialize Result payload owners unconditionally

## 概要

Resource IR owner summaries can leave enum payload owner projections in unconditional projection_returns when Result payload owners pass through generic ok/err helpers or match-bound raw i32 fields. This can materialize impossible Ok/Err payload owners together and can also miss raw owner seeds for Result payload fields.

## 対象

- `nepl-core/src/resource`

## 根拠

- `cargo test -p nepl-core --test resource_ir -- --nocapture` で `resource_ir_owner_check_reports_leaked_alloc`、`resource_ir_owner_check_resolves_unwrap_ok_raw_dealloc_consumption`、`resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind`、`resource_ir_owner_summary_consumes_owned_err_payload_from_unreachable_arm`、`resource_ir_owner_check_forwards_nested_byte_builder_result_owner` が失敗した。
- `make_result true` の `Result::Err` payload owner が runtime 到達不能であるにもかかわらず unconditional `projection_returns` として materialize され、caller の temporary `Result::Err.field0` leak になった。
- `unwrap_box` の `Result::Ok` payload field owner は、raw i32 owner seed の lightweight alias walk が `read %r -> tmp` / `match Result::Ok box` で projection を潰していたため、callee summary の parameter source に現れなかった。
- `EndScope` owner auto-drop は `ResourceDropRequirement::StateOnly` の candidate まで owner obligation を消費し、実際には drop code が生成されない raw allocation owner leak を隠し得た。

## 問題

Resource IR owner summaries can leave enum payload owner projections in unconditional projection_returns when Result payload owners pass through generic ok/err helpers or match-bound raw i32 fields. This can materialize impossible Ok/Err payload owners together and can also miss raw owner seeds for Result payload fields.

## 影響

Memory-safety owner checking can either hide raw leaks via StateOnly auto-drop or report false leaks after unwrap-style helpers because variant-specific owner obligations are no longer tied to the resolved Result variant.

## 修正方針

Preserve projection suffixes in raw owner alias summary walks, map local declarations and match payload bindings without losing owner suffixes, normalize only ambiguous multi-variant enum payload projection returns into variant owner returns, and avoid consuming raw/non-owning StateOnly obligations through auto-drop.

## 検証

cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reports_leaked_alloc resource_ir_owner_check_resolves_unwrap_ok_raw_dealloc_consumption resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind resource_ir_owner_summary_consumes_owned_err_payload_from_unreachable_arm resource_ir_owner_check_forwards_nested_byte_builder_result_owner

## 2026-05-12 修正

Resource IR owner summary の根本原因を、callee summary の projection 分類と raw owner seed の projection 喪失に分けて修正した。

- `owner_summary_raw_alias.rs` / `owner_summary_raw_use.rs` は `Read` / `Move` / `DeclareLocal` / `Assign` / aggregate construct / branch / match で alias projection suffix を保持する。`match` payload bind では `Result::Ok.field0` のような owner leaf を bind local の `field0` へ写すため、raw i32 owner seed が enum payload field を見落とさない。
- `owner_summary_raw_use.rs` は `RawMemoryOp::Store` の value 側を raw owner consumption として扱う。raw node field に格納した tail owner が callee summary の consumed parameter source から漏れない。
- `owner_summary_variant_projection.rs` は `Ok` / `Err` のような複数 variant payload owner が同時に `projection_returns` に残った場合だけ、unconditional return ではなく `OwnerVariantProjectionReturn` へ正規化する。単一 variant の helper return や self-update projection return は通常 projection として維持する。
- `owner_drop.rs` / `owner_drop_scope.rs` は `ResourceDropRequirement::StateOnly` のうち `str` などの状態所有 leaf は scope end で落としつつ、`i32` raw address や `MemPtr` のような非所有 pointer を auto-drop しない。実 drop code が走らない raw owner leak を checker 内部で消さない。

検証:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`: passed
- `cargo test -p nepl-core --test effects -- --nocapture`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo fmt --check -p nepl-core`: passed
- `node nodesrc/issues.js check`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reports_leaked_alloc -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_resolves_unwrap_ok_raw_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_summary_consumes_owned_err_payload_from_unreachable_arm -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_forwards_nested_byte_builder_result_owner -- --nocapture`: passed
