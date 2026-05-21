---
id: ISS-20260521T065624831Z-COLLECTION-SLOT-STATE-RETURN-SUMMARY-4591B626
title: "Collection slot state return summary misses enum payload owner transfers"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_summary_return*.rs"
---

# ISS-20260521T065624831Z-COLLECTION-SLOT-STATE-RETURN-SUMMARY-4591B626: Collection slot state return summary misses enum payload owner transfers

## 概要

Collection slot return summaries only record a direct returned parameter. A callee that returns an owner-preserving Result/enum payload such as Err(storage) does not summarize the parameter slot state transfer into the returned enum payload, so caller-side match binding can lose live/moved/released slot evidence.

## 対象

- `nepl-core/src/resource/collection_slot_summary_return*.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、collection slot state を通常の owner value transfer と call return summary に追従させ、stdlib module allowlist ではなく generic Resource IR proof boundary で検査する方針を Stage 6 に置いている。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、fallible collection update が owner を error/success payload に戻す必要を明記している。
- umbrella issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) の残件として、owner-preserving result が Resource IR 上でも slot state を保持することを確認する必要があった。

## 問題

Collection slot return summaries only record a direct returned parameter. A callee that returns an owner-preserving Result/enum payload such as Err(storage) does not summarize the parameter slot state transfer into the returned enum payload, so caller-side match binding can lose live/moved/released slot evidence.

## 影響

Fallible collection APIs can appear to recover owners while Resource IR forgets initialized non-Copy slots inside the error/success payload. That can hide storage-only dealloc of live payloads and blocks safe non-Copy collection support for self-host.

## 修正方針

Derive return transfers from source Resource IR value construction/control-flow rather than stdlib allowlists: follow enum/struct/tuple construct payloads and branch/match value forwarding to map parameter-relative slot state into return-value suffixes.

## 対応

- `collect_return_facts_from_terminator` に return block の `ResourceOp` を渡し、return value の producer を source-level Resource IR から辿れるようにした。
- direct parameter return だけでなく、`Construct` の enum / struct / tuple payload、`Branch` / `Match` の arm value、`DeclareLocal` / `Read` / `Move` / `Assign` の value forwarding を汎用的に追跡する `collect_return_transfers_from_ops` を追加した。
- `Result::Err(storage)` のような enum payload へ parameter owner が入る場合、caller 側では actual argument の collection slot state を `output.Err` payload へ移し、match binding で取り出した storage owner の dealloc に live slot が見えるようにした。
- stdlib 関数名や `Result` 固有の allowlist は追加していない。`AggregateKind` と `ResourceOp` の enum/match から source-derived に return transfer を構成する。

## 検証

Add Resource IR regression where a helper returns Result::Err(storage), caller matches Err payload, and storage dealloc must see the transferred live slot.

- `resource_ir_collection_slot_call_summary_transfers_caller_slot_through_returned_enum_payload` を追加し、callee が `Result::Err(storage)` として owner を返す場合でも、caller の `Err` match payload bind 後の storage dealloc が live slot を検出することを固定した。
