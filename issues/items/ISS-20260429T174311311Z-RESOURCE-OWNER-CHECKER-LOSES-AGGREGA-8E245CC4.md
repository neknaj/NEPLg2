---
id: ISS-20260429T174311311Z-RESOURCE-OWNER-CHECKER-LOSES-AGGREGA-8E245CC4
title: "Resource owner checker loses aggregate raw cell root through loop address views"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/owner_alias.rs, nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_flow.rs, nepl-core/src/resource/owner_state.rs, nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/owner_transfer.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260429T174311311Z-RESOURCE-OWNER-CHECKER-LOSES-AGGREGA-8E245CC4: Resource owner checker loses aggregate raw cell root through loop address views

## 概要

After raw address loads are treated as non-owning views, HashMap insert still leaves the entries owner under hdr.StorageOffset(8).Deref. The remaining leak appears when the loaded entries view is used through helper address calculations and loop/branch flow before returning the aggregate.

## 対象

- `nepl-core/src/resource/owner_check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `HashMap insert` の `ready` aggregate は `field::get ready "hdr"` で header pointer を non-owning view として読み、`load_i32 add hdr 8` で backing entries pointer を読む。その後 helper call / loop / branch を経由して `ready` を返すため、raw address alias と owner table の合流が同じ raw cell owner を同一 root として扱う必要がある。
- 旧実装では raw alias graph と owner table を別々に merge していたため、同じ owner が `ready.field0.StorageOffset(8).Deref` と `hdr.StorageOffset(8).Deref` に分裂した。
- さらに raw pointer の copy で projected alias group が存在する場合、基底 pointer の direct alias が seed されず、`field::get` の単なる pointer read が owner move と誤判定される経路があった。
- 戻り値へ owner を移したあとに残る旧 alias の `Moved` entry を aliased descendant として再診断しており、正しく移動済みの stale alias が false positive になっていた。

## 問題

After raw address loads are treated as non-owning views, HashMap insert still leaves the entries owner under hdr.StorageOffset(8).Deref. The remaining leak appears when the loaded entries view is used through helper address calculations and loop/branch flow before returning the aggregate.

## 影響

HashMap insert remains rejected by the strict Resource owner gate even though the backing entries owner should stay attached to the returned HashMap. This blocks collection integration tests and self-host collection work.

## 修正方針

Add a focused Resource IR regression for a returned aggregate whose backing raw address is loaded as a view, used inside a loop/branch for raw writes, and then returned. Preserve the aggregate raw cell owner root across loop/branch alias merges and helper address views.

## 修正内容

- raw pointer copy は projected alias を引き継ぐ場合でも、基底 pointer の direct alias を必ず保持するようにした。
- branch / loop / match の owner merge は、先に merge した raw alias graph を使って raw owner cell address を canonicalize してから owner state を merge するようにした。
- owner transfer / move / summary は raw alias 経由の descendant owner を扱うようにし、aggregate の戻り値に raw backing owner が残ることを証明できるようにした。
- owner を戻り値側へ移した後に残る `Moved` / `Freed` の stale alias は aliased descendant の実 owner として扱わないようにした。
- ResourceIR 回帰を追加し、loop address view、enum payload、field alias replacement の各経路で raw cell owner が正しい aggregate root に残ることを固定した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_keeps_aggregate_raw_cell_root_through_loop_address_views -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_moves_aliased_raw_cell_owner_into_enum_payload -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_returns_aggregate_with_raw_cell_owner_stored_through_field_alias -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reinitializes_self_update_report_projection_returns -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_ -- --nocapture`: 40 passed
- `cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture`: `insert...` 内の `hdr.StorageOffset(8).Deref` leak は解消。残りは `main` 側の `map1` header / entries 未解放で、`ISS-20260429T120339805Z-FALLIBLE-OWNING-COLLECTION-UPDATES-L-21EF56CB` の HashMap read/free API 契約残件として継続する。
- `cargo fmt --check`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
