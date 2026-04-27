---
id: ISS-20260427T164419173Z-MEMORY-LAYOUT-RULES-ARE-DUPLICATED-A-FDB20787
title: "memory layout rules are duplicated across compiler passes"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/typecheck.rs, nepl-core/src/passes/move_check.rs, nepl-core/src/passes/drop_insertion.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs"
---

# ISS-20260427T164419173Z-MEMORY-LAYOUT-RULES-ARE-DUPLICATED-A-FDB20787: memory layout rules are duplicated across compiler passes

## 概要

storage size、field offset、aggregate layout の規則が typecheck、move_check、drop_insertion、wasm codegen、llvm codegen に重複実装されている。compiler が memory safety を保証するには、検査と codegen が同じ layout plan を参照する必要がある。

## 対象

- `nepl-core/src/typecheck.rs, nepl-core/src/passes/move_check.rs, nepl-core/src/passes/drop_insertion.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs`

## 根拠

- `nepl-core/src/typecheck.rs:9018` 以降に `type_storage_size_bytes` / generic mapping 付き layout 計算がある。
- `nepl-core/src/passes/move_check.rs:956` 以降にも raw place overlap 判定用の storage size / offset 計算がある。
- `nepl-core/src/passes/drop_insertion.rs:583` 以降にも drop 対象 field の offset 計算がある。
- `nepl-core/src/codegen_wasm.rs:192` / `369` と `nepl-core/src/codegen_llvm.rs:3710` / `3863` に backend 別の storage size / aggregate field layout がある。
- `ISS-20260425T000000Z-RV-CORE-018-CF97C6F2` では generic named struct の layout 解決差分により nested aggregate の 2 番目以降が壊れていた。
- `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04` の raw aggregate field read 対応でも、typecheck と move_check の field offset/byte range 解釈を揃える必要があった。

## 問題

現状の責務分割では、`core/mem.nepl` は `size_of<T>` / `align_of<T>` を compiler intrinsic として利用し、compiler 側は複数 pass が独自に同じ layout を再計算している。これでは「どの byte range がどの field / owner か」という memory safety の根拠が pass ごとにずれる。`mem.nepl` ではなく compiler が layout の単一責任者になるべきで、pass は共有された layout plan だけを参照する必要がある。

## 影響

layout が 1 箇所でもずれると、move checker がある byte range を moved と判断しても codegen は別 byte range を読み書きする。これは false positive / false negative だけでなく、nested aggregate、enum payload、generic collection storage の値化けや drop 対象の取り違えにつながる。

## 修正方針

`nepl-core/src/layout.rs` のような共有 module を導入し、`StorageLayout` / `FieldLayout` / `EnumLayout` / `GenericSubstitution` を一元化する。typecheck、move_check、drop_insertion、WASM/LLVM codegen はこの module の query 結果だけを使い、各 pass 内の local layout helper を削除する。`core/mem.nepl` の `size_of` / `align_of` intrinsic も同じ layout engine から値を得る。

## 検証

struct、tuple、enum payload、generic `Apply`、nested aggregate、raw aggregate field access の layout snapshot を追加する。WASM/LLVM の codegen 出力と Resource IR/move_check の byte range が一致することを確認し、`type_storage_size_bytes` 系 helper が pass ごとに再導入されない source policy も追加する。
