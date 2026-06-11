---
id: ISS-20260611T164150059Z-WEB-PLAYGROUND-SEMANTIC-ANALYSIS-MUS-488205A3
title: "Web Playground semantic analysis must not block the UI thread"
area: tools
status: fixed
resolved: true
priority: P0
type: performance
created: 2026-06-11
updated: 2026-06-12
target: "web/src/language/neplg2/neplg2-provider.ts; web/src/language/neplg2/neplg2-analysis-worker.ts; web/src/editor-core/language-analysis.ts; web/src/main.ts"
---

# ISS-20260611T164150059Z-WEB-PLAYGROUND-SEMANTIC-ANALYSIS-MUS-488205A3: Web Playground semantic analysis must not block the UI thread

## 概要

NEPLg2LanguageProvider debounces and idles analysis scheduling, but analyze_semantics/analyze_semantics_with_vfs and structural parse still run as synchronous WASM calls on the UI thread. Stale guards prevent publishing old payloads, but they do not prevent obsolete analysis from blocking file open and key input once started.

## 対象

- `web/src/language/neplg2/neplg2-provider.ts; web/src/language/neplg2/neplg2-analysis-worker.ts; web/src/editor-core/language-analysis.ts; web/src/main.ts`

## 根拠

- `NEPLg2LanguageProvider._analyzeAndPublish` が debounce / idle 後に `analyze_semantics` / `analyze_semantics_with_vfs` を main thread で同期実行していた。
- `getHoverInfo` が fresh semantic payload を確認したあとも `_ensureStructuralParse` を呼び、hover 表示で `analyze_parse` を main thread 同期実行できた。
- `panelManager.redraw()` より後に compiler asset metadata を設定していたため、復元タブで provider が作られる時点では worker を初期化できず、同期 fallback に落ち得た。

## 問題

NEPLg2LanguageProvider debounces and idles analysis scheduling, but analyze_semantics/analyze_semantics_with_vfs and structural parse still run as synchronous WASM calls on the UI thread. Stale guards prevent publishing old payloads, but they do not prevent obsolete analysis from blocking file open and key input once started.

## 影響

Large files and VFS-backed analysis can still freeze editor input after the debounce fires. Hover can also force a synchronous structural parse and bypass the intended idle scheduling.

## 修正方針

Move semantic and structural analysis to a dedicated worker protocol with document path/version/VFS revision in requests and responses. Semantic-derived editor features must use current analyzed snapshots only and reschedule after fresh analysis completes.

## 検証

Add large-source editor tests proving continuous key input does not invoke synchronous wasm analysis on the UI thread, stale worker responses are dropped, and hover before structural analysis does not synchronously call analyze_parse.

## 解決

- `web/src/language/neplg2/neplg2-analysis-worker.ts` を追加し、semantic analysis と structural parse を module worker 側で実行するようにした。
- worker request は compiler asset metadata、path、text、VFS snapshot、requestId を持ち、provider は documentVersion / analysisVersion と一致する結果だけを fresh payload として採用する。
- 編集や文書切り替えで active worker request が残っている場合は worker を terminate し、古い response は publish しない。
- `getHoverInfo` は structural parse を同期実行せず、fresh semantic snapshot だけから hover を返す。AST が必要な経路は worker structural parse を schedule し、未完了なら `null` を返す。
- compiler asset metadata を `PlaygroundPanelManager.redraw()` より前に設定し、復元タブで作られる NEPLg2 provider も worker 経路を選べるようにした。

## 解決時の検証

- pass: `npm --prefix web run build:ts`
- pass: `node nodesrc/test_playground_analysis_freshness.js`
- pass: `node nodesrc/test_playground_editor_performance_policy.js`
- pass: `node nodesrc/test_neplg2_language_provider_vfs.js`
