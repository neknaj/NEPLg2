---
id: ISS-20260507T185938110Z-RESOURCE-OWNER-RETURN-MODULE-EXCEEDS-40B2E737
title: "Resource owner_return module exceeds responsibility split limit after unknown callback fix"
area: resource
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/owner_return.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T185938110Z-RESOURCE-OWNER-RETURN-MODULE-EXCEEDS-40B2E737: Resource owner_return module exceeds responsibility split limit after unknown callback fix

## 概要

After syncing remote main bf95da31, node nodesrc/test_resource_checker_responsibility.js reports owner_return.rs has 240 lines while the responsibility split limit is 220. The unknown callback non-owning view fix added policy-sensitive return selection logic back into the owner_return orchestration file.

## 対象

- `nepl-core/src/resource/owner_return.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `owner_return.rs has 240 lines; responsibility split limit is 220` で失敗する。
- `nepl-core/src/resource/owner_return.rs` は owner return orchestration を担う memory-safety-critical module であり、責務分割 policy では 220 lines を上限としている。
- `bf95da31` の unknown callback non-owning view 修正後、unknown callback return fallback と non-owning candidate handling が orchestration file に追加され、policy 上の分割境界を再び超えた。

## 問題

After syncing remote main bf95da31, node nodesrc/test_resource_checker_responsibility.js reports owner_return.rs has 240 lines while the responsibility split limit is 220. The unknown callback non-owning view fix added policy-sensitive return selection logic back into the owner_return orchestration file.

## 影響

ResourceIR owner return logic is a memory-safety-critical static checker boundary. If owner-return orchestration, unknown callback fallback, and non-owning view candidate handling keep accumulating in one file, future changes can bypass enum/match-oriented review boundaries and hide ownership transfer mistakes.

## 修正方針

Split the unknown callback return candidate selection or adjacent helper responsibility out of owner_return.rs into a dedicated module without loosening the source-policy limit. Keep owner_return.rs as orchestration, preserve exhaustive OwnerState handling, and add regression coverage for the module boundary.

## 検証

node nodesrc/test_resource_checker_responsibility.js; cargo test -p nepl-core --test resource_ir unknown_callback -- --nocapture; cargo test -p nepl-core --test resource_ir -- --nocapture; trunk build
