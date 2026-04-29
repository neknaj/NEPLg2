---
id: ISS-20260429T020330179Z-RESOURCE-OWNER-CHECKER-EXCEEDS-RESPO-AB6E0E0E
title: "resource owner checker exceeds responsibility split limit"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_raw_memory.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260429T020330179Z-RESOURCE-OWNER-CHECKER-EXCEEDS-RESPO-AB6E0E0E: resource owner checker exceeds responsibility split limit

## 概要

After fixing the resource checker responsibility policy import detection, the same Source policy test fails because nepl-core/src/resource/owner_check.rs has grown to 930 lines over the 800-line responsibility split limit. Owner checking now mixes traversal, diagnostics, owner transfer, storage-origin propagation, raw alias lookup, and raw memory operation handling.

## 対象

- `nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_raw_memory.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After fixing the resource checker responsibility policy import detection, the same Source policy test fails because nepl-core/src/resource/owner_check.rs has grown to 930 lines over the 800-line responsibility split limit. Owner checking now mixes traversal, diagnostics, owner transfer, storage-origin propagation, raw alias lookup, and raw memory operation handling.

## 影響

GitHub Actions Source policy regressions remain red, and Stage 4 Resource IR owner checking is accumulating raw-memory and storage-origin responsibilities in the main owner checker instead of keeping a maintainable boundary.

## 修正方針

Split raw memory/storage-origin specific owner operations out of owner_check.rs into a dedicated resource module while preserving diagnostics and owner semantics. Keep the 800-line owner_check limit rather than raising it.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, focused resource owner tests, cargo check -p nepl-core --tests, node nodesrc/issues.js check, and git diff --check.
