---
id: ISS-20260507T015042055Z-MOVE-EFFECT-FIELD-GET-FIXTURES-STILL-F9F8D1C4
title: "move_effect field::get fixtures still use glob core/field imports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-07
updated: 2026-05-07
target: tests/compiler/move_effect.n.md
---

# ISS-20260507T015042055Z-MOVE-EFFECT-FIELD-GET-FIXTURES-STILL-F9F8D1C4: move_effect field::get fixtures still use glob core/field imports

## 概要

After the Resource IR authority migration, several move_effect fixtures call field::get with a qualified module path while importing core/field as *. Current NEPLg2 import semantics do not bind field::get for glob imports, so the fixtures fail in name resolution before reaching the Resource IR cell diagnostics they are meant to assert.

## 対象

- `tests/compiler/move_effect.n.md`

## 根拠

- `tmp/move_effect_agent1_after_raw_view_origin.json` では `tests/compiler/move_effect.n.md` が total=110, passed=101, failed=9 だった。
- 残り 9 件はいずれも `field::get` を使う fixture が `#import "core/field" as *` のままで、`resolve.identifier.undefined` が先に出ていた。
- これらの fixture は `resource.cell.moved` や raw aggregate field move を検証する目的であり、名前解決エラーで落ちる状態では Resource IR の回帰テストとして機能しない。

## 問題

After the Resource IR authority migration, several move_effect fixtures call field::get with a qualified module path while importing core/field as *. Current NEPLg2 import semantics do not bind field::get for glob imports, so the fixtures fail in name resolution before reaching the Resource IR cell diagnostics they are meant to assert.

## 影響

The failures hide whether aggregate/MemPtr alias and raw aggregate field move checks report resource.cell.moved correctly. They also create pressure to weaken diagnostics even though the compiler is rejecting the stale fixture shape before static resource checks run.

## 修正方針

Update only the fixtures that call field::get with a qualified path to import core/field as field. Keep unqualified get fixtures as glob imports so the re-export coverage remains meaningful.

## 検証

- `NEPL_TEST_CASE_TIMEOUT_MS=120000 node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree --dist web/dist -o tmp/move_effect_agent1_after_field_imports.json -j 1 --assert-io`: total=110, passed=110, failed=0

## 対応結果

qualified `field::get` を使う fixture のみ `#import "core/field" as field` に変更した。unqualified `get` の re-export / glob import fixture はそのまま残し、import 形式の回帰検査と Resource IR cell diagnostic の回帰検査を混ぜないようにした。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
