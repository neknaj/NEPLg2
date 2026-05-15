---
id: ISS-20260515T110646911Z-CHECKED-MEMPTR-PROOF-DROPS-REGIONTOK-B846CF4C
title: "checked MemPtr proof drops RegionToken return provenance"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource/effect_return_summary_filter.rs, nepl-core/src/resource/effect_summary_identity.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260515T110646911Z-CHECKED-MEMPTR-PROOF-DROPS-REGIONTOK-B846CF4C: checked MemPtr proof drops RegionToken return provenance

## 概要

Resource IR raw identity summary used the same owner-protection filter as public raw escape diagnostics. RegionToken returns are owner-protected at the public surface, but their raw allocation identity is still required as internal provenance evidence for region_ptr/region_ptr_at and checked MemPtr wrappers. As a result safe alloc_region -> region_ptr_at/store/load and callback-returned region pointers fail with resource.raw.memory_outside_boundary.

## 対象

- `nepl-core/src/resource/effect_summary_identity.rs`

## 根拠

- 静的検査大規模修正 Stage 6 では、checked `MemPtr` wrapper を stdlib 名の allowlist で信頼せず、Resource IR の raw identity / pointer alias / owner provenance から証明する。
- `RegionToken<T>` は public surface では owner-protected なので raw address escape diagnostic の対象から外す必要がある。一方で、`region_ptr` / `region_ptr_at` が返す `MemPtr<T>` の checked access は、その `RegionToken<T>` が allocator 由来 owner token であることを内部証跡として必要とする。
- `str` のような高水準所有値は raw identity summary から隠すべきだが、`RegionToken` の owner provenance を同じ filter で隠すと、safe checked wrapper が証明不能になる。

## 問題

Resource IR raw identity summary used the same owner-protection filter as public raw escape diagnostics. RegionToken returns are owner-protected at the public surface, but their raw allocation identity is still required as internal provenance evidence for region_ptr/region_ptr_at and checked MemPtr wrappers. As a result safe alloc_region -> region_ptr_at/store/load and callback-returned region pointers fail with resource.raw.memory_outside_boundary.

## 影響

This is a static-check false positive in the memory-safety proof path. It pressures callers to bypass checked wrappers or weaken the boundary, and it prevents Stage 6 ResourceIR provenance from proving safe RegionToken-derived MemPtr operations.

## 修正方針

Separate public escape filtering from internal provenance summary filtering. Keep str and other opaque high-level values from exporting raw identity summaries, but allow RegionToken owner provenance to remain in RawIdentityReturnSummary so checked MemPtr proof can follow compiler-issued allocation identity through Result payloads and helper/callback returns. Public raw escape diagnostics must continue to treat RegionToken as owner-protected.

## 検証

Focused Rust ResourceIR tests for region_ptr, region_ptr_at, callback-returned region MemPtr, effect summary filter tests, and tests/stdlib/memory_safety.n.md must pass.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 2026-05-15 解決内容

- `raw_identity_return_projection_requires_summary` から public escape diagnostic 用の owner-protection filter を外し、internal summary 用の opaque owner filter を別に定義した。
- internal summary では `str` の raw identity は引き続き抑止するが、`RegionToken` は抑止しない。これにより `alloc_region -> Result::Ok(RegionToken)`、`region_ptr_at -> Result::Ok(MemPtr)`、callback-returned `MemPtr` の証跡が `RawIdentityReturnSummary` を通って checked `store` / `load` / `fill` へ届く。
- public raw escape diagnostic 側の `raw_identity_return_projection_is_escape` は変更していない。`RegionToken` は public surface では owner-protected のままで、raw address escape として誤報告されない。
- ResourceIR integration test の stale な direct raw `store_u8 mem_ptr_addr p` は、現在の boundary 設計に合わせて checked `store_u8 p` へ更新した。これは raw memory authority を緩める修正ではなく、callback provenance を checked wrapper 経由で検証する回帰である。

検証:

- `cargo fmt -p nepl-core --check`
- `cargo test -p nepl-core --lib effect_return_summary_filter -- --nocapture`
- `cargo test -p nepl-core --test resource_ir checked_region -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_region_ptr_through_known_identity_callback -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_region_ptr_through_callback_parameter -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_region_token_forged -- --nocapture`
- `trunk build`
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-memory-safety-regiontoken-provenance-after.json -j 1 --dist web/dist --assert-io`: total=37, passed=37
