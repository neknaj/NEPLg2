---
id: ISS-20260429T015312801Z-MOVE-CHECK-PROVENANCE-MODULE-EXCEEDS-6F5A378F
title: "move_check provenance module exceeds responsibility split limit"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/passes/move_check/provenance.rs, nepl-core/src/passes/move_check/raw_memory_args.rs, nepl-core/src/passes/move_check.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260429T015312801Z-MOVE-CHECK-PROVENANCE-MODULE-EXCEEDS-6F5A378F: move_check provenance module exceeds responsibility split limit

## 概要

GitHub Actions Source policy regressions fail because nepl-core/src/passes/move_check/provenance.rs has grown to 645 lines over the 620-line responsibility split limit. The module now mixes field projection provenance, i32 constant evaluation, MemPtr/RegionToken provenance, and raw memory operation argument sizing.

## 対象

- `nepl-core/src/passes/move_check/provenance.rs, nepl-core/src/passes/move_check/raw_memory_args.rs, nepl-core/src/passes/move_check.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- GitHub Actions Source policy regressions と同じ `node nodesrc/test_static_check_boundary_responsibility.js` が `passes/move_check/provenance.rs has 645 lines; responsibility split limit is 620` で停止する。
- `provenance.rs` 末尾に raw memory operation の対象 place 復元と size 引数解析が同居し、Stage 1 の責務分割が再び曖昧になっていた。

## 問題

GitHub Actions Source policy regressions fail because nepl-core/src/passes/move_check/provenance.rs has grown to 645 lines over the 620-line responsibility split limit. The module now mixes field projection provenance, i32 constant evaluation, MemPtr/RegionToken provenance, and raw memory operation argument sizing.

## 影響

CI stops in Source policy regressions, and the Stage 1 boundary that prevents move_check from re-accumulating raw-memory responsibilities is no longer enforced.

## 修正方針

Split raw memory operation argument and size helpers out of provenance.rs into a dedicated move_check submodule. Keep provenance.rs focused on address/provenance derivation and keep the responsibility policy unchanged.

## 検証

Run node nodesrc/test_static_check_boundary_responsibility.js, focused cargo check/test for move_check imports, node nodesrc/issues.js check, and git diff --check.

- `rustfmt --check nepl-core\src\passes\move_check.rs nepl-core\src\passes\move_check\provenance.rs nepl-core\src\passes\move_check\raw_memory_args.rs nepl-core\src\passes\move_check\summary_build.rs nepl-core\src\passes\move_check\visitor.rs`: pass
- `node nodesrc\test_static_check_boundary_responsibility.js`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\agent1-provenance-split-move-effect-after-trunk.json -j 1`: total=110 passed=110
- `node nodesrc\issues.js check`: pass
- `git diff --check`: pass

## 対応結果

`move_check` の raw memory operation 引数解析を `nepl-core/src/passes/move_check/raw_memory_args.rs` へ分離した。`provenance.rs` は address / field / `MemPtr` / `RegionToken` provenance 復元に集中し、`raw_memory_args.rs` は `dealloc` / `realloc` / `store` / `fill` / `bulk_copy` の対象 place と byte size の判定を担当する。

Source policy には `raw_memory_args` module の存在と 180 行上限を追加し、今後この責務が再び `provenance.rs` へ戻った場合に検出できるようにした。分離後の `provenance.rs` は 552 行、`raw_memory_args.rs` は 103 行で、Stage 1 の module 境界を回復した。
