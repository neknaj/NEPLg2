---
id: ISS-20260429T173344520Z-RESOURCE-OWNER-CHECKER-MOVES-RAW-ADD-D665B59D
title: "Resource owner checker moves raw address owner on non-owning raw cell load"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core/src/resource/owner_check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260429T173344520Z-RESOURCE-OWNER-CHECKER-MOVES-RAW-ADD-D665B59D: Resource owner checker moves raw address owner on non-owning raw cell load

## 概要

Loading an i32 raw address from a raw cell that stores an owned backing allocation currently transfers the owner to the loaded local. HashMap insert reads entries from hdr + 8 only to probe and write buckets, but the checker reports the local entries owner may leak.

## 対象

- `nepl-core/src/resource/owner_check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture` で、`insert...` の local `entries` owner may leak が再現した。
- focused regression `resource_ir_owner_check_keeps_raw_address_load_as_nonowning_view` は、`make_box` の戻り値に projection owner summary が適用された後で `load_i32 add hdr 8` を行うと、修正前に `entries` leak と return-side `Moved` を報告した。
- `load_i32` の出力は non-owning raw address view として probe / write に使われるだけで、raw cell に保持された backing storage owner を load 時点で移動すべきではない。

## 問題

Loading an i32 raw address from a raw cell that stores an owned backing allocation currently transfers the owner to the loaded local. HashMap insert reads entries from hdr + 8 only to probe and write buckets, but the checker reports the local entries owner may leak.

## 影響

Valid collection methods that read an internal backing pointer as a non-owning address view are rejected, pushing stdlib code toward artificial free/reinsert patterns and weakening the planned MemPtr/non-owning pointer separation.

## 修正方針

Treat i32 loads from raw cells known to contain raw addresses as non-owning alias views. Keep the owner at the raw cell root so dealloc/return/store can still consume it through the alias when required. Add a focused Resource IR regression for reading entries, writing through the view, returning the aggregate, then freeing it at the caller.

## 修正内容

- raw memory load の出力型が `i32` で、load 元 raw cell が raw address value として alias tracking されている場合は、owner transfer ではなく non-owning alias view として扱うようにした。
- owner は raw cell root に残し、`dealloc_raw entries` や return など所有権を消費する操作では alias 解決により元の owner を消費できる。
- call return summary で projection owner が適用された aggregate から raw backing pointer を読んで使い、aggregate を返して caller で free する focused regression を追加した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_keeps_raw_address_load_as_nonowning_view -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_ -- --nocapture`: 38 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_stored_tail_owner_under_new_raw_node -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_returns_aggregate_with_raw_cell_owner_stored_through_field_alias -- --nocapture`: passed
- `cargo fmt --check`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/resource-owner-raw-load-view-move-effect.json -j 1 --dist web/dist`: total=110, passed=110
- `node nodesrc/issues.js check`: passed
- `cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture`: still fails, but local `entries` leak is gone. Remaining diagnostics are `insert... hdr.StorageOffset(8).Deref` and caller-side unfreed HashMap owners, tracked by `ISS-20260429T120339805Z-FALLIBLE-OWNING-COLLECTION-UPDATES-L-21EF56CB`.
- `cargo test -p nepl-core --test neplg2 llvm_hashmap_string_key_preserves_explicit_hasher_type_args -- --nocapture`: still fails with the same remaining `insert... hdr.StorageOffset(8).Deref` and caller-side unfreed HashMap owners.
