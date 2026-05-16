---
id: ISS-20260516T033336255Z-PLAYGROUND-EDITOR-ANALYSIS-FIXTURES--7D947BA4
title: "playground editor analysis fixtures are stale after source import expansion"
area: tools
status: fixed
resolved: true
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

- `analysis_payload_basic/source.nepl` と `analysis_hover_definition/source.nepl` は `#import "core/math" as *` を含む現在の source になっていた。
- 一方で両 case の `analysis.json` は import 追加前の token span / definition span / reference span を保持していた。
- `analysis_hover_definition/requests.json` は旧 source の `add` 参照位置 `22` を指しており、現在の source では import 行の内部を指すため hover / definition / occurrence がすべて `null` / `[]` になっていた。
- `analysis_payload_basic/expected.json` は旧 source の診断位置・folding range・semantic token / inlay hint 位置を期待していた。

## 問題

After trunk build, node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-source-capability-proof-walk.json reports 11/13 passed. analysis_payload_basic and analysis_hover_definition fail because source.nepl includes an import line while expected.json still expects offsets and hover/definition results for the old shorter source. The same stale fixture content exists on origin/main, so this is not caused by the source capability proof walker change.

## 影響

The required post-trunk nodesrc/cli.js JSON check cannot be used as a reliable local gate. It also hides real playground editor analysis regressions behind expected fixture drift.

## 修正方針

Refresh the affected analysis fixtures from the current source.nepl and analysis.json contract, or redesign the fixture runner so analysis.json/source.nepl/expected.json are generated from a single canonical source. Preserve CRLF normalization and keep JSON assertions deterministic.

## 検証

Run trunk build and node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-analysis-fixtures.json, then confirm caseCount=13, passedCount=13, failedCount=0.

## 2026-05-16 Agent 1 修正

`analysis_payload_basic` と `analysis_hover_definition` の `analysis.json` を、現在の `source.nepl` の import 行を含む span へ揃えた。`DirImport` token を追加し、`fn add`、`let x add`、unused variable diagnostic、semantic token / inlay hint の位置を現在の source offset / line / column に更新した。

`analysis_hover_definition/requests.json` は、旧 offset `22` ではなく現在の `add` 参照上の offset `47` を指すように更新した。`expected.json` は更新後の `analysis.json` と request を入力にした playground editor runner 出力から再生成し、definition target も現在の `fn add` definition offset `28` に揃えた。

検証:

- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-after.json`: `13/13 passed`
- `trunk build`: passed
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-after-trunk.json`: `13/13 passed`
