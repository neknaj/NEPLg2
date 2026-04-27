---
id: ISS-20260427T000313426Z-LIST-REVERSE-HIDES-ALLOCATION-FAILUR-E4B68FAA
title: "List reverse hides allocation failure behind unsafe unwraps"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/list.nepl, tests/stdlib/list_collections.n.md"
---

# ISS-20260427T000313426Z-LIST-REVERSE-HIDES-ALLOCATION-FAILUR-E4B68FAA: List reverse hides allocation failure behind unsafe unwraps

## 概要

List.reverse allocates a new list with unwrap_ok/uwok even though cons/new can fail, so a normal allocation-bearing operation has no Result surface.

## 対象

- `stdlib/alloc/collections/list.nepl, tests/stdlib/list_collections.n.md`

## 根拠

- 未記入

## 問題

List.reverse allocates a new list with unwrap_ok/uwok even though cons/new can fail, so a normal allocation-bearing operation has no Result surface.

## 影響

Self-host linked-list utilities can trap on allocation pressure and the API shape hides a real failure mode from callers.

## 修正方針

Add a Result-returning reverse variant or change reverse to return Result if compatible, update callers/tests, and add a regression that prevents unsafe helpers in List implementation allocation paths.

## 検証

Run List doctests, focused collection tests, source guard, stdlib suite, and nodesrc/issues.js check.
