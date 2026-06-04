---
id: ISS-20260604T034125917Z-CHAR-UTF-8-BYTE-ACCESSORS-RELY-ON-CA-AC31C3D4
title: "char UTF-8 byte accessors rely on caller length checks instead of typed absence"
area: stdlib
status: open
resolved: false
priority: P2
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: stdlib/core/char.nepl
---

# ISS-20260604T034125917Z-CHAR-UTF-8-BYTE-ACCESSORS-RELY-ON-CA-AC31C3D4: char UTF-8 byte accessors rely on caller length checks instead of typed absence

## 概要

Subagent audit found char_utf8_byte1/2/3 requiring callers to know byte length before access, rather than returning Option/Result for absent bytes. This conflicts with Zenn guidance to express nullable/absent states through Option and keep invalid state out of ordinary values.

## 対象

- `stdlib/core/char.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found char_utf8_byte1/2/3 requiring callers to know byte length before access, rather than returning Option/Result for absent bytes. This conflicts with Zenn guidance to express nullable/absent states through Option and keep invalid state out of ordinary values.

## 影響

Callers can accidentally read byte positions that do not exist for ASCII or shorter UTF-8 encodings, and the static checker cannot force match coverage for absent bytes.

## 修正方針

Introduce char_utf8_byte_at returning Option i32 or an encoded UTF-8 struct with explicit length, and make raw byteN helpers private/internal or documented precondition-only helpers.

## 検証

Add doctests and regular tests for ASCII, 2-byte, 3-byte, invalid index, and boundary access.
