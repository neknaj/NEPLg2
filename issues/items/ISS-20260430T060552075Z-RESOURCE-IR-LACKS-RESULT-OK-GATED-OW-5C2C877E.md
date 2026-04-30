---
id: ISS-20260430T060552075Z-RESOURCE-IR-LACKS-RESULT-OK-GATED-OW-5C2C877E
title: "Resource IR lacks Result::Ok-gated owner consumption summaries for checked MemPtr frees"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/owner_control.rs, stdlib/core/mem.nepl, tests/stdlib/memory_safety.n.md"
---

# ISS-20260430T060552075Z-RESOURCE-IR-LACKS-RESULT-OK-GATED-OW-5C2C877E: Resource IR lacks Result::Ok-gated owner consumption summaries for checked MemPtr frees

## 概要

Checked MemPtr free/realloc wrappers return Result and consume the storage owner only on Result::Ok. After call-site RawMemory lowering is reserved for direct raw operations, dealloc_ptr p in a Result::Ok arm leaves the p.raw owner obligation live, while restoring unconditional RawMemory::Dealloc at the wrapper call would consume the owner even on Err.

## 対象

- `nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/owner_control.rs, stdlib/core/mem.nepl, tests/stdlib/memory_safety.n.md`

## 根拠

- `dealloc_ptr p size` は内部で `dealloc raw size` の `Result` を match し、`Result::Ok` だけを外側の `Result::Ok` として返す。
- 既存の owner summary は関数全体で必ず消費される owner だけを表現していたため、`Result::Err` では owner を残し、`Result::Ok` では owner を消費する API を正確に呼び出し側へ伝播できなかった。
- summary 生成側で match arm を再実行するときに、通常の owner checker と異なり pending variant owner effect を arm 入口で適用していなかったため、`dealloc_ptr` が内側の `dealloc` の Ok-gated 消費を外側の Ok summary として再要約できなかった。

## 問題

Checked MemPtr free/realloc wrappers return Result and consume the storage owner only on Result::Ok. After call-site RawMemory lowering is reserved for direct raw operations, dealloc_ptr p in a Result::Ok arm leaves the p.raw owner obligation live, while restoring unconditional RawMemory::Dealloc at the wrapper call would consume the owner even on Err.

## 影響

Valid checked cleanup code is rejected with resource.owner.maybe_leak, and the unsafe alternative would hide double-free/leak paths on failed checked frees. This blocks strict memory-safety validation for core/mem and self-host storage cleanup.

## 修正方針

Add enum-variant-gated owner summaries parallel to initialized-cell variant summaries. Summarize owner consumption/return per Result::Ok/Err branch for checked MemPtr dealloc/realloc APIs, record the pending owner effect at the call result, and apply it only inside matching match arms.

## 検証

Add Resource IR regressions for dealloc_ptr/realloc_ptr where Ok consumes the owner and Err preserves it. Re-run tests/stdlib/memory_safety.n.md cleanup cases and owner_check focused tests.

## 2026-04-30 修正

- `OwnerReturnSummary` に enum variant で gated された owner 消費 summary を追加した。
- direct/indirect call の戻り値に pending variant owner effect を記録し、`match` の該当 variant arm に入った時だけ `CallArgument` として owner を消費するようにした。
- summary 生成時も通常の `check_match` と同じく、arm 入口で pending variant owner effect を適用するようにし、`dealloc_ptr` のような wrapper が内側の `dealloc` の Ok-gated 消費を外側の Ok summary へ再要約できるようにした。
- unconditional summary 側では `MaybeFreed` を「必ず消費済み」として扱わないようにし、Err 経路の owner を誤って消費しないようにした。
- `tests/stdlib/memory_safety.n.md` の基本 dealloc doctest は、`store_i32` / `dealloc_ptr` が Err を返した経路で残る owner を明示的に閉じる形へ更新した。

## 検証結果

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_result_ok_mem_ptr_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 142 passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-result-ok-owner-consumption-after-cleanup.json -j 1`: 12 total / 7 passed / 5 failed

## 分離した残件

- `ISS-20260430T063111361Z-RESOURCE-IR-LACKS-VALUE-REFINED-OWNE-9B53C97C`: `realloc/realloc_ptr` は `Result::Ok(0)` と owner-carrying `Result::Ok(new_ptr)` が同じ variant に合流するため、値条件付き owner return summary または API 契約分割が必要。
