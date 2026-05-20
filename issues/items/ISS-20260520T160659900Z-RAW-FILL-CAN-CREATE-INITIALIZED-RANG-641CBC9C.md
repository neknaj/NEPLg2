---
id: ISS-20260520T160659900Z-RAW-FILL-CAN-CREATE-INITIALIZED-RANG-641CBC9C
title: "Raw fill can create initialized ranges for non-Copy values in Resource IR"
area: compiler
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "nepl-core/src/resource/initialized_raw_fill.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260520T160659900Z-RAW-FILL-CAN-CREATE-INITIALIZED-RANG-641CBC9C: Raw fill can create initialized ranges for non-Copy values in Resource IR

## 概要

Resource IR initialized-state checking lets RawMemoryOp::Fill mark an initialized element range using the fill value type without proving that the value is Copy. A non-Copy fill value would model repeated initialized owner cells, which is an unsound proof shape for future non-Copy collection slot initialization.

## 対象

- `nepl-core/src/resource/initialized_raw_fill.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `RawMemoryOp::Fill` は同じ value を複数 slot へ書く操作として Resource IR に現れる。
- `CellTable::mark_initialized_raw_byte_range` は範囲内の任意 element load を initialized とみなすため、non-Copy value type をそのまま range evidence にすると owner payload が複数 slot に複製されたことになる。
- non-Copy collection payload support では、slot ごとの move / drop state を正確に追う必要がある。raw fill は Copy payload 初期化の proof に限定し、non-Copy payload の初期化は個別 store / move / drop proof へ接続する必要がある。

## 問題

Resource IR initialized-state checking lets RawMemoryOp::Fill mark an initialized element range using the fill value type without proving that the value is Copy. A non-Copy fill value would model repeated initialized owner cells, which is an unsound proof shape for future non-Copy collection slot initialization.

## 影響

Non-Copy collection payload support depends on raw storage cells representing moves and drops exactly. If raw fill can manufacture initialized non-Copy ranges, collection grow/clear/drop work could accidentally rely on shallow duplication rather than per-cell move/drop proof.

## 修正方針

Gate element-range initialization from RawMemoryOp::Fill on TypeCtx::is_copy(value.ty). Non-Copy fill must not create initialized raw cell range evidence; subsequent load/dealloc checks should report uninitialized or live-cell diagnostics instead of accepting duplicated owner cells.

## 検証

Add a Resource IR regression where Fill uses a non-Copy value type and a later load of that type is rejected, while existing Copy/u8 raw range tests continue to pass.

## 2026-05-20 Agent 1 修正

`ResourceCheckEngine::check_raw_memory_fill_words` が `RawMemoryOp::Fill` から element range initialization evidence を作る条件に `TypeCtx::is_copy(value.ty)` を追加した。

これにより `fill<T>` 由来の `InitializedRawRangeUnit::Elements` は Copy value の範囲初期化だけを表す。non-Copy value を使った raw fill は initialized range evidence を作らないため、後続の `load<T>` は raw cell が未初期化であることを診断する。

この修正は stdlib の個別関数名に依存しない。Resource IR の範囲初期化 proof 自体を Copy payload に限定することで、将来の non-Copy collection slot lifecycle が shallow duplication に依存できないようにする。

関連設計:

- [NEPLg2 静的検査の複雑化解消計画](https://github.com/neknaj/NEPLg2/blob/main/doc/neplg2/static_check_complexity_reduction_plan.md)
- [Non-Copy collection payload support issue](https://github.com/neknaj/NEPLg2/blob/main/issues/items/ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md)

focused verification:

- `cargo test -p nepl-core --test resource_ir raw_fill -- --test-threads=1`: 3/3 passed
