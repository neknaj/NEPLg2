---
id: ISS-20260518T030154409Z-SELFHOST-CLI-REPORTER-DOCTESTS-EXCEE-6D30C865
title: "selfhost CLI reporter doctests exceed local compile timeout"
area: TEST
status: fixed
resolved: true
priority: P1
type: performance
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/selfhost_cli_reporter.n.md, stdlib/neplg2/cli/reporter.nepl, stdlib/neplg2/cli/reporter/render, nepl-core/src/resource/effect_return_summary_filter.rs, nepl-core/src/resource/raw_pointer_type.rs"
---

# ISS-20260518T030154409Z-SELFHOST-CLI-REPORTER-DOCTESTS-EXCEE-6D30C865: selfhost CLI reporter doctests exceed local compile timeout

## 概要

Focused selfhost CLI reporter doctests time out during compile even with NEPL_TEST_CASE_TIMEOUT_MS=300000. The timeout happens before run output is produced and affects all three reporter doctests.

## 対象

- `tests/stdlib/selfhost_cli_reporter.n.md, stdlib/neplg2/cli/reporter.nepl, stdlib/neplg2/core/infra/diag.nepl, nepl-core/src/resource`

## 根拠

- `node nodesrc/tests.js -i tests\stdlib\selfhost_cli_reporter.n.md --no-tree -o tmp\agent1-selfhost-cli-reporter-report.json -j 1 --dist web\dist --assert-io` は 3 doctest すべてで `wasm test case timeout after 60000ms` になった。
- `$env:NEPL_TEST_CASE_TIMEOUT_MS='300000'; node nodesrc/tests.js -i tests\stdlib\selfhost_cli_reporter.n.md --no-tree -o tmp\agent1-selfhost-cli-reporter-report-300s.json -j 1 --dist web\dist --assert-io` でも 3 doctest すべてが `wasm test case timeout after 300000ms` になった。
- timeout の `last_phase` は `compile` で、run output 生成前に止まっている。

## 問題

Focused selfhost CLI reporter doctests time out during compile even with NEPL_TEST_CASE_TIMEOUT_MS=300000. The timeout happens before run output is produced and affects all three reporter doctests.

## 影響

Reporter fixture changes cannot be locally validated by focused doctest execution, and CI may spend excessive time on selfhost diagnostic rendering cases. The cause may be compiler/static-check cost in the selfhost diagnostic import graph rather than the generated wasm runtime.

## 修正方針

Fixed.

根本原因は `StringBuilder` / `ByteBuilder` のような owner token を内包する structural owner carrier を、`len` / `cap` などの plain `i32` field まで public raw pointer / raw identity carrier として扱っていたことだった。

`append_six` のように `sb_append` を6回連鎖するだけの最小再現で、`resource_effect_boundaries` が 155 秒級まで肥大化した。これは generated wasm runtime ではなく、Resource IR の raw pointer alias summary と raw identity return summary が owner carrier 内部の metadata field を探索し続ける compile-time 問題だった。

修正後は、`TypeCtx` の compiler-memory type 証明を使い、`MemPtr` は non-owning raw pointer、`RegionToken` / structural owner carrier は free obligation owner として分離した。`RegionToken` や `StringBuilder` / `ByteBuilder` のような owner carrier は raw pointer alias summary / raw identity summary の public propagation 対象から外し、`MemPtr` や bare raw `i32` の public return は引き続き summary 対象にする。stdlib module 名の allowlist ではなく、compiler-memory owner token の型証明に基づく判定である。

Reporter 側は compile surface を下げるため、facade、single renderer、collection renderer、stdio write boundary に分割した。render-only doctest は stdio write boundary を import せず、single diagnostic renderer は `&SelfhostDiagnostic` を受けることで owner move を避ける。

## 検証

- `cargo test -p nepl-core summary_filter -- --nocapture`: passed
- `cargo test -p nepl-core summary_carrier -- --nocapture`: passed
- `cargo build -p nepl-cli`: passed
- `target\debug\nepl-cli.exe --check -i tmp\agent1_append_six.nepl --target std` with `NEPL_COMPILE_STAGE_TIMING=1`: `elapsed_ms=6059`, `resource_effect_boundaries=67ms`, `resource_static_check=2833ms`
- `trunk build`: passed
- `node nodesrc/tests.js -i tests\stdlib\selfhost_cli_reporter.n.md --no-tree -o tmp\agent1-selfhost-cli-reporter-split-borrow.json -j 1 --dist web\dist --assert-io`: total=3, passed=3
- `node nodesrc/test_selfhost_cli_reporter_boundary.js`: passed
- `node nodesrc/test_selfhost_cli_reporter_report_contract.js`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/test_abstraction_static_verification_policy.js`: passed
