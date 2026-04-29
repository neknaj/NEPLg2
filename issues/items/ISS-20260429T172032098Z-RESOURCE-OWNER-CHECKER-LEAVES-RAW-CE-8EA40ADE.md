---
id: ISS-20260429T172032098Z-RESOURCE-OWNER-CHECKER-LEAVES-RAW-CE-8EA40ADE
title: "Resource owner checker leaves raw cell owner under local address alias"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core/src/resource/owner_check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260429T172032098Z-RESOURCE-OWNER-CHECKER-LEAVES-RAW-CE-8EA40ADE: Resource owner checker leaves raw cell owner under local address alias

## 概要

When a raw owner is stored through a local raw address alias derived from an owning aggregate field, returning the aggregate can leave the nested raw cell owner under the local alias. HashMap rehash stores new entries through hdr + 8 and returns hm, but Resource owner diagnostics still report hdr.StorageOffset(8).Deref as live.

## 対象

- `nepl-core/src/resource/owner_check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture` で `hashmap_rehash_to...` の `hdr.StorageOffset(8).Deref` owner leak が再現した。
- focused regression `resource_ir_owner_check_returns_aggregate_with_raw_cell_owner_stored_through_field_alias` は修正前に `replace_entries__HeaderBox__HeaderBox__imp` の `hdr.StorageOffset(8).Deref` leak で失敗した。
- `field::get box "hdr"` の read temporary は raw address alias を持つが、`let hdr` へ移す時点で、owner がまだ seed されていない通常の function check では alias が引き継がれなかった。

## 問題

When a raw owner is stored through a local raw address alias derived from an owning aggregate field, returning the aggregate can leave the nested raw cell owner under the local alias. HashMap rehash stores new entries through hdr + 8 and returns hm, but Resource owner diagnostics still report hdr.StorageOffset(8).Deref as live.

## 影響

Valid owning aggregate update code is rejected, and stdlib HashMap owner contract work is blocked unless the Resource IR owner gate is weakened. The checker must keep alias ownership roots consistent instead.

## 修正方針

Add a focused Resource IR regression for storing a replacement raw owner through a header alias and returning the aggregate, then canonicalize raw memory owner cells through the concrete owner root instead of the local raw alias.

## 修正内容

- raw memory load/store の owner cell address には、通常の raw address canonical ではなく owner cell 用 canonical を使うようにした。
- owner cell 用 canonical は、raw local alias よりも struct/tuple/enum projection を含む aggregate field alias を優先する。
- `let hdr = field::get box "hdr"` のような非 owning i32 raw address view では、既存 alias group がある場合に owner の有無だけへ依存せず alias を local 宣言/代入へ引き継ぐようにした。
- focused Resource IR regression を追加し、aggregate field alias 経由で raw cell owner を格納して aggregate を返すケースを固定した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_returns_aggregate_with_raw_cell_owner_stored_through_field_alias -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_ -- --nocapture`: 37 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_stored_tail_owner_under_new_raw_node -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_raw_address_stored_in_aggregate_field -- --nocapture`: passed
- `cargo fmt --check`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/resource-owner-alias-root-move-effect.json -j 1 --dist web/dist`: total=110, passed=110
- `node nodesrc/issues.js check`: passed
- `cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture`: still fails, but `hashmap_rehash_to... hdr.StorageOffset(8).Deref` leak is gone. Remaining diagnostics are `insert... entries` and caller-side unfreed HashMap owners, tracked by `ISS-20260429T120339805Z-FALLIBLE-OWNING-COLLECTION-UPDATES-L-21EF56CB`.
- `cargo test -p nepl-core --test neplg2 llvm_hashmap_string_key_preserves_explicit_hasher_type_args -- --nocapture`: still fails with the same remaining `insert... entries` and caller-side unfreed HashMap owners.
