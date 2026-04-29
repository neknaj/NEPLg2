---
id: ISS-20260429T034515729Z-RESOURCE-BORROW-NON-RETURN-CONFLICTS-12A5A5E4
title: "Resource borrow non-return conflicts remain shadow-only"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/compiler.rs, nepl-core/src/resource/borrow_check.rs, tests/compiler/move_check.n.md"
---

# ISS-20260429T034515729Z-RESOURCE-BORROW-NON-RETURN-CONFLICTS-12A5A5E4: Resource borrow non-return conflicts remain shadow-only

## 概要

ResourceBorrowCheckEngine reports Read/Assign/Move/Drop/SharedBorrow/UniqueBorrow conflicts, but compiler.rs only maps ReturnValue conflicts to diagnostics. Stage 4 borrow/lifetime checking therefore remains non-authoritative for normal source mutation/read conflicts even when Resource IR detects them.

## 対象

- `nepl-core/src/compiler.rs, nepl-core/src/resource/borrow_check.rs, tests/compiler/move_check.n.md`

## 根拠

- `nepl-core/src/resource/borrow_check.rs` は `Read`、`Assign`、`Move`、`Drop`、`SharedBorrow`、`UniqueBorrow` の `BorrowConflict` を既に報告している。
- `nepl-core/src/compiler.rs` は `ReturnValue` だけを `D3099` に昇格し、それ以外の borrow conflict を `None` として shadow-only にしていた。
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は borrow / lifetime を Resource IR 上の検査へ移す計画であり、通常の read / mutation / borrow 作成の conflict を shadow-only に残すと検査が形骸化する。

## 問題

ResourceBorrowCheckEngine reports Read/Assign/Move/Drop/SharedBorrow/UniqueBorrow conflicts, but compiler.rs only mapped ReturnValue conflicts to diagnostics. Stage 4 borrow/lifetime checking therefore remained non-authoritative for normal source mutation/read conflicts even when Resource IR detected them.

## 影響

Resource IR borrow/lifetime checking can regress or remain incomplete while the compiler still succeeds through old move_check parity gaps. This keeps borrow checks partially formal and conflicts with the static_check_complexity_reduction_plan Stage 4 goal.

## 修正方針

Promote all ResourceBorrowDiagnostic::BorrowConflict operations to compiler diagnostics after old move_check and lowering coverage pass. `ReturnValue` remains `D3099`; source read/move/assign/drop/borrow conflicts map to the existing borrow/move diagnostic IDs (`D3051`, `D3052`, `D3055`, `D3056`, `D3057`, `D3058`, `D3061`, `D3062`) instead of a raw-memory-specific diagnostic.

## 検証

- `rustfmt --check nepl-core\src\compiler.rs`: pass
- `cargo test -p nepl-core compiler::tests::resource_borrow_gate -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_borrow_check -- --nocapture`: 12 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_check.n.md --no-tree -o tmp\agent1-resource-borrow-conflict-gate-move-check.json -j 1`: total=52, passed=52, failed=0
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\agent1-resource-borrow-conflict-gate-move-effect.json -j 1`: total=110, passed=110, failed=0

## 対応結果

`resource_borrow_diagnostic_to_error` を `ReturnValue` 専用ではなく全 `BorrowConflict` に対応させた。戻り値 escape は従来通り `D3099`、通常の borrow conflict は旧 `move_check` と同じ意味の診断 ID に写像するため、Resource IR gate が新しい別種のエラー体系にならず、既存の borrow safety 仕様と揃う。

これにより Resource IR borrow checker が検出した read / move / assign / drop / borrow 作成の conflict は compiler pipeline で authoritative になる。旧 `move_check` は先に実行されるため既存防壁は維持しつつ、旧 checker の parity gap がある場合でも Resource IR 側で補完できる。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
