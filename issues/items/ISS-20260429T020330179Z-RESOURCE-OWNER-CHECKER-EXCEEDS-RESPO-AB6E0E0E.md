---
id: ISS-20260429T020330179Z-RESOURCE-OWNER-CHECKER-EXCEEDS-RESPO-AB6E0E0E
title: "resource owner checker exceeds responsibility split limit"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_flow.rs, nepl-core/src/resource/mod.rs, nepl-core/src/resource/summary.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260429T020330179Z-RESOURCE-OWNER-CHECKER-EXCEEDS-RESPO-AB6E0E0E: resource owner checker exceeds responsibility split limit

## 概要

After fixing the resource checker responsibility policy import detection, the same Source policy test fails because nepl-core/src/resource/owner_check.rs has grown to 930 lines over the 800-line responsibility split limit. Owner checking now mixes traversal, diagnostics, owner transfer, storage-origin propagation, raw alias lookup, and raw memory operation handling.

## 対象

- `nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_flow.rs, nepl-core/src/resource/mod.rs, nepl-core/src/resource/summary.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc\test_resource_checker_responsibility.js` が `owner_check.rs has 930 lines; responsibility split limit is 800` で停止した。
- `owner_check.rs` は traversal と operation dispatch に加え、owner transfer、return summary application、diagnostic emission、raw alias / storage origin resolution を同じ impl に抱えていた。

## 問題

After fixing the resource checker responsibility policy import detection, the same Source policy test fails because nepl-core/src/resource/owner_check.rs has grown to 930 lines over the 800-line responsibility split limit. Owner checking now mixes traversal, diagnostics, owner transfer, storage-origin propagation, raw alias lookup, and raw memory operation handling.

## 影響

GitHub Actions Source policy regressions remain red, and Stage 4 Resource IR owner checking is accumulating raw-memory and storage-origin responsibilities in the main owner checker instead of keeping a maintainable boundary.

## 修正方針

Split raw memory/storage-origin specific owner operations out of owner_check.rs into a dedicated resource module while preserving diagnostics and owner semantics. Keep the 800-line owner_check limit rather than raising it.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, focused resource owner tests, cargo check -p nepl-core --tests, node nodesrc/issues.js check, and git diff --check.

- `rustfmt --check nepl-core\src\resource\mod.rs nepl-core\src\resource\owner_check.rs nepl-core\src\resource\owner_flow.rs nepl-core\src\resource\summary.rs`: pass
- `node nodesrc\test_resource_checker_responsibility.js`: pass
- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check -- --nocapture`: 18 passed
- `cargo test -p nepl-core compiler::tests::resource_owner_gate -- --nocapture`: 3 passed
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\agent1-owner-flow-split-move-effect.json -j 1`: total=110 passed=110
- `node nodesrc\issues.js check`: pass
- `git diff --check`: pass

## 対応結果

`nepl-core/src/resource/owner_flow.rs` を追加し、`ResourceOwnerCheckEngine` の owner transfer、return summary application、construct field owner movement、overwrite leak reporting、dealloc/realloc release、live owner diagnostics、raw alias / storage origin からの owner place resolution を移した。

`owner_check.rs` は function/block/op traversal と state table orchestration に集中し、`owner_flow.rs` が owner state mutation の意味論を担当する。Source policy には `owner_flow.rs` の存在と 620 行上限を追加した。分離後は `owner_check.rs` が 447 行、`owner_flow.rs` が 492 行となり、Stage 4 の Resource owner checker 境界を回復した。
