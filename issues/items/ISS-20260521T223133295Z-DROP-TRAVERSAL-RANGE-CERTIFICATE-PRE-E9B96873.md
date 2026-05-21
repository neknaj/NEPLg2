---
id: ISS-20260521T223133295Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-PRE-E9B96873
title: "Drop traversal range certificate preserve check ignores anchor consumption"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-22
target: "nepl-core/src/resource/collection_slot_summary_build_range_preserve.rs, nepl-core/src/resource/collection_slot_summary_build_range_certificate.rs, nepl-core/src/resource/collection_slot_summary_build_range_certificate_tests.rs"
---

# ISS-20260521T223133295Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-PRE-E9B96873: Drop traversal range certificate preserve check ignores anchor consumption

## 概要

ForallInitializedRange certificate construction treats loop bodies as preserving storage/count when protected anchors are consumed through Assign/DeclareLocal/Construct/Call/EndScope paths without being written directly.

## 対象

- `nepl-core/src/resource/collection_slot_summary_build_range_preserve.rs, nepl-core/src/resource/collection_slot_summary_build_range_certificate.rs, nepl-core/src/resource/collection_slot_summary_build_range_certificate_tests.rs`

## 根拠

- Parent issue: [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md)
- 設計段階: [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-resource-ir-と-stdlib-境界の完成)
- 直前の関連修正: [ISS-20260521T222108027Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-PRE-A3B78241](./ISS-20260521T222108027Z-DROP-TRAVERSAL-RANGE-CERTIFICATE-PRE-A3B78241.md)
- 監査観点: full-range drop traversal certificate は stdlib helper 名や module allowlist ではなく、Resource IR の loop induction、range bound、actual drop witness、protected anchor preservation による generic proof として扱う必要がある。

## 問題

ForallInitializedRange certificate construction treats loop bodies as preserving storage/count when protected anchors are consumed through Assign/DeclareLocal/Construct/Call/EndScope paths without being written directly.

## 影響

A callee summary can claim full initialized-range drop traversal after a loop body consumed the owner/count anchor, weakening the generic Resource IR proof boundary for non-Copy collection cleanup.

## 修正方針

Make body preservation use typed consume semantics consistently with ResourceOp execution and reject writes or non-Copy consumption of protected anchors before accepting a range certificate.

## 対応

- `body_preserves_place` を `ResourceCheckEngine` と `RawCellAddressAliases` を受け取る逐次判定に変更し、operation ごとに alias propagation を反映しながら protected anchor への touch / write / non-Copy consumption を判定するようにした。
- `Assign` の value、`DeclareLocal` initializer、`Construct` inputs、`EndScope` result、`Call` args を Resource IR の消費 semantics として preservation 条件に含めた。
- pure user call は protected anchor またはその raw/scalar alias を引数に取る場合、callee 側 preservation summary なしの generic 証明として扱わない。これは個別 stdlib module の allowlist ではなく、Resource IR op と alias state に基づく conservative な proof boundary である。
- loop certificate 生成は condition facts 適用後の alias state を preservation 判定に渡し、step 後 tail index preservation も step までの alias propagation を反映して判定するようにした。

## 検証

Add regression tests for Assign/Construct/Call consumption paths and run focused nepl-core collection_slot_summary_loop_induction tests plus cargo check.

- `cargo test -p nepl-core --lib collection_slot_summary_loop_induction -- --nocapture`
- `cargo test -p nepl-core --lib body_preserve -- --nocapture`
- `cargo test -p nepl-core --lib collection_slot_summary_loop_certificate -- --nocapture`
