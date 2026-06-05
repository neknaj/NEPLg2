---
id: ISS-20260604T034206714Z-PAINT-GUI-EXAMPLE-USES-FIXED-STROKE--B21EFEC7
title: "Paint GUI example uses fixed stroke slots and sentinel values instead of typed stroke storage"
area: examples
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-05
target: examples/gui_paint.nepl
---

# ISS-20260604T034206714Z-PAINT-GUI-EXAMPLE-USES-FIXED-STROKE--B21EFEC7: Paint GUI example uses fixed stroke slots and sentinel values instead of typed stroke storage

## 概要

Subagent audit found gui_paint.nepl using three stroke slots and a 255 sentinel for missing or inactive values. This conflicts with the Zenn Option/Result/enum guidance and leaves the example closer to pointer-event smoke testing than a paint application.

## 対象

- `examples/gui_paint.nepl`

## 根拠

- `examples/gui_paint.nepl` の `PaintModel` を `slot0/slot1/slot2: Option PaintCell` と `count` で表す形へ変更した。
- `PaintCell` は cell index と描画時点の `PaintColor` を保持するため、stroke 再描画が現在の palette state に依存しない。
- `paint_set_cell_result` / `paint_update_event_result` は `Result PaintModel PaintUpdateError` を返し、capacity overflow と canvas 外 pointer を `PaintUpdateErrorKind` で区別する。
- `paint_present_slot` は `Option::Some` / `Option::None` を `match` し、`255` sentinel と `eq slot 255` による欠如表現を削除した。

## 問題

Subagent audit found gui_paint.nepl using three stroke slots and a 255 sentinel for missing or inactive values. This conflicts with the Zenn Option/Result/enum guidance and leaves the example closer to pointer-event smoke testing than a paint application.

## 影響

Multiple strokes, clear/history, capacity overflow, and pointer cancellation cannot be modeled cleanly, and sentinel state can hide invalid interactions.

## 修正方針

Represent strokes with Option or bounded collection storage, remove sentinel values from the model, and return Result on capacity overflow or invalid pointer state.

## 検証

- `node nodesrc/test_web_gui_paint_model_contract.js`
- `node nodesrc/test_web_gui_shared_event_queue.js`
- `node nodesrc/tests.js -i examples/gui_paint.nepl --no-tree -o tmp/agent2-gui-paint-typed-strokes-tests.json -j 1 --dist web/dist --assert-io`
- `cargo run -p nepl-cli -- --check -i examples/gui_paint.nepl --target std --stdlib-root stdlib`
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/agent2-gui-paint-typed-strokes-playground-editor.json`
- `node nodesrc/tests.js -i examples/gui_paint.nepl --no-tree -o tmp/agent2-gui-paint-typed-strokes-tests-after-trunk.json -j 1 --dist web/dist --assert-io`
