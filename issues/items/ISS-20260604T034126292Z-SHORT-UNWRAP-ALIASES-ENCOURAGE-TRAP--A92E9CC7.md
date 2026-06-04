---
id: ISS-20260604T034126292Z-SHORT-UNWRAP-ALIASES-ENCOURAGE-TRAP--A92E9CC7
title: "short unwrap aliases encourage trap-based Result handling in public examples"
area: stdlib
status: open
resolved: false
priority: P3
type: maintenance
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/core/result.nepl, stdlib/**"
---

# ISS-20260604T034126292Z-SHORT-UNWRAP-ALIASES-ENCOURAGE-TRAP--A92E9CC7: short unwrap aliases encourage trap-based Result handling in public examples

## 概要

Subagent audit found public short aliases such as uwok and uwerr around unwrap_ok/unwrap_err. Zenn guidance emphasizes Result + match and explicit failure handling; short trap aliases make failure handling easy to hide in examples and stdlib code.

## 対象

- `stdlib/core/result.nepl, stdlib/**`

## 根拠

- 未記入

## 問題

Subagent audit found public short aliases such as uwok and uwerr around unwrap_ok/unwrap_err. Zenn guidance emphasizes Result + match and explicit failure handling; short trap aliases make failure handling easy to hide in examples and stdlib code.

## 影響

Examples and docs can normalize trap-based Result handling instead of enum/match handling, making error contracts less visible and reducing the value of static checking.

## 修正方針

Move short unwrap aliases to test/internal or unsafe-style namespace, keep full names heavily documented, and prefer match or unwrap_or in public docs/examples.

## 検証

Add source policy that stdlib implementation and examples do not use short unwrap aliases outside approved tests.
