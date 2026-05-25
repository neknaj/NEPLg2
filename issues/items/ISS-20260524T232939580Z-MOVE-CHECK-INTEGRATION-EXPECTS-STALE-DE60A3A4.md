---
id: ISS-20260524T232939580Z-MOVE-CHECK-INTEGRATION-EXPECTS-STALE-DE60A3A4
title: "move_check integration expects stale legacy diagnostic text"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-24
updated: 2026-05-25
target: "nepl-core/tests/move_check.rs; nepl-core/src/diagnostic_codes.rs; nepl-core/src/resource/report.rs; nepl-core/src/resource/initialized_control.rs"
---

# ISS-20260524T232939580Z-MOVE-CHECK-INTEGRATION-EXPECTS-STALE-DE60A3A4: move_check integration expects stale legacy diagnostic text

## 概要

move_check integration tests still assert legacy human-readable messages after Resource IR emits structured cell and borrow diagnostic codes. Full cargo test fails even though the compiler emits the expected Resource IR violations.

## 対象

- `nepl-core/tests/move_check.rs; nepl-core/src/diagnostic_codes.rs; nepl-core/src/resource/report.rs; nepl-core/src/resource/initialized_control.rs`

## 根拠

- `cargo test -p nepl-core` は Resource IR diagnostic text への移行後、`nepl-core/tests/move_check.rs` の旧 human-readable message assertion で失敗していた。
- 独立調査でも、現在の正規の静的検査結果は `DiagnosticCode::Resource(...)` として保持されており、旧文言へ assertion を寄せるのではなく structured code に寄せるべきだと確認した。
- `move_in_loop` は旧文言 assertion だけではなく、Resource loop checker が body post-state から次反復を診断用に再実行していない実装上の穴も露呈したため、別 issue `ISS-20260524T234608312Z-RESOURCE-LOOP-CHECKER-DOES-NOT-REPLA-51B65237` として分離した。

## 問題

move_check integration tests still assert legacy human-readable messages after Resource IR emits structured cell and borrow diagnostic codes. Full cargo test fails even though the compiler emits the expected Resource IR violations.

## 影響

Full nepl-core verification is red and can confuse real move/borrow regressions with stale message wording. Tests also remain brittle against deliberate diagnostic text improvements.

## 修正方針

Move move_check assertions to DiagnosticCode-based helpers for moved, possibly moved, borrow conflict, and return escape cases. Keep raw-memory boundary failures as structured ResourceRaw diagnostics where appropriate; do not weaken the compiler checks or accept arbitrary errors.

## 検証

- `cargo test -p nepl-core --test move_check -- --nocapture`: 55/55 passed.
- `cargo test -p nepl-core` は未再実行。直前の full run は `move_check.rs` の 39 failures のみが残件であり、focused `move_check` suite はこの修正後に green。

## 2026-05-25 修正結果

`move_check.rs` の assertion を旧 human-readable message 依存から `DiagnosticCode` ベースの helper へ移行した。これにより、診断表示文言を改善しても、静的検査として期待する `resource.cell.*` / `resource.move.*` / `resource.borrow.*` の意味は弱めずに検査できる。

raw memory boundary の fixture は、通常の `<test>` path ではなく raw boundary 用の stdlib-root path で compile するようにし、raw boundary policy と同じ条件で期待を確認するようにした。

loop 由来の move failure は stale assertion ではなく Resource loop checker の backedge 診断不足だったため、実装修正は `ISS-20260524T234608312Z-RESOURCE-LOOP-CHECKER-DOES-NOT-REPLA-51B65237` に記録した。
