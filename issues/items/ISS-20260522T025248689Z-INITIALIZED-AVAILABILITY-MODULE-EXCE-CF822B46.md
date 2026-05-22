---
id: ISS-20260522T025248689Z-INITIALIZED-AVAILABILITY-MODULE-EXCE-CF822B46
title: "initialized availability module exceeds responsibility limit"
area: core
status: open
resolved: false
priority: P2
type: maintenance
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/resource/initialized_availability.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260522T025248689Z-INITIALIZED-AVAILABILITY-MODULE-EXCE-CF822B46: initialized availability module exceeds responsibility limit

## 概要

After splitting collection_slot_state_release_alias.rs, the resource responsibility monitor reaches initialized_availability.rs and fails because the module has 173 lines while its split limit is 120. The module mixes argument availability orchestration, by-value consumption, unavailable diagnostics, and collection-slot certified raw-cell acceptance helpers.

## 対象

- `nepl-core/src/resource/initialized_availability.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After splitting collection_slot_state_release_alias.rs, the resource responsibility monitor reaches initialized_availability.rs and fails because the module has 173 lines while its split limit is 120. The module mixes argument availability orchestration, by-value consumption, unavailable diagnostics, and collection-slot certified raw-cell acceptance helpers.

## 影響

The resource checker responsibility gate remains blocked after the collection-slot release alias split, so future static-check changes lose an automated signal for initialized availability module growth.

## 修正方針

Split initialized_availability.rs into focused modules for argument availability/consumption and collection-slot certified raw-cell acceptance, without raising the limit.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js after the split, plus cargo check -p nepl-core and issue validation.
