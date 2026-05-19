---
id: ISS-20260519T172250950Z-RESOURCE-IR-BYTEBUILDER-REGRESSION-C-5E7800F8
title: "Resource IR ByteBuilder regression calls private raw append helper"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-19
updated: 2026-05-19
target: nepl-core/tests/resource_ir.rs
---

# ISS-20260519T172250950Z-RESOURCE-IR-BYTEBUILDER-REGRESSION-C-5E7800F8: Resource IR ByteBuilder regression calls private raw append helper

## 概要

The ByteBuilder Resource IR regression still called byte_builder_push_bytes_ref directly after the raw MemPtr append helper was made private. The test source no longer matched the typed public API that ordinary code is allowed to use.

## 対象

- `nepl-core/tests/resource_ir.rs`

## 根拠

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_keeps_byte_builder_source_ref_deallocatable -- --exact --nocapture` が、`byte_builder_push_bytes_ref` の未定義と後続の型不一致で失敗した。
- `byte_builder_push_bytes_ref` は `ISS-20260518T115751573Z-BYTEBUILDER-PUBLIC-RAW-BYTE-APPEND-D-D94BB3A0` で private helper に戻しており、ordinary source から直接呼べないこと自体は正しい。
- 旧 regression は「borrowed raw MemPtr source を copy しても caller が source owner を保持する」ことを検査していたが、現在の public contract は raw pointer/length pair ではなく `str` / checked slice / owned `ByteBuf` から readable span を導出する設計へ変わっている。

## 問題

The ByteBuilder Resource IR regression still called byte_builder_push_bytes_ref directly after the raw MemPtr append helper was made private. The test source no longer matched the typed public API that ordinary code is allowed to use.

## 影響

Focused Resource IR tests fail at typecheck time and the regression no longer proves the current ByteBuilder source-object boundary. Leaving this stale fixture would encourage re-exposing raw MemPtr plus length append instead of testing the public typed wrapper.

## 修正方針

Replace the private raw helper call with byte_builder_push_str and assert that the source str remains usable after append. This keeps the raw copy helper private while still exercising Resource IR owner behavior through monomorphized stdlib code.

## 解決内容

- test 名を `resource_ir_owner_check_keeps_byte_builder_string_source_usable` に変更し、旧 `RegionToken -> MemPtr -> byte_builder_push_bytes_ref` fixture を削除した。
- `byte_builder_push_str b0 text` の後で同じ `text` に対して `len text` を読むようにし、typed public wrapper 経由の copy が source `str` を消費または reserve しないことを Resource IR owner checker で固定した。
- raw `MemPtr` append helper の direct import を戻さず、private helper は monomorphized stdlib implementation の内部としてだけ検査される。

## 対応 stage

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6: ByteBuilder の raw pointer/length append を public API から外した後の Resource IR regression を、source object 由来の readable extent proof に合わせ直す整備。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_keeps_byte_builder_string_source_usable -- --exact --nocapture`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
