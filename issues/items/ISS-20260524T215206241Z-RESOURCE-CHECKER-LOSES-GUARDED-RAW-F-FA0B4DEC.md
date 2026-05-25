---
id: ISS-20260524T215206241Z-RESOURCE-CHECKER-LOSES-GUARDED-RAW-F-FA0B4DEC
title: "Resource checker loses guarded raw fill range after stream scanner calls"
area: core
status: open
resolved: false
priority: P2
type: bug
created: 2026-05-24
updated: 2026-05-24
target: nepl-core/src/resource
---

# ISS-20260524T215206241Z-RESOURCE-CHECKER-LOSES-GUARDED-RAW-F-FA0B4DEC: Resource checker loses guarded raw fill range after stream scanner calls

## 概要

A raw prefix-sum fixture that fills an i32 buffer and guards every dynamic load still fails after StreamScanner read calls with RawMemoryLoadCell PossiblyMoved on pref + symbolic offset. The same user-facing KP fixture should use Vec, but the raw proof precision gap remains relevant for compiler-owned raw boundary tests.

## 対象

- `nepl-core/src/resource`

## 根拠

- 未記入

## 問題

A raw prefix-sum fixture that fills an i32 buffer and guards every dynamic load still fails after StreamScanner read calls with RawMemoryLoadCell PossiblyMoved on pref + symbolic offset. The same user-facing KP fixture should use Vec, but the raw proof precision gap remains relevant for compiler-owned raw boundary tests.

## 影響

Future raw-memory boundary code can be pushed toward weakening RawMemoryLoadCell or avoiding valid guarded layouts when stdlib scanner calls and loop dynamic stores are present in the same function.

## 修正方針

Create a smaller compiler-owned raw boundary regression and preserve initialized fill_i32 element-range evidence across StreamScanner call summaries and loop/path merges without accepting unguarded raw loads.

## 検証

Run the new focused Resource IR regression, the existing word-fill guarded and unguarded tests, and cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 -- --nocapture.
