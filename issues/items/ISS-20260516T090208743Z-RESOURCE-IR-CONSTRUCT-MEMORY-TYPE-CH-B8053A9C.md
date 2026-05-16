---
id: ISS-20260516T090208743Z-RESOURCE-IR-CONSTRUCT-MEMORY-TYPE-CH-B8053A9C
title: "Resource IR construct memory type checks still use constructor names"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/resource/lower_raw_address_return.rs, nepl-core/src/resource/owner_flow.rs, nodesrc/test_resource_checker_responsibility.js, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260516T090208743Z-RESOURCE-IR-CONSTRUCT-MEMORY-TYPE-CH-B8053A9C: Resource IR construct memory type checks still use constructor names

## 概要

After compiler memory type identity was moved into TypeCtx, Resource IR still has construct-time checks that call compiler_memory_type_from_constructor_name on AggregateKind or HirExpr constructor names. These paths can drift from the proven TypeCtx identity model and reintroduce name-based static-check behavior.

## 対象

- `nepl-core/src/resource/lower_raw_address_return.rs, nepl-core/src/resource/owner_flow.rs, nodesrc/test_resource_checker_responsibility.js, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `nepl-core/src/resource/lower_raw_address_return.rs` の transparent return projection が `HirExprKind::StructConstruct { name, .. }` の constructor 名から `MemPtr` を判定していた。
- `nepl-core/src/resource/owner_flow.rs` の `region_token_construct_kind` が `AggregateKind::Struct { name, .. }` の constructor 名から `RegionToken` を判定していた。
- `ISS-20260516T083402106Z-COMPILER-MEMORY-TYPE-IDENTITY-IS-INF-3F3AF6C8` で `TypeCtx` に証明済み compiler memory type identity を入れた後も、Resource IR construct path だけが名前文字列判定へ戻れる状態だった。

## 問題

After compiler memory type identity was moved into TypeCtx, Resource IR still has construct-time checks that call compiler_memory_type_from_constructor_name on AggregateKind or HirExpr constructor names. These paths can drift from the proven TypeCtx identity model and reintroduce name-based static-check behavior.

## 影響

Static-check correctness depends on residual string-name checks instead of source/type proof. Future OwnedBuffer or owner-token work could add more ad hoc construct checks, and same-name user structs may receive raw-address or owner-token treatment if surrounding guards change.

## 修正方針

Make construct-sensitive Resource IR code query the proven TypeCtx compiler memory type identity through type_is_raw_pointer/type_is_owner_token, and update source policies so Resource IR checkers cannot call compiler_memory_type_from_constructor_name for semantic classification.

## 対応

- `lower_raw_address_return.rs` の raw pointer construct return 判定を `type_is_raw_pointer(env.types, expr.ty)` に変更し、constructor 名を Resource IR lowering の semantic proof に使わないようにした。
- `owner_flow.rs` の owner-token construct extent 判定を `type_is_owner_token(self.types, output.ty)` に変更し、`AggregateKind` の struct 名ではなく construct output の証明済み type identity を見るようにした。
- `resource_primitives.rs` の unit test を `resource_primitives_tests.rs` へ分離し、primitive registry 本体の line-limit policy を緩めずに維持した。
- `nodesrc/test_resource_checker_responsibility.js` と `nodesrc/test_static_check_boundary_responsibility.js` に、Resource IR 側で `compiler_memory_type_from_constructor_name` を使わないことを監視する policy を追加した。
- 同名 user struct `RegionToken` が owner-token construct extent check を受けない Resource IR regression を追加した。

## 検証

- `cargo fmt -p nepl-core --check`: passed
- `cargo test -p nepl-core resource_primitives --lib -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_uses_proven_region_token_identity_for_construct_extent -- --exact --nocapture`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `cargo check -p nepl-core`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
