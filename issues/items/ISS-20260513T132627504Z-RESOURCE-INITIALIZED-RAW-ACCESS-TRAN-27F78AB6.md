---
id: ISS-20260513T132627504Z-RESOURCE-INITIALIZED-RAW-ACCESS-TRAN-27F78AB6
title: "Resource initialized raw access transitions remain coupled to raw-memory dispatch"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/resource/initialized_raw_memory.rs; nepl-core/src/resource/initialized_raw_memory_access.rs; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260513T132627504Z-RESOURCE-INITIALIZED-RAW-ACCESS-TRAN-27F78AB6: Resource initialized raw access transitions remain coupled to raw-memory dispatch

## 概要

Resource IR initialized raw-memory checking still keeps Load and Store cell-state transitions inside initialized_raw_memory.rs together with the RawMemoryOp dispatcher. The file is at 299/300 policy lines, so future static-check fixes have no room except by raising the limit or mixing access semantics into the dispatch module.

## 対象

- `nepl-core/src/resource/initialized_raw_memory.rs; nepl-core/src/resource/initialized_raw_memory_access.rs; nodesrc/test_resource_checker_responsibility.js`

## 関連計画

- [静的検査の不必要な複雑化の解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行)

## 根拠

- `initialized_raw_memory.rs` は `RawMemoryOp` dispatcher と、Load / Store 固有の initialized-cell / non-Copy move / raw-address alias / initialized byte range 遷移を同じ関数に保持していた。
- 同 file は 299 行で policy 上限 300 行の直前まで増えており、次の raw-memory static-check 修正で上限引き上げか責務混在を選びやすい状態だった。
- Load / Store は raw memory safety の入口なので、dispatcher と access semantics を分離しておかないと、今後の Dealloc / Realloc / Fill / BulkCopy 変更時に access invariant を巻き込みやすい。

## 問題

Resource IR initialized raw-memory checking still keeps Load and Store cell-state transitions inside initialized_raw_memory.rs together with the RawMemoryOp dispatcher. The file is at 299/300 policy lines, so future static-check fixes have no room except by raising the limit or mixing access semantics into the dispatch module.

## 影響

Load/Store are the raw memory access point where initialized cell state, non-Copy moves, raw-address aliases, and initialized byte ranges meet. Coupling these transitions to the dispatcher increases the risk of weakening memory-safety checks during future raw-memory fixes.

## 修正方針

Move raw Load/Store access transition logic and the zero-initialized runtime-cell helper into a dedicated initialized_raw_memory_access module. Keep initialized_raw_memory.rs as the operation dispatcher and update the resource responsibility policy to require the new module with tight line limits.

## 検証

Run cargo check -p nepl-core --tests, trunk build, focused memory/move doctests, node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, and node nodesrc/issues.js check --dir issues.

## 対応内容

- `initialized_raw_memory_access.rs` を追加し、`RawMemoryOp::Load` / `RawMemoryOp::Store` の cell-state transition を専用moduleへ分離した。
- `initialized_raw_memory.rs` は `RawMemoryOp` dispatch と、alloc/dealloc/realloc/fill/bulk/default のoperation routingへ責務を戻した。
- zero-initialized runtime raw load 判定と raw-address output type 判定も access module 側へ移し、Load判定の補助責務を同じ境界にまとめた。
- Resource checker responsibility policy に新moduleの必須存在、`mod` declaration、行数上限を追加し、`initialized_raw_memory.rs` の上限を 300 行から 190 行へ下げた。

## 検証結果

- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir raw_memory -- --nocapture`: 3 passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/agent1-initialized-raw-access-move-effect.json -j 1 --dist web/dist`: total=113, passed=113
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check --dir issues`: passed

## 追加で発見した問題

- `tests/stdlib/memory_safety.n.md` の全体実行は doctest#16 / #22 の constructor boundary regression で 27/29 だった。
- この失敗は今回の Load / Store 分離範囲ではなく、direct constructor restriction の typecheck-level signal が安定していない問題として `ISS-20260513T133223124Z-MEMORY-SAFETY-CONSTRUCTOR-BOUNDARY-R-A6590141` に分離した。
