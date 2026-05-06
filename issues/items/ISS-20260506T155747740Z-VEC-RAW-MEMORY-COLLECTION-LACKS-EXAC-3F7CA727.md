---
id: ISS-20260506T155747740Z-VEC-RAW-MEMORY-COLLECTION-LACKS-EXAC-3F7CA727
title: "Vec raw-memory collection lacks exact loader effect boundary"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/loader.rs, stdlib/alloc/collections/vec.nepl, stdlib/neplg2/core/infra/text.nepl, stdlib/neplg2/core/resolve/name_resolver.nepl"
---

# ISS-20260506T155747740Z-VEC-RAW-MEMORY-COLLECTION-LACKS-EXAC-3F7CA727: Vec raw-memory collection lacks exact loader effect boundary

## 概要

Focused selfhost doctests that import Vec now fail in compile phase with effect.pure.calls_impure because alloc/collections/vec.nepl uses load/store raw memory operations but the loader exact raw-memory boundary table does not grant that configured stdlib path.

## 対象

- `nepl-core/src/loader.rs, stdlib/alloc/collections/vec.nepl, stdlib/neplg2/core/infra/text.nepl, stdlib/neplg2/core/resolve/name_resolver.nepl`

## 根拠

- 未記入

## 問題

Focused selfhost doctests that import Vec now fail in compile phase with effect.pure.calls_impure because alloc/collections/vec.nepl uses load/store raw memory operations but the loader exact raw-memory boundary table does not grant that configured stdlib path.

## 影響

Selfhost modules using Vec cannot be validated under mandatory static checks; the failure masks later owner/type diagnostics and encourages weakening the effect checker instead of declaring the stdlib raw-memory boundary explicitly.

## 修正方針

Audit Vec as an internal raw-memory-backed collection module, add only the configured stdlib exact path if this remains the approved Stage 6 design, and add Rust/source-policy regressions so future collection splits update the table deliberately.

## 検証

Run cargo effect tests, trunk build, and focused selfhost doctests for source_text/name_resolver after the Vec boundary decision.
