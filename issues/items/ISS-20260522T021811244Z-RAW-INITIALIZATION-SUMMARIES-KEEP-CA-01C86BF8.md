---
id: ISS-20260522T021811244Z-RAW-INITIALIZATION-SUMMARIES-KEEP-CA-01C86BF8
title: "Raw initialization summaries keep callee-local symbolic offsets"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/resource/initialized_summary*.rs"
---

# ISS-20260522T021811244Z-RAW-INITIALIZATION-SUMMARIES-KEEP-CA-01C86BF8: Raw initialization summaries keep callee-local symbolic offsets

## 概要

Raw cell initialization summaries store parameter-cell suffixes as plain PlaceProjection values, so symbolic offsets that refer to callee parameters can survive summary application unchanged instead of being instantiated with caller arguments.

## 対象

- `nepl-core/src/resource/initialized_summary*.rs`

## 根拠

- `CollectionSlotLifecycleSummaryPlace` は `ResourceOffset::Symbolic` / `ScaledSymbolic` operand を parameter-relative な typed summary projection として保持していたが、raw initialization summary の `param_cells` / `param_byte_ranges` / variant param facts は plain `PlaceProjection` のままだった。
- そのため callee 内の `byte_off` parameter を使って raw cell / raw byte range を summary 化すると、caller replay 時に caller actual へ置換されず、callee-local symbolic place を含む raw cell state が残り得た。
- `MemPtr<T>` と `RegionToken<T>` の alias を型だけで同一視する修正は unsafe であるため、alias precondition とは独立に、まず summary suffix 自体を source parameter に相対化できる場合だけ発行する必要がある。

## 問題

Raw cell initialization summaries store parameter-cell suffixes as plain PlaceProjection values, so symbolic offsets that refer to callee parameters can survive summary application unchanged instead of being instantiated with caller arguments.

## 影響

Caller-side Resource IR can retain callee-local symbolic offset cells, causing false CellUnavailable reports and risking proof checks that reason about the wrong symbolic address. This blocks sound non-Copy collection helpers that use byte-offset parameters.

## 修正方針

Use the shared typed summary projection model for raw initialization parameter cells, byte ranges, and variant parameter facts so symbolic offsets are represented as parameterized places and instantiated at every callsite.

## 検証

Add focused unit coverage for caller argument substitution in raw initialization parameter summaries and run the affected nepl-core tests plus issue validation.

## 対応内容

- `summary_projection` を Resource IR 共通の typed summary projection として切り出し、collection slot summary と raw initialization summary が同じ enum / exhaustive match による parameterized suffix instantiate を使うようにした。
- raw initialization summary の `param_cells`、`param_byte_ranges`、variant param cell / byte range / requirement / condition を plain `PlaceProjection` から `SummaryProjection` に移行した。
- summary build は suffix 内の symbolic operand が function parameter に相対化できる場合だけ summary fact を発行し、caller apply / variant pending state は caller actual に instantiate した concrete projection だけを Resource IR state に入れる。
- `RawCellInitializationParamCell` と `RawCellInitializationParamByteRange` の focused unit test を追加し、callee summary の scaled symbolic offset が caller argument に置換されることを固定した。
- `summary_projection.rs` と focused test module を `nodesrc/test_resource_checker_responsibility.js` の監視対象に追加した。監視はこの追加分を通過した後、既存の `collection_slot_state_release_alias.rs` 130/120 行超過で停止するため、別 issue [ISS-20260522T023831896Z-COLLECTION-SLOT-RELEASE-ALIAS-MODULE-0B5B1690](./ISS-20260522T023831896Z-COLLECTION-SLOT-RELEASE-ALIAS-MODULE-0B5B1690.md) として分離した。

## 検証結果

- `cargo test -p nepl-core resource::initialized_summary_apply_param_tests -- --test-threads=1`: passed
- `cargo test -p nepl-core resource::collection_slot_summary_target_tests -- --test-threads=1`: passed
- `cargo test -p nepl-core --test collection_slot_full_range public_owner_collection_api_uses_private_lifecycle_helpers -- --test-threads=1 --exact`: passed
- `cargo check -p nepl-core`: passed
- `cargo fmt -p nepl-core --check`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed with CRLF normalization warnings only
