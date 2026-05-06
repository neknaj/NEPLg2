---
id: ISS-20260506T172012873Z-RESOURCE-IR-DYNAMIC-RAW-ADDRESS-VIEW-77E94B53
title: "Resource IR dynamic raw address views lose stable local value origins"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/initialized_alias.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260506T172012873Z-RESOURCE-IR-DYNAMIC-RAW-ADDRESS-VIEW-77E94B53: Resource IR dynamic raw address views lose stable local value origins

## 概要

RawCellAddressAliases ValueOrigin only resolved exact places. After fill_i32 initialized a range through one read of a stable local raw address, a later add of another read of the same local with a dynamic offset produced tmp[+?] instead of the stable local origin plus offset, so Resource IR reported resource.cell.uninit for initialized dynamic loads.

## 対象

- `nepl-core/src/resource/initialized_alias.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `fill_i32 pref pref_len 0` は Resource IR 上で `pref[+?].deref` の Copy cell を initialized にする。
- 後続の `let prev_ptr add pref prev_off` は、`pref` を再度 read した一時値に `StorageOffset(None)` を付けた raw address view として lowering される。
- 既存の `ValueOrigin` は `tmp -> %pref` の exact place だけを解決し、`tmp[+?] -> %pref[+?]` のように projection suffix を保持して stable origin へ戻せなかった。
- 通常 i32 copy を raw alias group として seed し直すと、`ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232` で修正した alias explosion を再発させる。

## 問題

RawCellAddressAliases ValueOrigin only resolved exact places. After fill_i32 initialized a range through one read of a stable local raw address, a later add of another read of the same local with a dynamic offset produced tmp[+?] instead of the stable local origin plus offset, so Resource IR reported resource.cell.uninit for initialized dynamic loads.

## 影響

KP prefix-sum style code and self-host scanners can be rejected even when the source initializes a dynamic raw range before reading it. Adding broad raw alias groups would hide the problem and reintroduce compile-time alias explosion, so the fix must keep scalar copies out of raw alias groups while preserving stable origins structurally.

## 修正方針

Resolve ValueOrigin through place prefixes and rebuild the suffix on the stable origin, so tmp[+?] canonicalizes to local[+?] without seeding ordinary scalar copies as raw alias groups. Add a Resource IR regression for fill_i32 followed by a later dynamic add/load through another local read.

## 検証

cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_dynamic_fill_origin_across_local_reads -- --nocapture

## 2026-05-06 修正

`RawCellAddressAliases::value_origin` を exact place のみではなく prefix 対応にし、origin 登録済み place の descendant であれば suffix を stable origin 側へ付け替えるようにした。

これにより、通常 scalar copy は raw alias group を seed しないまま、`tmp[+?]` のような projected dynamic raw address view だけが `%pref[+?]` へ正規化される。`fill_i32` が初期化した dynamic Copy range と、後続の別 read から作った dynamic load が同じ Resource IR cell fact を参照できる。

回帰として `resource_ir_cell_check_preserves_dynamic_fill_origin_across_local_reads` を追加した。このテストは `fill_i32 pref pref_len 0` の後、別の `pref` read から `add pref prev_off` を作り、`load_i32` が `resource.cell.uninit` にならないことを確認する。

追加確認:

- `cargo fmt --check`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_dynamic_fill_origin_across_local_reads -- --nocapture`: passed
- `cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 -- --nocapture`: dynamic range の `resource.cell.uninit` は解消。次の別件として fs/stdio scratch dealloc の `resource.owner.no_free_obligation` が残る。

## 関連

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38](./ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38.md)
- [ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232](./ISS-20260506T154254968Z-RESOURCE-AUTHORITY-DEEP-PREFIX-REGRE-14D21232.md)
