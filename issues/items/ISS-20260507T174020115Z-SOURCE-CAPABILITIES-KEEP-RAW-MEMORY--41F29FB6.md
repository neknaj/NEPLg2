---
id: ISS-20260507T174020115Z-SOURCE-CAPABILITIES-KEEP-RAW-MEMORY--41F29FB6
title: "Source capabilities keep raw memory boundary as ad hoc boolean"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/source_map.rs, nepl-core/src/loader.rs, nepl-core/tests/effects.rs"
---

# ISS-20260507T174020115Z-SOURCE-CAPABILITIES-KEEP-RAW-MEMORY--41F29FB6: Source capabilities keep raw memory boundary as ad hoc boolean

## 概要

SourceCapabilities stores the compiler-owned raw-memory-boundary privilege as a dedicated boolean. As Stage 5/6 adds more compiler capabilities, boolean fields make privilege classification ad hoc instead of enum-first.

## 対象

- `nepl-core/src/source_map.rs, nepl-core/src/loader.rs, nepl-core/tests/effects.rs`

## 根拠

- `SourceCapabilities` が `raw_memory_boundary: bool` を直接保持していた。
- Stage 5/6 では raw memory boundary 以外にも compiler-owned capability が増える可能性がある。
- boolean field を増やす設計では capability の分類が構造化されず、静的検査方針の enum-first / match-first に沿わない。

## 問題

SourceCapabilities stores the compiler-owned raw-memory-boundary privilege as a dedicated boolean. As Stage 5/6 adds more compiler capabilities, boolean fields make privilege classification ad hoc instead of enum-first.

## 影響

Raw memory boundary checks can grow by adding flags rather than typed capability variants, weakening exhaustiveness and making static-check privilege audits harder.

## 修正方針

Introduce a SourceCapability enum and store capabilities as an enum-keyed set. Keep raw-memory-boundary behavior unchanged while making future source privileges explicit typed variants.

## 検証

Run cargo fmt/check focused source capability and loader tests, trunk build if Rust output changes, focused memory_safety doctests, node nodesrc/issues.js check, and source policy regressions.

## 対応結果

`SourceCapability` enum を追加し、`SourceCapabilities` は enum-keyed set として capability を保持するようにした。既存の `raw_memory_boundary_allowed` / `allows_raw_memory_boundary` API と loader の exact path whitelist の意味は変えていない。

これにより、raw-memory-boundary は単なる boolean field ではなく compiler-owned privilege の enum variant になった。今後 capability を追加する場合も、型上の variant として追加し、source privilege audit を文字列や個別 boolean の増殖へ戻さない。

## 検証結果

- `cargo test -p nepl-core --lib source_map::tests -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test effects raw_memory_boundary -- --nocapture`: 4 passed
- `cargo fmt --check -p nepl-core`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-source-capability-memory-safety.json -j 1 --dist web/dist`: 19 passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `git diff --check`: passed
