---
id: ISS-20260429T030655089Z-RESOURCE-OWNER-GATE-REGRESSES-NM-DIR-98E651E0
title: "Resource owner gate regresses nm direct serializer fixtures with D3100"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource, stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, tests/stdlib/nm.n.md"
---

# ISS-20260429T030655089Z-RESOURCE-OWNER-GATE-REGRESSES-NM-DIR-98E651E0: Resource owner gate regresses nm direct serializer fixtures with D3100

## 概要

Current tests/stdlib/nm.n.md fails 5/5 in compile phase with D3100 owner obligation leaks in document_to_json and nm_inline_to_html, even though the nm direct serializer had previously passed after raw-memory detours were removed.

## 対象

- `nepl-core/src/resource, stdlib/nm/parser.nepl, stdlib/nm/html_gen.nepl, tests/stdlib/nm.n.md`

## 根拠

- `node nodesrc/tests.js -i tests\stdlib\nm.n.md --no-tree -o tmp\byte-scanner-nm.json -j 1` は 5 件すべて compile phase で失敗。
- top issue は `document_to_json__Document__str__pure` の `D3100 resource ir owner obligation may leak` と、`nm_inline_to_html__str__str__pure` の `D3100 ... found MaybeFreed`。
- `ISS-20260428T003718356Z-NM-PARSER-AND-HTML-GEN-RAW-MEMORY-DE-99175378` では direct serializer 化後に nm suite が 10/10 passed していたため、現在の Resource owner gate の残件または回帰として切り分ける。

## 問題

Current tests/stdlib/nm.n.md fails 5/5 in compile phase with D3100 owner obligation leaks in document_to_json and nm_inline_to_html, even though the nm direct serializer had previously passed after raw-memory detours were removed.

## 影響

NM parser/html_gen refactors cannot be behaviorally verified, and documentation/self-host markdown rendering regressions can be hidden behind Resource IR owner diagnostics.

## 修正方針

Trace Resource IR owner flow for StringBuilder/direct serializer temporaries in document_to_json and nm_inline_to_html without weakening D3100. Reopen behavioral nm fixtures once owner flow is corrected.

## 検証

node nodesrc/tests.js -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-resource-owner-fixed.json -j 1; node nodesrc/tests.js -i stdlib/nm/parser.nepl -i stdlib/nm/html_gen.nepl -i tests/stdlib/nm.n.md --no-tree -o tmp/nm-resource-owner-suite-fixed.json -j 1
