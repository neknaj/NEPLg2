---
id: ISS-20260428T115405922Z-RESOURCE-OWNER-CHECKER-LOSES-AGGREGA-DE066CEF
title: "Resource owner checker loses aggregate owner projections returned by helpers"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T115405922Z-RESOURCE-OWNER-CHECKER-LOSES-AGGREGA-DE066CEF: Resource owner checker loses aggregate owner projections returned by helpers

## 概要

ResourceOwnerCheckEngine can now move owners into aggregate projections and move those projections with aggregate values, but owner return summaries still describe only exact return places. A helper that allocates an owner, constructs a wrapper, and returns the wrapper either reports a callee-side leak or fails to attach the owner obligation to the caller output projection.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ISS-20260428T114547680Z-RESOURCE-OWNER-CHECKER-DOES-NOT-MOVE-0F2678E0` と `ISS-20260428T114942018Z-RESOURCE-OWNER-CHECKER-DOES-NOT-MOVE-B551A456` で、owner は aggregate output projection へ移り、aggregate value movement にも追従するようになった。
- しかし `OwnerReturnSummary` は exact return place だけを表現しており、return value 配下の field / payload projection にある owner を caller output projection へ渡せなかった。
- `ResourceOwnerCheckEngine::move_owner_out` も descendant owner を return move-out しないため、helper 側 leak と caller 側 obligation 欠落のどちらかが残る構造だった。
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は function boundary でも owner/free obligation を落とさないことを要求している。

## 問題

ResourceOwnerCheckEngine can now move owners into aggregate projections and move those projections with aggregate values, but owner return summaries still describe only exact return places. A helper that allocates an owner, constructs a wrapper, and returns the wrapper either reports a callee-side leak or fails to attach the owner obligation to the caller output projection.

## 影響

Owner/free-obligation checks remain incomplete at function boundaries for structs, tuples, and enum payloads. Self-host or stdlib wrapper APIs can hide allocation obligations inside aggregate return values.

## 修正方針

Extend owner return summaries with returned projection owners. Move descendant owners out on return and attach fresh or parameter-derived projection obligations to the caller output projection when direct calls or known function values return aggregates.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build; node nodesrc/issues.js check

## 2026-04-28 Stage 4 aggregate owner return summary 対応

`OwnerReturnSummary` を projection owner に拡張した。summary は exact return owner に加えて、return value 配下の owner projection suffix、型、fresh owner か parameter-derived owner かを保持する。

`ResourceOwnerCheckEngine::move_owner_out` は return value 配下の descendant owner も move-out するようにした。caller 側では direct call / known function value の owner return summary 適用時に、caller output の同じ projection へ owner obligation を付ける。

`nepl-core/tests/resource_ir.rs` に、helper が raw allocation を struct field に入れて wrapper を返し、caller が解放しない場合に caller 側 output field の leak として報告される回帰を追加した。
