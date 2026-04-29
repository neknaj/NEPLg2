---
id: ISS-20260429T004211995Z-RESOURCE-BORROW-RETURN-ESCAPE-GATE-R-298B4E8A
title: "Resource borrow return escape gate remains shadow-only"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/compiler.rs, nepl-core/src/resource/borrow_check.rs, tests/compiler/move_check.n.md"
---

# ISS-20260429T004211995Z-RESOURCE-BORROW-RETURN-ESCAPE-GATE-R-298B4E8A: Resource borrow return escape gate remains shadow-only

## 概要

Resource IR borrow checker detects active borrow tokens returned across a function boundary, but compiler.rs only emitted the borrow report in verbose shadow mode. A regression in Resource IR borrow summaries could therefore remain non-authoritative even when old move_check does not see the Resource IR token flow.

## 対象

- `nepl-core/src/compiler.rs, nepl-core/src/resource/borrow_check.rs, tests/compiler/move_check.n.md`

## 根拠

- `nepl-core/src/resource/borrow_check.rs` は `ResourceBorrowOperation::ReturnValue` として active borrow token の return escape を検出できる。
- `nepl-core/src/compiler.rs` は Resource IR lowering coverage、raw cell gate、raw identity escape gate は compiler diagnostic に接続しているが、borrow lifetime report は verbose shadow report のみに残っていた。
- Stage 4 の authoritative 化を進めるには、旧 `move_check` を通過した後でも Resource IR 側で関数境界 lifetime escape を補完できる必要がある。

## 問題

Resource IR borrow checker detects active borrow tokens returned across a function boundary, but compiler.rs only emitted the borrow report in verbose shadow mode. A regression in Resource IR borrow summaries could therefore remain non-authoritative even when old move_check does not see the Resource IR token flow.

## 影響

Stage 4 cannot replace HIR move_check for lifetime checks while borrow return escape diagnostics are shadow-only. Higher-order or helper-returned borrow tokens can only be trusted if the Resource IR report is connected to compiler diagnostics.

## 修正方針

After old move_check and Resource IR lowering coverage pass, run check_resource_borrow_lifetimes and map ReturnValue BorrowConflict diagnostics to D3099. Keep non-return borrow conflicts shadow-only until their old-checker parity is verified.

## 検証

- `cargo test -p nepl-core compiler::tests::resource_borrow_gate -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_borrow_check -- --nocapture`: 12 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_check.n.md --no-tree -o tmp\agent1-resource-borrow-gate-alias-move-check.json -j 1`: total=52, passed=52, failed=0
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\agent1-resource-borrow-gate-alias-move-effect.json -j 1`: total=110, passed=110, failed=0

## 対応結果

旧 `move_check`、Resource IR lowering coverage、raw cell gate の後に `check_resource_borrow_lifetimes` を実行し、`ReturnValue` の `BorrowConflict` だけを compiler diagnostic `D3099` に昇格する gate を追加した。read / assign / unique borrow などの非 return conflict は旧 checker との parity が未確認なため shadow-only に残している。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
