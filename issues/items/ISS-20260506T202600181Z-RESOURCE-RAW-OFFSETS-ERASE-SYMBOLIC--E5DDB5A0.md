---
id: ISS-20260506T202600181Z-RESOURCE-RAW-OFFSETS-ERASE-SYMBOLIC--E5DDB5A0
title: "Resource raw offsets erase symbolic dynamic index identity"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-07
target: "nepl-core/src/resource/model.rs, nepl-core/src/resource/lower_raw_address*.rs, nepl-core/src/resource/cell_state.rs, nepl-core/tests/resource_ir.rs"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行"
---

# ISS-20260506T202600181Z-RESOURCE-RAW-OFFSETS-ERASE-SYMBOLIC--E5DDB5A0: Resource raw offsets erase symbolic dynamic index identity

## 概要

RawAddressOffset::Unknown and ResourceOffset { bytes: None } collapse every dynamic offset into the same wildcard. Relational guard facts such as i < len cannot be tied to the concrete base + i raw address, so Resource IR cannot later prove guarded initialized ranges without over-approximating memory safety.

## 対象

- `nepl-core/src/resource/model.rs, nepl-core/src/resource/lower_raw_address*.rs, nepl-core/src/resource/cell_state.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ResourceOffset` が `bytes: Option<usize>` だけを持っていたため、`None` が「unsupported arithmetic」「任意の動的 index」「unknown wildcard」を同時に表していた。
- `RawAddressOffset` も `Known(i64)` / `Unknown` だけだったため、`mem_ptr_add ptr idx` や `add raw off` の `idx` / `off` が Resource IR 上で区別不能になっていた。
- 直前の relation fact 対応で `i < len` は `ResourceConditionFact::I32Relation` として残るようになったが、raw address 側が `base[+?]` に潰れると guard と offset を結び付けられない。

## 問題

RawAddressOffset::Unknown and ResourceOffset { bytes: None } collapse every dynamic offset into the same wildcard. Relational guard facts such as i < len cannot be tied to the concrete base + i raw address, so Resource IR cannot later prove guarded initialized ranges without over-approximating memory safety.

## 影響

Length-guarded initialized range summaries cannot be implemented precisely. Different dynamic indices alias as the same unknown offset, which forces either false positives or unsafe broad acceptance.

## 修正方針

Replace byte-option storage offsets with a typed ResourceOffset enum and preserve simple symbolic offset places through raw address lowering. Keep a distinct Unknown fallback for unsupported arithmetic so exhaustive matches distinguish known, symbolic, and unknown offsets.

## 検証

Add Resource IR dump regression that mem_ptr_add with a dynamic index keeps a symbolic offset instead of [+?], and run focused resource tests plus issue/source-policy checks.

## 2026-05-07 対応結果

`ResourceOffset` を `Known(usize)` / `Symbolic { place }` / `Unknown` の enum に変更した。これにより、byte offset の種類は Rust の `match` で網羅的に扱われ、`Option<usize>` の `None` に複数の意味を詰め込む設計は残さない。

`RawAddressOffset` も `Known(i64)` / `Symbolic { place }` / `Unknown` に変更し、`mem_ptr_add ptr idx`、`add raw off`、transparent raw-address return projection の simple dynamic offset を symbolic place として保持するようにした。unsupported な複合 arithmetic や負 offset は引き続き `Unknown` に落とすが、これは Symbolic とは別 variant なので後続の range summary が「保持された identity」と「安全側 fallback」を区別できる。

安全性のため、一般の raw address overlap 判定では `Symbolic` と `Unknown` を保守的に may-overlap として扱う。今回の変更は dynamic offset を無条件に initialized とみなすものではなく、将来の typed range summary が relation fact と照合できる情報を Resource IR に残すための前提整備である。

回帰として `resource_ir_lowering_preserves_symbolic_mem_ptr_add_offset` を追加し、`mem_ptr_add` の dynamic offset が `ResourceOffset::Symbolic` として lowering されることを enum 構造で確認した。

残る親 issue の本体は、`ResourceConditionFact::I32Relation` と `ResourceOffset::Symbolic` を用いて、`i < len` が証明された場合だけ `base + i` の initialized range を許可する summary model を実装することである。
