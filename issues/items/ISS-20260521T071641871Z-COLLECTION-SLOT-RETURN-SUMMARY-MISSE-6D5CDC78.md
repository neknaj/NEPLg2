---
id: ISS-20260521T071641871Z-COLLECTION-SLOT-RETURN-SUMMARY-MISSE-6D5CDC78
title: "Collection slot return summary misses nested callee return transfers"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource/collection_slot_summary_return_collect.rs
---

# ISS-20260521T071641871Z-COLLECTION-SLOT-RETURN-SUMMARY-MISSE-6D5CDC78: Collection slot return summary misses nested callee return transfers

## 概要

Collection slot return summary follows direct parameter returns and aggregate payload construction, but stops when the returned value was produced by another function call. A wrapper such as return Result::Err(identity_storage(storage)) can therefore lose caller slot state before the caller matches the returned enum payload.

## 対象

- `nepl-core/src/resource/collection_slot_summary_return_collect.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、collection slot lifecycle を stdlib/module allowlist ではなく Resource IR の generic proof boundary へ載せる方針を定めている。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、fallible owner-preserving API が collection owner と payload owner を失わないことを要求している。
- 直前の [ISS-20260521T065624831Z-COLLECTION-SLOT-STATE-RETURN-SUMMARY-4591B626](./ISS-20260521T065624831Z-COLLECTION-SLOT-STATE-RETURN-SUMMARY-4591B626.md) で direct `Result::Err(storage)` payload transfer は固定したが、`Result::Err(identity_storage(storage))` のような helper composition は call output producer で探索が止まっていた。

## 問題

Collection slot return summary follows direct parameter returns and aggregate payload construction, but stops when the returned value was produced by another function call. A wrapper such as return Result::Err(identity_storage(storage)) can therefore lose caller slot state before the caller matches the returned enum payload.

## 影響

Owner-preserving stdlib/self-host helper composition can hide live non-Copy collection slots from Resource IR, weakening memory-safety checks around fallible APIs that recover collection storage through nested helpers.

## 修正方針

Compose callee CollectionSlotLifecycleReturnTransfer facts through call outputs while collecting the wrapper return summary. Instantiate callee sources with the wrapper call args, canonicalize raw owner aliases, map them back to the wrapper parameters, and append the wrapper target suffix without stdlib or Result-specific allowlists.

## 対応

- `collection_slot_summary_return_collect` が return value producer として `ResourceOp::Call` / `ResourceOp::IndirectCall` を見た場合、callee の `CollectionSlotLifecycleReturnTransfer` を wrapper の return transfer へ合成するようにした。
- 合成時は `instantiate_summary_target` で callee source を wrapper call actual へ写し、`RawCellAddressAliases::canonicalize_owner_cell_address` 後に `summary_place_for_params` で wrapper parameter-relative source へ戻す。
- target 側は wrapper return value から call output までの structural suffix に callee transfer の suffix を append し、`target_ty` は append 後の callee target type を保持する。
- direct call と function-alias indirect call の両方を同じ helper へ通し、stdlib module 名、`Result` 名、特定 wrapper 名の allowlist は追加していない。
- `resource_ir_collection_slot_call_summary_transfers_caller_slot_through_nested_returned_enum_payload` を追加し、`identity_storage(storage)` の返り値を wrapper が `StorageResult::Err` に包んだ場合でも caller match bind 後の `StorageDealloc` が `LiveSlotDuringStorageDealloc` を報告することを固定した。

## 検証

Add a Resource IR regression where a wrapper calls identity_storage, wraps the returned storage in StorageResult::Err, the caller matches Err, and StorageDealloc of the recovered storage reports LiveSlotDuringStorageDealloc. Run focused resource_ir collection slot tests, cargo check -p nepl-core, cargo fmt --check, node nodesrc/issues.js check --dir issues, and git diff --check.

実施:

- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary_transfers_caller_slot_through_nested_returned_enum_payload -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_call_summary -- --test-threads=1`: passed
- `cargo check -p nepl-core`: passed
- `cargo fmt --check`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
