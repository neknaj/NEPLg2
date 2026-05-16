---
id: ISS-20260516T033336255Z-PLAYGROUND-EDITOR-ANALYSIS-FIXTURES--7D947BA4
title: "playground editor analysis fixtures are stale after source import expansion"
area: tools
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: "tests/playground_editor/analysis_payload_basic/**, tests/playground_editor/analysis_hover_definition/**, nodesrc/playground_editor_test_runner.js"
---

# ISS-20260516T033336255Z-PLAYGROUND-EDITOR-ANALYSIS-FIXTURES--7D947BA4: playground editor analysis fixtures are stale after source import expansion

## 概要

After trunk build, node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-source-capability-proof-walk.json reports 11/13 passed. analysis_payload_basic and analysis_hover_definition fail because source.nepl includes an import line while expected.json still expects offsets and hover/definition results for the old shorter source. The same stale fixture content exists on origin/main, so this is not caused by the source capability proof walker change.

## 対象

- `tests/playground_editor/analysis_payload_basic/**, tests/playground_editor/analysis_hover_definition/**, nodesrc/playground_editor_test_runner.js`

## 根拠

- 未記入

## 問題

After trunk build, node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-source-capability-proof-walk.json reports 11/13 passed. analysis_payload_basic and analysis_hover_definition fail because source.nepl includes an import line while expected.json still expects offsets and hover/definition results for the old shorter source. The same stale fixture content exists on origin/main, so this is not caused by the source capability proof walker change.

## 影響

The required post-trunk nodesrc/cli.js JSON check cannot be used as a reliable local gate. It also hides real playground editor analysis regressions behind expected fixture drift.

## 修正方針

Refresh the affected analysis fixtures from the current source.nepl and analysis.json contract, or redesign the fixture runner so analysis.json/source.nepl/expected.json are generated from a single canonical source. Preserve CRLF normalization and keep JSON assertions deterministic.

## 検証

Run trunk build and node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-analysis-fixtures.json, then confirm caseCount=13, passedCount=13, failedCount=0.
