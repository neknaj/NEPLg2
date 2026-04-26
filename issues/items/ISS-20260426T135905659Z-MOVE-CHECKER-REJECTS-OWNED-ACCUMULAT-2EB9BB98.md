---
id: ISS-20260426T135905659Z-MOVE-CHECKER-REJECTS-OWNED-ACCUMULAT-2EB9BB98
title: "move checker rejects owned accumulator updated after fallible consuming loop call"
area: core
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/passes/move_check.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md"
source: "ByteBuilder implementation while handling ISS-20260426T010005Z-BYTEBUF-BUILDER-WASM-D63A7C11"
---

# ISS-20260426T135905659Z-MOVE-CHECKER-REJECTS-OWNED-ACCUMULAT-2EB9BB98: move checker rejects owned accumulator updated after fallible consuming loop call

## 概要

Owned accumulator values such as ByteBuilder cannot be updated through a fallible consuming call inside a loop without triggering D3065/D3054, even when the loop stops on Err and returns the last successful value.

## 対象

- `nepl-core/src/passes/move_check.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md`

## 根拠

During ByteBuilder implementation, a loop in byte_builder_push_leb_u32 that repeatedly called byte_builder_push_u8 current out failed with D3065 potentially moved value current, and a doctest helper that accumulated ByteBuilder in a loop failed the same way for cur.

## 問題

The move checker marks the loop-carried owned accumulator as potentially moved because one iteration may pass it to a consuming function before the result branch reassigns the accumulator. It does not model the control-flow invariant that Err exits the loop and Ok always installs the replacement value before the next iteration or final return.

## 影響

Stdlib and self-host compiler code must avoid natural owned-builder accumulator loops and use recursion or hand-unrolled code. That hides the real ownership-flow limitation, makes binary emitters harder to write, and can force awkward workarounds in ByteBuilder, parser, and codegen code.

## 修正方針

Teach move/borrow analysis to distinguish continuing and diverging branches when merging branch-local ownership state. After a consuming call, the old value is unavailable; however, a `never`-typed Err branch such as `unreachable` does not reach the post-match state, so only the Ok branch that installs the replacement should determine the continuing loop state. Add CFG-level tests covering owned accumulator loops and use-after-Err rejection.

## 検証

Add core tests where an owned non-Copy value is repeatedly replaced in a while loop through Result::Ok and returned after the loop; add negative tests where an Err path can continue without replacement and must still be rejected.

## 解決内容

- `move_check` の branch merge を `BranchStateSnapshot` に整理し、branch ごとに `continues`、変更差分、最終 state を保持するようにした。
- `if`、builtin `if`、`match` の merge で `never` 型の branch を post-branch state と返り値 borrow の merge 対象から外した。
- `Result::Err` 側が `#intrinsic "unreachable"` で diverge する owned accumulator loop は、continuing path だけを見て `cur` が再初期化済みであると判定するようにした。
- `Result::Err` 側が通常に継続し、消費済み accumulator を再初期化しないケースは引き続き `potentially moved` として拒否する回帰テストを追加した。
- stdlib 側の ByteBuilder 実装には触れず、Rust core の move checker の制御フロー merge を修正した。

## 検証結果

- `node nodesrc/issues.js check`: pass (`files=130`)
- `cargo fmt --all --check`: pass
- `git diff --check`: pass
- `cargo check -p nepl-core --test move_check`: pass
- `cargo test -p nepl-core --test move_check move_loop_owned_accumulator -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test move_check -- --nocapture`: 24 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md --no-tree -o tmp/move-check-loop-accumulator-after-generic-rebase.json -j 1`: `total=25`, `passed=25`, `failed=0`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-loop-accumulator-after-generic-rebase.json`: 13/13 passed
- `cargo check --workspace`: pass
