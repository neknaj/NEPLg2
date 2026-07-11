---
id: ISS-20260711T215752529Z-DEEP-RESULT-OWNER-RETURN-PROJECTION--1A86CA9D
title: "Deep Result owner return projection reuses moved payload"
area: RESOURCE
status: open
resolved: false
priority: P1
type: bug
created: 2026-07-11
updated: 2026-07-11
target: nepl-core/src/resource/owner_return_apply_projection.rs
---

# ISS-20260711T215752529Z-DEEP-RESULT-OWNER-RETURN-PROJECTION--1A86CA9D: Deep Result owner return projection reuses moved payload

## 概要

A caller that consumes a deeply nested move-only registered owner through Result<BudgetStep, StepError> is rejected at the call expression with resource.owner.use_after_move on ReturnValue projections even though every branch consumes or frees the owner exactly once. F5nxj registered path command sink runtime reproduction fails first at writer_step_budget(recovered_writer, 0).

## 対象

- `nepl-core/src/resource/owner_return_apply_projection.rs`

## 根拠

- 未記入

## 問題

A caller that consumes a deeply nested move-only registered owner through Result<BudgetStep, StepError> is rejected at the call expression with resource.owner.use_after_move on ReturnValue projections even though every branch consumes or frees the owner exactly once. F5nxj registered path command sink runtime reproduction fails first at writer_step_budget(recovered_writer, 0).

## 影響

Valid production owner chains cannot be exercised by runtime fixtures; F5nxj integration is blocked despite normal compile and source-policy gates passing.

## 修正方針

Add a minimized nested Result aggregate reproduction, correct ReturnValue projection summary application so moved input payload leaves do not get reprojected onto returned owner leaves, and preserve genuine use-after-move diagnostics.

## 検証

Run the minimized Resource IR regression and tests/stdlib/gui_font_registered_face.n.md with the F5nxj controlled 8-command runtime contract: read retry, zero/negative budget, partial seal, eight writes, terminal completion, checked seal, and cleanup-only push failure.
