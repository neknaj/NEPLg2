---
id: ISS-20260506T211740745Z-SYMBOLIC-COPY-STORES-ERASE-UNKNOWN-O-0BD91F6C
title: "Symbolic copy stores erase unknown-offset initialized copy facts"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-07
target: "nepl-core/src/resource/cell_state.rs, nepl-core/src/resource/initialized_raw_memory.rs"
---

# ISS-20260506T211740745Z-SYMBOLIC-COPY-STORES-ERASE-UNKNOWN-O-0BD91F6C: Symbolic copy stores erase unknown-offset initialized copy facts

## 概要

RawMemoryOp::Store clears all raw cells under a symbolic address before marking the stored cell initialized. When a prior fill/helper summary has established an unknown-offset initialized Copy fact such as pref[+?].deref, a later store_i32 to pref[+i] incorrectly erases the fact even though storing an initialized Copy value preserves initializedness.

## 対象

- `nepl-core/src/resource/cell_state.rs, nepl-core/src/resource/initialized_raw_memory.rs`

## 根拠

- `RawMemoryOp::Store` は `cells.clear_raw_cells_under(&address)` を呼んでから stored cell を initialized にしていた。
- `clear_raw_cells_under` は symbolic offset と unknown offset を may-overlap として扱うため、`pref[+i]` への store が `pref[+?].deref` の initialized Copy fact を消していた。
- Copy 型の値を同じ Copy 型の cell へ store する操作は、その cell を initialized に保つ。unknown-offset fact が store 前に正しかったなら、overlap する 1 cell を initialized Copy value で上書きしても initializedness は壊れない。
- 一方で non-Copy / moved / uninit state は従来どおり保守的に消す必要があるため、単純に clear をやめる設計にはできない。

## 問題

RawMemoryOp::Store clears all raw cells under a symbolic address before marking the stored cell initialized. When a prior fill/helper summary has established an unknown-offset initialized Copy fact such as pref[+?].deref, a later store_i32 to pref[+i] incorrectly erases the fact even though storing an initialized Copy value preserves initializedness.

## 影響

Copy buffers initialized by fill_i32 or equivalent summaries become uninitialized after any symbolic store, causing false ResourceIR Cell(Uninit) diagnostics and blocking safe prefix-buffer patterns. A broad relaxation would be unsound for non-Copy cells, so the store overwrite rule must be typed.

## 修正方針

Split raw cell clearing for stores into a typed CellTable operation that preserves overlapping Initialized Copy facts when the stored value has the same Copy type, while still clearing moved/uninit/non-Copy obligations conservatively.

## 検証

Add CellTable regression for unknown-offset initialized Copy facts surviving symbolic Copy stores, run focused resource tests, nepl-core cargo check, issue checks, and source policy regressions. KP prefixsum remains a separate loop-condition/range-summary blocker if it still fails.

## 2026-05-07 対応結果

`CellTable::clear_raw_cells_overwritten_by_store` を追加し、store overwrite 専用の typed clearing を導入した。overlap する entry でも `Initialized(entry_ty)` かつ stored value と同じ Copy 型として扱える場合は保持し、non-Copy obligation や moved/uninit state は従来どおり消す。

`RawMemoryOp::Store` は汎用の `clear_raw_cells_under` ではなく、この store 専用 API を使うようにした。これにより `fill_i32` / helper summary 由来の `pref[+?].deref` initialized Copy fact が、後続の `store_i32 pref[+i]` で失われない。

回帰として `CellTable` 単体で unknown-offset initialized Copy fact が symbolic Copy store 後も別 symbolic load に流れることを確認した。

なお `nepl-core::kp::kpread_to_kpwrite_prefixsum_i32` はこの修正後も `pref[+symbolic].deref` の `Cell(Uninit)` で失敗する。残件は loop condition fact / guarded range summary 側であり、この issue とは別に継続する。
