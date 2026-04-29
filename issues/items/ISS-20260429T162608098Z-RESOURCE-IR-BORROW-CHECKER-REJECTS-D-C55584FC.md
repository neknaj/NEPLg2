---
id: ISS-20260429T162608098Z-RESOURCE-IR-BORROW-CHECKER-REJECTS-D-C55584FC
title: "Resource IR borrow checker rejects drop overwrite fixture"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource, tests/compiler/drop_overwrite.n.md"
---

# ISS-20260429T162608098Z-RESOURCE-IR-BORROW-CHECKER-REJECTS-D-C55584FC: Resource IR borrow checker rejects drop overwrite fixture

## 概要

On current main after MemPtr explicit Clone merge, node nodesrc/tests.js -i tests/compiler/drop_overwrite.n.md --no-tree -o tmp/memptr-explicit-clone-drop-overwrite-after-merge.json -j 1 --dist web/dist reports total=1 failed=1 with resource.borrow.assign_during_shared. The conflict is Assign on local g while a Shared borrow is counted as active. This is outside the MemPtr explicit Clone path and appears to be a Resource IR borrow-lifetime regression.

## 対象

- `nepl-core/src/resource, tests/compiler/drop_overwrite.n.md`

## 根拠

- 未記入

## 問題

On current main after MemPtr explicit Clone merge, node nodesrc/tests.js -i tests/compiler/drop_overwrite.n.md --no-tree -o tmp/memptr-explicit-clone-drop-overwrite-after-merge.json -j 1 --dist web/dist reports total=1 failed=1 with resource.borrow.assign_during_shared. The conflict is Assign on local g while a Shared borrow is counted as active. This is outside the MemPtr explicit Clone path and appears to be a Resource IR borrow-lifetime regression.

## 影響

The drop overwrite fixture is a focused regression guard for overwrite/drop behavior. Rejecting it means the Resource IR borrow checker may be retaining shared borrows too long, which can block valid code and undermine confidence in the static memory safety gate.

## 修正方針

Trace Resource IR borrow activation and release around addr-of/deref and overwrite in the drop_overwrite fixture. Keep assign_during_shared strict for real aliasing, but ensure borrow scopes end at the correct expression boundary before a later assignment is checked.

## 検証

node nodesrc/tests.js -i tests/compiler/drop_overwrite.n.md --no-tree -o tmp/drop-overwrite-borrow-regression.json -j 1 --dist web/dist; cargo test -p nepl-core --test drop_overwrite -- --nocapture; then node nodesrc/tests.js -i tests/compiler --no-tree -o tmp/compiler-after-drop-overwrite-borrow.json -j 4 --dist web/dist
