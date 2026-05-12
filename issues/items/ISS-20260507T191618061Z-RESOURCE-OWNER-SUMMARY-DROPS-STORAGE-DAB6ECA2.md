---
id: ISS-20260507T191618061Z-RESOURCE-OWNER-SUMMARY-DROPS-STORAGE-DAB6ECA2
title: "Resource owner summary drops StorageOrigin on returned RegionToken"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-12
target: "nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/summary.rs, nepl-core/src/resource/owner_return_apply.rs, nepl-core/src/resource/storage_origin.rs, nepl-core/src/resource/owner_drop.rs, nepl-core/src/resource/owner_consumption.rs, nepl-core/src/resource/owner_raw_view.rs, nepl-core/src/resource/owner_raw_view_model.rs, nepl-core/src/resource/owner_summary_consumed.rs, nepl-core/src/resource/owner_summary_i32_leaf.rs, nepl-core/src/resource/owner_summary_parameters.rs, nepl-core/src/resource/owner_summary_raw_alias.rs, nepl-core/src/resource/owner_summary_raw_use.rs, nepl-core/src/resource/owner_summary_raw_view_return.rs, nepl-core/src/resource/owner_summary_storage_origin.rs, nepl-core/src/resource/owner_summary_variant_projection.rs, nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_summary_variant_return.rs, nepl-core/src/resource/owner_summary_variant_conditions.rs, nepl-core/src/resource/owner_check.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md, nodesrc/test_resource_checker_responsibility.js, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260507T191618061Z-RESOURCE-OWNER-SUMMARY-DROPS-STORAGE-DAB6ECA2: Resource owner summary drops StorageOrigin on returned RegionToken

## 概要

A function can return a RegionToken produced by region_new from a fixed raw MemPtr. The callee Resource IR contains StorageOrigin::Owned on token.ptr.raw, but the owner return summary does not carry that origin to the caller, so dealloc_region on the returned token is not rejected.

## 対象

- `nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/summary.rs, nepl-core/src/resource/owner_return_apply.rs, nepl-core/src/resource/storage_origin.rs, nepl-core/src/resource/owner_drop.rs, nepl-core/src/resource/owner_consumption.rs, nepl-core/src/resource/owner_raw_view.rs, nepl-core/src/resource/owner_raw_view_model.rs, nepl-core/src/resource/owner_summary_consumed.rs, nepl-core/src/resource/owner_summary_i32_leaf.rs, nepl-core/src/resource/owner_summary_parameters.rs, nepl-core/src/resource/owner_summary_raw_alias.rs, nepl-core/src/resource/owner_summary_raw_use.rs, nepl-core/src/resource/owner_summary_raw_view_return.rs, nepl-core/src/resource/owner_summary_storage_origin.rs, nepl-core/src/resource/owner_summary_variant_projection.rs, nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_summary_variant_return.rs, nepl-core/src/resource/owner_summary_variant_conditions.rs, nepl-core/src/resource/owner_check.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md, nodesrc/test_resource_checker_responsibility.js, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `region_new` が helper 内で返した `RegionToken.ptr.raw` の `StorageOrigin::Owned` は callee 内 Resource IR には存在したが、`OwnerReturnSummary` が returned value 配下の storage origin を表現していなかった。
- 調査中に、`alloc_ptr` / `alloc_region` の `Result::Ok` payload に入る fresh owner も `variant_projection_returns` が parameter source しか表現できず、fresh owner と maybe owner を variant payload return として caller 側へ伝播できないことを確認した。
- `mem_ptr_wrap` / `region_new` の raw-address alias は wrapper 内の raw field へ owner を移す境界でもあるが、alias table だけでは wrapper field への actual owner state が作られず、後段 summary が owner と storage origin を区別できなかった。

## 問題

A function can return a RegionToken produced by region_new from a fixed raw MemPtr. The callee Resource IR contains StorageOrigin::Owned on token.ptr.raw, but the owner return summary does not carry that origin to the caller, so dealloc_region on the returned token is not rejected.

## 影響

Owned storage provenance required for RegionToken can disappear at a function return boundary. This weakens the MemPtr non-owning pointer / RegionToken owner-token separation and allows forged owner-token-shaped values to cross helper boundaries.

## 修正方針

Represent returned storage origin obligations in the owner return summary and apply them at direct and indirect call return sites before owner consumption checks.

## 対応

- `OwnerReturnSummary` に `storage_origin_markers` を追加し、returned value 配下に残る `StorageOrigin::{Owned,Unmanaged,Internal}` を call output の対応 projection へ復元するようにした。
- `StorageOriginTable::entries_under` を追加し、return summary 作成時に exact place だけでなく returned aggregate 配下の storage origin を列挙できるようにした。
- `OwnerVariantProjectionReturn` を `OwnerProjectionReturnOwner::{Parameter,Fresh,Maybe}` で表す enum 設計に変更した。これにより `Result::Ok` payload に入った fresh owner / maybe owner も match arm 選択時または caller への再返却時に網羅的な `match` で伝播される。
- `RawAddressAlias` が raw owner をより深い wrapper projection へ入れる場合のみ、既存 transferable owner を wrapper field へ移動する。`mem_ptr_addr` のように wrapper から scalar raw address を読むだけの alias では owner を移さない。
- `StorageOriginTable` に copy origin の source place を保持させ、`read local -> tmp` のような by-value projection return で owner state を複製せずに「戻り値が元 owner を保存している」ことを summary / EndScope で判定できるようにした。
- `EndScope` の自動 drop は、戻り値配下の origin source が local owner と重なる場合、その local owner を drop 対象から外す。これにより `test_report_print_stdout` や aggregate identity helper が、戻り値へ owner を返しているにもかかわらず元 local を自動 drop してしまう問題を解消した。
- `BorrowKind::Shared` は owner / storage origin alias を作らず、non-owning raw view だけを伝播する。共有参照から `str` field を読む処理が元 `TestReport` の string owner を消費扱いにしないようにした。
- raw `i32` owner seed は「raw owner 消費」または「aggregate 内 raw i32 leaf の返却」に限定し、裸の `i32 -> i32` identity を owner transfer と誤認しないようにした。
- variant condition tracking は owner seed と分離し、通常の `i32` parameter / leaf も condition source として要約できるようにした。これにより `dealloc(ptr,size)` の `ptr<=0 || size<0` のような条件を、`size` を owner と誤認せず caller 側で利用できる。
- remote main 取り込み後に responsibility gate が再発したため、上限を緩めず、non-owning initializer 判定を `owner_raw_view.rs`、reserved call argument 判定を `owner_consumption.rs`、EndScope drop を `owner_drop.rs`、raw view ownership enum を `owner_raw_view_model.rs` に分けた。
- さらに returned summary 本体へ集まっていた consumed parameter 判定、parameter seed、raw i32 leaf、raw owner alias walk、non-owning raw view return、storage origin marker、variant projection source 整理を dedicated module へ分離し、`nodesrc/test_resource_checker_responsibility.js` に追加 module の存在と line count を固定した。
- `alloc_region` 由来の正当な `RegionToken` は返却後も caller で `dealloc_region` 可能であること、fixed raw `MemPtr` 由来の forged `RegionToken` は helper return を跨いでも `resource.owner.no_free_obligation` で拒否することを対の回帰テストで固定した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reinitializes_self_update -- --nocapture`: 5 passed
- `cargo test -p nepl-core --test resource_ir owner_return -- --nocapture`: 8 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_result_ok_raw_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir region_token_forged -- --nocapture`: 6 passed
- `cargo test -p nepl-core --test resource_ir borrowed_region_ptr -- --nocapture`: 4 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_dealloc_through_result_wrapped_str_addr_view -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir alloc_ptr -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir variant_owner -- --nocapture`: passed
- `cargo fmt --check -p nepl-core`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-return-storage-origin-memory-safety.json -j 1 --dist web/dist`: 23 passed

## 関連

- 親 issue: [MemPtr and RegionToken lack compiler owned provenance model](./ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF.md)
- 設計計画: [静的検査の不必要な複雑化の解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
