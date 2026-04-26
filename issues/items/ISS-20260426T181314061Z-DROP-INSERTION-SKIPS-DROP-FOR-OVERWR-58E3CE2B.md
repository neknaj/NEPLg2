---
id: ISS-20260426T181314061Z-DROP-INSERTION-SKIPS-DROP-FOR-OVERWR-58E3CE2B
title: "drop insertion skips Drop for overwritten values"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/passes/drop_insertion.rs, nepl-core/tests/drop_overwrite.rs, tests/compiler/drop_overwrite.n.md"
---

# ISS-20260426T181314061Z-DROP-INSERTION-SKIPS-DROP-FOR-OVERWR-58E3CE2B: drop insertion skips Drop for overwritten values

## 概要

Drop 対象の mutable binding を set で上書きしても、旧値に対する Drop 呼び出しが挿入されず、scope 終端の最終値だけが Drop される。

## 対象

- `nepl-core/src/passes/drop_insertion.rs, nepl-core/tests/drop_overwrite.rs, tests/compiler/drop_overwrite.n.md`

## 根拠

- `nepl-core/src/passes/drop_insertion.rs` の `HirExprKind::Set` は RHS を走査した後に対象 binding を `Valid` へ戻すだけで、上書き前の旧値に対する Drop 呼び出しを生成していなかった。
- scope 終端の `scope_drop_lines` は最後に残った値だけを Drop するため、`let mut g Guard 0; set g Guard 1` では `Guard 0` の Drop が失われる。

## 問題

Drop 対象の mutable binding を set で上書きしても、旧値に対する Drop 呼び出しが挿入されず、scope 終端の最終値だけが Drop される。

## 影響

owned resource を保持する値を再代入するたびに旧 resource がリークし、self-host compiler のバッファやファイル handle の deterministic release 前提が崩れる。

## 修正方針

set の RHS を評価した後、旧値がまだ有効で Drop capability を持つ場合に旧値の Drop を挿入してから新値を代入する drop elaboration にする。RHS 評価順は保持する。

## 検証

set overwrite で RHS effect の後に旧値 Drop、scope 終端で新値 Drop が起きる回帰テストを追加する。

## 解決

- `set` の RHS を一時 local に評価し、旧値がまだ `Valid` かつ Drop capability を持つ場合は旧値 Drop を挿入してから一時 local を代入する HIR block へ展開した。
- RHS 評価順を保持するため、旧値 Drop を `set` の前へ単純移動せず、`let __nepl_drop_assign_tmp_N <rhs>; drop old; set old __tmp` の順になるようにした。
- Rust integration test で `tick 1`（RHS effect）→ `tick 2`（旧値 Drop）→ `tick 2`（新値 scope Drop）の順序を固定した。
- `tests/compiler/drop_overwrite.n.md` に nodesrc compiler 経路の `set` overwrite 回帰を追加した。

## 検証結果

- `cargo test -p nepl-core --test drop_overwrite -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test drop -- --nocapture`: 7 passed
- `cargo test -p nepl-core --test move_check -- --nocapture`: 38 passed
- `cargo check --workspace`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/drop_overwrite.n.md --no-tree -o tmp/drop-overwrite-elaboration-after-trunk.json -j 1`: 1 passed
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md -i tests/compiler/move_effect.n.md --no-tree -o tmp/drop-overwrite-move-regression.json -j 1`: 66 passed
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-drop-overwrite.json`: 13/13 passed
- `node tests/compiler/tree/run.js`: 20 passed
