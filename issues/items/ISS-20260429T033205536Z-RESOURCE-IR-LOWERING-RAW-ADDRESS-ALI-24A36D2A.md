---
id: ISS-20260429T033205536Z-RESOURCE-IR-LOWERING-RAW-ADDRESS-ALI-24A36D2A
title: "Resource IR lowering raw address alias logic is concentrated in lower.rs"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/lower.rs, nepl-core/src/resource/lower_raw_address.rs, nepl-core/src/resource/lower_raw_memory.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260429T033205536Z-RESOURCE-IR-LOWERING-RAW-ADDRESS-ALI-24A36D2A: Resource IR lowering raw address alias logic is concentrated in lower.rs

## 概要

Resource IR lowering keeps raw memory operation classification, MemPtr/RegionToken wrapper aliasing, raw address helper return summaries, literal offset arithmetic, and lowering traversal in one lower.rs file. This makes the Stage 4 MemPtr = non-owning pointer model difficult to audit and encourages adding more call-name summaries to the main lowering pass.

## 対象

- `nepl-core/src/resource/lower.rs, nepl-core/src/resource/lower_raw_address.rs, nepl-core/src/resource/lower_raw_memory.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `nepl-core/src/resource/lower.rs` は 1740 行規模になり、HIR traversal、raw memory operation classification、`MemPtr` / `RegionToken` wrapper aliasing、helper return raw address summary、literal offset arithmetic、raw aggregate field projection を同じ file に持っていた。
- Stage 4 の `MemPtr = non-owning pointer` 方針では、raw address alias は free obligation owner ではなく pointer projection として扱う必要があるため、この意味論を general lowering traversal から分離して監査可能にする必要があった。
- 既存の `nodesrc/test_resource_checker_responsibility.js` は Resource checker / coverage 分割は守っていたが、Resource IR lowering 側の raw address alias logic が main lowering file に戻る退行は検出していなかった。

## 問題

Resource IR lowering keeps raw memory operation classification, MemPtr/RegionToken wrapper aliasing, raw address helper return summaries, literal offset arithmetic, and lowering traversal in one lower.rs file. This makes the Stage 4 MemPtr = non-owning pointer model difficult to audit and encourages adding more call-name summaries to the main lowering pass.

## 影響

Stage 4/5 owner, raw cell, and effect gates trust Resource IR lowering as their input. If raw address alias semantics remain embedded in the general HIR lowering traversal, future MemPtr/RegionToken fixes can accidentally change lowering coverage or cell state inputs without a focused module boundary or source policy guard.

## 修正方針

Split raw address alias semantics and raw memory operation classification out of lower.rs into dedicated resource lowering modules. Keep lower.rs responsible for traversal and ResourceOp construction orchestration, and add source policy checks so raw address alias lowering does not grow back into the main file.

## 検証

Run rustfmt on resource lowering modules, node nodesrc/test_resource_checker_responsibility.js, cargo test -p nepl-core --test resource_ir coverage -- --nocapture, cargo check -p nepl-core --tests, trunk build, focused move_effect doctests, node nodesrc/issues.js check, and git diff --check.

- `rustfmt --check nepl-core\src\resource\lower.rs nepl-core\src\resource\lower_raw_address.rs nepl-core\src\resource\lower_raw_memory.rs nepl-core\src\resource\mod.rs`: pass
- `node nodesrc\test_resource_checker_responsibility.js`: pass
- `cargo test -p nepl-core --test resource_ir coverage -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test resource_ir raw -- --nocapture`: 36 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\agent1-resource-lower-raw-address-split-move-effect.json -j 1`: total=110, passed=110, failed=0
- `node nodesrc\issues.js check`: pass
- `git diff --check`: pass

## 対応結果

`lower.rs` から raw memory operation classification を `lower_raw_memory.rs`、`MemPtr` / `RegionToken` wrapper aliasing と raw address helper return summary を `lower_raw_address.rs` へ分離した。`lower.rs` は HIR traversal、scope 管理、ResourceOp construction orchestration に集中し、raw address alias の意味論は dedicated module で扱う。

分離後の行数は `lower.rs` 1044 行、`lower_raw_address.rs` 626 行、`lower_raw_memory.rs` 45 行である。`nodesrc/test_resource_checker_responsibility.js` には lowering module の存在確認、`lower.rs` から `RawAddressSource` が再導入されないこと、各 file の line limit を追加した。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
