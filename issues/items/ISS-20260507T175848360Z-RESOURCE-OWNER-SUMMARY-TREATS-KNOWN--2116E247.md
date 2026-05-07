---
id: ISS-20260507T175848360Z-RESOURCE-OWNER-SUMMARY-TREATS-KNOWN--2116E247
title: "Resource owner summary treats known higher-order non-owning MemPtr return as consumption"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-08
target: "nepl-core/src/resource/owner_return.rs, nepl-core/src/resource/owner_return_apply.rs, nepl-core/src/resource/owner_return_apply_source.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md"
---

# ISS-20260507T175848360Z-RESOURCE-OWNER-SUMMARY-TREATS-KNOWN--2116E247: Resource owner summary treats known higher-order non-owning MemPtr return as consumption

## 概要

A borrowed MemPtr from region_ptr remains non-owning, but when it is passed through a higher-order helper whose function-typed parameter is bound to a known identity callback, the owner summary for the helper is computed as unknown indirect call consumption. Caller-side application reports resource.owner.no_free_obligation at the helper call instead of preserving the non-owning pointer return.

## 対象

- `nepl-core/src/resource/owner_return.rs`
- `nepl-core/src/resource/owner_return_apply.rs`
- `nepl-core/src/resource/owner_return_apply_source.rs`
- `nepl-core/tests/resource_ir.rs`
- `tests/stdlib/memory_safety.n.md`

## 根拠

- `apply_ptr <(MemPtr<u8>, (MemPtr<u8>)->MemPtr<u8>)->MemPtr<u8>>` の unknown indirect call summary が、同じ `MemPtr<u8>` 形状の引数と返り値を生の `TypeId` 不一致で別型扱いし、返却候補として扱えなかった。
- その結果、`apply_ptr p @id_ptr` は本来「既知の identity callback から非所有 `MemPtr` view が返る」経路であるにもかかわらず、summary 適用時に `p.field0` の owner consumption として扱われ、`resource.owner.no_free_obligation` が callback helper 呼び出し位置で出ていた。
- `MemPtr = non-owning pointer` と `RegionToken = free obligation owner` の分離では、非所有 pointer view を callback helper 経由で保存する一方、そこから owner token を forged する経路だけを拒否する必要がある。

## 問題

A borrowed MemPtr from region_ptr remains non-owning, but when it is passed through a higher-order helper whose function-typed parameter is bound to a known identity callback, the owner summary for the helper is computed as unknown indirect call consumption. Caller-side application reports resource.owner.no_free_obligation at the helper call instead of preserving the non-owning pointer return.

## 影響

Safe non-owning pointer projections cannot be passed through callback-based helpers without false owner-consumption diagnostics. This is a safety-preserving false positive, but it shows that Resource IR owner summaries cannot yet express callback-specialized non-owning pointer flow, which is required by the MemPtr = non-owning pointer / owner token split.

## 修正方針

Represent known callback actuals during owner summary application or summary instantiation so higher-order helpers can reuse the callback owner/non-owning return summary. Keep unknown indirect calls conservative and keep callbacks that deallocate or consume owner obligations rejected.

## 対応内容

- unknown indirect call の「引数が返り値として戻る可能性」判定を `arg.ty == output.ty` から `TypeCtx::same_type(arg.ty, output.ty)` に変更し、同じ意味の型が異なる `TypeId` になった場合でも Resource owner summary が返却 source を失わないようにした。
- owner summary 適用時、返却 source が caller 側では非所有 raw address view である場合に owner transfer ではなく non-owning raw view / raw alias / storage origin を出力へ写すようにした。
- `owner_return_apply.rs` の責務上限を超えないよう、summary source 解決・消費適用・非所有返却補助を `owner_return_apply_source.rs` に分割した。
- unknown indirect call 自体の保守性は残し、非所有 `MemPtr` から forged `RegionToken` を作って解放しようとする経路は `resource.owner.no_free_obligation` のまま拒否する。

## 検証

Add Resource IR and compile_fail regressions for a borrowed region_ptr passed through an identity callback and then forged into RegionToken, plus a consuming callback case that must still report resource.owner.no_free_obligation.

- `cargo fmt --check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_region_ptr_through_known_identity_callback -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged_from_higher_order_region_ptr -- --nocapture`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-higher-order-nonowning-memory-safety.json -j 1 --dist web/dist`: total=20, passed=20
- `node nodesrc/test_diagnostic_code_first_boundary.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `git diff --check`: passed
