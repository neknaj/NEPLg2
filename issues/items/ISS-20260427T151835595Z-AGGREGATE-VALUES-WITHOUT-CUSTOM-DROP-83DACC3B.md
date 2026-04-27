---
id: ISS-20260427T151835595Z-AGGREGATE-VALUES-WITHOUT-CUSTOM-DROP-83DACC3B
title: "aggregate values without custom Drop skip droppable fields"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/passes/drop_insertion.rs, nepl-core/tests/drop.rs"
---

# ISS-20260427T151835595Z-AGGREGATE-VALUES-WITHOUT-CUSTOM-DROP-83DACC3B: aggregate values without custom Drop skip droppable fields

## 概要

drop_insertion only emits a whole-value Drop call when the aggregate type itself has a Drop impl. A struct or tuple that does not implement Drop but contains fields with Drop impls reaches scope end without dropping those fields.

## 対象

- `nepl-core/src/passes/drop_insertion.rs, nepl-core/tests/drop.rs`

## 根拠

- `DropInsertionContext::drop_lines_for_info` は、local の型そのものに Drop impl がある場合だけ whole-value drop を挿入していた。
- `moved_fields` が空の plain aggregate では `aggregate_fields_with_offsets` を見ないため、Drop impl を持つ field を内包していても scope end で field drop が生成されなかった。

## 問題

drop_insertion only emits a whole-value Drop call when the aggregate type itself has a Drop impl. A struct or tuple that does not implement Drop but contains fields with Drop impls reaches scope end without dropping those fields.

## 影響

Owned resources wrapped inside plain aggregate containers can leak because compiler drop elaboration treats the outer aggregate as non-droppable. This weakens memory/resource safety and forces stdlib code to rely on manual free paths instead of structural cleanup.

## 修正方針

When an aggregate has no custom Drop impl, recursively enumerate its direct and nested struct/tuple fields that do have Drop impls and emit field-address Drop calls for those initialized fields. Preserve existing custom Drop behavior and partial-move handling.

## 検証

Add a Rust drop trace regression where a plain struct contains a Guard field with Drop but has no Drop impl itself; scope exit must call the field destructor exactly once.

## 対応結果

- outer aggregate に custom Drop impl がない場合、struct / tuple field を再帰的に走査し、Drop impl を持つ leaf field に対して field-address Drop call を生成するようにした。
- partial move 済み aggregate でも、move されていない field の内側にある droppable leaf を drop するようにした。
- custom Drop impl を持つ型は従来通り whole-value Drop call を優先し、既存の明示 destructor semantics は変えていない。

## 実施した検証

- `cargo test -p nepl-core --test drop auto_drop_plain_struct_drops_droppable_fields`: pass
- `cargo test -p nepl-core --test drop`: `9 passed`
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/drop.n.md -i tests/compiler/drop_overwrite.n.md --no-tree -o tmp/structural-field-drop.json -j 1`: `total=5`, `passed=5`
