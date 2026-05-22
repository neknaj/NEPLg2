---
id: ISS-20260522T023831896Z-COLLECTION-SLOT-RELEASE-ALIAS-MODULE-0B5B1690
title: "Collection slot release alias module exceeds responsibility limit"
area: core
status: open
resolved: false
priority: P2
type: maintenance
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/resource/collection_slot_state_release_alias.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260522T023831896Z-COLLECTION-SLOT-RELEASE-ALIAS-MODULE-0B5B1690: Collection slot release alias module exceeds responsibility limit

## 概要

After registering the new summary projection module, the resource responsibility monitor reaches collection_slot_state_release_alias.rs and fails because the module has 130 lines while its split limit is 120.

## 対象

- `nepl-core/src/resource/collection_slot_state_release_alias.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After registering the new summary projection module, the resource responsibility monitor reaches collection_slot_state_release_alias.rs and fails because the module has 130 lines while its split limit is 120.

## 影響

The resource checker responsibility gate cannot pass on current main, so future static-check refactors lose an automated signal for release/dealloc proof module growth.

## 修正方針

Split collection_slot_state_release_alias.rs by release-state query versus release-state mutation, or otherwise reduce the module below its current responsibility budget without raising the limit.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js after the split, plus cargo check -p nepl-core and issue validation.
