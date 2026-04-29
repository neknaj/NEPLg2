---
id: ISS-20260429T160211035Z-RESOURCE-OWNER-ASSIGN-MISSES-SELF-UP-01475305
title: "Resource owner assign misses self-update aggregate projection returns"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_flow.rs, tests/compiler/move_check.n.md, tests/stdlib/std_test_collect.n.md"
---

# ISS-20260429T160211035Z-RESOURCE-OWNER-ASSIGN-MISSES-SELF-UP-01475305: Resource owner assign misses self-update aggregate projection returns

## 概要

While removing shallow Copy from std/test reports, the focused fixture exposed that set report test_report_push report ... leaves report string projections Moved and leaks the temporary returned projections. This indicates the Resource IR owner assign path does not fully reinitialize a target when a call consumes aggregate projections from the same local and returns replacement projections.

## 対象

- `nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_flow.rs, tests/compiler/move_check.n.md, tests/stdlib/std_test_collect.n.md`

## 根拠

- `TestReport` / `TestAssertion` の浅い `Copy` を削除した状態で、`std_test_collect_continues_after_string_allocation` を `let mut report ...; set report test_report_push report ...` の形に戻すと compile fail した。
- 代表診断は `DeclareInitializer on Place { root: Local("report"), projections: [Field { index: 3 ... }] } found Moved` と、同時に `Temporary(ResourceId(...))` の returned projection leak である。
- 同じ `test_report_push` chain を pipeline temporary として書くと `tests/stdlib/std_test_collect.n.md` は 3/3 pass するため、report API の戻り値自体ではなく self-update assignment の owner transfer 順序が焦点である。

## 問題

While removing shallow Copy from std/test reports, the focused fixture exposed that set report test_report_push report ... leaves report string projections Moved and leaks the temporary returned projections. This indicates the Resource IR owner assign path does not fully reinitialize a target when a call consumes aggregate projections from the same local and returns replacement projections.

## 影響

Correct move-update style for non-Copy aggregate accumulators remains fragile. stdlib users may be forced into pipeline temporaries to avoid a checker false positive, and real self-update ownership bugs could be misclassified.

## 修正方針

Add a focused Resource IR regression for call-return projection summaries assigned back to the same aggregate local, then fix assign/summary application so returned projection owners are transferred to the target after consumed parameter projections are marked moved.

## 検証

- 修正時は、元の mutable fixture 形で `set report test_report_push report ...` が compile できることを確認する。
- `TestReport` / `TestAssertion` の `Copy` を復活させずに通すこと。
- `tests/stdlib/std_test_collect.n.md` の pipeline 版も引き続き pass すること。
