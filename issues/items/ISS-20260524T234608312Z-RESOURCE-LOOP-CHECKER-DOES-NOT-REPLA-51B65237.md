---
id: ISS-20260524T234608312Z-RESOURCE-LOOP-CHECKER-DOES-NOT-REPLA-51B65237
title: "Resource loop checker does not replay moved body state on backedge"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-24
updated: 2026-05-25
target: "nepl-core/src/resource/initialized_control.rs; nepl-core/tests/move_check.rs"
---

# ISS-20260524T234608312Z-RESOURCE-LOOP-CHECKER-DOES-NOT-REPLA-51B65237: Resource loop checker does not replay moved body state on backedge

## 概要

A value moved inside a while body can be accepted because the resource checker merges the first body pass without replaying a possible next iteration from the body post-state.

## 対象

- `nepl-core/src/resource/initialized_control.rs; nepl-core/tests/move_check.rs`

## 根拠

- `move_in_loop` は `while cnd:` の body で非 Copy 値 `x` を move した後、次反復で同じ body が再び `x` を読む可能性があるため reject されるべきだった。
- 修正前は `cargo test -p nepl-core --test move_check -- --nocapture` で `move_in_loop` が compile success となり、期待していた move diagnostic が出なかった。
- loop の最終状態自体は exit path と body path の merge でよいが、診断としては body post-state を loop backedge へ戻した次反復の condition/body を少なくとも一度検査する必要がある。

## 問題

A value moved inside a while body can be accepted because the resource checker merges the first body pass without replaying a possible next iteration from the body post-state.

## 影響

Loop-carried move violations can escape Resource IR checking, so non-Copy values may be read again after a previous iteration consumed them.

## 修正方針

After checking the first body pass, enumerate body path alternatives and replay one next condition/body pass from each body post-state only for diagnostics. Keep the final loop state as the merge of exit and body states.

## 検証

- `cargo test -p nepl-core --test move_check move_in_loop -- --nocapture`: passed.
- `cargo test -p nepl-core --test move_check -- --nocapture`: 55/55 passed.

## 2026-05-25 修正結果

`ResourceCheckEngine::check_loop` は body の path alternatives を `ResourceCheckState` として保持し、各 body post-state から診断用に次の condition/body を一度だけ replay するようにした。この replay は diagnostics を収集するための検査であり、最終的に外へ出す loop state は従来通り exit path と body path の merge で構成する。

これにより、1回目の loop body で move された非 Copy 値を、次反復の body が再び読むケースを Resource IR cell state violation として検出できる。
