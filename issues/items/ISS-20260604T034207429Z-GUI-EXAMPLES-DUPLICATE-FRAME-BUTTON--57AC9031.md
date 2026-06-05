---
id: ISS-20260604T034207429Z-GUI-EXAMPLES-DUPLICATE-FRAME-BUTTON--57AC9031
title: "GUI examples duplicate text-button stdout emission instead of sharing typed helpers"
area: examples
status: fixed
resolved: true
priority: P2
type: maintenance
created: 2026-06-04
updated: 2026-06-05
target: "examples/gui_*.nepl, stdlib/platforms/gui/web"
---

# ISS-20260604T034207429Z-GUI-EXAMPLES-DUPLICATE-FRAME-BUTTON--57AC9031: GUI examples duplicate text-button stdout emission instead of sharing typed helpers

## 概要

Subagent audit found repeated text-button stdout emission code across GUI examples. The duplicated `fill_rect -> text_run -> action_rect` sequence made protocol ordering drift likely and conflicted with Zenn zero-cost abstraction, struct-based data grouping, `Result`-based error handling, and platform-boundary guidance.

## 対象

- `examples/gui_*.nepl, stdlib/platforms/gui/web`

## 根拠

- Zenn 記事の方針では、本質的に同じ処理は抽象化し、まとめるべきデータは struct にし、失敗は `Result` / enum で扱う。
- `doc/neplg2/gui_standard_library_spec.md` は Web stdout protocol を formal host ABI 前の platform backend detail とし、Canvas / DOM / TS simulation を stdlib/exampless 側へ漏らさない方針を示している。
- subagent review により、application 固有の `ActionId` decode と event loop は example 側に残すべきで、共通化対象は text label を持つ button の stdout protocol emission に絞るべきだと確認した。

## 問題

`examples/gui_counter.nepl`、`examples/gui_life.nepl`、`examples/gui_mandelbrot.nepl`、`examples/gui_calculator.nepl`、`examples/gui_scientific_calculator.nepl`、`examples/gui_paint.nepl`、`examples/gui_breakout.nepl` が、text label 付き button の描画と hit target 出力をそれぞれ手書きしていた。

## 影響

各 example が protocol encoding、hit target handling、`Result` propagation を個別に持つため、Web stdout fallback や将来の formal host ABI に変更が入った時に drift しやすい。

## 修正方針

- `stdlib/platforms/gui/web/stdout_protocol.nepl` に `GuiWebButtonConfig` と `gui_web_stdout_button` を追加する。
- helper は text label 付き button の `fill_rect -> text_run -> action_rect` emission を保持し、example 側は `ActionId`、label、geometry、color だけを渡す。
- invalid rect、invalid action id、invalid text size は stdout へ 1 command も出す前に `GuiError::InvalidGeometry` として返す。
- Paint の palette swatch のような label なし color hit target は button ではないため、直接 `fill_rect` と `action_rect` を使う。
- application 固有の action decode と event loop は example 側に残す。

## 修正内容

- `GuiWebButtonConfig` / `gui_web_button_config` / `gui_web_stdout_button` を追加し、button stdout emission を platform helper に集約した。
- GUI examples の text label 付き button を shared helper 使用へ変更した。
- `nodesrc/test_web_gui_example_button_helper_contract.js` を追加し、examples 内の hand-rolled `fill_rect -> text_run -> action_rect` の再導入を検出する。
- `tests/stdlib/gui_web_stdout_protocol.n.md` を追加し、happy path と invalid config の no-partial-output contract を実行で検査する。
- GUI 仕様書と実装計画に stdout button helper checkpoint を記録した。

## 検証

- `node nodesrc/test_web_gui_example_button_helper_contract.js`
- `node nodesrc/test_web_gui_shared_event_queue.js`
- `node nodesrc/test_web_gui_stdout_protocol.js`
- `node nodesrc/tests.js -i tests/stdlib/gui_web_stdout_protocol.n.md --no-tree -o tmp/agent2-gui-web-stdout-button-negative.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i examples/gui_counter.nepl -i examples/gui_life.nepl -i examples/gui_mandelbrot.nepl -i examples/gui_calculator.nepl -i examples/gui_scientific_calculator.nepl -i examples/gui_paint.nepl -i examples/gui_breakout.nepl --no-tree -o tmp/agent2-gui-button-helper-examples-after-prevalidate.json -j 1 --dist web/dist --assert-io`
- `npm --prefix web run build:ts`
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/agent2-gui-button-helper-playground-editor.json`
- `node nodesrc/run_source_policy_regressions.js --warn-only` は今回差分由来の warning なし。既存別件の `nodesrc/test_resource_gate_order.js` と `nodesrc/test_diagnostic_code_first_boundary.js` の 2 warning は残る。
- subagent review 2 件で blocker なし、mergeable と確認した。
