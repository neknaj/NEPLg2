---
id: ISS-20260512T124702911Z-RESOURCE-BORROW-AND-OWNER-SUMMARY-ST-DAA86E59
title: "Resource borrow and owner summary still duplicate variant name normalization"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/borrow_usage.rs; nepl-core/src/resource/owner_summary_variant_ambiguous.rs; nepl-core/src/resource/variant_name.rs"
---

# ISS-20260512T124702911Z-RESOURCE-BORROW-AND-OWNER-SUMMARY-ST-DAA86E59: Resource borrow and owner summary still duplicate variant name normalization

## 概要

Resource IR borrow token propagation and ambiguous owner-summary variant return collection still canonicalize enum variant names with local rsplit logic instead of the shared variant_name utility.

## 対象

- `nepl-core/src/resource/borrow_usage.rs; nepl-core/src/resource/owner_summary_variant_ambiguous.rs; nepl-core/src/resource/variant_name.rs`

## 根拠

- `nepl-core/src/resource/borrow_usage.rs` は match arm payload へ borrow token tree を伝播する際に、pattern variant を file-local `rsplit("::")` で canonicalize していた。
- `nepl-core/src/resource/owner_summary_variant_ambiguous.rs` は ambiguous enum projection return の variant dedupe で、同じく local `rsplit("::")` による variant 比較を持っていた。
- `place_utils.rs` / initialized / owner variant checks は `variant_name.rs` へ移行済みであり、borrow / owner summary だけ別規則を持つと enum payload place key の一貫性が崩れる。

## 問題

Resource IR borrow token propagation and ambiguous owner-summary variant return collection still canonicalize enum variant names with local rsplit logic instead of the shared variant_name utility.

## 影響

Borrow-token transfer and owner summary variant projection returns can diverge from the Resource IR canonical enum payload key, weakening lifetime and owner checks around qualified enum payload match arms.

## 修正方針

Use variant_name::match_pattern_variant_name and variant_name::variant_names_match from the remaining Resource IR modules, remove local variant normalization helpers, and extend the responsibility policy to reject reintroduced local rsplit usage outside variant_name.rs.

## 検証

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_compiler_rejects_match_payload_borrow_move -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir owner_return -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir variant_owner -- --nocapture`: passed
- `node nodesrc/issues.js check --dir issues`: passed

## 対応記録

- `borrow_usage.rs` の match arm payload borrow propagation を `variant_name::match_pattern_variant_name` へ移行した。
- `owner_summary_variant_ambiguous.rs` の ambiguous variant dedupe を `variant_name::variant_names_match` へ移行した。
- `nodesrc/test_resource_checker_responsibility.js` へ、`variant_name.rs` 以外の Resource IR module が `rsplit("::")` による variant canonicalization を再導入しない source policy を追加した。
