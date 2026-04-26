---
id: ISS-20260426T175008731Z-OWNED-AGGREGATE-DECOMPOSITION-LACKS--48C352EE
title: "owned aggregate decomposition lacks safe multi-field move path"
area: core
status: verified
resolved: true
priority: P2
type: architecture
created: 2026-04-26
updated: 2026-04-27
target: "nepl-core/src/passes/move_check.rs, nepl-core/src/passes/drop_insertion.rs, nepl-core/tests/move_check.rs, nepl-core/tests/drop.rs, tests/compiler/move_check.n.md"
---

# ISS-20260426T175008731Z-OWNED-AGGREGATE-DECOMPOSITION-LACKS--48C352EE: owned aggregate decomposition lacks safe multi-field move path

## 概要

When an owning struct contains multiple non-Copy fields, helper code cannot move out more than one field with field::get because the owner is considered moved after the first non-Copy field extraction. Existing stdlib code works around similar cases with raw memory store/load detours.

## 対象

- `nepl-core/src/typecheck.rs, nepl-core/src/passes/move_check.rs, stdlib/alloc/diag/error.nepl, stdlib/neplg2/core/infra/outcome.nepl`

## 根拠

- 未記入

## 問題

When an owning struct contains multiple non-Copy fields, helper code cannot move out more than one field with field::get because the owner is considered moved after the first non-Copy field extraction. Existing stdlib code works around similar cases with raw memory store/load detours.

## 影響

Outcome-like values that need to return a Result and free or propagate diagnostics are pushed toward raw memory detours or indirect pointer layouts. This makes ownership intent harder to audit and can hide real move/borrow bugs from stdlib review.

## 修正方針

Design a safe owned aggregate decomposition path, such as compiler-supported struct destructuring or a checked multi-field move primitive, so code can consume an owner and bind all fields exactly once without raw memory round-trips.

## 検証

Add compiler tests that consume a struct with two non-Copy fields and bind both fields once, while still rejecting repeated moves, partial use-after-move, and borrow-live owner moves.

## 修正内容

- `field::get` が `Intrinsic "load"` に下がった後でも、load address から owner / field offset / field type を復元し、非 Copy field の move を owner 全体ではなく field 単位で記録するようにした。
- move checker は、異なる field の一度きり move を許可し、同一 field の二重 move、部分 move 後の owner 全体の use/move/drop、borrow 中 owner からの field move を拒否する。
- branch/loop merge では field move set が一致しない場合に `PossiblyMoved` として扱い、条件付き部分 move を通常の完全 owner と混同しないようにした。
- drop insertion は部分 move 済み aggregate を owner 全体として drop せず、未 move で Drop を持つ field だけを field address 経由で drop するようにした。
- stdlib 側の raw memory detour は別 agent の作業領域と競合しないため、この commit では core safety と回帰テストに限定した。

## 回帰テスト

- `nepl-core/tests/move_check.rs`: distinct non-Copy fields をそれぞれ一度だけ取り出せる正常系、同一 field 二重 move、部分 move 後 owner use、borrow live 中の field move を追加。
- `nepl-core/tests/drop.rs`: 部分 move 済み struct で、移動済み field は新しい束縛として drop され、owner の custom Drop は呼ばず、未 move field だけが drop されることを追加。
- `tests/compiler/move_check.n.md`: n.md doctest に同等の compile/run 回帰を追加。

## 検証結果

- `cargo fmt --all --check`: 成功
- `cargo check -p nepl-core`: 成功
- `cargo test -p nepl-core --test move_check -- --nocapture`: 51/51 passed
- `cargo test -p nepl-core --test drop -- --nocapture`: 8/8 passed
- `trunk build`: 成功
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md --no-tree -o tmp/owned-aggregate-decomposition-tests.json -j 1`: 52/52 passed
