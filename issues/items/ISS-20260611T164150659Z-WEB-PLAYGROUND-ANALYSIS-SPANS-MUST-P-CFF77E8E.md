---
id: ISS-20260611T164150659Z-WEB-PLAYGROUND-ANALYSIS-SPANS-MUST-P-CFF77E8E
title: "Web Playground analysis spans must preserve file identity and character offset mapping"
area: tools
status: open
resolved: false
priority: P0
type: bug
created: 2026-06-11
updated: 2026-06-11
target: "nepl-web/src/lib.rs; web/src/editor-core/language-analysis.ts; web/src/language/neplg2/neplg2-provider.ts"
---

# ISS-20260611T164150659Z-WEB-PLAYGROUND-ANALYSIS-SPANS-MUST-P-CFF77E8E: Web Playground analysis spans must preserve file identity and character offset mapping

## 概要

The Rust analysis payload can contain byte-based spans and file_path-bearing spans, but the TypeScript payload builder maps semantic expression spans and cross-file diagnostics/definitions as if they belong to the active editor text.

## 対象

- `nepl-web/src/lib.rs; web/src/editor-core/language-analysis.ts; web/src/language/neplg2/neplg2-provider.ts`

## 根拠

- 未記入

## 問題

The Rust analysis payload can contain byte-based spans and file_path-bearing spans, but the TypeScript payload builder maps semantic expression spans and cross-file diagnostics/definitions as if they belong to the active editor text.

## 影響

Non-ASCII source shifts inlay/hover ranges, and diagnostics or definitions from imported files can be underlined or navigated at wrong positions in the active file.

## 修正方針

Normalize all spans through a shared active-file-aware span mapper. Editor-local diagnostics/tokens must be filtered by active path, while definition targets should carry path plus target range.

## 検証

Add non-ASCII and cross-file import playground fixtures. Active editor must not display imported-file diagnostics at local offsets, and definition targets must preserve destination path.
