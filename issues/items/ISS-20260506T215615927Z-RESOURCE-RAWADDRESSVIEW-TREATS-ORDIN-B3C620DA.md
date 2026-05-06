---
id: ISS-20260506T215615927Z-RESOURCE-RAWADDRESSVIEW-TREATS-ORDIN-B3C620DA
title: "Resource RawAddressView treats ordinary i32 arithmetic as a raw pointer"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_alias_raw_view.rs, nepl-core/src/resource/initialized_raw_view.rs, nepl-core/src/resource/initialized_rekey.rs, nepl-core/src/resource/owner_raw_view.rs, nepl-core/tests/resource_ir.rs"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行
---

# ISS-20260506T215615927Z-RESOURCE-RAWADDRESSVIEW-TREATS-ORDIN-B3C620DA: Resource RawAddressView treats ordinary i32 arithmetic as a raw pointer

## 概要

ResourceOp::RawAddressView is emitted for address-like add/sub syntax, but the initialized and owner checkers currently promote every view target into a raw-address alias even when the source is only an ordinary i32 value with a stable origin. This can make non-pointer arithmetic participate in raw alias state and destabilize guarded range checking.

## 対象

- `nepl-core/src/resource/initialized_alias_raw_view.rs`
- `nepl-core/src/resource/initialized_raw_view.rs`
- `nepl-core/src/resource/initialized_rekey.rs`
- `nepl-core/src/resource/owner_raw_view.rs`
- `nepl-core/src/resource/cell_state.rs`
- `nepl-core/src/resource/place_utils.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- `ResourceOp::RawAddressView` は lowering 上、`add` / `sub` のような address-like arithmetic から生成されるが、この段階だけでは operand が raw pointer か通常の `i32` かを完全には判別できない。
- initialized checker / owner checker は従来、`RawAddressView` を無条件に `copy_explicit_raw_address_alias` へ渡していた。そのため `let l = sub l1 1` のような通常の scalar arithmetic でも raw alias group が作られ、非 pointer の値が non-owning raw-address view として扱われ得た。
- 一方で `let left_ptr = add pref left_off` のような正当な view は、`pref` が alias table だけでなく checked `alloc` / `fill_i32` 由来の raw storage/cell state によって raw address と証明される場合がある。alias table だけを証明源にすると、この正当な view も落としてしまう。
- さらに `left_ptr` のような storage-offset view を local に束縛すると、従来の `prefer_target` rekey が `pref[+?].deref` の broad initialized fact を view local 側へ移してしまい、元の base+symbolic query が `Uninit` になった。

## 問題

ResourceOp::RawAddressView is emitted for address-like add/sub syntax, but the initialized and owner checkers currently promote every view target into a raw-address alias even when the source is only an ordinary i32 value with a stable origin. This can make non-pointer arithmetic participate in raw alias state and destabilize guarded range checking.

## 影響

Static resource checks can either reject safe initialized Copy buffers through unrelated alias churn or accept ordinary scalar arithmetic as non-owning raw-address views. Both outcomes undermine memory-safety reasoning for Resource IR.

## 修正方針

Gate RawAddressView alias propagation on an existing typed raw-address proof: the exact source or a storage-offset prefix must already be tracked as a raw address. Do not use scalar value origin alone as raw-pointer evidence.

## 検証

Add a Resource IR regression where fill_i32 initializes a dynamic raw range, unrelated impure i32 reads feed scalar arithmetic, and later symbolic loads from the filled buffer remain accepted without classifying the scalar arithmetic as raw.

## 2026-05-07 修正

`RawAddressView` の伝播条件を、既存の raw-address proof に基づく判定へ変更した。

- alias table 上で exact source または storage-offset base prefix が raw address として追跡されている場合だけ、`RawAddressView` を raw alias / non-owning view として伝播する。
- initialized checker では checked API 経由の raw address も落とさないよう、canonicalized base に raw cell / owned raw storage / external raw storage が存在する場合も proof として扱う。
- owner checker では canonicalized base に owner state または storage origin が存在する場合も proof として扱う。
- scalar `ValueOrigin` は raw pointer の証明として扱わない。これにより、通常の `i32` copy / impure call result / arithmetic が raw alias state を seed しない。
- storage-offset を含む view local への束縛では raw cell/storage root を rekey しない。view は non-owning pointer expression であり、base storage fact の所有 canonical ではないため、`pref[+?].deref` の broad initialized fact を `left_ptr` に移さない。

回帰として `resource_ir_cell_check_preserves_dynamic_fill_across_impure_i32_reads` を追加した。このテストは `fill_i32 pref pref_len 0` の後、unrelated impure `next()` 由来の `i32` arithmetic を挟んでも、`load_i32 (add pref left_off/right_off)` が initialized Copy fact を参照できることを確認する。

確認結果:

- `cargo fmt -p nepl-core --check`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_dynamic_fill_across_impure_i32_reads -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_dynamic_fill_origin_across_local_reads -- --nocapture`: passed
- `cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed

## 関連

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38](./ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38.md)
