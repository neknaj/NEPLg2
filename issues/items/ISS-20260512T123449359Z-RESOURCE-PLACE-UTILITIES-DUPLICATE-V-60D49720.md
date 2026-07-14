---
id: ISS-20260512T123449359Z-RESOURCE-PLACE-UTILITIES-DUPLICATE-V-60D49720
title: "Resource place utilities duplicate variant name normalization"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-07-15
target: "nepl-core/src/resource/place_utils.rs; nepl-core/src/resource/variant_name.rs"
---

# ISS-20260512T123449359Z-RESOURCE-PLACE-UTILITIES-DUPLICATE-V-60D49720: Resource place utilities duplicate variant name normalization

## 概要

Resource IR place_utils.rs still canonicalizes enum variant payload names locally while initialized and owner variant checks use the shared variant_name utility.

## 対象

- `nepl-core/src/resource/place_utils.rs; nepl-core/src/resource/variant_name.rs`

## 根拠

- `nepl-core/src/resource/place_utils.rs` の `construct_aggregate_field_place` が enum aggregate payload projection を作る際に、Resource IR 共通の `variant_name::normalize_variant_name` ではなく file-local helper で variant 名を正規化していた。
- 同じ file の `match_arm_variant_payload_name` も match pattern payload 名を独自に `rsplit("::")` しており、initialized / owner / owner summary の variant 判定と別規則になる余地があった。
- Resource IR の enum payload place は initialized cell、owner transfer、raw-address alias / raw-view propagation の共通の足場なので、variant 名 canonicalization は単一 module に集約する必要がある。

## 問題

Resource IR place_utils.rs still canonicalizes enum variant payload names locally while initialized and owner variant checks use the shared variant_name utility.

## 影響

Enum payload place generation can diverge from initialized/owner summary variant matching, weakening static memory-safety checks around enum payload moves, borrows, and raw-address propagation.

## 修正方針

Route enum payload place construction and match-bind payload extraction through the shared variant_name utility and remove the local duplicate canonicalization helper.

## 検証

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_result_payload_raw_address_field -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir variant_owner -- --nocapture`: passed
- `node nodesrc/issues.js check --dir issues`: passed

## 対応記録

- `place_utils.rs` の enum aggregate payload projection を `variant_name::normalize_variant_name` へ移行した。
- match arm payload 名の抽出を `variant_name::match_pattern_variant_name` へ移行し、`match_bind_payload_place` と inactive sibling payload 判定が同じ canonical name を使うようにした。
- enum payload type lookup も `variant_name::variant_names_match` へ移行し、`place_utils.rs` から variant 名比較の local 規則を取り除いた。
- `nodesrc/test_resource_checker_responsibility.js` に `place_utils.rs` が shared variant utility を import すること、local `canonical_variant_name` を再導入しないことを追加した。

## 2026-07-15 regression restoration

exclusive enum payload sibling判定の追加時、qualified / generic variantのfamily tail抽出が`place_utils.rs`のlocal `rsplit`として再導入され、shared variant-name responsibility policyが再び失敗していた。family抽出を`variant_name::variant_family_name`へ移し、`qualified_name`の共有split/tail境界だけが`::`を解釈する構造へ戻した。

`Result::Ok`、generic引数内にqualified nameを持つ`Result<core::Foo,str>::Ok`、qualified familyの`core::Result::Err`、unqualified `Err`をunit回帰で固定した。exclusive sibling判定のsame-family/different-family契約と既存owner alias修正は変更していない。

responsibility policyにはlocal `rsplit`禁止だけでなく、`place_utils.rs`がshared `variant_family_name`をimportして使用するpositive assertionも追加した。別形式のlocal family splitへ置換して責務境界を迂回する変更も検出する。
