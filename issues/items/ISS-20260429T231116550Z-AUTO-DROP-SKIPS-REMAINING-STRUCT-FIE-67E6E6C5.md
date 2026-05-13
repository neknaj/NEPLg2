---
id: ISS-20260429T231116550Z-AUTO-DROP-SKIPS-REMAINING-STRUCT-FIE-67E6E6C5
title: "auto drop skips remaining struct field after partial move"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-05-14
target: "nepl-core/tests/drop.rs, nepl-core/tests/resource_ir.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/initialized_drop_scope.rs, nepl-core/src/passes/drop_insertion.rs"
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

## 2026-05-14 再発調査

- clean `main` で `cargo test -p nepl-core --test drop auto_drop_partially_moved_struct_drops_remaining_fields -- --nocapture` を再実行し、再び `left: [1]`, `right: [1, 2]` になることを確認した。
- 2026-04-30 の修正により `field::get p "left"` は Resource IR 上で `p.left` の move として表現されていたが、`auto_drop_scope_locals_with_record` が root local `p` 全体の availability だけを見て `Moved` と判定し、初期化済みの `p.right` を含む partial auto-drop を生成していなかった。
- 追加で、scope local は逆宣言順で auto-drop されるべきだが、Resource lowering が `BTreeMap` から `EndScope.locals` を作っていたため、宣言順ではなく名前順になっていた。部分move済み root の auto-drop が復活すると、`p` と `left` の drop 順が `[2, 1]` になり、この lowering 側の順序バグも表面化した。

## 2026-05-14 追加対応

- `LoweringContext` の local scope を名前解決用 map と宣言順 list に分離し、`EndScope.locals` を宣言順で生成するようにした。これにより `auto_drop_candidates_for_end_scope` の逆順 traversal が、名前順ではなく逆宣言順に基づく。
- `auto_drop_scope_locals_with_record` は root local が部分move済みでも、型レイアウトと `CellTable` の状態から初期化済み descendant だけの `ResourceDropRequirement::Structural` を再構成するようにした。
- 部分move済み descendant は除外し、初期化済み descendant のみを drop 対象にするため、既に move された field の二重dropと残存 field の leak の両方を避ける。
- `resource_ir_lowering_preserves_scope_local_declaration_order` を追加し、Resource IR lowering が `EndScope.locals` の宣言順を保持することを固定した。

## 2026-05-14 検証

- `cargo test -p nepl-core --test drop auto_drop_partially_moved_struct_drops_remaining_fields -- --nocapture`: passed
- `cargo test -p nepl-core --test drop -- --nocapture`: `17 passed`
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_preserves_scope_local_declaration_order -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_drop_insertion_consumes_checked_scope_and_assignment_points -- --nocapture`: passed
- `cargo fmt --package nepl-core --check`: passed
