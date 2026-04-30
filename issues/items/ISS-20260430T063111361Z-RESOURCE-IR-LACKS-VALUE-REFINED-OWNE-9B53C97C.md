---
id: ISS-20260430T063111361Z-RESOURCE-IR-LACKS-VALUE-REFINED-OWNE-9B53C97C
title: "Resource IR lacks value-refined owner returns for realloc Result::Ok payloads"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/owner_summary_variant_build.rs, nepl-core/src/resource/owner_summary_variant_paths.rs, nepl-core/src/resource/owner_summary_variant_return.rs, nepl-core/src/resource/owner_return.rs, nepl-core/src/resource/owner_control.rs, nepl-core/src/resource/owner_variant.rs, stdlib/core/mem.nepl"
---

# ISS-20260430T063111361Z-RESOURCE-IR-LACKS-VALUE-REFINED-OWNE-9B53C97C: Resource IR lacks value-refined owner returns for realloc Result::Ok payloads

## 概要

realloc/realloc_ptr merges Result::Ok(0) for new_size <= 0 with Result::Ok(new_ptr) for successful growth. The current owner summary is keyed only by enum variant, so it cannot distinguish the Ok payload that carries a transferred owner from the Ok payload that carries no free obligation.

## 対象

- `nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/owner_return.rs, nepl-core/src/resource/owner_control.rs, stdlib/core/mem.nepl`

## 根拠

- 未記入

## 問題

realloc/realloc_ptr merges Result::Ok(0) for new_size <= 0 with Result::Ok(new_ptr) for successful growth. The current owner summary is keyed only by enum variant, so it cannot distinguish the Ok payload that carries a transferred owner from the Ok payload that carries no free obligation.

## 影響

Checked realloc wrappers either reject valid positive-size cleanup with resource.owner.maybe_leak/OwnerUnavailable, or would become unsound if Ok payload ownership were marked unconditionally. This blocks precise memory-safe realloc use without weakening owner checks.

## 修正方針

Add value-refined owner return summaries or split the realloc API contract so the owner-carrying Ok payload is represented separately from zero-size deallocation. The caller-side summary application must transfer the old owner to the returned MemPtr only when the success payload is proven owner-carrying.

## 検証

Add Resource IR regressions for realloc_ptr p old_size positive_new_size: Ok transfers the old owner to q and Err preserves p; also cover new_size <= 0 so Ok(0) does not create a fake owner.

## 2026-04-30 修正

- owner return summary に `variant_projection_returns` を追加し、`Result::Ok` payload へ parameter owner が返る場合も variant 選択後にだけ transfer するようにした。
- summary 生成側は branch / match の戻り値 path を再帰的にたどり、`realloc` のような nested branch の `Ok` payload owner return も variant-gated summary として収集する。
- `check_match` では pending variant owner return を bind local へ移す前に materialize し、通常の match owner transfer と同じ順序で扱う。
- raw address 条件 fact による `NoFreeObligation` 化は、その条件 place 自体が owner state を持つ raw alloc/realloc 結果に限定した。`mem_ptr_addr` で得た非所有 raw alias の `raw < 1` 判定が元 `MemPtr` owner を破壊しないようにした。
- `owner_summary_variant_build` を entry point、`owner_summary_variant_paths` を branch/match path traversal、`owner_summary_variant_return` を returned payload owner 抽出に分割し、Resource checker の責務分割ポリシーに収めた。
- `stdlib/core/mem.nepl` の safe `realloc` は `new_size<=0` を `Err` に分け、owner-carrying `Ok(new_ptr)` と zero-size deallocation を同じ `Ok` variant に混ぜない契約へ変更した。0 サイズ化は `dealloc` を明示的に使う。

## 検証結果

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_safe_realloc_variant_return_preserves_err_owner -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_result_ok_mem_ptr_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 143 passed
- `node nodesrc/issues.js check`: passed (`files=453`)
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/move-effect-variant-owner-return-final.json -j 1 --dist web/dist`: 110 total / 110 passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-variant-owner-return-final.json -j 1 --dist web/dist`: 12 total / 7 passed / 5 failed

`memory_safety.n.md` の doctest#1 は本修正で通過した。残る 5 件は `ISS-20260430T060600668Z-CHECKED-MEMPTR-LOAD-VARIANT-REQUIREM-1A1ADF53` および MemPtr/RegionToken owner model 整理の既存 issue 側で扱う。

## 分離した残件

- `ISS-20260430T070449791Z-FALLIBLE-OWNER-EFFECTS-DO-NOT-RESERV-32CC9198`: fallible owner effect の result を match/refine する前に元 owner を再利用できる可能性は、path-dependent owner reservation として別 issue で扱う。
