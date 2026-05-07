---
id: ISS-20260507T014059777Z-RESOURCE-RAWADDRESSVIEW-GATE-DROPS-Z-FED82745
title: "Resource RawAddressView gate drops zero-offset first-store origins"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_alias_origin.rs, nepl-core/src/resource/initialized.rs, nepl-core/tests/resource_ir.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260507T014059777Z-RESOURCE-RAWADDRESSVIEW-GATE-DROPS-Z-FED82745: Resource RawAddressView gate drops zero-offset first-store origins

## 概要

After RawAddressView propagation was gated to avoid ordinary i32 arithmetic becoming raw pointers, unproven raw address views also lost their stable value origin. A helper such as slot_ptr(base, 0) used before the first raw store was stored under a temporary address, so a later load from base was reported as resource.cell.uninit.

## 対象

- `nepl-core/src/resource/initialized_alias_origin.rs, nepl-core/src/resource/initialized.rs, nepl-core/tests/resource_ir.rs, tests/compiler/move_effect.n.md`

## 根拠

- `tests/compiler/move_effect.n.md` の focused run で `doctest#30` が positive test なのに `resource.cell.uninit` で拒否された。
- 該当 fixture は `slot_ptr<LocalToken,i32> p 0` に `store<LocalToken>` した後、同じ `p` から `load<LocalToken>` する。`slot_ptr(..., 0)` は Resource IR 上では exact zero-offset raw address view なので、store cell は後続 load と同じ raw cell に正規化されるべきである。
- `ISS-20260506T215615927Z-RESOURCE-RAWADDRESSVIEW-TREATS-ORDIN-B3C620DA` の修正で `RawAddressView` を raw alias group に昇格する条件を正しく絞ったが、未証明 view の stable origin まで `clear(target)` で消していたため、first store が temporary address 配下に孤立した。

## 問題

After RawAddressView propagation was gated to avoid ordinary i32 arithmetic becoming raw pointers, unproven raw address views also lost their stable value origin. A helper such as slot_ptr(base, 0) used before the first raw store was stored under a temporary address, so a later load from base was reported as resource.cell.uninit.

## 影響

Resource IR rejects valid raw-memory code that computes an exact zero-offset address before the first store. This makes move_effect doctest#30 fail and pressures tests toward weakening cell diagnostics instead of preserving precise raw address origin facts.

## 修正方針

Keep the RawAddressView gate for alias groups, but record non-owning view value origins separately when the source is not yet proven raw. Raw memory operations can then canonicalize the temporary back to the stable base/offset once the value is actually used as an address, without seeding ordinary scalar arithmetic as raw alias state.

## 検証

- `cargo fmt --check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_untracked_literal_helper_zero_offset_for_first_store -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_dynamic_fill_across_impure_i32_reads -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `trunk build`: passed
- `NEPL_TEST_CASE_TIMEOUT_MS=120000 node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree --dist web/dist -o tmp/move_effect_agent1_after_raw_view_origin.json -j 1 --assert-io`: total=110, passed=101, failed=9。今回対象の `doctest#28/#29/#30` は解消。残りは `field::get` fixture の import 形式が現在の qualified import 規則に合っていない別件。

## 対応結果

`RawAddressView` の gate は維持したまま、source がまだ raw address と証明されていない場合に target の raw alias group は作らず、stable value origin だけを `RawValueOrigins` に保存するようにした。これにより通常の i32 演算は raw pointer として昇格しない一方、raw memory operation が view target を実際に address として使った時は、temporary を base/offset に正規化して同じ Resource IR cell state を参照できる。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
