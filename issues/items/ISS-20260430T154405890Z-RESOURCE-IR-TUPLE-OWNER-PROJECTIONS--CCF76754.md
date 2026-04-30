---
id: ISS-20260430T154405890Z-RESOURCE-IR-TUPLE-OWNER-PROJECTIONS--CCF76754
title: "Resource IR tuple owner projections leak after aggregate field extraction"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource, tests/compiler/overload.n.md"
---

# ISS-20260430T154405890Z-RESOURCE-IR-TUPLE-OWNER-PROJECTIONS--CCF76754: Resource IR tuple owner projections leak after aggregate field extraction

## 概要

tests/compiler/overload.n.md::doctest#10 still fails with resource.owner.leak for `parts` tuple field owner projections after extracting Vec fields from an aggregate result.

## 対象

- `nepl-core/src/resource, tests/compiler/overload.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/overload-owner-pipeline-agent1.json -j 1 --dist web/dist` で `doctest#10` が `resource.owner.leak` を返した。
- diagnostic は `Local("parts")` の tuple field `0` / `1` 以下に残る `Vec` owner projection が `StorageId(0)` / `StorageId(1)` を保持したまま関数終了することを示している。

## 問題

tests/compiler/overload.n.md::doctest#10 still fails with resource.owner.leak for `parts` tuple field owner projections after extracting Vec fields from an aggregate result.

## 影響

Compiler fixture remains failing under strict Resource IR owner checking. The leak must be solved by precise tuple/field owner transfer, not by weakening owner diagnostics.

## 修正方針

Review aggregate field extraction and tuple projection owner transfer so moving fields out of tuple-like results retires the original tuple field obligations exactly once.

## 検証

node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/overload-tuple-owner.json -j 1 --dist web/dist
