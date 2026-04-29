---
id: ISS-20260429T160211035Z-RESOURCE-OWNER-ASSIGN-MISSES-SELF-UP-01475305
title: "Resource owner assign misses self-update aggregate projection returns"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/owner_alias.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/std_test_collect.n.md"
---

# ISS-20260429T160211035Z-RESOURCE-OWNER-ASSIGN-MISSES-SELF-UP-01475305: Resource owner assign misses self-update aggregate projection returns

## 概要

While removing shallow Copy from std/test reports, the focused fixture exposed that set report test_report_push report ... leaves report string projections Moved and leaks the temporary returned projections. This indicates the Resource IR owner assign path does not fully reinitialize a target when a call consumes aggregate projections from the same local and returns replacement projections.

## 対象

- `nepl-core/src/resource/owner_alias.rs`
- `nepl-core/tests/resource_ir.rs`
- `tests/stdlib/std_test_collect.n.md`

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

## 修正結果

- 根本原因は assign そのものではなく、`raw_address_alias` と local read alias が通常の owning aggregate root にも残り、call return の projection owners が temporary 側に存在しているのに `resolve_owner_alias_place` が古い local root へ逆解決していたことだった。
- `resolve_owner_alias_place` は、要求された place 自身に tracked descendant owner state がある場合、その place を canonical として優先するようにした。これにより、call return output に projection owner summary が適用済みなら、古い raw/root alias ではなく output 側の owner state を使って assign / declare できる。
- `std_test_collect_continues_after_string_allocation` は pipeline 回避形から、本来の `let mut report` + `set report test_report_push report ...` 形へ戻した。
- `TestReport` / `TestAssertion` の `Copy` は復活させていない。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reinitializes_self_update_report_projection_returns -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reinitializes_self_update -- --nocapture`: 5 passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/std_test_collect.n.md --no-tree -o tmp/std-test-self-update-owner-alias-fixed.json -j 1 --dist web/dist`: total=3, passed=3
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
- `TestReport` / `TestAssertion` の `Copy` を復活させずに通す。
