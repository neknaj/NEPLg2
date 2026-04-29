---
id: ISS-20260429T231116550Z-AUTO-DROP-SKIPS-REMAINING-STRUCT-FIE-67E6E6C5
title: "auto drop skips remaining struct field after partial move"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core/tests/drop.rs, nepl-core/src/passes/drop_insertion.rs"
---

# ISS-20260429T231116550Z-AUTO-DROP-SKIPS-REMAINING-STRUCT-FIE-67E6E6C5: auto drop skips remaining struct field after partial move

## 概要

`auto_drop_partially_moved_struct_drops_remaining_fields` で、struct の一部 field を move した後に scope end auto drop が残り field を drop せず、drop log が期待 `[1, 2]` に対して `[1]` になる。

## 対象

- `nepl-core/tests/drop.rs, nepl-core/src/passes/drop_insertion.rs`

## 根拠

- `cargo test -p nepl-core --test drop auto_drop_partially_moved_struct_drops_remaining_fields -- --nocapture`: `left: [1]`, `right: [1, 2]`
- `field::get p "left"` は typecheck 後に `get_field` intrinsic になるが、drop insertion は `get_field` を field projection として扱わず、default intrinsic traversal で base variable `p` 全体を move 済みにしていた。
- 同じ root cause により、Copy field read でも非 Copy owner 全体が move 済みになり、残り droppable field の auto drop が消える。

## 問題

`auto_drop_partially_moved_struct_drops_remaining_fields` で、struct の一部 field を move した後に scope end auto drop が残り field を drop せず、drop log が期待 `[1, 2]` に対して `[1]` になる。

## 影響

partially moved aggregate の残存 owner field が自動解放されず leak する可能性があり、型安全・メモリ安全を必達にする静的検証方針に反する。

## 修正方針

drop insertion の partial move state と remaining-field traversal を調査し、move 済み field だけを除外して未 move field の drop を維持するように修正する。

## 対応

- `DropInsertionContext` に module の string literal table を渡し、`get_field` intrinsic の static selector を解決できるようにした。
- `get_field` が static selector 付きの非 Copy field read であれば owner 全体ではなく該当 field offset だけを moved field として記録するようにした。
- Copy field read では owner 全体を move 済みにしないようにし、残存 droppable field の auto drop を維持した。
- `auto_drop_copy_field_read_keeps_struct_owner_alive` を追加し、Copy field read 後に owner の droppable field が解放されることを固定した。

## 検証

- `cargo test -p nepl-core --test drop auto_drop_partially_moved_struct_drops_remaining_fields -- --nocapture`: reproduced before fix
- `cargo test -p nepl-core --test drop -- --nocapture`: `17 passed`
- `rustfmt nepl-core/src/passes/drop_insertion.rs nepl-core/tests/drop.rs`: passed
