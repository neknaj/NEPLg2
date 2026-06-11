---
id: ISS-20260611T164150059Z-WEB-PLAYGROUND-SEMANTIC-ANALYSIS-MUS-488205A3
title: "Web Playground semantic analysis must not block the UI thread"
area: tools
status: open
resolved: false
priority: P0
type: performance
created: 2026-06-11
updated: 2026-06-11
target: "web/src/language/neplg2/neplg2-provider.ts; web/src/editor-core/language-analysis.ts; nepl-web/src/lib.rs"
---

# ISS-20260611T164150059Z-WEB-PLAYGROUND-SEMANTIC-ANALYSIS-MUS-488205A3: Web Playground semantic analysis must not block the UI thread

## 概要

NEPLg2LanguageProvider debounces and idles analysis scheduling, but analyze_semantics/analyze_semantics_with_vfs and structural parse still run as synchronous WASM calls on the UI thread. Stale guards prevent publishing old payloads, but they do not prevent obsolete analysis from blocking file open and key input once started.

## 対象

- `web/src/language/neplg2/neplg2-provider.ts; web/src/editor-core/language-analysis.ts; nepl-web/src/lib.rs`

## 根拠

- 未記入

## 問題

NEPLg2LanguageProvider debounces and idles analysis scheduling, but analyze_semantics/analyze_semantics_with_vfs and structural parse still run as synchronous WASM calls on the UI thread. Stale guards prevent publishing old payloads, but they do not prevent obsolete analysis from blocking file open and key input once started.

## 影響

Large files and VFS-backed analysis can still freeze editor input after the debounce fires. Hover can also force a synchronous structural parse and bypass the intended idle scheduling.

## 修正方針

Move semantic and structural analysis to a dedicated worker protocol with document path/version/VFS revision in requests and responses. Semantic-derived editor features must use current analyzed snapshots only and reschedule after fresh analysis completes.

## 検証

Add large-source editor tests proving continuous key input does not invoke synchronous wasm analysis on the UI thread, stale worker responses are dropped, and hover before structural analysis does not synchronously call analyze_parse.
