---
id: ISS-20260528T093534569Z-TYPECHECK-CALL-RESOLUTION-EXCEEDS-RE-9BBA7B96
title: "typecheck call resolution exceeds responsibility split limit after public signature test split"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-28
updated: 2026-05-28
target: "nepl-core/src/typecheck/call_resolution.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260528T093534569Z-TYPECHECK-CALL-RESOLUTION-EXCEEDS-RE-9BBA7B96: typecheck call resolution exceeds responsibility split limit after public signature test split

## 概要

After moving public signature contract tests out of driver.rs, the static check boundary policy reaches the next blocker: typecheck/call_resolution.rs has 802 implementation lines while the responsibility split limit is 760.

## 対象

- `nepl-core/src/typecheck/call_resolution.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `node nodesrc/test_static_check_boundary_responsibility.js` は、`driver.rs` の public signature 契約テスト移動後に `typecheck/call_resolution.rs has 802 implementation lines; responsibility split limit is 760` を報告する。
- `call_resolution.rs` は call candidate boundary の判断に加えて、outer consumer expected-type 推論、pipe segment reduction、unresolved overload deferral、arity selection、capture/reference collection を同じ file に持っている。
- source policy の行数監視はコメント行ではなく実装行を数えるため、コメントや doccomment の増加を妨げるものではない。
- しきい値を上げると、NEPLg2.1 の prefix call 解決で重要な責務境界が再び曖昧になるため、モジュール分割で解く。

## 問題

After moving public signature contract tests out of driver.rs, the static check boundary policy reaches the next blocker: typecheck/call_resolution.rs has 802 implementation lines while the responsibility split limit is 760.

## 影響

The call-resolution module now mixes outer consumer inference, pipe segment reduction, unresolved overload deferral, arity selection, and capture/reference collection in one oversized responsibility surface, reducing reviewability and weakening the source-policy signal.

## 修正方針

Split call_resolution responsibilities into focused modules without changing the policy limit. Candidate first split points are outer consumer expected-type inference and pipe-specific pending-segment reduction, leaving call_resolution.rs as the facade or orchestration layer.

## 検証

node nodesrc/test_static_check_boundary_responsibility.js; cargo check -p nepl-core; node nodesrc/issues.js check --dir issues; git diff --check

## 2026-05-28 修正

- `call_resolution.rs` から pipe-specific pending segment resolution を `typecheck/call_pipe.rs` へ分離した。
- 分離対象は `pipe_target_input_type`、`reduce_pipe_pending_segment_with_target`、`pipe_pending_base`、`pipe_segment_reduces_to_single_value` である。
- `call_resolution.rs` は outer consumer inference、unresolved overload 判定、callable signature extraction、arity selection の責務に絞り、`call_pipe.rs` は `|>` の pending segment を target argument type と rollback-scoped reduction で 1 値へ畳む責務を持つ。
- static check boundary policy は変更していない。
- `nodesrc/test_type_expectation_model_policy.js` は、helper split 後も typed outer-consumer expectation construction が `call_resolution.rs` / `call_pipe.rs` の分割境界を越えて保持されることを見るように追従した。
