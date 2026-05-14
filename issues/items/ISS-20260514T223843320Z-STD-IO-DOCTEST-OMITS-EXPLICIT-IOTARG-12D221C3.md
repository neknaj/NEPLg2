---
id: ISS-20260514T223843320Z-STD-IO-DOCTEST-OMITS-EXPLICIT-IOTARG-12D221C3
title: "std/io doctest omits explicit iotarget import"
area: stdlib
status: open
resolved: false
priority: P1
type: test
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/std/io.nepl, stdlib/std/iotarget.nepl"
---

# ISS-20260514T223843320Z-STD-IO-DOCTEST-OMITS-EXPLICIT-IOTARG-12D221C3: std/io doctest omits explicit iotarget import

## 概要

The std/io facade doctest imports std/io and core/result, then constructs WriteStream::Stdio directly. WriteStream is defined in std/iotarget and is not exported by std/io, so the doctest fails with resolve.identifier.undefined when run as a module doctest.

## 対象

- `stdlib/std/io.nepl, stdlib/std/iotarget.nepl`

## 根拠

- 未記入

## 問題

The std/io facade doctest imports std/io and core/result, then constructs WriteStream::Stdio directly. WriteStream is defined in std/iotarget and is not exported by std/io, so the doctest fails with resolve.identifier.undefined when run as a module doctest.

## 影響

Focused std/io verification fails for an import drift that is unrelated to the checked text conversion boundary. This obscures real std/io regressions and leaves documentation examples inaccurate about where target enums come from.

## 修正方針

Either make std/io intentionally re-export the target enum surface, or update the doctest to import std/iotarget explicitly after confirming the facade design. Do not add implicit raw or unrelated stdlib exports.

## 検証

Run stdlib/std/io.nepl focused doctest and issue validation.
