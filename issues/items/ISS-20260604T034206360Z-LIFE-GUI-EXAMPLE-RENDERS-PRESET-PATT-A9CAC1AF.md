---
id: ISS-20260604T034206360Z-LIFE-GUI-EXAMPLE-RENDERS-PRESET-PATT-A9CAC1AF
title: "Life GUI example renders preset patterns instead of a pure Conway board model"
area: examples
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-05
target: examples/gui_life.nepl
---

# ISS-20260604T034206360Z-LIFE-GUI-EXAMPLE-RENDERS-PRESET-PATT-A9CAC1AF: Life GUI example renders preset patterns instead of a pure Conway board model

## 概要

Subagent audit found gui_life.nepl modeling step/animate/cell_size while glider and blinker are drawn directly. The Life rules were not represented as Model + Event -> Model state transition logic. This conflicted with the Elm Architecture direction and Zenn pure core / explicit state guidance.

## 対象

- `examples/gui_life.nepl`

## 根拠

- `examples/gui_life.nepl` は `life_present_glider_phase*` / `life_present_blinker` / `life_present_patterns` で live cell を描画しており、board storage と neighbour count を持っていなかった。
- 現行 resource checker は owner-backed aggregate を user source の struct に埋め込むことを禁止するため、`BitSet` を `LifeModel` に直接入れず、event loop が裸の board owner として保持する必要がある。

## 問題

`gui_life.nepl` は `step` と hardcoded pattern phase だけで描画を変えており、Conway の cell state、neighbour count、next generation を model/update へ持っていなかった。

## 影響

Next Step and Animate could not be trusted as tests of the actual Game of Life algorithm, and the example could not scale to arbitrary board sizes or HD cell layouts without rewriting the model.

## 修正方針

Fixed by rewriting `examples/gui_life.nepl` so that:

- `LifeModel` keeps Copy scalar TEA state: `generation`, `animate`, `cell_size`.
- board storage is a real NEPL `BitSet` owner held explicitly by the event loop, avoiding user-defined owner-backed aggregate structs.
- `LifeCellState` and `life_cell_next_state` encode Conway rules with enum + `match`.
- `life_board_neighbor_count` and `life_board_next_generation` derive the next board from the stored board.
- animation advances only from `GuiEvent::Timer` and `gui_web_stdout_animation_timer`, not from timeout `Option::None`.
- source policy `nodesrc/test_web_gui_life_model_contract.js` prevents regression to hardcoded pattern rendering or owner-backed aggregate board structs.

## 検証

- `node nodesrc/test_web_gui_life_model_contract.js`
- `node nodesrc/run_doctest.js -i examples/gui_life.nepl -n 1`
- `node nodesrc/tests.js -i examples/gui_life.nepl --no-tree -o tmp/agent2-life-gui-board-model-after-review.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/issues.js index --dir issues && node nodesrc/issues.js check --dir issues`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/agent2-life-gui-board-playground-editor-after-review.json`
