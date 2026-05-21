---
id: ISS-20260521T050436552Z-SOURCE-LEVEL-NON-COPY-COLLECTION-SLO-D0BB9BAF
title: "Source-level non-Copy collection slot lifecycle lacks raw value-flow regression"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/raw_cell_value_flow*.rs, nepl-core/src/resource/collection_slot_*proof*.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260521T050436552Z-SOURCE-LEVEL-NON-COPY-COLLECTION-SLO-D0BB9BAF: Source-level non-Copy collection slot lifecycle lacks raw value-flow regression

## 概要

Manual Resource IR tests covered non-Copy collection slot StoreValue and MoveOutLoadedCell proofs, but source-level compiler-owned stdlib fixtures did not prove that raw store/load calls lower to the same generic collection slot lifecycle proof boundary.

## 対象

- `nepl-core/src/resource/raw_cell_value_flow.rs`
- `nepl-core/src/resource/raw_cell_value_flow_alias.rs`
- `nepl-core/src/resource/raw_cell_value_flow_cell.rs`
- `nepl-core/src/resource/raw_cell_value_flow_proof.rs`
- `nepl-core/src/resource/collection_slot_owner_transfer_proof.rs`
- `nepl-core/src/resource/collection_slot_drop_proof.rs`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection payload support を stdlib module allowlist ではなく Resource IR の generic proof boundary に載せることを要求している。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、collection slot lifecycle と raw cell initialized/moved/drop state を enum / match / generic proof として扱うことを完了条件にしている。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、collection slot lifecycle producer を stdlib function name allowlist ではなく compiler core の typed proof boundary として扱う方針を明記している。

## 問題

Manual Resource IR tests covered non-Copy collection slot StoreValue and MoveOutLoadedCell proofs, but source-level compiler-owned stdlib fixtures did not prove that raw store/load calls lower to the same generic collection slot lifecycle proof boundary.

実装を確認すると、source lowering では `region_ptr` / `mem_ptr_addr` 由来の raw store/load fact と、`collection_slot_*` intrinsic の owner-cell target が、同じ storage cell を指していても `RawCellAddressAliases` と explicit zero offset (`[+0]`) を跨いで照合されていなかった。手書き Resource IR test は同じ `Place` を直接使うためこのずれを隠していた。

## 影響

Stdlib collection implementation could appear safe in manual Resource IR tests while source lowering or source capability plumbing silently stops connecting raw value-flow proof to collection slot lifecycle events. この状態で non-Copy collection API を開くと、source-level production lowering だけが `OwnerTransferRequiresValueProof` に落ち、self-host 向けの owner payload collection を進められない。

## 修正方針

`RawCellValueFlowFacts` の照合を、raw address alias と zero storage offset の正規化を含む generic raw-cell equivalence で行う。collection slot owner-transfer/drop proof は alias-aware raw value-flow lookup を使い、stdlib function 名や module 名ではなく、同じ raw cell に対する typed value-flow fact が存在するかで証明する。

source-level compiler-owned stdlib Resource IR regression を追加し、`raw store -> InitializeEmpty` と `raw load -> MoveOut` が non-Copy payload で通ることを固定する。併せて non-zero offset は同じ proof として扱わない unit test を追加し、alias 正規化が広すぎる proof にならないことを確認する。

## 検証

- `cargo test -p nepl-core --lib resource::raw_cell_value_flow_tests::raw_value_flow_alias_matching_treats_zero_offset_as_same_cell_only -- --exact --test-threads=1`
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_non_copy -- --test-threads=1`
- `cargo test -p nepl-core --lib collection_slot -- --test-threads=1`
- `cargo check -p nepl-core`
- `node nodesrc/test_resource_checker_responsibility.js`
- `cargo fmt --check`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
