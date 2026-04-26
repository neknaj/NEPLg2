---
id: ISS-20260426T081514183Z-NM-RENDERER-DIVERGES-FROM-GLOSS-HTML-10480257
title: "nm renderer diverges from Gloss HTML and escape contract"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, tests/stdlib/nm.n.md"
---

# ISS-20260426T081514183Z-NM-RENDERER-DIVERGES-FROM-GLOSS-HTML-10480257: nm renderer diverges from Gloss HTML and escape contract

## 概要

The NEPL nm stdlib is intended to support the gloss/nm dialect, but the renderer outputs ruby without nm-ruby/rb markup, annotation gloss as generic span elements, section class nest instead of nm-sec, and HTML escaping omits apostrophe. Parser JSON escaping also keeps a hand-written finite decision tree that is easy to drift from the text escape contract.

## 対象

- `stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, tests/stdlib/nm.n.md`

## 根拠

- 未記入

## 問題

The NEPL nm stdlib is intended to support the gloss/nm dialect, but the renderer outputs ruby without nm-ruby/rb markup, annotation gloss as generic span elements, section class nest instead of nm-sec, and HTML escaping omits apostrophe. Parser JSON escaping also keeps a hand-written finite decision tree that is easy to drift from the text escape contract.

## 影響

Generated HTML cannot be styled or compared with the reference Gloss output, and weak regression tests allow renderer/parser drift to recur.

## 修正方針

Align html_gen output with Gloss ruby/annotation/section markup, classify escape decisions through enums and match expressions, and add exact regression tests for ruby, annotation, section class, and escaping.

## 検証

Run focused nm doctests and the stdlib suite; inspect JSON output for pass counts.
