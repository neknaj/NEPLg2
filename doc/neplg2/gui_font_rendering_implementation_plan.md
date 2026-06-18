# NEPLg2 GUI font rendering implementation plan

作成日: 2026-06-13

## 実装開始 gate

実装前に次を満たす。

1. `gui_font_rendering_spec.md`、`gui_font_rendering_detailed_design.md`、`gui_font_rendering_implementation_plan.md` が存在する。
2. `gui_font_rendering_design.md` と `gui_2d_rendering_design.md` の方針と矛盾しない。
3. Zenn 方針に照らして、platform abstraction、Option / Result、enum / match、fallback 禁止、契約と現状実装の分離が満たされている。
4. subagent が文書を確認し、`implementation may start` 相当の結論を返す。
5. blocker / required 指摘がある場合は doc を修正し、再 review する。

## Phase F1: core font and render style contract

目的:

- Font renderer と 2D renderer の共有 contract を no_alloc value として追加する。
- 本格 TTF parser の前に、layout / renderer が依存できる型境界を固定する。

変更:

- `stdlib/core/gui/font.nepl` を追加する。
- `GuiFontFaceId`、`GuiGlyphId`、`GuiFontSize`、`GuiWritingMode`、`GuiFontMetrics`、`GuiGlyphMetrics`、`GuiRenderedTextMetrics` を追加する。
- `GuiFontErrorKind`、`GuiShadowRunId`、`GuiShadowRef` を追加する。
- `gui_font_size_result` は denominator 0 以下を `GuiError::InvalidCommand` として返す。
- `stdlib/core/gui/render_style.nepl` を追加する。
- `GuiBlendMode`、`GuiShadow`、`GuiGlyphPaint` を追加する。
- `GuiGlyphPaint` は `shadows GuiShadowRef` を持ち、alloc-backed multi-shadow は `GuiShadowRef::ShadowRun` で参照する。
- `gui_glyph_paint_result` は fill と stroke が両方 `None` の場合 `GuiError::InvalidCommand` を返す。
- `core/gui/prelude.nepl` から font / render_style を公開する。
- `tests/stdlib/gui_core.n.md` に doctest を追加する。

検証:

```powershell
node nodesrc/tests.js -i tests/stdlib/gui_core.n.md --no-tree -o tmp_gui_core_font.json -j 1
node nodesrc/test_stdlib_gui_layering_policy.js
git diff --check
```

## Phase F5cw: std row tile RLE present host execution action boundary

目的:

- F5cr の `GuiRgba8888RowTileRlePresentHostImportRequest` を、actual Web / native / bare executor が直接 match できる std layer row tile RLE present host execution action boundary に写す。
- `GuiRgba8888RowTileRlePresentHostExecutionAction` は flat target x record action であり、Window / Offscreen / Device と BeginFrame / RunRecord / EndFrame の直積を invalid-state-free enum として持つ。
- F5cw は action decoding だけを行い、does not execute host imports。actual host import execution、dispatch loop、scheduler、queue、timer、platform API、Canvas / DOM / minifb、video memory、fallback、silent no-op には進まない。

変更:

- `stdlib/std/gui/tile_present_host_execution.nepl` を追加する。
- Window action は `WindowId` と descriptor / run record を同時に保持する payload struct を使う。
- Offscreen / Device action は target が variant 名に含まれるため descriptor または run record を直接持つ。
- main mapping は F5cr request accessor で target と record を読み、F5cq record の enum を match して action に写す。

完了条件:

- F5cw は F5cr request accessor と F5cq record / run record accessor だけを使う。
- F5cv / F5cu / F5ct / F5cs / F5cp / F5co direct call、F5cr request constructor call、raw packet storage、queue、timer、scheduler、host execution API、platform API、video memory、fallback、silent no-op を持たない。
- focused doctest は import smoke だけでなく、WindowBegin / WindowRun / WindowEnd / OffscreenRun / DeviceEnd の representative mapping を検査する。
- source policy、F5cr / F5cv regression、`git diff --check` が通る。
- subagent implementation review で action shape、direct action return、禁止依存、representative mapping coverage が承認される。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_execution.n.md --no-tree -o tmp_gui_std_tile_present_host_execution_f5cw.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_execution.nepl --no-tree -o tmp_gui_std_tile_present_host_execution_module_f5cw.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_import.n.md --no-tree -o tmp_gui_std_tile_present_host_import_f5cw_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_dispatch_loop.n.md --no-tree -o tmp_gui_std_tile_present_dispatch_loop_f5cw_regression.json -j 1
git diff --check
```

## Phase F5cx: std row tile RLE present host execution report boundary

目的:

- F5cw の `GuiRgba8888RowTileRlePresentHostExecutionAction` と executor outcome を、action context and executor outcome を失わない std layer row tile RLE present host execution report boundary に束ねる。
- `GuiRgba8888RowTileRlePresentHostExecutionReport` は action と `GuiRgba8888RowTileRlePresentHostExecutionReportKind` を持つ。
- report kind は `Succeeded` または `Failed GuiError` であり、failure は string / bool / silent no-op へ落とさない。
- F5cx は not actual execution and not pending completion である。actual Web / native / bare executor、F5cv pending completion、scheduler、queue、timer、platform API、Canvas / DOM / minifb、video memory、fallback、silent no-op には進まない。
- caller は `report_outcome` で元の `Result unit GuiError` を取り出し、F5cv `complete_request` へ渡す。

plan review:

- Dirac plan review は `PLAN_APPROVED`。
- F5cw が executor action、F5cx が action-retaining outcome envelope を担う分割は、platform executor の前に置く root-cause slice として妥当と確認された。
- `Result unit GuiError` は executor が既に返した outcome なので、report construction は direct value return とし、新しい `Result` failure mode を作らない。
- F5cx は F5cv から独立させ、`report_outcome` だけを公開して caller が one-shot pending completion に渡す。

変更:

- `stdlib/std/gui/tile_present_host_execution_report.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostExecutionReportKind`、`GuiRgba8888RowTileRlePresentHostExecutionReport`、constructor helper、accessor、`report_for_request`、`report_outcome` を追加する。
- `report_for_request` は F5cw action decoding を 1 回だけ呼び、supplied outcome を report に束ねる。capability validation、request construction、dispatch loop completion は行わない。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_execution_report.n.md` を追加し、Succeeded / Failed、request-to-action report、`report_outcome` roundtrip を検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5cx source policy を追加し、F5cw/F5cr import、F5cv/F5cu/F5ct/F5cs/F5cp/F5co 禁止、raw / platform / host / queue / timer / scheduler / fallback 禁止、NEPL parentheses 禁止を固定する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_execution_report.n.md --no-tree -o tmp_gui_std_tile_present_host_execution_report_f5cx.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_execution_report.nepl --no-tree -o tmp_gui_std_tile_present_host_execution_report_module_f5cx.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_execution.n.md --no-tree -o tmp_gui_std_tile_present_host_execution_f5cx_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_dispatch_loop.n.md --no-tree -o tmp_gui_std_tile_present_dispatch_loop_f5cx_regression.json -j 1
git diff --check
```

## Phase F5cy: std row tile RLE present host executor validation boundary

目的:

- F5cw の `GuiRgba8888RowTileRlePresentHostExecutionAction` と F5cx の report を、actual Web / native / bare executor の手前で検査する std layer row tile RLE present host executor boundary を追加する。
- `GuiRgba8888RowTileRlePresentHostExecutorSupport` は Window / Offscreen / Device とその非空の組み合わせだけを表す enum とし、supports-nothing state を型で表現しない。
- `GuiRgba8888RowTileRlePresentHostExecutorError` は `UnsupportedAction` / `ReportActionMismatch`、category、expected action、reported action option を持つ typed error とする。
- `validate_report_for_action` は support validation の後、report action と expected action の full action identity を比較する。
- full action identity は variant だけでなく、window、surface、frame、packet metadata、run offset、run count、RGBA channel を含む。
- matching action の failed report は association としては valid であり、execution outcome の解釈は F5cx `report_outcome` と F5cv completion に残す。
- F5cy は actual host import execution、F5cv completion、F5cu / F5ct / F5cs / F5cp / F5co、F5cr request construction、raw storage、host API、platform API、queue、timer、scheduler、Canvas / DOM / minifb、video memory、fallback、silent no-op には進まない。

plan review:

- 最初の計画は support が loose bool 群で空 support を表現でき、report/action の検査も弱くなりうるため `PLAN_BLOCKED` となった。
- 修正版では non-empty support enum、typed executor error、full action equality、`validate_report_for_action` の順序を明確化し、Dirac plan review で `PLAN_APPROVED` となった。
- `UnsupportedAction` では `reported = None`、`ReportActionMismatch` では `reported = Some reported_action` とする。
- F5cv は caller-owned completion として残し、この phase では pending request を消費しない。

変更:

- `stdlib/std/gui/tile_present_host_executor.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostExecutorSupport`、`GuiRgba8888RowTileRlePresentHostExecutorActionKind`、`GuiRgba8888RowTileRlePresentHostExecutorErrorKind`、`GuiRgba8888RowTileRlePresentHostExecutorError` を追加する。
- `gui_rgba8888_row_tile_rle_present_host_executor_require_supported` は `Result unit GuiRgba8888RowTileRlePresentHostExecutorError` を返し、bool-only public API にはしない。
- `gui_rgba8888_row_tile_rle_present_host_executor_action_same` は public accessor だけを使い、descriptor/run/action payload の完全一致を検査する。
- `gui_rgba8888_row_tile_rle_present_host_executor_validate_report_for_action` は supported action 検査、report action 取得、full action equality の順序で実行する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_executor.n.md` を追加し、supported action、unsupported target、matching report、same-variant mismatch、failed report preservation を検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5cy source policy を追加し、F5cw/F5cx import、F5cv/F5cu/F5ct/F5cs/F5cp/F5co 禁止、F5cr request construction 禁止、raw / platform / host / queue / timer / scheduler / fallback 禁止、NEPL parentheses 禁止を固定する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_executor.n.md --no-tree -o tmp_gui_std_tile_present_host_executor_f5cy.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_executor.nepl --no-tree -o tmp_gui_std_tile_present_host_executor_module_f5cy.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_execution_report.n.md --no-tree -o tmp_gui_std_tile_present_host_execution_report_f5cy_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_dispatch_loop.n.md --no-tree -o tmp_gui_std_tile_present_dispatch_loop_f5cy_regression.json -j 1
git diff --check
```

## Phase F5cz: std row tile RLE present host report loop bridge boundary

目的:

- F5cv の `GuiRgba8888RowTileRlePresentDispatchLoopPendingRequest`、F5cw の action decoding、F5cx の report outcome、F5cy の executor validation を接続する std layer row tile RLE present host report loop bridge boundary を追加する。
- validation before completion を contract とし、support / full action identity の検査が通った場合だけ pending value を F5cv `complete_request` へ渡す。
- validation failure は pending の previous state を保持し、F5cv completion を呼ばない。
- matching action の failed report は validation failure にせず、F5cx `report_outcome` から F5cv `HostImportExecutionFailed` へ進める。
- wrong action report と unsupported target は `GuiRgba8888RowTileRlePresentHostReportLoopBridgeError` の `ExecutorValidationFailed` として止める。
- F5cz は actual host import execution、F5cu / F5ct / F5cs / F5cp / F5co、F5cr request construction、raw storage、host API、platform API、queue、timer、scheduler、Canvas / DOM / minifb、video memory、fallback、silent no-op には進まない。

plan review:

- Dirac plan review は `PLAN_APPROVED`。
- 承認条件は pending を value として消費すること、pending request / previous state を completion 前に読むこと、F5cw action decode、F5cy validation、F5cx `report_outcome`、F5cv `complete_request` の順序を固定することだった。
- validation failure では F5cv completion を呼ばず、completion failure では F5cv error kind / category / state を wrapper に保持する。

変更:

- `stdlib/std/gui/tile_present_host_report_loop_bridge.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostReportLoopBridgeErrorKind` は `ExecutorValidationFailed lower_executor_error` と `LoopCompletionFailed lower_loop_error` を持つ。
- `GuiRgba8888RowTileRlePresentHostReportLoopBridgeError` は kind、category、loop state を保持する。
- `gui_rgba8888_row_tile_rle_present_host_report_loop_bridge_complete` は pending request から expected action を作り、F5cy validation 成功後だけ F5cx `report_outcome` と F5cv `complete_request` を呼ぶ。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_report_loop_bridge.n.md` を追加し、success report、failed report completion error、unsupported support、wrong action report を検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5cz source policy を追加し、F5cv/F5cw/F5cx/F5cy import、validation before completion、F5cu/F5ct/F5cs/F5cp/F5co 禁止、F5cr request construction 禁止、raw / platform / host / queue / timer / scheduler / fallback 禁止、NEPL parentheses 禁止を固定する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_report_loop_bridge.n.md --no-tree -o tmp_gui_std_tile_present_host_report_loop_bridge_f5cz.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_report_loop_bridge.nepl --no-tree -o tmp_gui_std_tile_present_host_report_loop_bridge_module_f5cz.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_executor.n.md --no-tree -o tmp_gui_std_tile_present_host_executor_f5cz_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_dispatch_loop.n.md --no-tree -o tmp_gui_std_tile_present_dispatch_loop_f5cz_regression.json -j 1
git diff --check
```

## Phase F5da: std row tile RLE present host execution driver boundary

目的:

- F5cv の `GuiRgba8888RowTileRlePresentDispatchLoopPendingRequest` を、actual Web / native / bare / headless executor が読む action と one-shot pending value の組へ束ねる std layer row tile RLE present host execution driver boundary を追加する。
- `GuiRgba8888RowTileRlePresentHostExecutionDriverPending` は pending value と F5cw action を保持し、platform executor は `pending_action` だけを読んで実行する。
- executor は `Result unit GuiError` だけを返し、F5da は F5cx report construction と F5cz bridge を通して validation / completion に戻す。
- completion は F5cz を authority とし、F5cv `complete_request`、F5cy `validate_report_for_action`、F5cr request construction は直接呼ばない。
- F5da は actual platform API、DOM / Canvas / minifb、video memory、queue、timer、scheduler、F5cu / F5ct / F5cs / F5cp / F5co、raw storage、fallback、silent no-op には進まない。

plan review:

- Dirac plan review は `PLAN_APPROVED`。
- 承認条件は `DriverPending` が Clone / Copy を持たないこと、prepare が pending request を borrow して action を 1 回だけ導出すること、complete_outcome が action を読んでから `field::get driver "pending"` で pending を move し、F5cx report と F5cz bridge だけを呼ぶことだった。
- source policy で direct F5cv completion、F5cy validation reimplementation、F5cu / F5ct / F5cs / F5cp / F5co、F5cr request construction、platform / raw / fallback leakage を禁止する。

変更:

- `stdlib/std/gui/tile_present_host_execution_driver.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostExecutionDriverPending` は pending と action を持つ owner-bearing struct とし、Clone / Copy を実装しない。
- `GuiRgba8888RowTileRlePresentHostExecutionDriverErrorKind` は `BridgeFailed lower_bridge_error` を持ち、driver error は category と loop state を保持する。
- `gui_rgba8888_row_tile_rle_present_host_execution_driver_prepare` は F5cv pending request accessor と F5cw action decoding だけで driver pending を作る。
- `gui_rgba8888_row_tile_rle_present_host_execution_driver_complete_outcome` は stored action と executor outcome から F5cx report を作り、F5cz bridge へ渡す。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_execution_driver.n.md` を追加し、action exposure、success completion、failed outcome、unsupported support を検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5da source policy を追加する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_execution_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_execution_driver_f5da.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_execution_driver.nepl --no-tree -o tmp_gui_std_tile_present_host_execution_driver_module_f5da.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_report_loop_bridge.n.md --no-tree -o tmp_gui_std_tile_present_host_report_loop_bridge_f5da_regression.json -j 1
git diff --check
```

## Phase F5db: std row tile RLE present virtual host executor boundary

目的:

- F5da の one-shot driver pending を、headless / test 用の deterministic virtual executor で実行する std layer row tile RLE present virtual host executor boundary を追加する。
- `GuiRgba8888RowTileRlePresentVirtualExecutor` は F5cy support と F5cs virtual drain を保持し、actual Web / native / bare executor と同じ F5cw action shape を消費する。
- virtual executor は fallback ではなく、platform host import を持たない deterministic validation backend である。
- support rejection と drain failure でも F5da `complete_outcome` を呼び、pending を one-shot cleanup する。
- F5db は actual platform API、DOM / Canvas / minifb、video memory、queue、timer、scheduler、F5cv direct completion、F5cz direct bridge、F5cr request construction、F5cu / F5ct / F5cp / F5co、raw storage、fallback、silent no-op には進まない。

plan review:

- Dirac の initial plan review は `PLAN_CHANGES`。`InconsistentCompletion` の明示、drain failure pending consumption、support rejection cleanup、total action-to-record mapping、F5cq source policy、error recovery state の明確化が必要とされた。
- 修正版では `SupportRejected`、`DrainFailed`、`DriverFailed`、`InconsistentCompletion` を typed error kind とし、error は category、recovery executor、optional driver error を保持する。
- drain failure の recovery executor は failed drain state ではなく original executor とし、lower drain error 側に diagnostic drain state を保持する。
- 修正版の Dirac plan review は `PLAN_APPROVED`。

変更:

- `stdlib/std/gui/tile_present_virtual_executor.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentVirtualExecutor` は support と virtual drain を保持する Copy state とする。
- action-to-record helper は F5cw の 9 action variant を F5cq `GuiRgba8888RowTileRlePresentHostCommandRecord` に total mapping する。
- execute は F5cy `require_supported` を先に呼び、support preflight 成功後だけ F5cs `virtual_drain_step` を呼ぶ。
- support rejection / drain failure / success のすべてで F5da `complete_outcome` を経由し、direct F5cv completion と direct F5cz bridge を使わない。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_virtual_executor.n.md` を追加し、facade import smoke と coverage label を固定する。begin success、support preflight rejection、drain failure pending consumption、begin/run/end success sequence の順序契約は source policy と module doctest で検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5db source policy を追加する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_virtual_executor.n.md --no-tree -o tmp_gui_std_tile_present_virtual_executor_f5db.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_virtual_executor.nepl --no-tree -o tmp_gui_std_tile_present_virtual_executor_module_f5db.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_execution_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_execution_driver_f5db_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5db.json -j 1
git diff --check
```

## Phase F5dc: std row tile RLE present host action sink boundary

目的:

- actual Web / native / bare presenter が返した executor-supplied outcome を、F5cw action と一緒に `GuiRgba8888RowTileRlePresentHostActionSinkStep` へ包む std layer row tile RLE present host action sink boundary を追加する。
- F5dc は F5cy support validation を通過した action だけを step にし、unsupported target は typed error にする。
- F5dc does not manufacture success。`Result::Ok unit` や fallback success は作らず、caller が渡した `Result unit GuiError` をそのまま保持する。
- F5dc は actual platform execution、F5da pending ownership、F5da completion、F5cx report、F5cz bridge、F5cr request construction、queue、timer、scheduler、raw storage、DOM / Canvas / minifb、video memory、fallback、silent no-op には進まない。

plan review:

- Dirac initial plan review は `PLAN_CHANGES`。`accept` / `reject` helper が success / failure を std layer で作ると silent no-op success path になり得るため禁止された。
- 修正版では `gui_rgba8888_row_tile_rle_present_host_action_sink_step support action outcome` だけを public constructor とし、outcome は executor-supplied outcome として caller から受け取る。Dirac revised plan review は `PLAN_APPROVED`。

実装:

- `stdlib/std/gui/tile_present_host_action_sink.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostActionSinkStep` は action と `Result unit GuiError` outcome だけを保持する。
- `GuiRgba8888RowTileRlePresentHostActionSinkErrorKind` は `UnsupportedAction` だけを持ち、lower F5cy support error と category を保持する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_action_sink.n.md` を追加し、facade import smoke と coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dc source policy を追加する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_action_sink.n.md --no-tree -o tmp_gui_std_tile_present_host_action_sink_f5dc.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_action_sink.nepl --no-tree -o tmp_gui_std_tile_present_host_action_sink_module_f5dc.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_virtual_executor.n.md --no-tree -o tmp_gui_std_tile_present_virtual_executor_f5dc_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_execution_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_execution_driver_f5dc_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dc.json -j 1
git diff --check
```

## Phase F5dd: std row tile RLE present host action sink driver boundary

目的:

- F5dc の support preflight / executor-supplied outcome packaging と F5da one-shot driver completion を接続する std layer row tile RLE present host action sink driver boundary を追加する。
- F5dc が support rejection を返した場合、F5da completion を呼ばず、original driver pending を `SinkRejected` の owner-bearing error として返す。
- F5dc が support success を返した場合だけ、同じ caller-supplied outcome を F5da `complete_outcome` へ渡し、sink step と dispatch loop completion を返す。
- F5dd does not manufacture executor outcome。`Result::Ok unit` や synthetic `Result::Err` を作らず、actual executor から渡された `Result unit GuiError` だけを扱う。
- F5dd は actual platform execution、F5cv direct completion、F5cz direct bridge、F5cx report construction、F5cr request construction、F5db virtual executor、F5cu / F5ct / F5cs / F5cp / F5co、queue、timer、scheduler、raw storage、DOM / Canvas / minifb、video memory、fallback、silent no-op には進まない。

plan review:

- Dirac initial plan review は `PLAN_CHANGES`。success payload を nested F5da completion result にせず、`Ok GuiRgba8888RowTileRlePresentHostActionSinkDriverStep` / `Err GuiRgba8888RowTileRlePresentHostActionSinkDriverError` の単一 `Result` にするよう指摘された。
- Dirac は support preflight failure で synthetic outcome を作らず、owner-bearing `SinkRejected` error に original driver pending を戻す方針を承認した。
- 修正版では `SinkRejected` が F5dc sink error と driver pending を所有し、`DriverCompletionFailed` が F5da driver error と sink step だけを保持する。Dirac revised plan review は `PLAN_APPROVED`。

実装:

- `stdlib/std/gui/tile_present_host_action_sink_driver.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostActionSinkDriverStep` は sink step と dispatch loop completion を保持する。
- `GuiRgba8888RowTileRlePresentHostActionSinkDriverRejected` は sink error と driver pending を保持し、Clone / Copy を実装しない。
- `GuiRgba8888RowTileRlePresentHostActionSinkDriverError` は owner-bearing error なので Clone / Copy を実装しない。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_action_sink_driver.n.md` を追加し、facade import smoke と coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dd source policy を追加する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_action_sink_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_action_sink_driver_f5dd.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_action_sink_driver.nepl --no-tree -o tmp_gui_std_tile_present_host_action_sink_driver_module_f5dd.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_action_sink.n.md --no-tree -o tmp_gui_std_tile_present_host_action_sink_f5dd_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_execution_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_execution_driver_f5dd_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dd.json -j 1
git diff --check
```

## Phase F5de: std row tile RLE present host action attempt driver boundary

目的:

- actual Web / native / bare executor が返した action attempt と F5da driver pending の action identity を、F5dd action sink driver completion の前に検査する std layer row tile RLE present host action attempt driver boundary を追加する。
- driver pending の expected action と attempt の attempted action を F5cy full action equality で比較し、variant-only comparison を使わない。
- action mismatch では F5dd を呼ばず、expected action、attempted action、`GuiError::InvalidCommand` category、original driver pending を `AttemptActionMismatch` owner-bearing error として返す。
- action match の場合だけ、attempt outcome を F5dd `gui_rgba8888_row_tile_rle_present_host_action_sink_driver_step` に委譲する。
- F5de does not manufacture executor outcome。`Result::Ok unit` や synthetic `Result::Err` を作らず、actual executor から渡された `Result unit GuiError` だけを扱う。
- F5de は F5dc direct call、actual platform execution、F5cv direct completion、F5cz direct bridge、F5cx report construction、F5cr request construction、F5db virtual executor、queue、timer、scheduler、raw storage、DOM / Canvas / minifb、video memory、fallback、silent no-op には進まない。

plan review:

- Dirac plan review は `PLAN_APPROVED`。
- action 比較は F5cy `gui_rgba8888_row_tile_rle_present_host_executor_action_same &expected &attempted` を使う。
- mismatch は F5dd を呼ばず、original F5da driver pending を `AttemptActionMismatch` payload に保持する。category は `Some GuiError::InvalidCommand` とする。
- `SinkDriverFailed` は lower F5dd error を丸ごと保持し、top-level error は Clone / Copy を実装しない。

実装:

- `stdlib/std/gui/tile_present_host_action_attempt_driver.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostActionAttempt` は attempted action と executor-supplied outcome を保持する。
- `GuiRgba8888RowTileRlePresentHostActionAttemptDriverStep` は attempt と F5dd sink driver step を保持する。
- `GuiRgba8888RowTileRlePresentHostActionAttemptMismatch` は expected action、attempted action、category、driver pending を保持し、Clone / Copy を実装しない。
- `GuiRgba8888RowTileRlePresentHostActionAttemptDriverError` は owner-bearing error なので Clone / Copy を実装しない。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_action_attempt_driver.n.md` を追加し、facade import smoke と coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5de source policy を追加する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_action_attempt_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_action_attempt_driver_f5de.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_action_attempt_driver.nepl --no-tree -o tmp_gui_std_tile_present_host_action_attempt_driver_module_f5de.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_action_sink_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_action_sink_driver_f5de_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_executor.n.md --no-tree -o tmp_gui_std_tile_present_host_executor_f5de_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5de.json -j 1
git diff --check
```

## Phase F5df: std row tile RLE present run-span boundary

目的:

- F5cq host-command run record の tile-local linear pixel offset を、actual Web / native / bare / headless presenter が共通に消費できる 1 行 span stream に分解する std layer row tile RLE present run-span boundary を追加する。
- platform rect、DrawTarget / RenderTarget、video memory、host import execution へ進む前に、run crossing row boundary を正しく分割する。
- start は descriptor と run を checked arithmetic で検査し、invalid cursor を作らない。
- step は `SpanReady span next_cursor` または explicit Completed を返し、empty span、silent no-op、fallback success を作らない。
- F5df does not call platform import。F5da-F5de action / driver、F5cs virtual drain、F5cp / F5co lower cursor、packet record reader、raw storage、queue、scheduler、DOM / Canvas / minifb、video memory、fallback には進まない。

plan review:

- Dirac plan review は `PLAN_CHANGES`。
- `start record` は必ず `Result Cursor Error` とし、検証を `step` へ遅らせない。
- span は platform / renderer rect ではなく専用 value 型にする。高さを持たない row span とし、必要な accessor は 1 を返す。
- run offset は tile-local linear pixel offset と明記する。座標変換は `local_row = offset / width`、`x = offset % width`、`y = row_start + local_row` とする。
- step result は explicit Completed を持ち、remaining 0 を no-op として流さない。
- width / height / row range / tile rows / pixel count / run bounds を enum error で分ける。
- source policy は F5cq host command、F5cn descriptor accessor、RLE run accessor、packet descriptor metadata だけを許可する。

実装:

- `stdlib/std/gui/tile_present_run_span.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentRunRowSpan` は x、y、width、color を保持し、高さは accessor で 1 を返す。
- `GuiRgba8888RowTileRlePresentRunSpanCursor` は F5cq run record、next pixel offset、remaining pixel count を保持する。
- `GuiRgba8888RowTileRlePresentRunSpanStepResult` は `SpanReady` と `Completed` を持つ。
- start error kind は width / height / row start / row count / row extent / row count vs tile rows / pixel count / run offset / run count / run end を分ける。
- step error kind は descriptor invalid、cursor offset / remaining inconsistency、local row out of bounds、row y overflow、span advance overflow、span width invalid を分ける。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_run_span.n.md` を追加し、row crossing run が 2 span に分かれ、3 step 目で Completed になることを検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5df source policy を追加する。

完了条件:

- F5df は F5cq host-command run record と F5cn descriptor metadata を authority とし、F5cq を bypass しない。
- row crossing run は `width - x` を超えて 1 span にしない。
- start が invalid descriptor / run を拒否し、step は invalid cursor を typed error とする。
- focused doctest、source policy、F5cq / F5de regression、`git diff --check` が通る。
- subagent implementation review で start-time validation、row-span invariant、explicit Completed、禁止依存が承認される。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_run_span.n.md --no-tree -o tmp_gui_std_tile_present_run_span_f5df.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_run_span.nepl --no-tree -o tmp_gui_std_tile_present_run_span_module_f5df.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_command.n.md --no-tree -o tmp_gui_std_tile_present_host_command_f5df_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_action_attempt_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_action_attempt_driver_f5df_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5df.json -j 1
git diff --check
```

## Phase F5dg: std row tile RLE present host span operation boundary

目的:

- F5cw host execution action を、actual Web / native / bare presenter が 1 operation ずつ消費できる target-qualified operation stream に写す。
- F5dg は std layer row tile RLE present host span operation boundary である。
- `GuiRgba8888RowTileRlePresentHostSpanOperationCursor` は Begin / End を `SinglePending operation` として one-shot operation にし、Run を `RunPending target run_span_cursor` として F5df run-span cursor で保持する。
- `start action` は Run action の場合だけ F5df `run_span_start` を 1 回呼び、失敗時は original F5cw action を保持する typed error を返す。
- `step cursor` は F5df `run_span_step` を最大 1 回だけ呼び、SpanReady を WindowRunSpan / OffscreenRunSpan / DeviceRunSpan に target-qualified mapping する。
- actual host import execution、F5da-F5de action driver、F5cs virtual drain、F5cp / F5co lower cursor、packet record / raw storage、queue、scheduler、platform API、DOM / Canvas / minifb、video memory、DrawTarget / RenderTarget、fallback、silent no-op には進まない。

plan review:

- Dirac plan review は `PLAN_CHANGES`。
- 指摘に従い、cursor phase は `SinglePending operation` / `RunPending target run_span_cursor` / `Completed` に固定する。
- step result は `OperationReady operation next_cursor` / `Completed` とし、Begin / End は 1 回目で operation、2 回目で explicit Completed にする。
- Run start failure は original action、Run step failure は current cursor を保持する error にする。
- source policy で public step が F5df step を 1 回だけ呼び、F5df start を毎 step 呼び直さないことを固定する。

変更:

- `stdlib/std/gui/tile_present_host_span_operation.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperation`、operation cursor、ready、start / step error を追加する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation.n.md` を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dg source policy を追加する。

完了条件:

- Begin / End action が one-shot operation と explicit Completed になる。
- Run action が F5df cursor を保持し、row crossing run を target-qualified span operation に分解する。
- F5df start / step error が action / cursor context と category を保持する。
- focused doctest、source policy、F5df / F5cw regression、`git diff --check` が通る。
- subagent implementation review で cursor phase、F5df call location、禁止依存が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_f5dg.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_module_f5dg.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_run_span.n.md --no-tree -o tmp_gui_std_tile_present_run_span_f5dg_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_execution.n.md --no-tree -o tmp_gui_std_tile_present_host_execution_f5dg_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dg.json -j 1
git diff --check
```

## Phase F5dh: std layer row tile RLE present scheduled span operation boundary

目的:

- F5dg host span operation stream を actual presenter の手前で deterministic slice budget に区切る。
- F5dh は F5dg operation stream に対して exact budget only の yield rule を適用し、`resume_slice` では F5dg cursor を保持する。
- F5ct record scheduler は再利用しない。F5ct は F5cq record 単位、F5dh は F5dg operation stream 単位の scheduler であり、RunRecord と RunSpan の cost model は異なる。
- `GuiRgba8888RowTileRlePresentScheduledSpanOperationState` は F5dg cursor と slice-local operation / pixel counters だけを持つ。
- `OperationReady` は operation、post phase、next state を同時に保持し、exact budget 到達時にも operation を失わない。
- Begin / End は operation cost 1、pixel cost 0 とし、RunSpan は F5df accessor で `span.width * span.height` を checked arithmetic で計算する。
- actual host import execution、record scheduler direct call、action driver、raw storage、queue、timer、platform API、DOM / Canvas / minifb、video memory、DrawTarget / RenderTarget、fallback、silent no-op には進まない。

plan review:

- Dirac plan review は `PLAN_CHANGES`。
- 指摘に従い、既存 `tile_present_schedule` の state / policy は再利用しない。
- F5dg / F5df を stream authority とし、F5cs / F5ct / F5cu を再実行しない。
- policy は `max_operations_per_slice` と `max_pixels_per_slice` を持つ新規 value にする。
- `Yield` は valid operation 消費後の exact budget 到達だけを表し、ready payload が operation と phase と next state を同時に持つ。
- `resume_slice` は cursor を保持し、slice counters だけを reset する。

変更:

- `stdlib/std/gui/tile_present_scheduled_span_operation.nepl` を追加する。
- scheduled span operation policy、state、ready、step result、start / step error を追加する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_scheduled_span_operation.n.md` を追加する。heavy action scenario は compile timeout を避けるため import smoke に限定し、behavior は source policy で実装構造を直接検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dh source policy を追加する。
- GUI / font rendering docs と `todo.md` / `note.n.md` を更新する。

完了条件:

- `start` が F5dg start を 1 回だけ呼び、state の counter を 0 で初期化する。
- `step` が F5dg step を最大 1 回だけ呼び、operation と post phase と next state を同時に返す。
- exact budget 到達は `Yield`、budget 超過は typed error になる。
- `resume_slice` が cursor continuation を保持し、counter だけ reset する。
- focused import smoke doctest、source policy、F5dg / F5df regression、`git diff --check` が通る。
- subagent implementation review で F5ct 再利用禁止、F5dg authority、禁止依存、budget error が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_scheduled_span_operation.n.md --no-tree -o tmp_gui_std_tile_present_scheduled_span_operation_f5dh.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_scheduled_span_operation.nepl --no-tree -o tmp_gui_std_tile_present_scheduled_span_operation_module_f5dh.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_f5dh_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_run_span.n.md --no-tree -o tmp_gui_std_tile_present_run_span_f5dh_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dh.json -j 1
git diff --check
```

## Phase F5di: std layer row tile RLE present host span operation attempt boundary

目的:

- F5dh scheduled ready と actual Web / native / bare / headless presenter が返した caller supplied outcome を、completion や queue へ進む前に対応検査する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationAttempt` は attempted operation と `Result unit GuiError` だけを保持し、std layer では success / failure outcome を作らない。
- `attempt_step` は support before equality を固定する。support は F5cy support enum を target support set としてだけ使い、F5cy action validation / action equality へ戻らない。
- operation equality は 9 variants をすべて扱い、Window variants は `window_id_raw`、Begin / End は descriptor、RunSpan は x / y / width / height / RGBA channel を public accessor で比較する。
- unsupported と mismatch は scheduled ready と attempt を保持する typed error にする。
- Yield phase is data only とし、この boundary では resume、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op へ進まない。

plan review:

- Dirac plan review は `PLAN_CHANGES`。
- F5cy の `require_supported` は F5cw action 用なので使わず、F5di 側で span operation target support helper を exhaustive match で作る。
- unsupported error は support context、category `Some GuiError::Unsupported`、scheduled ready、attempt を保持する。
- mismatch error は category `Some GuiError::InvalidCommand`、expected operation、attempted operation、scheduled ready、attempt を保持する。
- attempt constructor と step は caller supplied outcome をそのまま保持し、`Result::Ok unit` や synthetic `Result::Err GuiError` を作らない。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_attempt.nepl` を追加する。
- attempt、attempt step、unsupported payload、mismatch payload、attempt error enum、target support helper、operation equality helper、attempt step を追加する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_attempt.n.md` を追加する。heavy presenter scenario は compile timeout を避けるため import smoke に限定し、behavior は source policy で検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5di source policy を追加する。
- GUI / font rendering docs と `todo.md` / `note.n.md` を更新する。

完了条件:

- support validation が equality と success より前に実行される。
- operation equality が Window / Offscreen / Device の Begin / RunSpan / End をすべて比較する。
- unsupported と mismatch が scheduled ready と attempt を失わない。
- attempt outcome は caller supplied outcome の passthrough であり、std layer で生成しない。
- focused import smoke doctest、source policy、F5dh import smoke、`git diff --check` が通る。
- subagent implementation review で F5cy action helper 不使用、F5dh ready preservation、no platform / no fallback が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_attempt.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_attempt_f5di.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_attempt.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_attempt_module_f5di.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_scheduled_span_operation.n.md --no-tree -o tmp_gui_std_tile_present_scheduled_span_operation_f5di_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5di.json -j 1
git diff --check
```

## Phase F5dj: std layer row tile RLE present host span operation completion boundary

目的:

- F5di の検査済み `GuiRgba8888RowTileRlePresentHostSpanOperationAttemptStep` を AttemptStep only の入力として受け、caller supplied outcome と ready phase を completion value へ写す。
- `GuiRgba8888RowTileRlePresentHostSpanOperationCompletion` は `Continue state` / `Yield state` だけを持つ。
- F5dh `Completed` は operation を持たない terminal なので、per-operation completion does not create Completed。
- host outcome failure does not publish state とし、`Err host_error` では host error、ready、attempt、category `Some host_error` を typed error に保持する。
- F5di association validation、F5dh step / start / resume、F5cs / F5ct / F5cu、F5cy action validation、F5cw action equality、F5da-F5de action driver、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op へ進まない。

plan review:

- Dirac plan review は `PLAN_APPROVED`。
- `Completed` を F5dj で作らない判断は正しい。F5dh `Completed` は operation なし terminal なので、per-operation completion が作ると successful operation completion と end-of-stream completion を混同する。
- F5di errors を wrap せず `AttemptStep` only input にする判断は正しい。association failure は F5di の責務であり、F5dj は検査済み step の host outcome completion だけに絞る。
- `completion_step` は F5di `step_ready` / `step_attempt`、F5di `attempt_outcome`、F5dh `ready_phase` / `ready_state` の public accessor だけを使う。
- outcome が `Err host_error` の場合は `HostOutcomeFailed` を返し、Continue / Yield state を publish しない。category は `Some host_error` とし、通常 failure に `None` を使わない。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_completion.nepl` を追加する。
- completion enum、completion step、host failed payload、completion error enum、completion step function、public accessors を追加する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_completion.n.md` を追加する。heavy presenter scenario は compile timeout を避けるため import smoke に限定し、behavior は source policy で検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dj source policy を追加する。
- GUI / font rendering docs と `todo.md` / `note.n.md` を更新する。

完了条件:

- `completion_step` が F5di `attempt_step` validation 関数を呼ばず、検査済み `AttemptStep` の accessor だけを読む。
- `Err host_error` は ready / attempt / `Some host_error` を保持し、state を completion として返さない。
- `Ok` の場合だけ ready phase / state を Continue / Yield へ写す。
- `Completed` variant を per-operation completion に持たせない。
- focused import smoke doctest、source policy、F5di import smoke、`git diff --check` が通る。
- subagent implementation review で no scheduler / no platform / no fallback と AttemptStep only が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_completion.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_completion_f5dj.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_completion.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_completion_module_f5dj.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_attempt.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_attempt_f5dj_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dj.json -j 1
git diff --check
```

## Phase F5dk: std layer row tile RLE present host span operation presenter step boundary

目的:

- actual Web / native / bare / headless presenter wrapper が 1 span operation を試行した後の戻り道を、F5di before F5dj の順序で固定する。
- 入力は support set、F5dh ready、presenter supplied attempt だけにし、F5dk does not execute host imports を contract とする。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterStep` は F5dj の completion step を保持する success value とする。
- F5di rejection は `AttemptRejected` として support、ready、attempt、lower F5di error、lower category を保持する。
- F5dj rejection は `CompletionRejected` として attempt step、lower F5dj error、lower category を保持する。
- Completed、F5dh start / step / resume、F5dg start / step、F5cy / F5cw action validation、F5da-F5de action drivers、F5cs / F5ct / F5cu、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op へ進まない。

plan review:

- Dirac plan review は `PLAN_APPROVED`。
- F5dj の後続 boundary として妥当であり、platform execution や scheduling を早く結合しすぎていない。
- `presenter_step` は F5di `attempt_step` を先に呼び、F5di `Ok attempt_step` branch でだけ F5dj `completion_step` を呼ぶ。
- `AttemptRejected` は support、original ready、original attempt、lower F5di error、public accessor から得た category を保持する。
- `CompletionRejected` は F5di AttemptStep と lower F5dj error を保持する。caller が ready / attempt context を復元するために lower variant を解析しなくてよい。
- source policy は F5di call、F5di Err wrap、F5di Ok branch、F5dj call、F5dj Err wrap、success step の順序を固定する。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_step.nepl` を追加する。
- presenter step、attempt rejected、completion rejected、presenter step error enum、category mapping helper、public accessors を追加する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_step.n.md` を追加する。heavy presenter scenario は compile timeout を避けるため import smoke に限定し、behavior は source policy で検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dk source policy を追加する。
- GUI / font rendering docs と `todo.md` / `note.n.md` を更新する。

完了条件:

- F5dk `presenter_step` が F5di attempt step を F5dj completion step より先に呼ぶ。
- F5di error branch では F5dj を呼ばず、support / ready / attempt / lower / category を保持する。
- F5dj error branch では attempt step / lower / category を保持する。
- success path は F5dj completion step だけを F5dk success value に包む。
- Completed variant、scheduler、queue、timer、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op を持たない。
- focused import smoke doctest、source policy、F5dj / F5di regression、`git diff --check` が通る。
- subagent implementation review で no scheduler / no platform / no fallback と F5di-before-F5dj order が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_step.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_step_f5dk.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_step.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_step_module_f5dk.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_completion.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_completion_f5dk_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_attempt.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_attempt_f5dk_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dk.json -j 1
git diff --check
```

## Phase F5dl: std layer row tile RLE present host span operation presenter loop boundary

目的:

- actual Web / native / bare / headless presenter loop が F5dh step と F5dk presenter step を直接呼ばず、LoopState / request / completion contract だけを扱う境界を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterLoopState` は support、F5dh policy、scheduled state を同じ value に保持し、次 request に必要な context を side state へ逃がさない。
- `presenter_loop_start` は F5dh start を 1 回だけ呼び、success path だけで support / policy / scheduled state を LoopState へ束ねる。
- `presenter_loop_request` は F5dh step を 1 回だけ呼び、F5dh `OperationReady` を presenter request、F5dh operation-less terminal を loop `Completed` へ写す。
- `presenter_loop_complete` は F5dk presenter step を 1 回だけ呼び、F5dk success branch でだけ F5dj completion step を読み、Continue / Yield scheduled state を support / policy 付き LoopState へ再包装する。
- F5dh `resume_slice`、F5di / F5dj direct call、F5dg start / step、F5cs / F5ct / F5cu、F5da-F5de action drivers、F5cy / F5cw validation、queue、timer、real scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op へ進まない。

plan review:

- Dirac plan review は最初 `PLAN_CHANGES`。
- F5dl の `Completed` は F5dh operation-less terminal を受ける場合に限り妥当であり、F5dk / F5dj の per-operation completion とは混同しない contract が必要と指摘された。
- completion 後も次 request に support / policy が必要なので、`Continue` / `Yield` は scheduled state だけではなく LoopState を返す必要があると指摘された。
- request は ready だけでなく support / policy / ready を保持し、caller が support / policy を side state として持たない形にする必要がある。
- `start` を含め、F5dh start、F5dh step、F5dk presenter step の exact call order を source policy で固定する必要がある。
- F5dh `resume_slice`、F5di / F5dj direct call、F5dg / F5cs / F5ct / F5cu / F5da-F5de / F5cy / F5cw、queue、timer、real scheduler、platform、fallback、silent no-op、per-operation `Completed` creation を禁止する必要がある。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_loop.nepl` を追加する。
- LoopState、presenter request、loop step result、loop completion、start / request / complete error payload、category mapping helper、public accessors を追加する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_loop.n.md` を追加する。heavy presenter scenario は compile timeout を避けるため import smoke に限定し、behavior は source policy で検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dl source policy を追加する。
- GUI / font rendering docs と `todo.md` / `note.n.md` を更新する。

完了条件:

- F5dl `presenter_loop_start` が F5dh start を 1 回だけ呼び、support / policy / scheduled state を LoopState に保持する。
- F5dl `presenter_loop_request` が F5dh step を 1 回だけ呼び、F5dh `Completed` だけを loop `Completed` へ写し、F5dh `OperationReady` から support / policy / ready を保持する request を作る。
- F5dl `presenter_loop_complete` が F5dk presenter step を 1 回だけ呼び、success branch でだけ F5dj completion step から Continue / Yield を取り出して LoopState へ再包装する。
- F5dk error では next state を publish せず、request、lower F5dk error、public accessor derived category を typed error に保持する。
- F5dh `resume_slice`、F5di / F5dj direct call、F5dg start / step、action drivers、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、per-operation `Completed` creation を持たない。
- focused import smoke doctest、source policy、F5dk / F5dh regression、`git diff --check` が通る。
- subagent implementation review で LoopState preservation、F5dh / F5dk exact call order、no scheduler / no platform / no fallback が承認される。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_loop.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_loop.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_loop.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_loop_f5dl.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_loop.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_loop_module_f5dl.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_step.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_step_f5dl_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_scheduled_span_operation.n.md --no-tree -o tmp_gui_std_tile_present_scheduled_span_operation_f5dl_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dl.json -j 1
git diff --check
```

## Phase F5dm: std layer row tile RLE present host span operation presenter outcome boundary

目的:

- actual Web / native / bare / headless presenter glue が F5dl request から operation を読み、caller supplied outcome を F5di attempt constructor へ渡して F5dl complete へ戻すための typed bridge を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterOutcomeRequest` は F5dl request と F5dh ready operation accessor から得た expected operation を保持する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterOutcomeAttempt` は original F5dl request と F5di attempt を保持する。
- OutcomeRequest / OutcomeAttempt / OutcomeCompleteError は Clone / Copy にしない。同じ request / outcome bridge の replay を static に避ける。
- `outcome_request` は F5dl request ready を読み、F5dh ready operation accessor を 1 回だけ使う。
- `outcome_attempt` は OutcomeRequest を value として消費し、F5di attempt constructor を 1 回だけ呼ぶ。
- `outcome_complete` は OutcomeAttempt を value として消費し、F5dl complete を 1 回だけ呼ぶ。
- host import、F5di validation、F5dk presenter step、F5dj completion step、F5dh start / step / resume、F5dg、F5cs / F5ct / F5cu、F5da-F5de action drivers、F5cy / F5cw validation、queue、timer、real scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、loop `Completed` creation へ進まない。

plan review:

- Dirac plan review は `PLAN_CHANGES`。
- F5dm は次 boundary として妥当だが、OutcomeRequest / OutcomeAttempt を Copy にすると same request / outcome bridge の replay が起きやすいため、non-Copy / non-Clone にする必要があると指摘された。
- required flow は `F5dl request -> OutcomeRequest -> borrowed operation inspection -> consume OutcomeRequest + caller outcome -> OutcomeAttempt -> consume OutcomeAttempt -> F5dl complete`。
- `presenter_outcome_attempt` は OutcomeRequest を value として消費し、F5di validation ではなく F5di attempt constructor だけを呼ぶ。
- `presenter_outcome_complete` は OutcomeAttempt を value として消費し、lower error に original request、F5di attempt、F5dl lower error、F5dl public accessor 由来 category を保持する。
- source policy は Clone / Copy 禁止、F5di attempt constructor のみ許可、F5dl complete の exact call order、F5di validation / F5dk / F5dj / F5dh start-step-resume / scheduler / platform / fallback 禁止を固定する。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_outcome.nepl` を追加する。
- OutcomeRequest、OutcomeAttempt、OutcomeCompleteError、request / attempt / complete functions、public accessors を追加する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_outcome.n.md` を追加する。heavy presenter scenario は compile timeout を避けるため import smoke に限定し、behavior は source policy で検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dm source policy を追加する。
- GUI / font rendering docs と `todo.md` / `note.n.md` を更新する。

完了条件:

- F5dm `outcome_request` が F5dl request ready を読み、F5dh ready operation accessor から operation を得る。
- OutcomeRequest / OutcomeAttempt / OutcomeCompleteError が Clone / Copy 実装を持たない。
- F5dm `outcome_attempt` が OutcomeRequest を value として消費し、F5di attempt constructor を 1 回だけ呼ぶ。
- F5dm `outcome_complete` が OutcomeAttempt を value として消費し、F5dl complete を 1 回だけ呼ぶ。
- F5dl lower error は request / attempt / category / lower として typed error に保持される。
- host import、F5di validation、F5dk presenter step、F5dj completion step、F5dh start / step / resume、action drivers、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、loop `Completed` creation を持たない。
- focused import smoke doctest、source policy、F5dl / F5di regression、`git diff --check` が通る。
- subagent implementation review で non-Copy bridge、value-consuming flow、no scheduler / no platform / no fallback が承認される。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_outcome.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_outcome.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_outcome.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_outcome_f5dm.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_outcome.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_outcome_module_f5dm.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_loop.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_loop_f5dm_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_attempt.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_attempt_f5dm_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dm.json -j 1
git diff --check
```

## Phase F5dn: std layer row tile RLE present host span operation presenter driver boundary

目的:

- actual Web / native / bare / headless presenter loop が F5dl start / request と F5dm outcome request / attempt / complete を直接ばらばらに呼ばず、DriverState / DriverRequestResult / DriverCompletion の contract だけを扱えるようにする。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterDriverState` は F5dl loop state を保持する non-Copy state とし、同じ state から複数 request を作る replay を避ける。
- `request` は DriverState を value として消費し、F5dl request を 1 回だけ呼ぶ。
- F5dm outcome request は F5dl `Request` branch でだけ呼ぶ。F5dl `Completed` と F5dl request error では呼ばない。
- `complete` は OutcomeRequest と caller supplied outcome を value として受け、F5dm outcome attempt と F5dm outcome complete を 1 回ずつ呼んで F5dl Continue / Yield を次の DriverState へ再包装する。
- host import、F5dl complete direct call、F5di constructor / validation direct call、F5dh start / step / resume direct call、action drivers、queue、timer、real scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `GuiError` creation へ進まない。

plan review:

- Dirac plan review は `PLAN_CHANGES`。
- F5dn は presenter-facing driver として妥当だが、`DriverState` は non-Copy / non-Clone にする必要がある。Copy state は同じ F5dl state から複数 request を作る replay を許すため、F5dm の replay 防止と矛盾する。
- `presenter_driver_request` は DriverState を value として消費し、F5dl request error だけ original DriverState を typed error に戻す。
- `DriverRequestResult` は OutcomeRequest を含むため non-Copy / non-Clone にする。`DriverCompletion` も DriverState を含むため non-Copy / non-Clone にする。
- `presenter_driver_request` は F5dl `Request` branch でだけ F5dm `outcome_request` を呼ぶ。F5dl `Completed` と F5dl request error では F5dm を呼ばない。
- `presenter_driver_complete` の error は F5dm lower error と F5dm public accessor 由来 category だけを保持すればよい。request / attempt の重複保存は lower F5dm error が既に持つ。
- source policy は F5dl start / request と F5dm outcome_request / outcome_attempt / outcome_complete だけを許可し、F5dl complete direct call、F5di constructor / validation direct call、F5dh start / step / resume direct call、`Result::Ok unit`、synthetic `Completed` creation、scheduler / platform / fallback を禁止する。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_driver.nepl` を追加する。
- DriverState、DriverRequestResult、DriverCompletion、start / request / complete error payload、start / request / complete functions、public accessors を追加する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_driver.n.md` を追加する。heavy presenter scenario は compile timeout を避けるため import smoke に限定し、behavior は source policy で検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dn source policy を追加する。
- GUI / font rendering docs と `todo.md` / `note.n.md` を更新する。

完了条件:

- F5dn `start` が F5dl start を 1 回だけ呼び、success path だけで DriverState を作る。
- DriverState / DriverRequestResult / DriverCompletion / driver errors が Clone / Copy 実装を持たない。
- F5dn `request` が DriverState を value として消費し、F5dl request error では original DriverState を typed error に戻す。
- F5dn `request` は F5dl `Request` branch でだけ F5dm outcome request を呼び、F5dl `Completed` branch では driver `Completed` だけを返す。
- F5dn `complete` が F5dm outcome attempt と F5dm outcome complete だけを呼び、F5dl Continue / Yield を DriverCompletion へ再包装する。
- F5dn complete error は F5dm lower error と F5dm category accessor 由来 category だけを保持する。
- host import、F5dl complete direct call、F5di constructor / validation direct call、F5dh start / step / resume direct call、action drivers、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `GuiError` creation を持たない。
- focused import smoke doctest、source policy、F5dm / F5dl regression、`git diff --check` が通る。
- subagent implementation review で value-consuming driver state、F5dl/F5dm exact bridge、no scheduler / no platform / no fallback が承認される。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_driver.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_driver.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_driver_f5dn.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_driver.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_driver_module_f5dn.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_outcome.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_outcome_f5dn_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_loop.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_loop_f5dn_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dn.json -j 1
git diff --check
```

## Phase F5do: std layer row tile RLE present host span operation presenter executor boundary

目的:

- actual Web / native / bare / headless presenter executor が F5dn OutcomeRequest を受け取り、executor supplied attempt を F5dn complete へ戻す直前の validation boundary を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorRequest` は OutcomeRequest、OutcomeRequest 由来 support、expected span operation を保持する。
- support は caller 引数から受けず、OutcomeRequest に保持された F5dl request の public accessor から読む。
- unsupported operation では F5dn complete へ合成 `Err Unsupported` を渡さず、request owner を保持した typed error を返す。
- executor supplied attempt は span operation と caller supplied outcome を持ち、complete 前に expected operation と reported operation を payload まで比較する。
- host import、platform API、DOM / Canvas / minifb、video memory、queue、timer、scheduler、fallback、silent no-op、old F5cw action mapping へ進まない。

plan review:

- Dirac plan review は `PLAN_CHANGES`。
- F5do は `OutcomeRequest -> ExecutorRequest -> ExecutorAttempt -> F5dn complete` の value-consuming bridge に絞る。
- support は新 enum を作らず、既存 `GuiRgba8888RowTileRlePresentHostExecutorSupport` を使う。
- support は OutcomeRequest 内の F5dl request から public accessor 経由で読む。別引数にすると F5dn start 時の support と F5do support が食い違う。
- support rejection では F5dn complete に合成 `Err Unsupported` を渡さない。`UnsupportedOperation` error に non-Copy request wrapper を保持して返す。
- executor から戻る値は `Result unit GuiError` だけでなく、`operation + outcome` の typed attempt にする。stale or別 operation の outcome を現在 request に適用する事故を防ぐ。
- action identity check は support check の後、F5dn complete の前に行う。
- F5do から直接呼んでよいのは F5dm / F5dn の public accessors と F5dn complete、F5dl request support accessor、span operation public accessors に限る。
- source policy は no platform / DOM / Canvas / minifb / video_memory / raw / queue / timer / scheduler / fallback / silent no-op、no F5dh / F5dk / F5dj direct calls、no F5dl complete direct、no F5di attempt constructor / validation direct、no F5cw action mapping、no F5da-F5de drivers、no `Result::Ok unit` / synthetic `Result::Err GuiError` outcome creation、no Clone / Copy for request / attempt / owner-bearing errors、no parentheses を固定する。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor.nepl` を追加する。
- ExecutorRequest、ExecutorAttempt、unsupported operation error、attempt mismatch error、driver complete error、request / complete error enum、request / attempt / complete functions、public accessors を追加する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor.n.md` を追加する。heavy presenter scenario は compile timeout を避けるため import smoke に限定し、behavior は source policy で検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5do source policy を追加する。
- GUI / font rendering docs と `todo.md` / `note.n.md` を更新する。

完了条件:

- F5do `request` が OutcomeRequest 内の F5dl request から support を読み、別引数 support を受け取らない。
- ExecutorRequest / ExecutorAttempt / owner-bearing errors が Clone / Copy 実装を持たない。
- unsupported operation では F5dn complete、F5dm outcome attempt / complete、F5di constructor / validation を呼ばない。
- F5do `complete` が request operation と attempt operation を span-operation payload まで比較し、一致した場合だけ F5dn complete を呼ぶ。
- mismatch error は original request と attempt を保持する。
- driver complete error は F5dn lower error と F5dn category accessor 由来 category だけを保持する。
- host import、F5dl complete direct call、F5dm outcome attempt / complete direct call、F5di constructor / validation direct call、F5cw action mapping、action drivers、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `GuiError` outcome creation を持たない。
- focused import smoke doctest、source policy、F5dn / F5dm regression、`git diff --check` が通る。
- subagent implementation review で OutcomeRequest support source、unsupported owner retention、attempt identity check、F5dn-only completion が承認される。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_executor.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_f5do.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_executor.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_module_f5do.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_driver_f5do_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_outcome.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_outcome_f5do_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5do.json -j 1
git diff --check
```

## Phase F5dp: std layer row tile RLE present host span operation presenter executor loop boundary

目的:

- actual Web / native / bare / headless presenter loop が F5dn request と F5do executor request / complete を手動で interleave しないための std loop boundary を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorLoopState` は F5dn DriverState を保持する non-Copy loop state とする。
- request は LoopState を value として消費し、F5dn request を 1 回だけ呼ぶ。
- F5dn `Completed` branch では F5do を呼ばず、F5dn `Operation` branch でだけ F5do executor request を呼ぶ。
- complete は F5do executor complete だけを 1 回呼び、F5dn DriverCompletion を Continue / Yield loop completion へ再包装する。
- F5dp is not actual Web / native / bare / headless execution であり、real scheduler policy でもない。

plan review:

- Dirac plan review は `PLAN_APPROVED`。
- F5dp は real scheduler / platform backend の前に置く正しい std boundary であり、F5dn request と F5do executor request / complete の残る manual interleaving を除く。
- LoopState、request result、completion、owner-bearing errors は non-Copy / non-Clone にする。
- request order は F5dn `driver_request` once、F5dn `Operation` の場合だけ F5do `executor_request` once とする。
- Driver `Completed` は F5do を呼ばない。
- complete は F5do `executor_complete` だけを 1 回呼ぶ。F5dn complete、F5dm、F5di、F5dl、F5dh を直接呼ばない。
- F5do unsupported / mismatch semantics を維持し、synthetic `Err Unsupported`、synthetic `Ok unit`、owner loss を作らない。
- source policy は F5dn start / request と F5do request / complete、public category accessors だけを許可し、F5dn complete direct、F5dm/F5dl/F5di/F5dh/F5dk/F5dj direct calls、F5cw/F5da-F5de paths、platform / raw / queue / timer / scheduler、fallback / silent no-op、括弧を禁止する。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_loop.nepl` を追加する。
- LoopState、RequestResult、LoopCompletion、start / request / complete errors、start / request / complete functions、public accessors を追加する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_loop.n.md` を追加する。heavy presenter scenario は compile timeout を避けるため import smoke に限定し、behavior は source policy で検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dp source policy を追加する。
- GUI / font rendering docs と `todo.md` / `note.n.md` を更新する。

完了条件:

- F5dp `start` が F5dn start を 1 回だけ呼び、success path だけで LoopState を作る。
- LoopState / RequestResult / LoopCompletion / owner-bearing errors が Clone / Copy 実装を持たない。
- F5dp `request` が LoopState を value として消費し、F5dn request を 1 回だけ呼ぶ。
- F5dp `request` は F5dn `Operation` branch でだけ F5do executor request を呼び、F5dn `Completed` branch では F5do を呼ばない。
- F5dp `complete` が F5do executor complete だけを呼び、F5dn DriverCompletion を LoopCompletion へ再包装する。
- F5dn complete direct、F5dm / F5dl / F5di / F5dh / F5dk / F5dj direct call、F5cw action mapping、action drivers、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `GuiError` outcome creation を持たない。
- focused import smoke doctest、source policy、F5do / F5dn regression、`git diff --check` が通る。
- subagent implementation review で F5dn/F5do exact bridge、no scheduler / no platform / no fallback が承認される。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_executor_loop.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_loop.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_loop.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_loop_f5dp.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_executor_loop.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_loop_module_f5dp.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_f5dp_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_driver_f5dp_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dp.json -j 1
git diff --check
```

## Phase F5dq: std layer row tile RLE present host span operation presenter executor attempt driver boundary

目的:

- actual Web / native / bare / headless presenter executor が返した executor supplied attempt を F5dp executor loop completion へ戻す std layer boundary を追加する。
- F5dq は F5dp complete wrapper であり、actual execution、headless virtual drain、fallback、real scheduler policy ではない。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorAttemptDriverStep` は completion-only success value とし、F5dp complete が value として消費した request / attempt を保持し直さない。
- failure は category と lower F5dp error だけを持ち、lower F5dp error を recovery authority とする。
- F5dq は `Result::Ok unit` / `Result::Err GuiError` を作らず、executor supplied attempt が持つ outcome を F5dp に渡すだけにする。

plan review:

- Dirac plan review 1 は `PLAN_BLOCKED`。
- 初期案は success step と failure payload に request / attempt を保持しようとしていたが、F5dp complete が request / attempt を value 消費するため、non-Copy ownership model と衝突すると指摘された。
- 修正版では success step は completion-only、failure は category + lower F5dp error のみとし、lower F5dp error chain を recovery authority とする。
- Dirac revised plan review は `PLAN_APPROVED`。
- F5dq は F5dp の post-completion wrapper として妥当であり、request / attempt を再保持しないこと、F5dp complete exactly once、synthetic outcome 禁止、old action path / platform / raw / scheduler / fallback 禁止を source policy で固定する。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_attempt_driver.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorAttemptDriverStep`、`CompleteRejected`、`Error`、step function、public accessors を追加する。
- step function は `gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_loop_complete request attempt` を 1 回だけ呼ぶ。
- success は completion-only step、failure は lower F5dp error と category だけを返す。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_attempt_driver.n.md` を追加する。heavy presenter scenario は compile timeout を避けるため import smoke に限定し、behavior は source policy で検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dq source policy を追加する。
- GUI / font rendering docs と `todo.md` / `note.n.md` を更新する。

完了条件:

- F5dq `step` が F5dp complete を 1 回だけ呼ぶ。
- success step は completion だけを保持し、request / attempt を保持しない。
- failure payload は category と lower F5dp error だけを保持し、request / attempt を保持しない。
- Step / CompleteRejected / Error は Clone / Copy 実装を持たない。
- F5do direct complete / request、F5dn / F5dm / F5dl / F5di / F5dh / F5dk / F5dj direct call、old F5cw / F5da-F5de action paths、F5db virtual executor、F5cs virtual drain、F5cu / F5ct / F5cr / F5cp / F5co、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `Result::Err GuiError` outcome creation を持たない。
- focused import smoke doctest、source policy、F5dp / F5do regression、`git diff --check` が通る。
- subagent implementation review で ownership correction、completion-only success、lower error recovery authority、no synthetic outcome、no scheduler / platform / fallback が承認される。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_executor_attempt_driver.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_attempt_driver.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_attempt_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_attempt_driver_f5dq.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_executor_attempt_driver.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_attempt_driver_module_f5dq.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_loop.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_loop_f5dq_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_f5dq_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dq.json -j 1
git diff --check
```

## Phase F5dr: std layer row tile RLE present host span operation presenter executor session boundary

目的:

- actual Web / native / bare / headless presenter loop が ready state、executor pending request、completion result、terminal completed state を sentinel / null なしで保持できる std layer session boundary を追加する。
- F5dr は F5dp executor loop と F5dq attempt driver を session contract に包むが、actual execution、headless virtual drain、fallback、real scheduler policy ではない。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionState` は `Ready` と `Completed` を持つ。
- `Completed` state への request は明示 terminal behavior として request result `Completed` を返し、F5dp request を呼ばない。
- `Ready` state だけが F5dp request を 1 回だけ呼ぶ。
- pending request は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionPending` に value として移す。
- `session_complete` は pending request と executor attempt を value として消費し、F5dq attempt driver step を 1 回だけ呼ぶ。
- Continue / Yield は Ready session state に写す。
- request / complete error recovery は lower F5dp / F5dq error chain を authority とする。

plan review:

- Dirac plan review は `PLAN_APPROVED`。
- `Completed` を `SessionState` に含める設計は sentinel / null を避けるので妥当と判断された。
- `session_request Completed -> Completed` は明示 terminal behavior として document すれば silent no-op ではない。
- `session_request` は state を value 消費し、`Ready` だけで F5dp request を 1 回呼ぶ。
- `session_complete` は F5dq `attempt_driver_step` だけを authority にし、F5dp / F5do / F5dn へ戻らない。
- F5dq success step から completion を取り出し、Continue / Yield だけを Ready session state へ包む。
- request / complete は lower error chain を recovery authority とし、private field 復元や consumed request / attempt 再構築を禁止する。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session.nepl` を追加する。
- SessionState、SessionPending、SessionRequestResult、SessionCompletion、start / request / complete errors、start / request / complete functions、public accessors を追加する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session.n.md` を追加する。heavy presenter scenario は compile timeout を避けるため import smoke に限定し、behavior は source policy で検査する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dr source policy を追加する。
- GUI / font rendering docs と `todo.md` / `note.n.md` を更新する。

完了条件:

- `session_start` が F5dp start を 1 回だけ呼ぶ。
- `session_request` は Ready branch だけで F5dp request を 1 回だけ呼ぶ。
- `session_request` の Completed branch は F5dp request を呼ばず、terminal Completed result だけを返す。
- `session_complete` が F5dq attempt driver step を 1 回だけ呼び、F5dp complete / F5do / F5dn を直接呼ばない。
- SessionState / SessionPending / request result / completion / errors は Clone / Copy 実装を持たない。
- `SessionState::Completed` は request mapping と enum definition 以外で作らない。
- F5do / F5dn / F5dm / F5dl / F5di / F5dh / F5dk / F5dj direct call、old F5cw / F5da-F5de action paths、F5db virtual executor、F5cs virtual drain、F5cu / F5ct / F5cr / F5cp / F5co、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `Result::Err GuiError` outcome creation を持たない。
- focused import smoke doctest、source policy、F5dq / F5dp regression、`git diff --check` が通る。
- subagent implementation review で session state shape、terminal Completed behavior、pending owner transfer、F5dq authority、lower error recovery authority、no scheduler / platform / fallback が承認される。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_f5dr.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_module_f5dr.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_attempt_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_attempt_driver_f5dr_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_loop.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_loop_f5dr_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dr.json -j 1
git diff --check
```

## Phase F5ds: std layer row tile RLE present host span operation presenter executor session turn boundary

目的:

- F5dr の session state と pending executor request を、actual Web / native / bare / headless scheduler が 1 turn 分の owner-bearing state として扱える std layer boundary を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnState` は `Session` と `Pending` だけを持ち、no separate Completed turn state とする。
- terminal completed state の authority は F5dr `SessionState` と F5dr session request に残し、F5ds は F5dr `Ready` / `Completed` variant を直接 match しない。
- `turn_poll` は state を value として消費し、`Pending` なら executor へ owner transfer し、`Session` だけが F5dr session request を 1 回呼ぶ。
- `turn_complete` は F5dr session complete を 1 回だけ呼び、Continue / Yield を `Session` turn state へ包む。
- F5ds は real scheduler policy、queue、timer、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic outcome creation に進まない。

plan review:

- Dirac plan review 1 は `PLAN_CHANGES`。`Ready %SessionState` と separate `Completed` turn variant では `TurnState::Ready SessionState::Completed` と `TurnState::Completed` が重複 terminal state になると指摘された。
- revised plan では preferred shape として `TurnState::Session %F5drSessionState | Pending %F5drSessionPending` を採用し、terminal completion を F5dr のみへ集約する。
- Dirac revised plan review は `PLAN_APPROVED`。`turn_poll` は state を value 消費すること、F5dr session variants を直接 match しないこと、F5dr start / request / complete だけを authority にすること、source policy で separate `TurnState::Completed` と lower bypass を禁止することが条件である。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnState`、`SessionTurnPollResult`、`SessionTurnCompleteResult`、start / poll / complete errors を追加する。
- `turn_start` は F5dr session start を 1 回だけ呼び、success で `Session` turn state を返す。
- `turn_poll` は `Pending` branch で pending を executor へ渡し、`Session` branch だけで F5dr session request を 1 回呼ぶ。
- `turn_complete` は F5dr session complete を 1 回だけ呼び、Continue / Yield を `Session` turn state に写す。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn.n.md` を追加し、facade、state owner、no duplicate terminal state、pending owner transfer、start / poll / complete order、lower recovery authority、no scheduler / platform / fallback の coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5ds source policy を追加し、F5dr-only boundary、separate terminal state 禁止、F5dr session variant direct match 禁止、F5dp / F5dq direct bypass 禁止、raw / platform / scheduler / fallback leakage 禁止、括弧なし prefix style を固定する。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_f5ds.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_module_f5ds.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_f5ds_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5ds.json -j 1
git diff --check
```

## Phase F5dt: std layer row tile RLE present host span operation presenter executor session turn step boundary

目的:

- F5ds の poll result と complete result を、future Web / native / bare / headless driver が同じ transient step result として扱える std layer boundary を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnStepResult` は `Execute`、`Continue`、`Yield`、`Completed` を持つ。
- `Completed` は transient Completed result であり、persistent state ではない。terminal state authority は F5dr / F5ds に残す。
- start is setup authority なので `turn_step_start` は F5ds `turn_start` を 1 回だけ呼び、step result ではなく F5ds turn state を返す。
- `turn_step_poll` は F5ds `turn_poll` を 1 回だけ呼び、`Execute` / `Completed` を `TurnStepResult` へ写す。
- `turn_step_complete` は F5ds `turn_complete` を 1 回だけ呼び、`Continue` / `Yield` を `TurnStepResult` へ写す。
- F5dt は real scheduler policy、queue、timer、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic outcome creation に進まない。

plan review:

- Dirac plan review は `PLAN_APPROVED`。
- `turn_step_start -> Result TurnState StartError` は start が scheduler tick outcome ではないため適切と判断された。
- single transient `TurnStepResult` は F5ds poll / complete の戻り値を future scheduler code が直接 lower variant を見ずに消費するための std boundary として有効と判断された。
- `StepResult::Completed` は transient result のみに限定し、persistent completed state を追加しないことが承認条件である。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_step.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnStepResult`、start / poll / complete wrapper errors、start / poll / complete functions、category / lower accessors を追加する。
- `turn_step_start` は F5ds turn start を 1 回だけ呼び、success で turn state をそのまま返す。
- `turn_step_poll` は F5ds turn poll を 1 回だけ呼び、Execute / Completed を single step result へ写す。
- `turn_step_complete` は F5ds turn complete を 1 回だけ呼び、Continue / Yield を single step result へ写す。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_step.n.md` を追加し、facade、result owner、transient completed、start returns turn state、poll / complete normalization、lower recovery authority、no scheduler / platform / fallback の coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dt source policy を追加し、F5ds-only boundary、persistent completed state 禁止、lower bypass 禁止、old action path 禁止、raw / platform / scheduler / fallback leakage 禁止、括弧なし prefix style を固定する。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_step.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_step.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_step.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_step_f5dt.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_step.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_step_module_f5dt.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_f5dt_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dt.json -j 1
git diff --check
```

## Phase F5du: std layer row tile RLE present host span operation presenter executor session turn driver boundary

目的:

- F5dt `Execute` result を actual Web / native / bare / headless executor が扱える owner-bearing driver pending value へ包む。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnDriverPending` は F5dr session pending request を所有し、Clone / Copy しない。
- executor は caller supplied outcome だけを返し、operation identity は pending から borrowed expected operation として読む。
- `turn_driver_complete` は borrowed expected operation と caller supplied outcome から F5do `executor_attempt` を 1 回だけ作り、F5dt `turn_step_complete` を 1 回だけ呼ぶ。
- この boundary で prevents operation mismatch を固定し、F5do complete / request、F5dr / F5dp / F5dq direct completion、real scheduler policy、queue、timer、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit`、synthetic `Result::Err GuiError::` に進まない。

plan review:

- Cicero plan review は `PLAN_APPROVED`。
- F5dr / F5ds に borrowed pending request accessor を追加する方針は owner state を消費せず expected operation を読めるため適切と判断された。
- F5du が F5do を使う範囲は `executor_request_operation` と `executor_attempt` に限定する。
- `turn_driver_complete` は full attempt ではなく `Result unit GuiError` を受け取り、operation identity authority を pending request 側に残す。

実装:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session.nepl` に `session_pending_request_ref` を追加する。
- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn.nepl` に `session_turn_pending_request_ref` を追加する。
- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_driver.nepl` を追加する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_driver.n.md` を追加し、facade、pending owner、borrowed operation、outcome-only complete、poll / complete order、lower recovery authority、no scheduler / platform / fallback の coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5du source policy を追加し、borrowed accessor、F5dt-only completion、F5do usage whitelist、old path / raw / platform / scheduler / fallback leakage 禁止、括弧なし prefix style を固定する。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session.nepl stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn.nepl stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_driver.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_driver.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_driver_f5du.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_driver.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_driver_module_f5du.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_step.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_step_f5du_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5du.json -j 1
git diff --check
```

## Phase F5dv: std layer row tile RLE present host span operation presenter executor session turn scheduler decision boundary

目的:

- F5du driver step result を target-neutral scheduler decision に写し、actual Web / native / bare / headless scheduler backend の直前で実行方針だけを型で固定する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnSchedulerDecision` は `Execute`、`ContinueNow`、`ScheduleOneShot`、`Completed` を持つ。
- `ScheduleOneShot` は validated delay と turn state を持つ scheduled state であり、actual timer backend や queue backend は呼ばない。
- policy constructor と `scheduler_decide` はどちらも `yield_delay_ms >= 0` を検査する。
- invalid policy は owner-bearing policy error として original driver step owner を保持し、caller が pending executor request または turn state を回収できる。
- F5du start / poll / complete / pending operation helper、timer API、queue、real scheduler backend、platform API、DOM / Canvas / minifb、video memory、raw storage、DrawTarget / RenderTarget、fallback、silent no-op、synthetic `Result::Ok unit`、synthetic `Result::Err GuiError::` に進まない。

plan review:

- Cicero plan review 1 は `PLAN_CHANGES`。
- 指摘は、public policy を信頼すると手作り value の negative delay を `ScheduleOneShot` にできるため、`scheduler_decide` が `Result SchedulerDecision SchedulerDecisionError` を返し、policy を再検査する必要があるというものだった。
- invalid policy error は kind だけではなく original step owner を保持する owner-bearing policy error にし、caller recovery を失わないことが要求された。
- revised plan では private validation が validated delay を返し、`ScheduleOneShot` はその delay だけを使う。Cicero revised plan review は `PLAN_APPROVED`。

実装:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_scheduler.nepl` を追加する。
- `SchedulerPolicy`、`SchedulerPolicyErrorKind`、`SchedulerScheduledState`、`SchedulerDecision`、`SchedulerDecisionErrorKind`、`SchedulerDecisionError` を定義する。
- `scheduler_policy` は negative delay を `YieldDelayInvalid` にする。
- `scheduler_decide` は policy を再検査し、`Execute` / `Continue` / `Yield` / `Completed` を `Execute` / `ContinueNow` / `ScheduleOneShot` / `Completed` へ写す。
- `scheduled_state_turn_state` は consuming accessor とし、`scheduled_state_delay_ms` は borrowed accessor とする。
- `decision_error_step` は owner-bearing policy error から original step owner を回収する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_scheduler.n.md` を追加し、facade、policy validation、decision mapping、ScheduleOneShot、owner-bearing error、scheduled state recovery、no timer / platform / fallback の coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dv source policy を追加し、docs、facade、type shape、policy revalidation、validated delay use、owner recovery、forbidden timer / queue / platform / fallback、括弧なし prefix style を検査する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_scheduler.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_scheduler.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_scheduler.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_scheduler_f5dv.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_scheduler.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_scheduler_module_f5dv.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_driver.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_driver_f5dv_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dv.json -j 1
git diff --check
```

## Phase F5dw: std layer row tile RLE present host span operation presenter executor session turn timer request boundary

目的:

- F5dv scheduler decision を actual Web / native / bare / headless timer backend の直前で target-neutral timer request value に写す。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnTimerReady` は `Execute`、`ContinueNow`、`ScheduleTimer`、`Completed` を持つ。
- `ScheduleTimer` は owner-bearing timer pending を持ち、pending は F5dv scheduled state と std `TimerRequest` を所有する。
- timer policy は `WindowId` と `TimerId` を持つ。`TimerId` は unchecked raw wrapper なので、policy constructor と interpret の両方で `timer_id_raw > 0` を検査する。
- `TimerRequest` は policy と scheduled delay を検査した後だけ作り、one-shot timer request として `repeating false` を固定する。
- invalid policy と invalid scheduled delay は original scheduler decision を保持する owner-bearing interpret error にする。
- timer completion は pending request timer id、incoming `TimerEvent` timer id、tick を検査し、成功時だけ scheduled turn state を回収して F5dv `ContinueNow` decision を返す。
- F5du start / poll / complete / pending operation helper、actual timer backend registration、queue、real scheduler backend、platform API、DOM / Canvas / minifb、video memory、raw storage、DrawTarget / RenderTarget、fallback、silent no-op、synthetic `Result::Ok unit`、synthetic `Result::Err GuiError::` に進まない。

plan review:

- Cicero plan review 1 は `PLAN_CHANGES`。
- 指摘は、`TimerId` が unchecked raw wrapper であるため、timer policy は `Result Policy PolicyErrorKind` にし、`TimerIdInvalid` を持つ必要があるというものだった。
- `interpret_decision` は `SchedulerDecision` を match / consume する前に borrowed policy を再検査し、invalid policy path では original decision owner を保持する必要がある。
- `TimerRequest` は policy validation と scheduled delay validation の後にだけ作る。invalid scheduled delay path も original `ScheduleOneShot scheduled` decision を再構成して保持する。
- `complete_timer` は F5dv `SchedulerDecision::ContinueNow state` を返す形が正しい。pending と `TimerEvent` を保持する owner-bearing complete error は acceptable。
- revised plan では timer id positive validation、decision before-consume validation、scheduled delay revalidation、no pre-validation `TimerRequest`、no queue / platform / fallback を source policy で固定する。Cicero revised plan review は `PLAN_APPROVED`。

実装:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_timer.nepl` を追加する。
- `TurnTimerPolicy`、`TurnTimerPolicyErrorKind`、`TurnTimerPending`、`TurnTimerReady`、`TurnTimerInterpretErrorKind`、`TurnTimerInterpretError`、`TurnTimerCompleteErrorKind`、`TurnTimerCompleteError` を定義する。
- `turn_timer_policy` は `timer_id_raw > 0` を検査して `Result Policy PolicyErrorKind` を返す。
- private `validate_policy_for_interpret` は borrowed policy を再検査し、checked `TimerId` を返す。
- `turn_timer_interpret_decision` は policy を再検査してから scheduler decision を match し、`ScheduleOneShot` では delay を再検査してから `timer_request window timer delay_ms false` を作る。
- `turn_timer_complete` は pending request timer id、event timer id、tick、id match の順で検査し、成功時だけ scheduled turn state を回収して `SchedulerDecision::ContinueNow state` を返す。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_timer.n.md` を追加し、facade、policy validation、interpret order、one-shot request、owner-bearing interpret error、complete event validation、no backend / queue / fallback の coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dw source policy を追加し、docs、facade、type shape、policy revalidation、TimerRequest creation order、completion validation、forbidden backend / queue / platform / fallback、括弧なし prefix style を検査する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_timer.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_timer.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_timer.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_timer_f5dw.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_timer.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_timer_module_f5dw.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_scheduler.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_scheduler_f5dw_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui.nepl --no-tree -o tmp_gui_std_gui_facade_f5dw.json -j 1
git diff --check
```

## Phase F5dz: std layer row tile RLE present host span operation presenter executor session turn virtual timer bridge

目的:

- F5dw timer pending request と F5dy deterministic virtual timer scheduler を std layer で接続する。
- headless / offscreen test は actual Web / native / bare timer backend、queue、real scheduler loop を使わず、`GuiEvent::Timer` によって scheduled turn を再開できる。
- schedule / advance / complete の各 authority を F5dw / F5dy に残し、新規 bridge は owner recovery と接続順序だけを担う。

plan review:

- Cicero plan review 1 は `PLAN_BLOCKED`。
- 指摘は、advance failure、unexpected event、timer complete failure の recovery payload から virtual timer state が失われること、schedule error の lower `GuiError` が category に縮約されていること、recovery accessor が計画されていないことだった。
- revised plan では schedule error が original F5dw pending と original virtual state と lower `GuiError` を保持し、advance failure が original combined pending と lower `GuiError` を保持する。
- unexpected event は F5dw pending、advance-after virtual timer state、event を保持し、timer complete failure は F5dw complete error と advance-after virtual timer state を保持する。
- source policy は `gui_virtual_timer_schedule`、`gui_virtual_timer_advance`、F5dw `turn_timer_complete` をそれぞれ該当 path で 1 回だけ呼ぶこと、loop / drain / backend / queue / fallback に進まないことを固定する。Cicero revised plan review は `PLAN_APPROVED`。

実装:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerPending` は F5dw pending と `GuiVirtualTimerState` を所有する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerAdvance` は `Pending` または `Ready SchedulerDecision` を返す。
- schedule は F5dw pending から borrowed `TimerRequest` を読み、`gui_virtual_timer_schedule` を 1 回だけ呼ぶ。
- advance は `gui_virtual_timer_advance` を 1 回だけ呼び、`Option::None` は next pending、`GuiEvent::Timer` は F5dw `turn_timer_complete`、timer 以外は owner-bearing unexpected event error に写す。
- schedule error、advance failed error、unexpected event error、complete failed error に category / lower / pending / timer_state / event の recovery accessor を追加する。
- owner-bearing pending、advance、error payload、top-level advance error には Clone / Copy を実装しない。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.n.md` を追加し、facade、schedule owner recovery、advance owner recovery、timer complete state recovery、unexpected event、exact authority calls、no loop / backend / queue / fallback の coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` と `nodesrc/test_web_gui_offscreen_headless_contract.js` に F5dz source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`doc/neplg2/gui_redesign_detailed_design.md`、`doc/neplg2/gui_redesign_implementation_plan.md`、`note.n.md`、`todo.md` を更新する。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_offscreen_headless_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer_f5dz.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer_module_f5dz.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_virtual_timer.n.md --no-tree -o tmp_gui_std_virtual_timer_f5dz_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_timer.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_timer_f5dz_regression.json -j 1
git diff --check
```

## Phase F5ea: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler state boundary

目的:

- F5dv scheduler decision、F5dw timer request、F5dz virtual timer bridge を deterministic scheduler state として接続する。
- actual scheduler loop、timeslice policy、event queue、platform timer backend を実装する前に、headless / offscreen test が扱う phase-owned state を固定する。
- `GuiVirtualTimerState` を policy ではなく dynamic state として保持し、`ContinueNow` を no-progress decision reuse にしない。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState` を追加し、`Turn`、`WaitingTimer`、`Execute`、`Completed` を持たせる。
- `Turn`、`Execute`、`Completed` の payload は current `GuiVirtualTimerState` を保持し、`WaitingTimer` は F5dz pending に保持させる。
- decision boundary は F5dw `turn_timer_interpret_decision` を 1 回だけ呼び、`ContinueNow` を `Turn`、`ScheduleTimer` を F5dz schedule、`Execute` と `Completed` を明示 phase へ写す。
- timer advance boundary は F5dz `virtual_timer_advance` を 1 回だけ呼び、`Ready` decision では one-shot complete 済みとして `gui_virtual_timer_empty` を渡して decision boundary へ戻す。
- interpret failure、schedule failure、timer advance failure、ready decision failure は lower error と recovery payload を持つ owner-bearing error にする。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler.n.md` を追加し、phase state、ContinueNow -> Turn、schedule owner recovery、ready empty timer、exact authority calls、no loop / backend / queue / fallback の coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` と `nodesrc/test_web_gui_offscreen_headless_contract.js` に F5ea source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`doc/neplg2/gui_redesign_detailed_design.md`、`doc/neplg2/gui_redesign_implementation_plan.md`、`note.n.md`、`todo.md` を更新する。

非目標:

- general scheduler loop、loop drain、timeslice budget、real Web / native / bare timer backend、event queue、platform API、DOM、Canvas、minifb、video memory、DrawTarget、RenderTarget、fallback、silent no-op は実装しない。

検証:

```text
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler.n.md
node --check nodesrc/test_web_gui_font_rendering_contract.js
node --check nodesrc/test_web_gui_offscreen_headless_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_offscreen_headless_contract.js
node nodesrc/test_stdlib_gui_layering_policy.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_f5ea.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler.nepl --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_module_f5ea.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.n.md --no-tree -o tmp_gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer_f5ea_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_virtual_timer.n.md --no-tree -o tmp_gui_std_virtual_timer_f5ea_regression.json -j 1
git diff --check
```

subagent review:

- 実装前 review では `PLAN_BLOCKED` として dynamic timer state を policy に入れる設計、`ContinueNow` を reusable decision に戻す no-progress state、`Ready` 後 timer state の暗黙消失が指摘された。
- revised plan では dynamic state を phase payload に移し、`ContinueNow` を `Turn` phase、`Ready` 後を明示 `gui_virtual_timer_empty` として固定し、Cicero revised plan review は `PLAN_APPROVED`。
- 実装後に、上記の指摘がすべて満たされていること、bridge が real scheduler loop や presentation fallback に進んでいないことを確認させる。

## Phase F5dx: Web formal one-shot timer request backend boundary

目的:

- F5dw が作る `TimerRequest` を Web platform backend の `nepl_gui_web.request_timer` で実際に受ける。
- `platforms/gui/web/timer` は positive window id、positive timer id、non-negative interval だけを request shape として検査し、`repeating false` を invalid にしない。
- Web Shell は repeating timer を `setInterval`、one-shot timer を `setTimeout` に接続する。
- One-shot timer は `GuiEvent::Timer` を enqueue する前に active timer entry を clear する。
- 既存 timer の idempotent reuse は interval と repeating mode の両方が一致する場合だけ許す。
- `interval_ms == 0` は same window / timer id の clear request として扱う。

実装:

- `stdlib/platforms/gui/web/timer.nepl` の invalid 判定から `not repeating` rejection を除く。
- `web/src/terminal/shell.ts` の `GuiRuntimeTimerState` に repeating mode を保持し、clear 時に `clearInterval` / `clearTimeout` を使い分ける。
- `applyGuiRuntimeTimerRequest` は request mode を含めて existing timer を比較し、mode change では既存 timer を clear して再登録する。
- `queueGuiRuntimeTimerTick` は one-shot の場合に event payload を構築してから active timer entry を clear し、その後で `handleGuiInputEvent` に渡す。
- `doc/neplg2/gui_redesign_detailed_design.md` と `doc/neplg2/gui_redesign_implementation_plan.md` の old one-shot-invalid contract を更新する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5dx source policy を追加し、docs、NEPL wrapper validation、TS timer state、setTimeout / setInterval mapping、clear-before-enqueue ordering、forbidden fallback を検査する。
- `note.n.md` と `todo.md` を更新する。

非目標:

- general scheduler loop、time-slice budget、virtual scheduler / real scheduler unification、native / bare / headless timer backend はこの phase では実装しない。
- std / core / alloc へ DOM、Canvas、browser handle、stdout fallback、polling fallback を入れない。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
npm --prefix web run build:ts
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/platforms/gui/web/timer.nepl --no-tree -o tmp_platform_gui_web_timer_f5dx.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/platforms/gui/web.nepl --no-tree -o tmp_platform_gui_web_facade_f5dx.json -j 1
git diff --check
```

## Phase F5bf: sfnt simple glyph raster packed mask owner

目的:

- F5be の completed coverage mask owner を authority とし、raw coverage cell を normalized alpha cell へ変換する。
- completed packed mask owner は raw coverage cells を保持せず、completion 時に raw coverage cell Vec を解放して edge owner と alpha cell Vec だけを保持する。
- render2d command、pixel buffer、DrawTarget / RenderTarget、platform API、font fallback へ進まない。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。
- `alpha_cells.len == cell_index` と `alpha_cells.cap == shape.cell_count` の pack-owner invariant が budget/read/push/completion より前に必要と指摘された。
- `vec::push` failure で部分的に進んだ alpha Vec が戻る可能性を、通常継続可能状態として扱わない contract が必要と指摘された。
- revised plan では `AlphaStorageLenMismatch` / `AlphaStorageCapacityMismatch` と `pack_owner_invariants` を追加し、drain / step / complete の入口で必ず検査する。
- push failure から回収した owner は cleanup / diagnostic 用であり、次に処理へ戻す場合も invariant を通過したときだけ続行できると明文化する。
- Tesla revised plan review は `PLAN_APPROVED`。`alpha_max` only config、typed owner-bearing error、raw coverage cells の completion-time free、edge owner exactly once preservation は実装開始条件を満たす。

変更:

- `GuiSfntSimpleGlyphRasterPackedMaskConfig` を追加する。`alpha_max` だけを持つ value-only record とし、`Clone` / `Copy` を実装する。
- F5bd completed coverage owner の shape / cell_count / cells len / cells cap を読む内部 helper を追加する。
- `GuiSfntSimpleGlyphRasterPackedMaskPackOwner` を module-private transition owner として追加する。completed coverage owner、alpha cell Vec、config、cell_index を保持する。
- `GuiSfntSimpleGlyphRasterPackedMaskOwner` を module-private completed owner として追加する。edge owner、shape、alpha cell Vec、cell_count、alpha_max を保持し、raw coverage cells は保持しない。
- `RasterPackedMaskStartErrorKind` / `RasterPackedMaskStartError` を追加する。start error は original completed coverage owner を必ず保持し、storage allocation failure は lower `StdErrorKind` を保持する。
- start は `alpha_max > 0`、shape invariant、`coverage_max * alpha_max` overflow、raw coverage owner cell_count / len / cap、alpha cell Vec allocation の順に検査する。
- `RasterPackedMaskErrorKind` / `RasterPackedMaskError` を追加する。conversion error は pack owner を保持し、raw cell index / storage error を必要に応じて保持する。
- `pack_owner_invariants` を追加し、cell index、shape invariant、cell upper bound、alpha Vec len/cap、coverage owner cell_count / raw cells len/cap を検査する。
- drain / step / complete は budget handling、raw cell read、normalization、alpha push、completion より前に `pack_owner_invariants` を通す。
- raw coverage read helper を追加し、`vec::get None`、negative coverage、coverage exceeds max を typed error にする。
- alpha normalize helper を追加し、`coverage * alpha_max / coverage_max` を integer-only で行い、multiply overflow を typed error にする。
- step helper は 1 raw coverage cell を alpha cell へ変換して push し、push failure では lower storage error kind を読んでから recovered alpha Vec を pack owner に戻す。
- bounded drain terminal を追加する。`PackedMaskCompleted` と `StepBudgetExhausted` を success enum で分ける。
- completion は exact invariant だけで成功し、raw coverage cell Vec を free して edge owner / shape / alpha cell Vec を completed packed mask owner へ移す。
- pack owner / completed owner / terminal free helper を追加し、owner を exactly once close する。

完了条件:

- source policy が docs、plan review approval、config `Clone` / `Copy`、private pack/completed owner、owner no `Clone` / `Copy`、start validation order、start error coverage owner recovery、pack invariant before budget/read/push/completion、typed alpha storage errors、raw cell read typed errors、alpha normalization overflow guard、push failure recovery contract、bounded terminal、completion raw cell free before packed owner finalization、free functions、forbidden byte-backed / old traversal / zero-fill / render2d / DrawTarget / RenderTarget / platform / fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_packed_mask_owner.n.md` に config/start、shape/raw cell revalidation、pack invariant、raw cell read、alpha normalize、push recovery、budget/completion/free、no fallback/no render policy の coverage label を追加する。
- implementation review で raw coverage owner が start/error/free path から必ず回収可能であること、completed owner が raw coverage Vec を保持しないこと、edge owner が exactly once 引き継がれることを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の render2d glyph alpha mask boundary phase へ進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_packed_mask_owner.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_raster_packed_mask_owner_f5bf.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_coverage_scan_converter.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_raster_coverage_scan_converter_f5bf_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5bf.json -j 1
git diff --check
```

## Phase F5bg: sfnt simple glyph render fill alpha mask boundary

目的:

- F5bf の completed packed alpha mask owner を authority とし、後続 2D renderer が消費できる fill alpha mask owner へ所有権を移す。
- packed owner の shape / alpha storage invariant を再検査し、検査通過後に edge owner と alpha cell Vec を zero-copy で移す。
- full `GuiGlyphPaint` は受けない。F5bf の alpha cells は fill coverage なので、stroke / shadow / full paint は後続の専用境界で明示的に扱う。
- `RenderCommand` emission、DrawTarget / RenderTarget、platform API、font fallback へ進まない。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。
- 当初案の `GuiGlyphPaint` config は stroke-only や shadow-bearing paint を fill coverage として扱ってしまう危険があり、hidden stroke / shadow fallback になると指摘された。
- revised plan では `GuiGlyphPaint` を config から削除し、`GuiPaint` と `GuiBlendMode` を持つ fill alpha mask 専用 config に変更する。
- stroke / shadow / full glyph paint binding は F5bg では扱わず、後続境界で明示的に accept / reject する。
- Tesla revised plan review は `PLAN_APPROVED`。fill paint と blend の exact preservation、zero-copy owner handoff、shape / alpha storage revalidation before destructuring が実装条件である。

変更:

- `core/gui/geometry`、`core/gui/render_command`、`core/gui/render_style` の value contract を `glyf.nepl` から参照する。`std/gui`、`platforms`、DOM / Canvas / host API は参照しない。
- F5bf completed packed mask owner の shape / cell_count / alpha_max / alpha Vec len / cap を読む内部 helper を追加する。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskConfig` を追加する。`origin`、`fill_paint`、`blend` を持つ value-only record とし、`Clone` / `Copy` を実装する。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskOwner` を module-private completed owner として追加する。edge owner、shape、alpha cell Vec、cell_count、alpha_max、origin、fill_paint、blend を保持し、`Clone` / `Copy` は実装しない。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskStartErrorKind` と `GuiSfntSimpleGlyphRenderFillAlphaMaskStartError` を追加する。start error は original packed owner と config を保持する。
- start は shape invariant、`alpha_max > 0`、packed owner cell_count、alpha Vec len / cap を検査してから packed owner を destructure する。
- success path は edge owner、shape、alpha cell Vec、cell_count、alpha_max、origin、fill_paint、blend を completed render fill alpha mask owner へ移す。
- start error の kind / config / packed owner recovery helper と、start error free helperを追加する。
- completed owner の origin / fill_paint / blend / shape / size / cell_count / alpha_max / alpha cells len / cap accessor と free helper を追加する。

完了条件:

- source policy が docs、plan review blocker と revised approval、config `Clone` / `Copy`、owner / start error no `Clone` / `Copy`、private owner、shape / alpha storage invariant、start error recovery、fill paint / blend exact preservation、no `GuiGlyphPaint`、no stroke / shadow binding、no byte-backed / old traversal / zero-fill / RenderCommand emission / DrawTarget / RenderTarget / platform / fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_boundary.n.md` に config、shape / alpha revalidation、fill paint / blend preservation、owner handoff、recovery、free、no platform / no command policy の coverage label を追加する。
- implementation review で owner handoff と free order、full glyph paint を受けない設計、stroke / shadow の未対応を hidden success にしていないことを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は F5bg 後の render command / 2D compositor boundary、stroke / shadow / full glyph paint binding を明示した残件へ更新する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_boundary.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_render_fill_alpha_mask_boundary_f5bg.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_packed_mask_owner.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_raster_packed_mask_owner_f5bg_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5bg.json -j 1
git diff --check
```

## Phase F5bh: sfnt simple glyph render glyph paint binding boundary

目的:

- F5bg の fill alpha mask owner start を authority とし、full `GuiGlyphPaint` から現在の renderer が扱える fill-only subset を明示的に取り出す。
- stroke / shadow は黙って無視せず、owner-bearing typed error として返す。
- success owner は既存の `GuiSfntSimpleGlyphRenderFillAlphaMaskOwner` とし、F5bh では新しい completed owner や command stream を作らない。
- `RenderCommand` emission、DrawTarget / RenderTarget、2D compositor、platform API、font fallback へ進まない。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。
- lower F5bg error から kind を読む前に lower error を消費しないこと、start signature と success owner type を明記すること、stroke-only / shadow-only paint を `MissingFillPaint` で覆い隠さない validation precedence を固定することが指摘された。
- revised plan では start signature を `Result GuiSfntSimpleGlyphRenderFillAlphaMaskOwner GuiSfntSimpleGlyphRenderGlyphPaintStartError` と明記し、validation order を stroke -> shadow -> fill に固定する。
- F5bg lower error は `gui_sfnt_simple_glyph_render_fill_alpha_mask_start_error_kind &lower_error` で kind を読んだ後に packed owner recovery accessor で消費する。
- Tesla revised plan review は `PLAN_APPROVED`。

変更:

- `GuiSfntSimpleGlyphRenderGlyphPaintConfig` を追加する。`origin` と `paint` を持つ value-only record とし、`Clone` / `Copy` を実装する。
- `GuiSfntSimpleGlyphRenderGlyphPaintStartErrorKind` を追加する。`MissingFillPaint`、`UnsupportedStrokePaint`、`UnsupportedShadowPaint`、`FillAlphaMaskStartFailed` を持つ。
- `GuiSfntSimpleGlyphRenderGlyphPaintStartError` を追加する。kind、packed owner、config、lower F5bg error kind の `Option` を保持する。
- direct validation failure は `lower_kind = Option::None` として返す。
- F5bg delegated failure は lower kind を `Option::Some` として保持し、recovered packed owner と original config を返す。
- `gui_sfnt_simple_glyph_render_glyph_paint_owner_start` を追加する。stroke Some、non-NoShadow、fill None をこの順に検査し、fill-only の場合だけ F5bg config を作って `gui_sfnt_simple_glyph_render_fill_alpha_mask_owner_start` へ委譲する。
- error kind / config / lower kind / packed owner recovery helper と、start error free helper を追加する。

完了条件:

- source policy が docs、plan review blocker と revised approval、config `Clone` / `Copy`、start error no `Clone` / `Copy`、private boundary、return type、validation precedence、F5bg lower error kind read before recovery、owner-bearing error、no RenderCommand / DrawTarget / RenderTarget / platform / fallback / stroke rasterizer / shadow rasterizer、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_glyph_paint_binding.n.md` に config、fill-only accepted path、stroke-before-missing-fill reject、shadow-before-missing-fill reject、missing fill、lower error recovery、no platform / no command policy の coverage label を追加する。
- implementation review で unsupported stroke / shadow が hidden fill-only success になっていないこと、F5bg lower error owner recovery order、staged doctest と note/todo 更新を確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は F5bh 後の render command / 2D compositor boundary、stroke / shadow dedicated raster boundary を明示した残件へ更新する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_glyph_paint_binding.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_render_glyph_paint_binding_f5bh.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_boundary.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_render_fill_alpha_mask_boundary_f5bh_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5bh.json -j 1
git diff --check
```

## Phase F5bi: sfnt simple glyph render fill alpha mask sample cursor boundary

目的:

- F5bh / F5bg の completed fill alpha mask owner を authority とし、後続の 2D compositor / command bridge が消費できる sample cursor boundary を追加する。
- この phase では `RenderCommand`、DrawTarget / RenderTarget、platform API、2D compositor、stroke / shadow rasterization へ進まない。
- cursor read は absolute pixel position、alpha、alpha_max、fill paint、blend を返す。
- start / step error は owner-bearing とし、失敗時に completed owner または cursor を回収できる。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。
- F5bg に completed `GuiSfntSimpleGlyphRenderFillAlphaMaskOwner` の invariant helper がないため、F5bi 側で shape、alpha_max、cell_count、alpha Vec len/cap を再検査する helper が必要と指摘された。
- `cell_index >= cell_count` を completed と扱う計画では、`cell_index > cell_count` の壊れた progress を隠すため、`cell_index > cell_count` を `CellIndexOutOfRange` として completion より前に拒否する必要があると指摘された。
- `origin + local position` は i32 overflow を起こしうるため、`PositionXOverflow` / `PositionYOverflow` と checked addition を `gui_point_new` より前に置く必要があると指摘された。
- `sample_cursor_step` の失敗は cursor を保持する owner-bearing error とし、read / invariant / alpha / position failure から recovery / free helper まで明示する必要があると指摘された。
- revised plan では completed owner invariant、`cell_index > cell_count` before completion、checked addition、step error owner recovery を追加する。
- Tesla revised plan review は `PLAN_APPROVED`。

変更:

- `GuiSfntSimpleGlyphRenderFillAlphaMaskSample` を追加する。`position`、`alpha`、`alpha_max`、`fill_paint`、`blend` を持つ value-only record とし、`Clone` / `Copy` を実装する。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCursor` を追加する。completed fill alpha mask owner と `cell_index` を所有し、`Clone` / `Copy` は実装しない。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCursorErrorKind` を追加する。shape / alpha invariant、cell index bounds、alpha slot/range、position overflow、progress invariant の typed error を持つ。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCursorStartError` と `GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCursorError` を追加する。start error は owner、step error は cursor を回収できる。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCursorTerminal` を追加する。`Sampled sample next_cursor` と `Completed owner` を持つ。
- completed owner invariant helper を追加し、shape、alpha max、owner cell count、alpha Vec len/cap を再検査する。
- cursor invariant は `CellIndexNegative`、`CellIndexOutOfRange`、`Ok unit` の順に fail-closed とする。
- `checked_add_nonnegative_delta` を追加し、`gui_point_new` の前に x / y overflow を検査する。
- cursor start / read / step / recovery accessor / free helpers / terminal free helper を追加する。

完了条件:

- source policy が docs、plan review blocker と revised approval、sample `Clone` / `Copy`、owner-bearing cursor/start error/step error/terminal no `Clone` / `Copy`、private boundary、error kind、completed owner invariant、cursor bounds order、checked addition before `gui_point_new`、read alpha validation、step terminal/recovery/free、no RenderCommand / DrawTarget / RenderTarget / platform / fallback / stroke / shadow / compositor、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_sample_cursor.n.md` に start、completed owner invariant、bounds fail-closed、position overflow、alpha read、step terminal、recovery/free、no platform / no command policy の coverage label を追加する。
- implementation review で F5bi が command/compositor へ進んでいないこと、cursor overflow / bounds order が source policy と一致すること、note/todo 更新が staged set に含まれていることを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は F5bi 後の 2D compositor / render command bridge と stroke / shadow 専用境界を残件として更新する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_sample_cursor.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_render_fill_alpha_mask_sample_cursor_f5bi.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_glyph_paint_binding.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_render_glyph_paint_binding_f5bi_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5bi.json -j 1
git diff --check
```

## Phase F5bj: sfnt simple glyph render fill alpha mask sample command bridge boundary

目的:

- F5bi の sample cursor を authority とし、1 sample を 1 typed `RenderCommand::FillRect` へ変換する command bridge を追加する。
- これは高速 compositor が無い場合の fallback ではなく、alpha scale、paint access、cursor recovery、command emission の correctness boundary である。
- 現行 `FillRectCommand` は blend payload を持たないため、`GuiBlendMode::SourceOver` だけを受理し、それ以外は `UnsupportedBlendMode` で fail closed にする。
- command conversion が成功する前に cursor を進めず、失敗時は元 cursor と rejected sample を error から回収できるようにする。

plan review:

- Planck plan review 1 は `PLAN_BLOCKED`。
- `FillRectCommand` が blend を保持しないため、`sample.blend` を捨てると hidden fallback / semantic loss になると指摘された。
- F5bi の owning `sample_cursor_step` を先に呼ぶと、command conversion failure 時に sample 消費済みで command 未発行という partial completion になり得ると指摘された。
- revised plan では `SourceOver` only validation と、conversion succeeds before cursor advances rule を追加した。
- command conversion failure は元 cursor と `rejected_sample = Some sample` を保持する。F5bi invariant/read failure は `rejected_sample = None` とする。
- Planck revised plan review は `PLAN_APPROVED`。

変更:

- `core/gui/render_command.nepl` に `gui_paint_color` accessor を追加する。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCommandErrorKind` を追加する。`InvalidAlphaMax`、`AlphaNegative`、`AlphaExceedsMax`、`PaintAlphaMultiplyOverflow`、`ScaledAlphaOutOfRange`、`UnsupportedBlendMode` を持つ。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCommandError` を追加し、value-only rejected sample を保持する。
- sample command paint helper を追加し、`gui_paint_color` で RGB と paint alpha を読み、checked alpha scaling の後にだけ `cast i32 u8` を行う。
- sample render command helper を追加し、absolute sample position から 1x1 `GuiRect` を作って `render_command_fill_rect` を返す。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCommandCursorErrorKind` を追加し、F5bi invariant/read failure と command conversion failure を typed に区別する。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCommandCursorError` を追加し、cursor と `Option rejected_sample` を保持する。
- command cursor terminal を追加し、`Command RenderCommand next_cursor` と `Completed owner` を表す。
- command cursor step を追加する。`cell_index == cell_count` は read より前に completed とし、`cell_index < cell_count` では `read &cursor`、sample command conversion、owner handoff to next cursor の順に進める。
- error / terminal free helper を追加する。

完了条件:

- source policy が docs、Planck revised approval、`gui_paint_color` accessor、SourceOver-only validation、unsupported blend error、checked alpha scale before cast、transparent zero alpha command、exact one sample to one FillRect command、conversion-before-advance、rejected sample recovery、owner-bearing error/terminal no `Clone` / `Copy`、free helpers、no RenderTarget / DrawTarget / platform / fallback / zero-fill / stroke / shadow / partial completion、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_sample_command_bridge.n.md` に paint accessor、SourceOver-only、checked alpha scale、transparent zero alpha、FillRect emission、conversion-before-advance、rejected sample recovery、terminal free、no platform / target / fallback policy の coverage label を追加する。
- implementation review で F5bj が target/backend/platform に進んでいないこと、blend semantic loss と partial completion の blocker が解消されていること、note/todo 更新が staged set に含まれていることを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は F5bj 後の alpha-mask/tile command、2D compositor drain、stroke / shadow 専用 raster boundary を残件として更新する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_sample_command_bridge.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_render_fill_alpha_mask_sample_command_bridge_f5bj.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_sample_cursor.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_render_fill_alpha_mask_sample_cursor_f5bj_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/core/gui/render_command.nepl --no-tree -o tmp_gui_core_render_command_f5bj.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5bj.json -j 1
git diff --check
```

## Phase F5bk: sfnt simple glyph alpha mask render command boundary

目的:

- F5bj の per-sample `FillRect` correctness bridge を最終描画経路にせず、core render command に alpha mask resource handle command を追加する。
- `AlphaMaskId` と `AlphaMaskRectCommand` は no_alloc / Copy value とし、mask storage、renderer、host transport、font internals は core に入れない。
- `AlphaMaskRect` は SourceOver 専用 command とし、non-SourceOver glyph blend は command 構築前に fail closed とする。

plan review:

- Planck plan review は `PLAN_APPROVED`。
- 指摘として、`default paint semantics` のような曖昧な表現は使わず、SourceOver 専用 command と明記する。
- mask alpha と `GuiPaint` alpha の合成意味、RGB が `GuiPaint` 由来であること、mask storage / dimensions 解決が core 外であること、missing / unsupported resource が renderer / host の `Result` になることを docs に固定する。
- core 実装は `Vec` / alloc / std / platform / backend / `RenderTarget` / `DrawTarget` / Canvas / DOM / minifb / font internals / fallback を持ち込まない。

変更:

- `stdlib/core/gui/render_command.nepl` に `AlphaMaskId` を追加する。`raw %i32` を保持し、`Clone` / `Copy`、`alpha_mask_id_new`、`alpha_mask_id_raw` を持つ。
- `AlphaMaskRectCommand` を追加する。`mask_id %AlphaMaskId`、`rect %GuiRect`、`paint %GuiPaint` を保持し、`Clone` / `Copy` と field accessor を持つ。
- `RenderCommand` に `AlphaMaskRect %AlphaMaskRectCommand` を追加する。
- `render_command_alpha_mask_rect` を追加し、`AlphaMaskRectCommand` を `RenderCommand::AlphaMaskRect` に包む。
- `tests/stdlib/gui_core_alpha_mask_command.n.md` を追加し、handle、accessor、enum variant、SourceOver-only contract、no alloc / no platform / no fallback policy の coverage label と runnable smoke を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5bk source policy を追加する。

完了条件:

- source policy と render command doc contract が docs、Planck approval、`AlphaMaskId` handle、constructor/raw accessor、`AlphaMaskRectCommand` fields/accessors、`RenderCommand::AlphaMaskRect`、`render_command_alpha_mask_rect`、SourceOver-only docs、mask alpha / paint alpha 合成 contract、no allocation / no platform / no fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `stdlib/core/gui/render_command.nepl` doctest と `tests/stdlib/gui_core_alpha_mask_command.n.md` focused doctest が通る。
- implementation review で F5bk が storage / target / backend / platform / fallback / font internals に進んでいないこと、F5bj の blend semantic loss を再導入していないこと、note/todo 更新が staged set に含まれていることを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は F5bk 後の alpha-mask resource binding、tile / bitmap formal transport、2D compositor drain、stroke / shadow 専用 raster boundary を残件として更新する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
node --check nodesrc/test_stdlib_gui_render_command_doc_contract.js
node nodesrc/test_stdlib_gui_render_command_doc_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_core_alpha_mask_command.n.md --no-tree -o tmp_gui_core_alpha_mask_command_f5bk.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/core/gui/render_command.nepl --no-tree -o tmp_gui_core_render_command_f5bk.json -j 1
git diff --check
```

## Phase F5bl: sfnt simple glyph alpha mask resource reservation boundary

目的:

- F5bg / F5bh の completed fill alpha mask owner を、F5bk の `AlphaMaskId` と同じ owner-bearing reservation value に束ねる。
- F5bl は resource table 登録ではなく、`RenderCommand::AlphaMaskRect` emission でもない。未登録の `AlphaMaskId` command が owner と切り離されることを防ぐ内部 alloc/font 境界にする。
- 後続の table registration slice は F5bl reservation owner を消費して alpha storage を登録し、その後で初めて `render_command_alpha_mask_rect` を呼ぶ。

plan review:

- Planck plan review 1 は `PLAN_BLOCKED`。
- 当初案は borrowed helper で `render_command_alpha_mask_rect` を返す設計だったため、reservation owner を free した後にも Copy command が残り、dangling `AlphaMaskId` command を作れてしまうと指摘された。
- private reservation owner を std/render2d handoff と表現すると、後続 module が直接消費できるかが曖昧だと指摘された。
- revised plan では F5bl が command を一切発行せず、内部 alloc/font reservation owner だけを作る。docs では table 登録、renderability、std/render2d handoff を主張しない。
- success recovery helper は underlying `GuiSfntSimpleGlyphRenderFillAlphaMaskOwner` を消費回収できる形にし、`mask_id` / `rect` / `paint` は value accessor で読む。
- Planck revised plan review は `PLAN_APPROVED`。`render_command_alpha_mask_rect` / `render_command_fill_rect` を source policy で禁止し、SourceOver-only / owner invariant / no alpha copy / no target-platform-fallback を固定する。

変更:

- `GuiSfntSimpleGlyphRenderFillAlphaMaskResourceReservationConfig` を追加する。`mask_id %AlphaMaskId` だけを持つ value-only config とし、`Clone` / `Copy` を実装する。
- private `GuiSfntSimpleGlyphRenderFillAlphaMaskResourceReservationOwner` を追加する。completed fill alpha mask owner、mask id、rect、paint を保持し、`Clone` / `Copy` は実装しない。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskResourceReservationStartErrorKind` を追加する。`InvalidMaskId`、shape invariant、alpha invariant、`UnsupportedBlendMode` を enum variant として持つ。
- start error は original completed fill alpha mask owner と config を保持し、owner recovery helper と free helper を持つ。
- start は `AlphaMaskId.raw > 0`、completed owner shape invariant、`alpha_max > 0`、cell_count / alpha Vec len / cap、`GuiBlendMode::SourceOver` を fail-closed に検査してから reservation owner を作る。
- success path は owner の origin/size から rect を作り、fill paint をそのまま保持する。alpha Vec は copy しない。
- `render_command_alpha_mask_rect`、`render_command_fill_rect`、sample cursor、DrawTarget / RenderTarget、platform / host / backend API、font fallback、zero-fill fallback、2D compositor は呼ばない。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_resource_reservation.n.md` を追加し、source policy coverage label を固定する。

完了条件:

- source policy が docs、Planck revised approval、config/owner/error types、owner-bearing no `Clone` / `Copy`、config `Clone` / `Copy`、mask id validation、completed owner invariant validation、SourceOver-only validation、rect/paint derivation、owner recovery/free、no command emission、no platform/target/fallback、no per-sample FillRect fallback、no alpha Vec copy、括弧なし prefix style、focused doctest coverage label を検査する。
- focused doctest と F5bk / F5bj 回帰、`stdlib/alloc/gui/font/sfnt/glyf.nepl` doctest が通る。
- implementation review で dangling `AlphaMaskId` command を作っていないこと、F5bl docs が resource table 登録を主張していないこと、note/todo 更新が staged set に含まれていることを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は F5bl 後の resource table registration、tile / bitmap formal transport、2D compositor drain、stroke / shadow 専用 raster boundary を残件として更新する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_resource_reservation.n.md --no-tree -o tmp_gui_font_render_fill_alpha_mask_resource_reservation_f5bl.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_core_alpha_mask_command.n.md --no-tree -o tmp_gui_core_alpha_mask_command_f5bl_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_sample_command_bridge.n.md --no-tree -o tmp_gui_font_render_fill_alpha_mask_sample_command_bridge_f5bl_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5bl.json -j 1
git diff --check
```

## Phase F5bm: sfnt simple glyph alpha mask resource table boundary

目的:

- F5bl の reservation owner を消費し、alpha mask id と metadata を private metadata-only table に登録する。
- table は Copy な metadata record だけを保持し、alpha storage owner は registered resource owner に保持する。
- 成功型は updated table owner と registered resource owner を同時に持つ owner-bearing pair とし、table だけが残って resource owner を失う状態を作らない。
- この phase は host-visible resource 登録、transport、`RenderCommand::AlphaMaskRect` emission、2D compositor drain ではない。

plan review:

- Planck plan review は `PLAN_APPROVED`。
- table が owner-bearing resource を `Vec` に持つ設計は、この slice では避ける。`Vec` payload destructor で内部 reservation owner を確実に閉じる contract は未証明なので、F5bm は metadata-only table と registered resource owner の pair にする。
- lookup は `Option ResourceRecord` を返してよいが、storage / renderability / host-visible resource availability を証明しないことを docs に固定する。
- registration は push 前に nonzero id、reservation owner invariant、SourceOver、rect / paint metadata、duplicate id を検査する。
- push failure は元 table owner と reservation owner を typed error から回収できるようにし、partial registration を禁止する。
- implementation review 1 では split consuming accessor が table だけ / reservation だけを返せるため `REVIEW_BLOCKED` となった。修正後は success continuation と error recovery を callback 型の pair recovery にし、片側 owner だけを返す API を source policy で禁止する。

変更:

- `GuiSfntSimpleGlyphRenderFillAlphaMaskResourceRecord` を追加する。`mask_id`、`rect`、`paint`、`width_px`、`height_px`、`cell_count`、`alpha_max` を持つ value-only record とし、`Clone` / `Copy` を実装する。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskResourceTableOwner` を追加する。`records %Vec ResourceRecord` だけを持つ metadata-only owner であり、alpha storage owner は保持しない。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskRegisteredResourceOwner` を追加する。F5bl reservation owner と record を保持し、`Clone` / `Copy` は実装しない。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskResourceTableRegistrationOwner` を追加する。updated table と registered resource owner を同時に保持し、`Clone` / `Copy` は実装しない。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskResourceTableRegisterErrorKind` と owner-bearing error を追加する。invalid id、reservation invariant、metadata mismatch、unsupported blend、duplicate id、table push failure を enum / `Option StdErrorKind` で分離する。
- table new / len / contains / lookup / free、registration success free、registration error recovery / free を追加する。
- success owner を消費して updated table owner と registered resource owner を同時に callback へ渡す helper を追加する。
- register error から table owner と reservation owner を同時に保持する rejected owner を作り、その rejected owner を callback で同時回収する helper を追加する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_resource_table.n.md` を追加し、source policy coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5bm source policy を追加する。

完了条件:

- source policy が docs、Planck approval、metadata-only table、record `Clone` / `Copy`、owner-bearing registered / registration / error no `Clone` / `Copy`、nonzero id、reservation invariant revalidation、rect / paint metadata comparison、duplicate check before push、push failure recovery、no command emission、no platform/target/fallback、no alpha Vec copy、no owner-bearing Vec payload、括弧なし prefix style、focused doctest coverage label を検査する。
- focused doctest と F5bl / F5bk 回帰、`stdlib/alloc/gui/font/sfnt/glyf.nepl` doctest が通る。
- implementation review で metadata-only table と owner-bearing pair の境界、partial registration failure recovery、dangling `AlphaMaskId` command がないこと、note/todo 更新が staged set に含まれていることを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は F5bm 後の registered resource command emission、tile / bitmap formal transport、2D compositor drain、stroke / shadow 専用 raster boundary を残件として更新する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_resource_table.n.md --no-tree -o tmp_gui_font_render_fill_alpha_mask_resource_table_f5bm.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_resource_reservation.n.md --no-tree -o tmp_gui_font_render_fill_alpha_mask_resource_reservation_f5bm_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_core_alpha_mask_command.n.md --no-tree -o tmp_gui_core_alpha_mask_command_f5bm_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5bm.json -j 1
git diff --check
```

## Phase F5bn: sfnt simple glyph alpha mask prepared command owner boundary

目的:

- F5bm の registered resource owner を authority として、`RenderCommand::AlphaMaskRect` を validation 成功後にだけ作る。
- `RenderCommand` は Copy value なので、この phase では raw command を accessor / arbitrary callback で外へ出さない。
- resource owner と command を `PreparedCommandOwner` に閉じ込め、formal transport / drain owner ができるまで dangling `AlphaMaskId` command を作らない。
- この phase は command stream emission、tile / bitmap transport、2D compositor drain、host-visible resource upload ではない。

plan review:

- Planck plan review 1 は `PLAN_BLOCKED`。success callback が raw `RenderCommand` を arbitrary `.R` callback へ渡すと、callback が command だけを保持できるため dangling `AlphaMaskId` command を再導入すると指摘された。
- revised plan では raw command accessor / callback を禁止し、prepared owner 内部に command を保存するだけに変更した。
- Planck revised plan review は `PLAN_APPROVED`。raw Copy `RenderCommand` escape を source policy で禁止し、formal transport / drain owner を次の消費境界として残す条件で実装開始を承認された。

変更:

- `GuiSfntSimpleGlyphRenderFillAlphaMaskResourcePreparedCommandOwner` を追加する。registered resource owner と `RenderCommand` を保持し、`Clone` / `Copy` は実装しない。
- `PreparedCommandOwner` は raw `RenderCommand` を返す accessor、borrow accessor、`RenderCommand` を渡す arbitrary callback helper を持たない。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskResourcePreparedCommandErrorKind` と owner-bearing error を追加する。invalid id、reservation invariant、metadata mismatch、record mismatch、unexpected table-register state を enum で表す。
- registered resource owner から stored record を読み、internal reservation から expected record を再導出し、mask id、rect、paint、width、height、cell count、alpha max を比較する helper を追加する。
- `gui_sfnt_simple_glyph_render_fill_alpha_mask_registered_resource_prepare_command` を追加し、validation 成功後にだけ `render_command_alpha_mask_rect` を呼び、返った command を prepared owner 内部へ保存する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_resource_prepared_command.n.md` を追加し、source policy coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5bn source policy を追加する。

完了条件:

- source policy が docs、Planck blocker / revised approval、prepared owner no `Clone` / `Copy`、error no `Clone` / `Copy`、raw command accessor / callback 禁止、record revalidation、record equality、validated path only `render_command_alpha_mask_rect`、no command stream emission、no platform / target / fallback / table lookup / sample cursor / alpha Vec copy / tile / compositor、括弧なし prefix style、focused doctest coverage label を検査する。
- focused doctest と F5bm / F5bk 回帰、`stdlib/alloc/gui/font/sfnt/glyf.nepl` doctest が通る。
- implementation review で raw Copy `RenderCommand` escape がないこと、prepared owner が formal transport / drain owner より前の lifetime boundary として閉じていること、note/todo 更新が staged set に含まれていることを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は F5bn 後の formal transport / drain owner、tile / bitmap transport、2D compositor drain、stroke / shadow 専用 raster boundary を残件として更新する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_resource_prepared_command.n.md --no-tree -o tmp_gui_font_render_fill_alpha_mask_resource_prepared_command_f5bn.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_resource_table.n.md --no-tree -o tmp_gui_font_render_fill_alpha_mask_resource_table_f5bn_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_core_alpha_mask_command.n.md --no-tree -o tmp_gui_core_alpha_mask_command_f5bn_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5bn.json -j 1
git diff --check
```

## Phase F5bo: Software RGBA8888 surface owner boundary

目的:

- F5bn prepared command の次に進む前に、font rasterizer、widget renderer、offscreen screenshot が共有する software RGBA8888 surface owner を `alloc/gui/render2d` に置く。
- pixel storage は `RegionToken u8` owner とし、raw pointer / raw region accessor を public API に出さない。
- write は owner-consuming、read は borrow-only として、失敗時にも owner を回収できる contract にする。
- この phase では SourceOver drain、F5bn prepared owner consumption、RenderCommand emission、DrawTarget / RenderTarget backend、platform present は実装しない。

plan review:

- Planck plan review 1 は `PLAN_BLOCKED`。surface owner と compositor drain を `alloc/gui/font/sfnt/glyf.nepl` に置く設計は、pixel buffer を font 固有 detail に閉じ込めてしまうため不適切と指摘された。
- revised plan では `stdlib/alloc/gui/render2d.nepl` facade と `stdlib/alloc/gui/render2d/software_surface.nepl` を作り、F5bo は owner-bearing RGBA8888 software surface contract だけに限定する。
- Planck revised plan review は `PLAN_APPROVED`。safe `core/mem` facade のみ、raw accessor 禁止、constructor fail-closed、write failure owner recovery、read borrow-only、source policy による no platform / no font / no fallback を条件に実装開始が承認された。

変更:

- `alloc/gui/render2d.nepl` facade を追加し、`alloc/gui.nepl` から re-export する。
- `GuiRgba8888SoftwareSurfaceShape` を追加する。`width`、`height`、`stride_bytes`、`byte_len` を保持し、`Clone` / `Copy` を実装する。
- `GuiRgba8888SoftwareSurfaceOwner` を追加する。`storage %RegionToken u8` を保持し、`Clone` / `Copy` は実装しない。
- `GuiRgba8888SoftwareSurfaceErrorKind`、`GuiRgba8888SoftwareSurfaceCreateError`、`GuiRgba8888SoftwareSurfaceWriteError` を追加する。
- `gui_rgba8888_software_surface_shape` と `gui_rgba8888_software_surface_create` は invalid geometry、pixel count overflow、stride overflow、byte length overflow、allocation failure を enum error で返す。
- `gui_rgba8888_software_surface_write_pixel` は owner を消費し、失敗時は owner-bearing write error を返す。
- `gui_rgba8888_software_surface_read_pixel` は owner を借用し、`Rgba8888` を返す。
- `gui_rgba8888_software_surface_free` は surface owner を消費して storage を解放する。
- `tests/stdlib/gui_render2d_software_surface.n.md` を追加し、focused doctest coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5bo source policy を追加する。

完了条件:

- source policy が docs、Planck initial blocker / revised approval、render2d facade、alloc/gui facade re-export、owner no `Clone` / `Copy`、shape / error kind `Clone` / `Copy`、safe `core/mem` facade only、constructor validation order、bounds before offset、write owner recovery、read borrow-only、free、no SourceOver / no F5bn prepared owner consumption / no RenderTarget / no DrawTarget / no Canvas / no DOM / no minifb / no font glyf / no fallback / no silent no-op、focused doctest coverage label を検査する。
- focused doctest と F5bn / F5bm / GUI core alpha mask 回帰、source policy が通る。
- implementation review で pixel storage が render2d 共通 boundary に置かれていること、SourceOver drain を premature に実装していないこと、note/todo 更新が staged set に含まれていることを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は F5bo 後の SourceOver alpha-mask drain owner、tile / bitmap transport、stroke / shadow 専用 raster boundary を残件として更新する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_software_surface.n.md --no-tree -o tmp_gui_render2d_software_surface_f5bo.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_resource_prepared_command.n.md --no-tree -o tmp_gui_font_render_fill_alpha_mask_resource_prepared_command_f5bo_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_core_alpha_mask_command.n.md --no-tree -o tmp_gui_core_alpha_mask_command_f5bo_regression.json -j 1
git diff --check
```

## Phase F5bp: SourceOver alpha-mask software drain-start owner boundary

目的:

- F5bn prepared command owner と F5bo software RGBA8888 surface owner を同時に消費する。
- completed drain ではなく、後続の bounded drain step が使う cursor owner を作る。
- この phase では pixel write、SourceOver 合成、dirty region、tile / bitmap transport、host present は実行しない。
- raw `RenderCommand` escape を再導入せず、private command field は start validation helper 内だけで読む。
- error recovery は prepared owner と surface owner を pair のまま扱い、片方だけを取り出す consuming accessor を作らない。

plan review:

- Planck plan review 1 は `PLAN_BLOCKED`。drain 完了ではなく drain-start / drain-cursor owner boundary と明記すること、paired recovery を守ること、private command field の narrow carveout、registered resource の再検証、checked geometry、pixel write 禁止が条件として示された。
- revised implementation は blocker を反映し、F5bp を start validation と cursor owner 作成に限定する。SourceOver pixel write は次 phase に残す。

変更:

- `GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainOwner` を追加する。prepared owner、surface owner、`cell_index` を保持し、`Clone` / `Copy` は実装しない。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainErrorKind` と owner-bearing start error を追加する。
- start error から rejected owner を作り、`rejected_with` callback で prepared owner と surface owner を同時に回収する。split consuming accessor は作らない。
- start validation は prepared owner の registered resource を internal reservation から再検証し、stored record と rederived record を比較する。
- command payload validation は private `command` field を内部で読み、`RenderCommand::AlphaMaskRect` だけを受理し、mask id、rect、paint を rederived record と比較する。
- rect / surface validation は origin、size、checked right、checked bottom、surface containment を順に検査する。
- `gui_sfnt_simple_glyph_render_fill_alpha_mask_software_drain_start` は validation 成功時だけ `cell_index = 0` の owner を返す。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_software_drain.n.md` を追加し、focused doctest coverage label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5bp source policy を追加する。

完了条件:

- source policy が docs、Planck blocker、drain-start cursor boundary、owner no `Clone` / `Copy`、owner-bearing error no `Clone` / `Copy`、paired recovery、split accessor 禁止、private command field の start validation helper 限定、registered resource revalidation、command payload equality、checked geometry、surface containment、no pixel write、no target / platform / fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- focused doctest と F5bn / F5bo / GUI core alpha mask 回帰、source policy が通る。
- implementation review で pixel write へ進んでいないこと、raw command escape がないこと、paired owner recovery が維持されていることを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は F5bp 後の bounded SourceOver drain step、write failure recovery、dirty region、tile / bitmap transport、FHD 60fps batching を残件として更新する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_software_drain.n.md --no-tree -o tmp_gui_font_render_fill_alpha_mask_software_drain_f5bp.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_resource_prepared_command.n.md --no-tree -o tmp_gui_font_render_fill_alpha_mask_resource_prepared_command_f5bp_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_software_surface.n.md --no-tree -o tmp_gui_render2d_software_surface_f5bp_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_core_alpha_mask_command.n.md --no-tree -o tmp_gui_core_alpha_mask_command_f5bp_regression.json -j 1
git diff --check
```

## Phase F5bq: SourceOver alpha-mask software drain-step boundary

目的:

- F5bp の drain owner を bounded work slice として進め、RGBA8888 software surface へ alpha mask を SourceOver 合成する。
- SourceOver 算術は `alloc/gui/render2d/composite.nepl` に置き、font/glyf 固有の owner 境界から分離する。
- software surface write は 4 channel projection を store 前に完了し、projection failure で partial pixel update を起こさない。
- completed / budget exhausted / error を enum と owner-bearing payload で分ける。
- dirty region、tile / bitmap transport、host present、FHD 60fps batching、stroke、shadow は次 phase に残す。

plan review:

- Planck plan review 1 は `PLAN_BLOCKED`。既存 `write_pixel` が channel を順次 store するため、途中 store failure で partial pixel update を返し得ること、completion owner と SourceOver 整数式が未固定であることを指摘された。
- revised plan は `gui_rgba8888_software_surface_write_pixel` 自体を prevalidated projection path に置き換え、render2d SourceOver helper、completed owner、InvalidBudget、StepBudgetExhausted、write failure recovery を明文化した。
- Planck revised plan review は `PLAN_APPROVED`。条件は all channel projections before first store、SourceOver 式と overflow bound の doc/test 固定、no split completed owner accessor、write failure で cell_index を進めないこと、source policy で旧 FillRect bridge と raw command accessor を禁止することである。

変更:

- `alloc/gui/render2d/composite.nepl` を追加し、`GuiRgba8888SourceOverAlphaMaskErrorKind` と `gui_rgba8888_source_over_alpha_mask` を定義する。
- SourceOver RGB は `out_alpha_num = src_a * 255 + dest.a * (255 - src_a)` を分母として保持し、低 alpha 同士の合成で 255 を超える narrow cast に依存しない。
- `alloc/gui/render2d.nepl` facade から composite helper を再公開する。
- `alloc/gui/render2d/software_surface.nepl` の `gui_rgba8888_software_surface_write_pixel` 内部を all-channel projection before store に変更する。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainCompletedOwner`、`GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainTerminal`、`GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainStepError` を追加する。
- `gui_sfnt_simple_glyph_render_fill_alpha_mask_software_drain_owner_step_once` は 1 cell だけを read / composite / write し、成功後だけ `cell_index` を進める。
- `gui_sfnt_simple_glyph_render_fill_alpha_mask_software_drain_to_complete_budget` は completed、InvalidBudget、StepBudgetExhausted、recursive progress を明示的に分ける。
- completed owner finish helper は completed owner を消費し、prepared/resource side を free して surface owner だけを返す。
- `tests/stdlib/gui_render2d_source_over_alpha_mask.n.md` に runnable composite doctest を追加する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_software_drain.n.md` に F5bq source policy labels を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5bq source policy を追加する。

完了条件:

- render2d source policy が prevalidated all-channel projection、SourceOver numerator formula、rounding、`out_alpha_num == 0`、low alpha unpremultiply、u8 cast 前 range check、overflow bound、no platform / no fallback を検査する。
- glyf source policy が completed owner、StepBudgetExhausted、InvalidBudget、unchanged owner on read/composite failure、surface recovery on write failure、advance after successful write、old FillRect bridge禁止、raw command accessor禁止、alpha Vec clone/copy禁止、unchecked rect extent helper禁止を検査する。
- focused doctest、F5bp / F5bn / F5bo / core alpha mask 回帰、source policy、`git diff --check` が通る。
- implementation review で partial pixel update、split accessor、fallback、old command bridge がないことを確認する。
- `note.n.md` と `todo.md` を更新する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_source_over_alpha_mask.n.md --no-tree -o tmp_gui_render2d_source_over_alpha_mask_f5bq.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_software_surface.n.md --no-tree -o tmp_gui_render2d_software_surface_f5bq_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_software_drain.n.md --no-tree -o tmp_gui_font_render_fill_alpha_mask_software_drain_f5bq.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_resource_prepared_command.n.md --no-tree -o tmp_gui_font_render_fill_alpha_mask_resource_prepared_command_f5bq_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_core_alpha_mask_command.n.md --no-tree -o tmp_gui_core_alpha_mask_command_f5bq_regression.json -j 1
git diff --check
```

## Phase F5br: SourceOver alpha-mask dirty-region completion boundary

目的:

- F5bq の completed owner に `DirtyRegion` metadata を追加し、host present / tile transport へ進む前に「どの範囲が変わったか」を typed value として保持する。
- render2d 汎用の surface + dirty owner はこの phase では作らない。複数 glyph drain の dirty merge、tile list、bitmap transport は次 phase 以降で設計する。
- completed owner は `prepared + surface + dirty` を同時に保持し、prepared / surface split accessor は引き続き禁止する。
- dirty は `dirty_region_rect_checked` で作り、失敗時は `DirtyRegionInvalid` の owner-bearing step error で返す。Full / Empty fallback や silent no-op は禁止する。

plan review:

- Tesla plan review は `PLAN_APPROVED`。条件は `dirty_region_rect_checked` を使うこと、dirty construction failure を owner-bearing `StepError` にすること、dirty accessor だけを追加し prepared/surface split accessor を増やさないこと、finish 前に dirty を読む contract を docs/source policy に固定することである。
- Planck plan review は `PLAN_APPROVED`。`DirtyRegion` を completed owner の Copy metadata として持たせる境界は妥当であり、render2d surface+dirty owner は tile / bitmap transport / present 境界まで defer してよいとされた。completion branch では owner を分解する前に `dirty_region_rect_checked` を呼ぶこと、fallback しないことが条件である。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `core/gui/dirty_region` import を追加する。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainErrorKind` に `DirtyRegionInvalid` を追加する。
- `GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainCompletedOwner` に `dirty DirtyRegion` を追加し、Clone / Copy 禁止を維持する。
- `gui_sfnt_simple_glyph_render_fill_alpha_mask_software_drain_dirty_region` を追加し、record rect から `dirty_region_rect_checked` で dirty metadata を作る。
- completion branch は `dirty_region_rect_checked` 成功後だけ `field::get owner "prepared"` / `"surface"` を呼ぶ。
- `gui_sfnt_simple_glyph_render_fill_alpha_mask_software_drain_completed_owner_dirty` を追加する。prepared / surface accessor は追加しない。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_software_drain.n.md` に F5br source policy labels を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5br source policy を追加する。
- `note.n.md` と `todo.md` を更新する。

完了条件:

- docs が dirty metadata、finish 前 dirty read、owner consume 前 checked dirty construction、no generic transport scope、no fallback を説明する。
- source policy が completed owner の dirty field、dirty accessor、`dirty_region_rect_checked`、completion branch order、prepared/surface split accessor 禁止、host / platform / tile / bitmap / DrawTarget / RenderTarget / fallback 禁止を検査する。
- focused doctest、F5bq SourceOver doctest、software surface doctest、glyph full doctest、source policy、`git diff --check` が通る。
- implementation review で owner loss、split accessor、fallback、platform leakage がないことを確認する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_render_fill_alpha_mask_software_drain.n.md --no-tree -o tmp_gui_font_render_fill_alpha_mask_software_drain_f5br.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_source_over_alpha_mask.n.md --no-tree -o tmp_gui_render2d_source_over_alpha_mask_f5br_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_software_surface.n.md --no-tree -o tmp_gui_render2d_software_surface_f5br_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5br.json -j 1
git diff --check
```

## Phase F5bs: SourceOver dirty region set aggregation boundary

目的:

- F5br の completed dirty metadata を、正式な tile / bitmap transport より前に no_alloc fixed-capacity `DirtyRegionSet` へ取り込める contract を作る。
- render2d の generic `surface+dirty owner` はこの phase では作らない。複数 surface owner、tile list、bitmap payload、host present、scheduler policy は後続で設計する。
- `dirty_regions_push_region_checked` は `DirtyRegion::Empty`、`DirtyRegion::Full`、`DirtyRegion::Rect` を `match` で明示し、fallback や silent no-op にしない。
- `DirtyRegion::Rect` は `dirty_regions_push_checked` へ通し、`dirty_region_rect_unchecked` 由来の invalid rect も `GuiError::InvalidGeometry` で拒否する。
- fixed-capacity 2 rect policy を維持し、3 つ目の rect は existing push contract と同じく Full へ昇格する。

plan review:

- Planck plan review は `PLAN_APPROVED`。条件は `Rect` branch で `dirty_regions_push_unchecked` を使わないこと、Empty を silent no-op ではなく dirty なしの明示状態として doc 化すること、source policy で allocator / platform / present / tile / bitmap / transport / fallback と unchecked push direct use を禁止することである。
- Tesla plan review は `PLAN_APPROVED`。条件は `#import "core/gui/dirty_region" as *` を追加し、`DirtyRegion::Full` では必ず `dirty_regions_full` を返すこと、`DirtyRegion::Rect` では必ず `dirty_regions_push_checked` を使うこと、`dirty_region_merge` を使わないこと、`doc/neplg2/gui_standard_library_spec.md` も更新することである。

変更:

- `stdlib/core/gui/dirty_region_set.nepl` に `core/gui/dirty_region` import を追加する。
- `dirty_regions_push_region_checked` を追加する。
- `tests/stdlib/gui_dirty_region_set.n.md` に Empty / Rect / Full / invalid unchecked rect / no alloc-platform-fallback source policy labels と focused doctest を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5bs source policy を追加し、checked push 経由、no `dirty_region_merge`、no unchecked push、no platform / present / tile / bitmap / transport / fallback を検査する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- docs が `DirtyRegionSet` aggregation、`dirty_regions_push_region_checked`、fixed-capacity no_alloc policy、no fallback を説明する。
- source policy が helper の explicit match、checked push、`dirty_region_merge` 非使用、platform / transport 非使用、focused label を検査する。
- dirty region set focused doctest、source policy、core GUI regressions、`git diff --check` が通る。
- implementation review で aggregation policy、invalid unchecked rect rejection、deferred surface+dirty owner scope が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_dirty_region_set.n.md --no-tree -o tmp_gui_dirty_region_set_f5bs.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_dirty_region.n.md --no-tree -o tmp_gui_dirty_region_f5bs_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_software_surface.n.md --no-tree -o tmp_gui_render2d_software_surface_f5bs_regression.json -j 1
git diff --check
```

## Phase F5bt: Render2d surface + dirty owner boundary

目的:

- F5bo の `GuiRgba8888SoftwareSurfaceOwner` と F5bs の `DirtyRegionSet` を `alloc/gui/render2d` の surface + dirty owner boundary として束ねる。
- tile / bitmap transport、host present、app effect conversion、Web / native backend、font glyf integration、row batching、pixel writing には進まない。
- dirty metadata の更新は `dirty_regions_push_region_checked` だけを経由し、invalid unchecked rect では元 owner を owner-bearing error で返す。
- `finish_surface` は dirty metadata を捨てる recovery / teardown API として明記し、dirty が必要な caller は finish 前に borrowed dirty accessor で読む。
- surface の raw accessor、mutable accessor、split accessor は追加しない。

plan review:

- Planck plan review は `PLAN_APPROVED`。条件は `dirty_regions_push_region_checked` を `field::get owner "surface"` より前に呼ぶこと、失敗時に元 owner を owner-bearing error に入れること、owner / owner-bearing error に `Clone` / `Copy` を実装しないこと、surface の raw / mutable / split accessor を追加しないこと、`finish_surface` を recovery / teardown API として docs/source policy に明記することである。
- Tesla plan review は `PLAN_APPROVED`。条件は dirty を Copy metadata として読み、surface move 前に checked push を適用すること、`finish_surface` 前に dirty を読む contract を固定すること、free は typed `GuiRgba8888SoftwareSurfaceErrorKind` を返して silent drop しないこと、facade export と no `dirty_region_merge` / no unchecked push / no platform / no present / no tile / no bitmap / no transport / no fallback を source policy に入れることである。

変更:

- `stdlib/alloc/gui/render2d/dirty_surface.nepl` を追加する。
- `GuiRgba8888SoftwareSurfaceDirtyOwner` と `GuiRgba8888SoftwareSurfaceDirtyPushError` を追加する。
- clean owner constructor、shape / dirty borrowed metadata accessor、checked dirty push、error accessor / owner recovery / free、`finish_surface`、dirty owner free を追加する。
- `stdlib/alloc/gui/render2d.nepl` facade から dirty surface owner を再公開する。
- `tests/stdlib/gui_render2d_dirty_surface.n.md` を追加し、clean owner、checked rect push、invalid unchecked rect recovery、Full escalation、finish teardown、no split / platform / fallback label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5bt source policy を追加し、facade export、owner no `Clone` / `Copy`、checked push before surface move、no raw / mutable / split accessor、no platform / transport / fallback を検査する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- source policy が `GuiRgba8888SoftwareSurfaceDirtyOwner`、owner-bearing dirty push error、facade export、`dirty_regions_push_region_checked` 経由、surface move 順序、no split accessor、focused doctest label を検査する。
- focused doctest、source policy、software surface regression、dirty region set regression、`git diff --check` が通る。
- implementation review で surface + dirty owner の所有境界、finish_surface teardown contract、deferred transport scope が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_dirty_surface.n.md --no-tree -o tmp_gui_render2d_dirty_surface_f5bt.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_software_surface.n.md --no-tree -o tmp_gui_render2d_software_surface_f5bt_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_dirty_region_set.n.md --no-tree -o tmp_gui_dirty_region_set_f5bt_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/dirty_surface.nepl --no-tree -o tmp_gui_render2d_dirty_surface_doc_f5bt.json -j 1
git diff --check
```

## Phase F5bu: Render2d validated bitmap frame owner boundary

目的:

- F5bt の `GuiRgba8888SoftwareSurfaceDirtyOwner` を formal tile / bitmap transport の前段で使う validated bitmap frame owner に変換する。
- host present、video memory host import、Canvas / DOM / minifb、row byte copy、tile list、FHD batching、fallback には進まない。
- public struct constructor で forged された dirty owner を前提に、`finish_surface` 前に frame id、surface shape metadata、dirty bounds を再検証する。

plan review:

- Planck plan review 1 は `PLAN_BLOCKED`。当初案の `frame_id >= 0` は既存の positive id contract と衝突するため、`frame_id > 0` に変更する必要があると指摘された。
- Tesla plan review 1 は `PLAN_BLOCKED`。`GuiError::InvalidCommand` だけでは失敗分類が粗く、forged surface metadata と dirty bounds の再検証が不足していると指摘された。
- revised plan では `GuiRgba8888BitmapFramePrepareErrorKind`、positive `frame_id`、shape / stride / byte_len 再検証、DirtyRegionSet state match、dirty rect origin / size / right-bottom overflow / surface bounds validation、validation-before-`finish_surface` を追加する。
- Tesla revised plan review は `PLAN_APPROVED`。`category` は lower API failure ではなく coarse `GuiError` classification として `Option GuiError` で保持する。
- Planck revised plan review は `PLAN_APPROVED`。`frame_id > 0`、shape failure mapping、typed stride / byte length mismatch、dirty bounds validation、owner-bearing prepare error、no raw pixel / no host / no fallback が実装開始条件である。

変更:

- `stdlib/alloc/gui/render2d/bitmap_frame.nepl` を追加する。
- `GuiRgba8888BitmapFramePrepareErrorKind`、`GuiRgba8888BitmapFrameConfig`、`GuiRgba8888BitmapFrameOwner`、`GuiRgba8888BitmapFramePrepareError` を追加する。prepare error kind は `SurfaceStrideMismatch` と `DirtyRectOutOfBounds` を含み、shape mismatch と dirty containment failure を string ではなく typed state として保持する。
- config checked constructor、frame metadata accessors、prepare error kind / category / owner recovery / free、frame `finish_surface` / free を追加する。
- `prepare` は `frame_id > 0`、surface shape / stride / byte_len、dirty set state、dirty rect bounds を全て検査してから `gui_rgba8888_software_surface_dirty_owner_finish_surface` を呼ぶ。
- `stdlib/alloc/gui/render2d.nepl` facade から bitmap frame owner を再公開する。
- `tests/stdlib/gui_render2d_bitmap_frame.n.md` を追加し、positive id config、metadata success、invalid id recovery、forged stride recovery、dirty out-of-bounds recovery、finish teardown、no platform / fallback label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5bu source policy を追加し、docs、subagent approval、facade export、owner no `Clone` / no `Copy`、validation-before-finish order、no raw pixel / no byte copy / no host / no fallback を検査する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- source policy が `GuiRgba8888BitmapFrameOwner`、typed prepare error kind、owner-bearing recovery、positive frame id、surface metadata revalidation、dirty bounds validation、facade export、focused doctest label を検査する。
- focused doctest、source policy、dirty surface regression、software surface regression、dirty region set regression、`git diff --check` が通る。
- implementation review で pre-transport frame owner 境界、validation-before-`finish_surface`、deferred host / platform / fallback scope が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_bitmap_frame.n.md --no-tree -o tmp_gui_render2d_bitmap_frame_f5bu.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_dirty_surface.n.md --no-tree -o tmp_gui_render2d_dirty_surface_f5bu_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_software_surface.n.md --no-tree -o tmp_gui_render2d_software_surface_f5bu_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_dirty_region_set.n.md --no-tree -o tmp_gui_dirty_region_set_f5bu_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/bitmap_frame.nepl --no-tree -o tmp_gui_render2d_bitmap_frame_doc_f5bu.json -j 1
git diff --check
```

## Phase F5bv: Render2d row batch plan owner boundary

目的:

- F5bu の `GuiRgba8888BitmapFrameOwner` を formal byte payload / host present の前段で row batch scheduler が読める row batch plan owner へ変換する。
- dirty state を contiguous row span と batch count に畳むが、row byte payload、tile list、video memory host call、host present、Canvas / DOM / minifb、fallback には進まない。
- 通常 application code の owner aggregate 直 constructor は compiler が拒否するが、compiler memory boundary や trusted producer から forged bitmap frame owner が来てもよい前提で、row planning 前に frame id、surface shape metadata、dirty bounds を再検証する。

plan review:

- Planck plan review 1 は `PLAN_BLOCKED`。当初案では public `GuiRgba8888BitmapFrameOwner` の再検証、typed error、dirty bounds validation、bottom overflow、quotient/remainder batch count、source policy が不足していると指摘された。
- Tesla plan review 1 は `PLAN_BLOCKED`。row planner が F5bu の validated owner を信頼しすぎると、forged public frame metadata から invalid stride / dirty が scheduler authority になるため、F5bv 内で再検証する必要があると指摘された。
- revised plan では `GuiRgba8888RowBatchPlanPrepareErrorKind`、positive `max_rows_per_batch`、positive `frame_id`、shape / stride / byte_len 再検証、DirtyRegionSet state match、dirty rect origin / size / right-bottom overflow / surface bounds validation、contiguous row span、quotient / remainder batch count、owner-bearing prepare error を追加する。
- Planck revised plan review は `PLAN_APPROVED`。typed error、checked bottom arithmetic、dirty set validation、`finish_frame` recovery、no `finish_surface` / no byte payload / no host / no fallback が実装開始条件である。
- Tesla revised plan review は `PLAN_APPROVED`。`Empty` dirty を zero-row clean plan として明示し、`Two` dirty を contiguous row span へ畳み、row/tile transport を後続 phase に残す設計で進めてよい。

変更:

- `stdlib/alloc/gui/render2d/row_batch_plan.nepl` を追加する。
- `GuiRgba8888RowBatchPlanPrepareErrorKind`、`GuiRgba8888RowBatchPlanConfig`、`GuiRgba8888RowBatchPlanOwner`、`GuiRgba8888RowBatchPlanPrepareError` を追加する。prepare error kind は `MaxRowsPerBatchInvalid`、`FrameStrideMismatch`、`DirtyRectBottomOverflow`、`DirtyRectOutOfBounds` を含み、shape mismatch と dirty containment failure を string ではなく typed state として保持する。
- config checked constructor、plan metadata accessors、prepare error kind / category / owner recovery / free、plan `finish_frame` / free を追加する。
- `prepare` は `max_rows_per_batch > 0`、`frame_id > 0`、surface shape / stride / byte_len、dirty set state、dirty rect bounds を全て検査してから row span と batch count を計算する。
- `Empty` は row_start 0 / row_count 0 / batch_count 0 とし、`Full` は height 全体、`One` は single rect の y..bottom、`Two` は 2 rect を覆う contiguous row span にする。
- batch count は `row_count + max_rows_per_batch - 1` を使わず、signed quotient / remainder の ceil division で計算する。
- `stdlib/alloc/gui/render2d.nepl` facade から row batch plan owner を再公開する。
- `tests/stdlib/gui_render2d_row_batch_plan.n.md` を追加し、positive config、Empty / Full / Two span、forged stride recovery、dirty bounds recovery、dirty bottom overflow recovery、finish frame teardown、no platform / fallback label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5bv source policy を追加し、docs、subagent approval、facade export、owner no `Clone` / no `Copy`、metadata revalidation、dirty validation、quotient/remainder batch count、no raw pixel / no byte copy / no host / no fallback を検査する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- source policy が `GuiRgba8888RowBatchPlanOwner`、typed prepare error kind、owner-bearing recovery、positive max rows、positive frame id、frame metadata revalidation、dirty bounds validation、contiguous row span、quotient / remainder batch count、facade export、focused doctest label を検査する。
- focused doctest、module doctest、source policy、bitmap frame regression、dirty surface regression、`git diff --check` が通る。
- implementation review で pre-transport row batch plan 境界、frame revalidation、dirty row span、deferred host / platform / fallback scope が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_plan.n.md --no-tree -o tmp_gui_render2d_row_batch_plan_f5bv.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_batch_plan.nepl --no-tree -o tmp_gui_render2d_row_batch_plan_module_f5bv.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_bitmap_frame.n.md --no-tree -o tmp_gui_render2d_bitmap_frame_f5bv_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_dirty_surface.n.md --no-tree -o tmp_gui_render2d_dirty_surface_f5bv_regression.json -j 1
git diff --check
```

## Phase F5bw: Render2d row batch cursor owner boundary

目的:

- F5bv の `GuiRgba8888RowBatchPlanOwner` を、scheduler が 1 batch ずつ進められる `GuiRgba8888RowBatchCursorOwner` へ変換する。
- cursor は `status` と `next_batch` に分け、`Complete` の owner-bearing terminal wrapper は作らない。
- emitted batch は `GuiRgba8888RowBatchDescriptor` metadata と continuation cursor owner だけを持ち、row byte payload、row copy、tile list、video memory host call、host present、Canvas / DOM / minifb、fallback には進まない。

plan review:

- Tesla plan review は `PLAN_BLOCKED`。start は forged plan owner を前提に full `GuiRgba8888RowBatchPlanOwner` invariant を再検証し、frame_id、shape / stride / byte_len、dirty rect、dirty span、batch count mismatch を typed error として返す必要があると指摘された。また、drain / budget と cursor step を同じ slice に入れると owner-bearing terminal が増えすぎるため分離すべきと指摘された。
- Planck plan review は `PLAN_BLOCKED`。当初案の `StepTerminal` / `CompleteOwner` / duplicated plan invariant mapping は型検査負荷と owner 解析負荷を増やすため、`status + next_batch` に縮小し、`PlanInvariant GuiRgba8888RowBatchPlanInvariantErrorKind` payload で lower invariant precision を保持する必要があると指摘された。
- revised plan では full plan invariant revalidation を `start` に限定し、`status` と `next_batch` は start 済み cursor の local index boundary を検査する。drain / budget は F5bw から外し、後続 phase で `status` と `next_batch` を使って設計する。

変更:

- `stdlib/alloc/gui/render2d/row_batch_cursor.nepl` を追加する。
- `GuiRgba8888RowBatchCursorErrorKind`、`GuiRgba8888RowBatchCursorStatus`、`GuiRgba8888RowBatchDescriptor`、`GuiRgba8888RowBatchCursorOwner`、`GuiRgba8888RowBatchCursorStartError`、`GuiRgba8888RowBatchCursorStepError`、`GuiRgba8888RowBatchCursorBatchOwner` を追加する。
- cursor error kind は `PlanInvariant %GuiRgba8888RowBatchPlanInvariantErrorKind` を持ち、plan invariant enum を cursor layer に重複コピーしない。
- `gui_rgba8888_row_batch_cursor_start` は `gui_rgba8888_row_batch_plan_validate_invariants` を通してから batch_index 0 の cursor owner を返す。失敗時は plan owner を保持する start error を返す。
- `gui_rgba8888_row_batch_cursor_status` は `Ready` / `Complete` を返す。`batch_index < 0` と `batch_index > batch_count` は error、`batch_index == batch_count` だけが `Complete` である。
- `gui_rgba8888_row_batch_cursor_next_batch` は `Ready` cursor だけを 1 batch 進める。descriptor は frame_id、batch_index、row_start、row_count、width、height、stride_bytes、byte_len の Copy metadata だけを持ち、next cursor index は checked arithmetic で計算する。
- `stdlib/alloc/gui/render2d.nepl` facade から row batch cursor owner を再公開する。
- `tests/stdlib/gui_render2d_row_batch_cursor.n.md` を追加し、facade、start revalidation、empty dirty complete status、full dirty first descriptor、owner constructor restriction、no platform / fallback label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5bw source policy を追加し、docs、blocked review feedback、facade export、owner no `Clone` / no `Copy`、no `CompleteOwner` / no `StepTerminal` / no drain budget、payload plan invariant、status / next_batch order、checked next index、no raw pixel / no byte copy / no host / no fallback を検査する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row batch plan regression、source policy、module doctest、`git diff --check` が通る。
- implementation review で owner-bearing complete terminal が戻っていないこと、drain / budget が F5bw から分離されていること、plan invariant precision が payload enum で保持されていること、host / platform / fallback へ進んでいないことを確認する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_cursor.n.md --no-tree -o tmp_gui_render2d_row_batch_cursor_f5bw.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_plan.n.md --no-tree -o tmp_gui_render2d_row_batch_plan_f5bw_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_batch_cursor.nepl --no-tree -o tmp_gui_render2d_row_batch_cursor_module_f5bw.json -j 1
git diff --check
```

## Phase F5bx: Render2d row batch scheduler drain boundary

目的:

- F5bw の `GuiRgba8888RowBatchCursorOwner` を、scheduler の time slice 内で bounded に進める。
- terminal は owner-bearing struct と Copy status enum に分け、Resource checker が cursor owner を単純に追跡できる形にする。
- `emitted_count` はこの call で進めた batch descriptor 数だけを表し、row byte payload / tile / RLE / host present の authority にはしない。

plan review:

- 初期案では `StepBudgetExhausted cursor count` / `Completed cursor count` の enum variant に owner と count を直接持たせる設計だった。
- Planck plan review は `PLAN_BLOCKED`。complete 判定を budget より先に置くこと、zero budget と negative budget を分けること、negative budget を owner-bearing `InvalidBudget` にすること、descriptor index と continuation cursor index の progress invariant を検査すること、checked emitted count を固定することが指摘された。
- Tesla revised plan review は `PLAN_APPROVED`。status-before-budget、negative budget error、zero budget exhaustion、progress invariant、checked count、no payload / host / platform / fallback を source policy に固定する条件で承認された。
- 実装時に owner-bearing enum variant へ cursor を直接入れる形は Resource checker と parser 負荷が高かったため、`GuiRgba8888RowBatchDrainTerminal` struct と `GuiRgba8888RowBatchDrainStatus` Copy enum に分ける形へ修正した。

変更:

- `stdlib/alloc/gui/render2d/row_batch_drain.nepl` を追加する。
- `GuiRgba8888RowBatchDrainErrorKind`、`GuiRgba8888RowBatchDrainStatus`、`GuiRgba8888RowBatchDrainTerminal`、`GuiRgba8888RowBatchDrainError` を追加する。
- `GuiRgba8888RowBatchDrainErrorKind` は `CursorStepFailed %GuiRgba8888RowBatchCursorErrorKind`、`InvalidBudget`、`ProgressInvariantInvalid`、`EmittedCountOverflow` を持つ。
- `gui_rgba8888_row_batch_drain_budget` は status を budget より先に読み、complete cursor を budget exhaustion に隠さない。
- Ready cursor で `remaining_steps < 0` は `InvalidBudget`、`remaining_steps == 0` は `StepBudgetExhausted`、positive budget は `next_batch` 1 回と progress invariant 検査を繰り返す。
- descriptor batch index と previous cursor index、continuation cursor index と `previous + 1` を検査し、count 加算も checked arithmetic にする。
- `stdlib/alloc/gui/render2d.nepl` facade から row batch drain を再公開する。
- `tests/stdlib/gui_render2d_row_batch_drain.n.md` を追加し、complete-before-budget、negative budget error、zero budget exhausted、partial budget progress、completion count、no platform / fallback label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5bx source policy を追加し、docs、facade export、status struct terminal、owner no Clone / Copy、status-before-budget、negative / zero budget、progress invariant、checked count、forbidden payload / host / platform / fallback を検査する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row batch cursor / plan regression、source policy、module doctest、`git diff --check` が通る。
- implementation review で F5bx が scheduler-progress-only であり、row payload / host present / fallback に進んでいないこと、negative budget と zero budget が明確に分かれていること、owner-bearing terminal/error が Clone / Copy されないことを確認する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_drain.n.md --no-tree -o tmp_gui_render2d_row_batch_drain_f5bx.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_batch_drain.nepl --no-tree -o tmp_gui_render2d_row_batch_drain_module_f5bx.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_cursor.n.md --no-tree -o tmp_gui_render2d_row_batch_cursor_f5bx_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_plan.n.md --no-tree -o tmp_gui_render2d_row_batch_plan_f5bx_regression.json -j 1
git diff --check
```

## Phase F5by: Render2d row batch range metadata boundary

目的:

- F5bw の `GuiRgba8888RowBatchCursorBatchOwner` を authority とし、row batch が参照する row span と byte offset range の Copy metadata を作る。
- `GuiRgba8888RowBatchRangeOwner` は元の batch owner を保持し、`start_byte_offset` と `byte_count` は checked arithmetic で導出する。
- この phase は byte storage / row copy / tile / RLE / host present / video memory / platform API / fallback へ進まない。

plan review:

- Franklin plan review 1 は `PLAN_BLOCKED`。metadata-only なのに payload 名を使うと後続の actual byte authority と混ざるため、`row_batch_range` / `GuiRgba8888RowBatchRangeOwner` へ改名する必要があると指摘された。
- Helmholtz plan review 1 は `PLAN_BLOCKED`。continuation cursor を検査するには `GuiRgba8888RowBatchCursorBatchOwner` を消費しない borrowed accessor が必要であり、range prepare 内で `gui_rgba8888_row_batch_cursor_batch_finish_cursor` を呼んではならないと指摘された。
- Franklin plan review 2 は `PLAN_BLOCKED`。descriptor が内部的に正しくても embedded plan 由来か分からないため、cursor 側に borrowed descriptor authority helper を置き、batch owner 内の plan から正規 descriptor を再計算して比較する必要があると指摘された。
- Franklin plan review 3 は `PLAN_BLOCKED`。embedded plan も forged されうるため、authority helper は plan invariant path を先に通し、`PlanInvariant lower_kind` を保持する必要があると指摘された。
- revised plan では `gui_rgba8888_row_batch_cursor_batch_cursor_ref` と `gui_rgba8888_row_batch_cursor_batch_validate_descriptor_authority` を F5bw prerequisite として追加し、F5by prepare は authority validation、descriptor range validation、continuation validation の順にする。
- Helmholtz revised plan は `PLAN_APPROVED`。Franklin は plan invariant validation を加える条件で implementation start を認めた。

変更:

- `stdlib/alloc/gui/render2d/row_batch_cursor.nepl` に `BatchDescriptorMismatch`、borrowed cursor accessor、borrowed descriptor authority validator を追加する。
- authority validator は continuation cursor と embedded plan を借用し、plan invariant を再検査し、`continuation_index - 1` の descriptor を再計算して frame_id、batch_index、row_start、row_count、width、height、stride_bytes、byte_len を比較する。
- `stdlib/alloc/gui/render2d/row_batch_range.nepl` を追加する。
- `GuiRgba8888RowBatchRangePrepareErrorKind` は `BatchAuthorityInvalid %GuiRgba8888RowBatchCursorErrorKind` と `ContinuationCursorInvalid %GuiRgba8888RowBatchCursorErrorKind` を分ける。
- `GuiRgba8888RowBatchRange` は frame_id、batch_index、row_start、row_count、width、height、stride_bytes、byte_len、`start_byte_offset`、`byte_count` を持つ Copy metadata とする。
- `GuiRgba8888RowBatchRangeOwner` と `GuiRgba8888RowBatchRangePrepareError` は batch owner を保持するため `Clone` / `Copy` を実装しない。
- `gui_rgba8888_row_batch_range_prepare` は authority validation の後に checked stride / byte_len / row extent / offset range / continuation index / continuation status を検査する。
- `stdlib/alloc/gui/render2d.nepl` facade から row batch range を再公開する。
- `tests/stdlib/gui_render2d_row_batch_range.n.md` を追加し、facade、first batch metadata、partial batch offset、forged owner constructor restriction、no platform / fallback label を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5by source policy を追加し、docs、cursor helper、facade export、owner no `Clone` / no `Copy`、authority-before-range、checked arithmetic、continuation lower error preservation、forbidden byte storage / host / platform / fallback を検査する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row batch cursor / drain / plan regression、source policy、module doctest、`git diff --check` が通る。
- implementation review で descriptor authority validation が plan invariant を通すこと、range prepare が batch owner を検査中に消費しないこと、`BatchAuthorityInvalid` と `ContinuationCursorInvalid` が lower cursor error を保持すること、row byte storage / tile / RLE / host present / fallback に進んでいないことを確認する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_range.n.md --no-tree -o tmp_gui_render2d_row_batch_range_f5by.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_batch_range.nepl --no-tree -o tmp_gui_render2d_row_batch_range_module_f5by.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_cursor.n.md --no-tree -o tmp_gui_render2d_row_batch_cursor_f5by_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_drain.n.md --no-tree -o tmp_gui_render2d_row_batch_drain_f5by_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_plan.n.md --no-tree -o tmp_gui_render2d_row_batch_plan_f5by_regression.json -j 1
git diff --check
```

## Phase F5bz: Render2d row byte storage boundary

目的:

- F5by の `GuiRgba8888RowBatchRangeOwner` を、formal tile / RLE / host present の前段となる `GuiRgba8888RowByteStorageOwner` へ変換する。
- source surface storage は private sealed helper 内だけで借用し、public API へ source `RegionToken` / `MemPtr` / raw storage accessor を出さない。
- copied byte storage は exact `byte_count` で確保し、copy が完全に成功するまで range owner を消費しない。
- この phase は no tile / RLE / host present とし、video memory、platform API、Canvas、DOM、minifb、fallback、silent no-op へ進まない。

plan review:

- Dewey plan review 1 は `PLAN_BLOCKED`。source byte access boundary が曖昧で、source `RegionToken` / `MemPtr` が public に漏れないこと、success / error owner path、scratch dealloc failure、`RangeMetadataMismatch` が明示されていないと指摘された。
- Einstein plan review 1 は条件付きで進行可能。range authority の再検証、success-only cursor finish、scratch cleanup の typed error を source policy と focused doctest へ入れる必要があるとされた。
- revised plan は Dewey / Einstein とも `PLAN_APPROVED`。`row_byte_storage` だけが source storage を borrow し、public raw accessor を出さず、`gui_rgba8888_row_batch_range_owner_validate_authority` のあとに exact allocation / copy / success-only cursor finish を行う方針で承認された。

変更:

- `stdlib/alloc/gui/render2d/row_batch_range.nepl` に `RangeMetadataMismatch` と borrowed `gui_rgba8888_row_batch_range_owner_validate_authority` を追加する。
- `stdlib/alloc/gui/render2d/row_byte_storage.nepl` を追加する。
- `GuiRgba8888RowByteStorageCopyErrorKind`、`GuiRgba8888RowByteStoragePrepareErrorKind`、`GuiRgba8888RowByteStorageReadErrorKind`、`GuiRgba8888RowByteStorageFinishErrorKind` を enum として分ける。
- `GuiRgba8888RowByteStorageOwner` と prepare / finish error は owner-bearing なので `Clone` / `Copy` を実装しない。
- prepare は range owner authority を再検証し、exact `byte_count` の scratch storage を確保し、checked offset / bounds / projection / load / store で byte copy を行い、全 copy 成功後だけ continuation cursor を取り出す。
- copy 失敗時は scratch storage を dealloc し、dealloc 失敗は `ScratchDeallocFailed` として元の copy error と区別する。
- `stdlib/alloc/gui/render2d.nepl` facade から row byte storage を再公開する。
- `tests/stdlib/gui_render2d_row_byte_storage.n.md` を追加し、facade、authority revalidation、exact copy、checked byte reader、scratch cleanup policy、raw source escape 禁止、platform / fallback 禁止を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5bz source policy を追加し、docs、facade export、owner no `Clone` / no `Copy`、public raw source API 禁止、validate-before-copy、success-only cursor finish、checked copy、scratch cleanup、括弧なし実装を検査する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row byte storage module doctest、row batch range / cursor / drain / plan regression、source policy、`git diff --check` が通る。
- implementation review で private sealed source access、public source `RegionToken` / `MemPtr` escape 禁止、range owner authority revalidation、copy 成功前の cursor finish 禁止、`ScratchDeallocFailed` の typed cleanup が確認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_byte_storage.n.md --no-tree -o tmp_gui_render2d_row_byte_storage_f5bz.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_byte_storage.nepl --no-tree -o tmp_gui_render2d_row_byte_storage_module_f5bz.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_range.n.md --no-tree -o tmp_gui_render2d_row_batch_range_f5bz_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_cursor.n.md --no-tree -o tmp_gui_render2d_row_batch_cursor_f5bz_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_drain.n.md --no-tree -o tmp_gui_render2d_row_batch_drain_f5bz_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_batch_plan.n.md --no-tree -o tmp_gui_render2d_row_batch_plan_f5bz_regression.json -j 1
git diff --check
```

## Phase F5ca: Render2d row tile plan metadata boundary

目的:

- F5bz の `GuiRgba8888RowByteStorageOwner` を、formal tile payload / RLE / host present の前段となる `GuiRgba8888RowTilePlanOwner` へ変換する。
- `GuiRgba8888RowByteStorageOwner` が保持する continuation cursor と copied range から byte storage authority を借用で再検証する。
- tile descriptor は frame-absolute row range と storage-relative byte offset を持つ metadata に限定する。
- この phase は no RLE / host present とし、byte payload split、RLE encode、video memory、platform API、Canvas、DOM、minifb、fallback、silent no-op へ進まない。

plan review:

- Nietzsche plan review 1 は `PLAN_BLOCKED`。F5ca の抽象度自体は正しいが、先に byte storage owner の borrowed authority helper が必要であり、descriptor offset semantics と error taxonomy と source policy の禁止事項を明記する必要があると指摘された。
- Beauvoir plan review 1 は `PLAN_BLOCKED`。`descriptor_at` は owner を消費せず借用 API にすること、descriptor 計算前に `GuiRgba8888RowTilePlanInvariantErrorKind` と invariant validation を通すこと、`PlanInvariantInvalid lower_kind` を descriptor error に保持することが必要だと指摘された。
- revised plan は Nietzsche / Beauvoir とも `PLAN_APPROVED`。byte storage authority helper は original batch owner に頼れないため、continuation cursor の `batch_index - 1` から expected range を再計算すること、`row_tile_plan` では byte reader、raw memory、allocation、RLE、host/platform/fallback に進まないことが実装条件である。

変更:

- `stdlib/alloc/gui/render2d/row_byte_storage.nepl` に `GuiRgba8888RowByteStorageAuthorityErrorKind` と borrowed `gui_rgba8888_row_byte_storage_validate_authority` を追加する。
- authority helper は continuation cursor status、plan invariant、previous batch index、expected range metadata、stored copied range metadata の一致を再検証し、owner を消費しない。
- `stdlib/alloc/gui/render2d/row_tile_plan.nepl` を追加する。
- `GuiRgba8888RowTilePlanPrepareErrorKind`、`GuiRgba8888RowTilePlanInvariantErrorKind`、`GuiRgba8888RowTilePlanDescriptorErrorKind` を enum として分ける。
- `GuiRgba8888RowTilePlanOwner` と prepare error は owner-bearing なので `Clone` / `Copy` を実装しない。`GuiRgba8888RowTilePlan` と `GuiRgba8888RowTileDescriptor` は Copy metadata とする。
- prepare は byte storage authority を再検証し、`tile_rows > 0`、`byte_count == row_count * stride_bytes`、checked ceil `tile_count` を通して owner を作る。失敗時は byte storage owner を prepare error に保持する。
- `gui_rgba8888_row_tile_plan_validate_invariants` は storage authority、range metadata 一致、`stride_bytes == width * 4`、`row_start + row_count <= height`、`byte_count == row_count * stride_bytes`、`tile_count == ceil(row_count / tile_rows)` を再検証する。
- `gui_rgba8888_row_tile_plan_descriptor_at` は owner を借用し、invariant validation 後に storage-relative `byte_offset` と frame-absolute `row_start` を checked arithmetic で計算する。
- `stdlib/alloc/gui/render2d.nepl` facade から row tile plan を再公開する。
- `tests/stdlib/gui_render2d_row_tile_plan.n.md` を追加し、facade、positive config、storage authority、checked ceil、last partial tile、descriptor offsets、owner recovery、invariant revalidation、raw storage escape 禁止、platform / fallback 禁止を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5ca source policy を追加し、docs、facade export、owner no `Clone` / no `Copy`、borrowed authority、borrowed descriptor_at、checked ceil、no byte reader / raw memory / allocation / RLE / host / platform / fallback、括弧なし実装を検査する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row tile plan module doctest、row byte storage / row batch range regression、source policy、`git diff --check` が通る。
- implementation review で byte storage authority helper が borrowed metadata-only であること、descriptor offsets が storage-relative であること、owner recovery が壊れていないこと、no RLE / host present / fallback が守られていることを確認する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_plan.n.md --no-tree -o tmp_gui_render2d_row_tile_plan_f5ca.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_tile_plan.nepl --no-tree -o tmp_gui_render2d_row_tile_plan_module_f5ca.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_byte_storage.nepl --no-tree -o tmp_gui_render2d_row_byte_storage_f5ca_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_byte_storage.n.md --no-tree -o tmp_gui_render2d_row_byte_storage_test_f5ca_regression.json -j 1
git diff --check
```

## Phase F5cb: Render2d row tile payload view boundary

目的:

- F5ca の `GuiRgba8888RowTilePlanOwner` と tile index から `GuiRgba8888RowTilePayloadOwner` を作る。
- `GuiRgba8888RowTilePayloadOwner` は owned payload buffer ではなく、existing copied row storage 上の tile-scoped byte payload view とする。
- tile-relative byte read を typed boundary にし、descriptor offset から storage-relative index へ checked add する。
- この phase は no RLE / host present とし、追加 allocation、追加 copy、video memory、platform API、Canvas、DOM、minifb、fallback、silent no-op へ進まない。

plan review:

- Cicero plan review は `PLAN_APPROVED`。`gui_rgba8888_row_tile_plan_storage_ref` は raw `RegionToken` / `MemPtr` を返さず `&GuiRgba8888RowByteStorageOwner` の borrowed read-only authority に留めるなら許容された。
- `payload` という名前は owned payload buffer ではなく tile-scoped byte payload view / formal payload owner over existing copied row storage と docs に明記する条件で承認された。
- prepare failure は tile plan owner を owner-bearing error で返し、`byte_at` は tile-relative bounds と checked add の両方を通して lower storage read error を包むことを source policy と focused doctest で固定する。

変更:

- `stdlib/alloc/gui/render2d/row_tile_plan.nepl` に `gui_rgba8888_row_tile_plan_storage_ref` を追加する。
- `storage_ref` は raw storage、raw pointer、byte read を公開せず、`&GuiRgba8888RowByteStorageOwner` だけを返す。
- `stdlib/alloc/gui/render2d/row_tile_payload.nepl` を追加する。
- `GuiRgba8888RowTilePayloadPrepareErrorKind` と `GuiRgba8888RowTilePayloadReadErrorKind` を enum として分ける。
- `GuiRgba8888RowTilePayloadOwner` と prepare error は owner-bearing なので `Clone` / `Copy` を実装しない。
- prepare は `gui_rgba8888_row_tile_plan_descriptor_at &plan tile_index` を呼び、descriptor invalid なら `DescriptorInvalid lower_kind` と original plan owner を保持する。
- `gui_rgba8888_row_tile_payload_byte_at` は tile-relative index bounds、`descriptor.byte_offset + index` の checked add、`gui_rgba8888_row_byte_storage_byte_at`、lower error wrapping の順に進む。
- `stdlib/alloc/gui/render2d.nepl` facade から row tile payload を再公開する。
- `tests/stdlib/gui_render2d_row_tile_payload.n.md` を追加し、facade、descriptor authority、existing-storage view、tile-relative read、typed bounds error、owner recovery、raw storage escape 禁止、platform / fallback 禁止を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5cb source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row tile payload module doctest、row tile plan / row byte storage regression、source policy、`git diff --check` が通る。
- implementation review で storage_ref が typed borrowed authority だけを返すこと、payload が existing copied storage view であること、byte_at が tile-relative bounds / checked add / lower error wrap を守ること、no RLE / host present / fallback が守られていることを確認する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_payload.n.md --no-tree -o tmp_gui_render2d_row_tile_payload_f5cb.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_tile_payload.nepl --no-tree -o tmp_gui_render2d_row_tile_payload_module_f5cb.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_plan.n.md --no-tree -o tmp_gui_render2d_row_tile_plan_f5cb_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_byte_storage.nepl --no-tree -o tmp_gui_render2d_row_byte_storage_f5cb_regression.json -j 1
git diff --check
```

## Phase F5cc: Render2d row tile RLE cursor boundary

目的:

- F5cb の `GuiRgba8888RowTilePayloadOwner` を、tile 内 RGBA8888 pixel run を streaming で返す `GuiRgba8888RowTileRleCursorOwner` へ変換する。
- `GuiRgba8888RowTileRleRun` は `pixel_offset`、`pixel_count`、`Rgba8888 color` だけを持つ Copy metadata とする。
- cursor / step / error は owner-bearing value とし、payload owner を確実に回収できるようにする。
- この phase は encoded RLE buffer、`Vec`、raw storage、host present、video memory、platform API、Canvas、DOM、minifb、fallback、silent no-op へ進まない。

plan review:

- Cicero plan review 1 は `PLAN_BLOCKED`。`next_run` に complete cursor が渡された時に status / finish だけで扱える設計では public contract が弱く、owner-bearing typed error が必要だと指摘された。
- revised plan は `PLAN_APPROVED`。`CursorComplete` を `GuiRgba8888RowTileRleStepErrorKind` の明示 variant とし、`GuiRgba8888RowTileRleStepError` が cursor owner を保持すること、`pixel_index * 4` と channel offset `+1` / `+2` / `+3` をすべて checked にすること、payload read failure を `PayloadReadFailed lower_kind` に包むことが実装条件である。

変更:

- `stdlib/alloc/gui/render2d/row_tile_rle.nepl` を追加する。
- `GuiRgba8888RowTileRleStartErrorKind`、`GuiRgba8888RowTileRleStepErrorKind`、`GuiRgba8888RowTileRleCursorStatus`、`GuiRgba8888RowTileRleRun`、`GuiRgba8888RowTileRleCursorOwner`、`GuiRgba8888RowTileRleStep`、owner-bearing error を定義する。
- `gui_rgba8888_row_tile_rle_cursor_start` は payload byte count の正値と RGBA8888 alignment を検査する。
- `gui_rgba8888_row_tile_rle_cursor_next_run` は Ready cursor の同色 pixel run を走査し、Complete cursor では `CursorComplete` owner-bearing error を返す。
- `stdlib/alloc/gui/render2d.nepl` facade から row tile RLE cursor を再公開する。
- `tests/stdlib/gui_render2d_row_tile_rle.n.md` を追加し、facade、streaming cursor、pixel run sequence、complete error owner recovery、checked channel offsets、payload read error wrapping、no encoded buffer / no Vec、platform / fallback 禁止を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5cc source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row tile RLE module doctest、row tile payload / row tile plan regression、source policy、`git diff --check` が通る。
- implementation review で `CursorComplete` が owner-bearing typed error であること、offset math が全て checked であること、encoded buffer / `Vec` / raw storage / host present / platform / fallback に進んでいないことを確認する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_f5cc.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_tile_rle.nepl --no-tree -o tmp_gui_render2d_row_tile_rle_module_f5cc.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_payload.n.md --no-tree -o tmp_gui_render2d_row_tile_payload_f5cc_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_plan.n.md --no-tree -o tmp_gui_render2d_row_tile_plan_f5cc_regression.json -j 1
git diff --check
```

## Phase F5cd: Render2d row tile RLE drain boundary

目的:

- F5cc の `GuiRgba8888RowTileRleCursorOwner` を scheduler budget 内で bounded に進める。
- `GuiRgba8888RowTileRleDrainTerminal` は `status`、continuation cursor、`emitted_run_count` を持つ owner-bearing terminal とする。
- complete cursor は budget 判定より先に `Completed` へ分類し、Ready cursor の `remaining_steps < 0` だけを `InvalidBudget` にする。
- この phase は encoded RLE buffer、`Vec`、raw storage、host present、video memory、platform API、Canvas、DOM、minifb、fallback、silent no-op へ進まない。

plan review:

- Cicero plan review は `PLAN_APPROVED`。encoded transport へ直接進まず、scheduler semantics、run traversal、owner recovery、allocation / host concerns を分離するため、F5cc cursor の bounded drain を先に作る方針が承認された。
- 追加条件として、continuation cursor index だけでなく discard する Copy run metadata の `pixel_offset == previous_next_pixel_index` と `pixel_count > 0` も検査する。

変更:

- `stdlib/alloc/gui/render2d/row_tile_rle_drain.nepl` を追加する。
- `GuiRgba8888RowTileRleDrainErrorKind`、`GuiRgba8888RowTileRleDrainStatus`、`GuiRgba8888RowTileRleDrainTerminal`、`GuiRgba8888RowTileRleDrainError` を定義する。
- `gui_rgba8888_row_tile_rle_drain_budget` は status-before-budget で cursor を進め、`StepBudgetExhausted`、`Completed`、owner-bearing error を返す。
- `stdlib/alloc/gui/render2d.nepl` facade から row tile RLE drain を再公開する。
- `tests/stdlib/gui_render2d_row_tile_rle_drain.n.md` を追加し、facade、complete-before-budget、negative budget error、zero budget exhaustion、partial progress、completion count、run progress invariant、no encoded buffer / platform / fallback 禁止を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5cd source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row tile RLE drain module doctest、row tile RLE / payload / plan regression、source policy、`git diff --check` が通る。
- implementation review で status-before-budget、owner-bearing terminal / error、run metadata progress validation、encoded buffer / `Vec` / raw storage / host present / platform / fallback に進んでいないことを確認する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_drain.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_drain_f5cd.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_tile_rle_drain.nepl --no-tree -o tmp_gui_render2d_row_tile_rle_drain_module_f5cd.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_f5cd_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_payload.n.md --no-tree -o tmp_gui_render2d_row_tile_payload_f5cd_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_plan.n.md --no-tree -o tmp_gui_render2d_row_tile_plan_f5cd_regression.json -j 1
git diff --check
```

## Phase F5ce: Render2d row tile RLE count boundary

目的:

- F5cd の slice-local `emitted_run_count` を、formal encoded RLE transport の exact capacity 前段として累積する `GuiRgba8888RowTileRleCountOwner` を作る。
- `GuiRgba8888RowTileRleCountOwner` は `GuiRgba8888RowTileRleCursorOwner` と `accumulated_run_count` だけを保持する。
- `count_step_budget` は F5cd の `gui_rgba8888_row_tile_rle_drain_budget` に委譲し、RLE run を再走査しない。
- この phase は encoded RLE buffer、`Vec`、raw storage、host present、video memory、platform API、Canvas、DOM、minifb、fallback、silent no-op へ進まない。

plan review:

- subagent review で、encoded transport へ直接進む前に exact count owner boundary を置く方針を確認する。
- `count_start` は Ready cursor だけを許可し、Complete cursor は過去の run count evidence を持たないため `InitialCursorComplete` として拒否する。
- `AccumulatedRunCountOverflow` では cursor が既に進んでいる可能性があるため、fake continuation count owner を返さず、advanced cursor と prior `accumulated_run_count` を owner-bearing error に保持する。

実装:

- `stdlib/alloc/gui/render2d/row_tile_rle_count.nepl` を追加する。
- `GuiRgba8888RowTileRleCountErrorKind` に `InitialCursorInvalid %GuiRgba8888RowTileRleStepErrorKind`、`InitialCursorComplete`、`DrainFailed %GuiRgba8888RowTileRleDrainErrorKind`、`AccumulatedRunCountOverflow` を定義する。
- `GuiRgba8888RowTileRleCountStepStatus` は `Pending` / `Completed` の Copy enum とする。
- `GuiRgba8888RowTileRleCountOwner`、`GuiRgba8888RowTileRleCountStep`、`GuiRgba8888RowTileRleCountError` は owner-bearing value とし、Clone / Copy を実装しない。
- `stdlib/alloc/gui/render2d.nepl` facade から row tile RLE count を再公開する。
- `tests/stdlib/gui_render2d_row_tile_rle_count.n.md` を追加し、facade、zero budget pending、partial accumulation、completion total、negative budget lower error wrapping、initial complete rejection、overflow fatal no fake owner policy、no encoded buffer / platform / fallback 禁止を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5ce source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row tile RLE count module doctest、row tile RLE drain / cursor / payload / plan regression、source policy、`git diff --check` が通る。
- implementation review で count boundary が drain delegation のみであること、initial complete rejection、overflow fatal recovery、owner-bearing non-Copy values、encoded buffer / `Vec` / raw storage / host present / platform / fallback に進んでいないことを確認する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_count.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_count_f5ce.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_tile_rle_count.nepl --no-tree -o tmp_gui_render2d_row_tile_rle_count_module_f5ce.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_drain.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_drain_f5ce_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_f5ce_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_payload.n.md --no-tree -o tmp_gui_render2d_row_tile_payload_f5ce_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_plan.n.md --no-tree -o tmp_gui_render2d_row_tile_plan_f5ce_regression.json -j 1
git diff --check
```

## Phase F5cf: Render2d row tile RLE completed count boundary

目的:

- F5ce の `GuiRgba8888RowTileRleCountOwner` を、formal encoded RLE transport の exact capacity evidence として使える completed count owner へ昇格する。
- cursor status が `Complete` で、`accumulated_run_count` が正の場合だけ `GuiRgba8888RowTileRleCountCompletedOwner` を作る。
- pending count owner を encoded transport へ渡せないように、`CountNotCompleted` の owner-bearing error で明示的に拒否する。
- この phase は encoded RLE buffer、`Vec`、raw storage、host present、video memory、platform API、Canvas、DOM、minifb、fallback、silent no-op へ進まない。

plan review:

- subagent review で、encoded transport へ直接進む前に completed count evidence boundary を置く方針を確認する。
- completed module は count owner の private field を直接読まず、`row_tile_rle_count.nepl` が提供する borrowed helper を通す。
- `prepare` は status first とし、`CursorInvalid lower_kind`、`CountNotCompleted`、`TotalRunCountInvalid` を `Result` で返す。

実装:

- `stdlib/alloc/gui/render2d/row_tile_rle_count.nepl` に `gui_rgba8888_row_tile_rle_count_owner_cursor_status` を追加する。
- `stdlib/alloc/gui/render2d/row_tile_rle_count_completed.nepl` を追加する。
- `GuiRgba8888RowTileRleCountCompletedErrorKind` に `CursorInvalid %GuiRgba8888RowTileRleStepErrorKind`、`CountNotCompleted`、`TotalRunCountInvalid` を定義する。
- `GuiRgba8888RowTileRleCountCompletedOwner` と `GuiRgba8888RowTileRleCountCompletedError` は owner-bearing value とし、Clone / Copy を実装しない。
- `stdlib/alloc/gui/render2d.nepl` facade から row tile RLE completed count を再公開する。
- `tests/stdlib/gui_render2d_row_tile_rle_count_completed.n.md` を追加し、facade、completed success total、pending rejection、error owner recovery、no encoded buffer / platform / fallback 禁止を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5cf source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row tile RLE completed count module doctest、row tile RLE count / drain / cursor / payload / plan regression、source policy、`git diff --check` が通る。
- implementation review で completed evidence が status first validation であり、pending owner を拒否し、owner-bearing recovery を持ち、encoded buffer / `Vec` / raw storage / host present / platform / fallback に進んでいないことを確認する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_count_completed.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_count_completed_f5cf.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_tile_rle_count_completed.nepl --no-tree -o tmp_gui_render2d_row_tile_rle_count_completed_module_f5cf.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_count.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_count_f5cf_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_drain.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_drain_f5cf_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_f5cf_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_payload.n.md --no-tree -o tmp_gui_render2d_row_tile_payload_f5cf_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_plan.n.md --no-tree -o tmp_gui_render2d_row_tile_plan_f5cf_regression.json -j 1
git diff --check
```

## Phase F5cg: Render2d row tile RLE encode seed boundary

目的:

- F5cf の `GuiRgba8888RowTileRleCountCompletedOwner` を、formal encoded RLE transport 前の payload seed owner へ変換する。
- `GuiRgba8888RowTileRleEncodeSeedOwner` に `GuiRgba8888RowTilePayloadOwner` と exact `total_run_count` を保持させ、後続 encoded writer が capacity evidence を失わないようにする。
- internal misuse や将来の constructor 変更に備え、`total_run_count <= 0` は `TotalRunCountInvalid` の owner-bearing error として拒否する。
- この phase は cursor restart、RLE run drain、payload byte read、encoded RLE buffer、`Vec`、raw storage、host present、video memory、platform API、Canvas、DOM、minifb、fallback、silent no-op へ進まない。

plan review:

- subagent review で、F5cg は cursor restart まで進まず、completed evidence から payload seed へ所有権を移す境界に留める方針を確認する。
- `prepare` は total count を検査してから completed owner を消費し、成功時は `completed -> count -> cursor -> payload` の順に finish する。
- cursor restart の start error は payload owner を保持し、invalid total error は completed owner を保持するため、mixed owner error を避ける目的で restart は後続 phase に分離する。

実装:

- `stdlib/alloc/gui/render2d/row_tile_rle_encode_seed.nepl` を追加する。
- `GuiRgba8888RowTileRleEncodeSeedErrorKind` に `TotalRunCountInvalid` を定義する。
- `GuiRgba8888RowTileRleEncodeSeedOwner` と `GuiRgba8888RowTileRleEncodeSeedError` は owner-bearing value とし、Clone / Copy を実装しない。
- `stdlib/alloc/gui/render2d.nepl` facade から row tile RLE encode seed を再公開する。
- `tests/stdlib/gui_render2d_row_tile_rle_encode_seed.n.md` を追加し、facade、completed-to-seed success total、test-side payload restart、error owner recovery label、no cursor restart / encoded buffer / platform / fallback 禁止を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5cg source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row tile RLE encode seed module doctest、row tile RLE completed count / count / drain / cursor / payload / plan regression、source policy、`git diff --check` が通る。
- implementation review で encode seed が payload-seed-only boundary であり、`cursor_start`、drain、`cursor_next_run`、payload byte read、encoded buffer、`Vec`、raw storage、host present、platform、fallback に進んでいないことを確認する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_encode_seed.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_encode_seed_f5cg.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_tile_rle_encode_seed.nepl --no-tree -o tmp_gui_render2d_row_tile_rle_encode_seed_module_f5cg.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_count_completed.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_count_completed_f5cg_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_count.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_count_f5cg_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_drain.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_drain_f5cg_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_f5cg_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_payload.n.md --no-tree -o tmp_gui_render2d_row_tile_payload_f5cg_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_plan.n.md --no-tree -o tmp_gui_render2d_row_tile_plan_f5cg_regression.json -j 1
git diff --check
```

## Phase F5ch: Render2d row tile RLE encode cursor boundary

目的:

- F5cg の `GuiRgba8888RowTileRleEncodeSeedOwner` を、formal encoded RLE writer 前の ready cursor owner へ変換する。
- `GuiRgba8888RowTileRleEncodeCursorOwner` に `GuiRgba8888RowTileRleCursorOwner` と exact `total_run_count` を保持させる。
- cursor restart failure は lower start error を owner-bearing error として返し、seed owner を曖昧に再構成しない。

plan review:

- Ramanujan plan review は `PLAN_APPROVED`。F5ch は seed-to-ready-cursor ownership transfer だけを行い、encoded transport には進まない。
- `cursor_start` は positive aligned payload を `next_pixel_index = 0` と positive `pixel_count` の cursor にするため、`cursor_status` の追加検査は不要である。
- start failure recovery は lower `GuiRgba8888RowTileRleStartError` と `total_run_count` を保持する形でよく、seed owner を再構成しない。
- source policy では `cursor_start` がちょうど 1 回であること、`cursor_status`、`cursor_next_run`、drain、payload read、raw storage、`Vec`、encoded buffer、host / platform、fallback、silent no-op、括弧を禁止する。

実装:

- `stdlib/alloc/gui/render2d/row_tile_rle_encode_cursor.nepl` を追加する。
- `GuiRgba8888RowTileRleEncodeCursorErrorKind` に `CursorStartFailed %GuiRgba8888RowTileRleStartErrorKind` を定義する。
- `GuiRgba8888RowTileRleEncodeCursorOwner` と `GuiRgba8888RowTileRleEncodeCursorError` は owner-bearing value とし、Clone / Copy を実装しない。
- `gui_rgba8888_row_tile_rle_encode_cursor_start` は seed の total count を読んでから payload を finish し、`gui_rgba8888_row_tile_rle_cursor_start` に 1 回だけ委譲する。
- `stdlib/alloc/gui/render2d.nepl` facade から row tile RLE encode cursor を再公開する。
- `tests/stdlib/gui_render2d_row_tile_rle_encode_cursor.n.md` を追加し、facade、seed-to-ready-cursor success、total count preservation、start error owner recovery label、no status / drain / encoded buffer / platform / fallback 禁止を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5ch source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row tile RLE encode cursor module doctest、row tile RLE encode seed / completed count / count / drain / cursor / payload / plan regression、source policy、`git diff --check` が通る。
- implementation review で encode cursor が ready cursor boundary であり、`cursor_status`、drain、`cursor_next_run`、payload byte read、encoded buffer、`Vec`、raw storage、host present、platform、fallback に進んでいないことを確認する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_encode_cursor.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_encode_cursor_f5ch.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_tile_rle_encode_cursor.nepl --no-tree -o tmp_gui_render2d_row_tile_rle_encode_cursor_module_f5ch.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_encode_seed.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_encode_seed_f5ch_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_count_completed.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_count_completed_f5ch_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_count.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_count_f5ch_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_drain.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_drain_f5ch_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_f5ch_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_payload.n.md --no-tree -o tmp_gui_render2d_row_tile_payload_f5ch_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_plan.n.md --no-tree -o tmp_gui_render2d_row_tile_plan_f5ch_regression.json -j 1
git diff --check
```

## Phase F5ci: Render2d row tile RLE writer plan boundary

目的:

- F5ch の `GuiRgba8888RowTileRleEncodeCursorOwner` を、formal encoded RLE writer の capacity plan owner へ変換する。
- fixed RLE run wire layout を `pixel_offset i32`、`pixel_count i32`、`Rgba8888` 4 bytes の 12 bytes として document し、`total_run_count * 12` を checked arithmetic で検査する。
- invalid count / overflow は original ready cursor owner を保持する owner-bearing error とし、cursor owner を fake reconstruction しない。

plan review:

- Ramanujan plan review は `PLAN_APPROVED`。F5ci は ready cursor owner と future encoded writer/storage の間に置く capacity-only boundary として妥当である。
- `total_run_count > 0` の再検査は、formal capacity boundary として forged / internal misuse を fail-closed にするため許容される。
- invalid count / overflow の error は original `GuiRgba8888RowTileRleEncodeCursorOwner` を保持し、`finish_cursor` は success path だけで行う。
- source policy では checked multiply by `12`、owner/error no Clone/Copy、no status / drain / next_run / payload read / raw storage / `Vec` / encoded buffer / platform / fallback / silent no-op / 括弧禁止を固定する。

実装:

- `stdlib/alloc/gui/render2d/row_tile_rle_writer_plan.nepl` を追加する。
- `GuiRgba8888RowTileRleWriterPlanErrorKind` に `TotalRunCountInvalid` と `EncodedByteCountOverflow` を定義する。
- `GuiRgba8888RowTileRleWriterPlanOwner` は `cursor`、`total_run_count`、`encoded_byte_count` を持つ。
- `GuiRgba8888RowTileRleWriterPlanError` は `ready` owner と `total_run_count` を保持する。
- `gui_rgba8888_row_tile_rle_writer_plan_prepare` は total を読んで正値検査を行い、`total_run_count * 12` を checked multiply で検査してから success path だけで cursor owner を finish する。
- `stdlib/alloc/gui/render2d.nepl` facade から row tile RLE writer plan を再公開する。
- `tests/stdlib/gui_render2d_row_tile_rle_writer_plan.n.md` を追加し、facade、ready-to-capacity success、encoded byte count、owner recovery label、no status / drain / payload read / encoded buffer / platform / fallback 禁止を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5ci source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row tile RLE writer plan module doctest、row tile RLE encode cursor / seed / completed count / count / drain / cursor / payload / plan regression、source policy、`git diff --check` が通る。
- implementation review で capacity-only boundary であり、allocation、RLE data write、payload byte read、host present、platform API、fallback に進んでいないことを確認する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_writer_plan.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_writer_plan_f5ci.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_tile_rle_writer_plan.nepl --no-tree -o tmp_gui_render2d_row_tile_rle_writer_plan_module_f5ci.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_encode_cursor.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_encode_cursor_f5ci_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_encode_seed.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_encode_seed_f5ci_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_count_completed.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_count_completed_f5ci_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_count.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_count_f5ci_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_drain.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_drain_f5ci_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_f5ci_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_payload.n.md --no-tree -o tmp_gui_render2d_row_tile_payload_f5ci_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_plan.n.md --no-tree -o tmp_gui_render2d_row_tile_plan_f5ci_regression.json -j 1
git diff --check
```

## Phase F5cj: Render2d row tile RLE encoded storage boundary

目的:

- F5ci の `GuiRgba8888RowTileRleWriterPlanOwner` を、future encoded writer の exact byte storage owner へ変換する。
- writer plan の stored byte count と total run count を再検査し、`total_run_count * 12` の checked recompute と stored byte count の一致を確認してから exact allocation へ進む。
- allocation failure を含む全 prepare failure で original writer plan owner を保持し、fallback や fake owner reconstruction を行わない。

plan review:

- Descartes plan review は `PLAN_APPROVED`。F5cj は F5ci writer plan と future run writer の間に置く allocation / reservation only boundary として妥当である。
- prepare failure path は allocation failure だけでなく metadata mismatch / invalid count / overflow も original `GuiRgba8888RowTileRleWriterPlanOwner` を保持する必要がある。
- 検査順は encoded byte count 正値、total run count 正値、checked multiply、stored byte count との一致、allocation の順にする。
- source policy と docs では、F5cj が `cursor_next_run`、drain、payload read、byte write、`Vec`、host present、platform API、fallback に進まないことを固定する。

実装:

- `stdlib/alloc/gui/render2d/row_tile_rle_storage.nepl` を追加する。
- `GuiRgba8888RowTileRleStoragePrepareErrorKind` に `EncodedByteCountInvalid`、`TotalRunCountInvalid`、`EncodedByteCountOverflow`、`EncodedByteCountMismatch`、`AllocationFailed` を定義する。
- `GuiRgba8888RowTileRleStorageOwner` は `cursor`、`total_run_count`、`encoded_byte_count`、`RegionToken u8 storage` を持つ。
- `GuiRgba8888RowTileRleStoragePrepareError` は original writer plan owner を保持する。
- `gui_rgba8888_row_tile_rle_storage_prepare` は capacity evidence を再検査し、allocation 成功後にだけ `gui_rgba8888_row_tile_rle_writer_plan_owner_finish_cursor` を呼ぶ。
- `gui_rgba8888_row_tile_rle_storage_finish_cursor` は storage を dealloc してから continuation cursor を返す。
- `stdlib/alloc/gui/render2d.nepl` facade から row tile RLE encoded storage を再公開する。
- `tests/stdlib/gui_render2d_row_tile_rle_storage.n.md` を追加し、facade、writer-plan-to-storage success、exact byte count、owner recovery label、allocation-only / no platform / no fallback を固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5cj source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- focused doctest、row tile RLE storage module doctest、writer plan / encode cursor / seed / completed count / count / drain / cursor / payload regression、source policy、`git diff --check` が通る。
- implementation review で allocation / reservation only boundary であり、RLE data write、payload byte read、host present、platform API、fallback に進んでいないことを確認する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_storage.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_storage_f5cj.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/render2d/row_tile_rle_storage.nepl --no-tree -o tmp_gui_render2d_row_tile_rle_storage_module_f5cj.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_writer_plan.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_writer_plan_f5cj_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_encode_cursor.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_encode_cursor_f5cj_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_encode_seed.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_encode_seed_f5cj_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_count_completed.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_count_completed_f5cj_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_count.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_count_f5cj_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_drain.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_drain_f5cj_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_f5cj_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_payload.n.md --no-tree -o tmp_gui_render2d_row_tile_payload_f5cj_regression.json -j 1
git diff --check
```

## Phase F5ck: render2d row tile RLE run writer cursor

目的:

- F5cj の encoded storage owner を exact run writer cursor へ変換する。
- storage への 12 byte record write が全て成功した後でだけ lower RLE cursor を進める。
- store / projection / advance failure では owner-bearing error により original cursor、storage、unchanged written counts を回収できるようにする。
- encoded byte reader、payload byte reader、host present、tile transport ABI、platform API、fallback には進まない。

plan review:

- Descartes plan review 1 は `PLAN_BLOCKED`。当初案では consuming `cursor_next_run` を write 前に呼ぶため、store failure 時に pre-step cursor を正しく返せないと指摘された。
- revised plan では `row_tile_rle` に borrowed `cursor_peek_run` と consuming `cursor_advance_by_run` を追加し、writer は peek、write、advance の順にする。
- Descartes revised plan review は `PLAN_APPROVED`。write-success-before-advance、unchanged written counts、uncommitted slot、reader 禁止、little-endian layout、completion 明示を source policy と docs に固定する条件で承認された。

実装:

- `stdlib/alloc/gui/render2d/row_tile_rle.nepl` に `gui_rgba8888_row_tile_rle_cursor_peek_run` を追加する。
- `stdlib/alloc/gui/render2d/row_tile_rle.nepl` に `gui_rgba8888_row_tile_rle_cursor_advance_by_run` を追加する。
- `advance_by_run` 用に `RunPixelOffsetMismatch`、`RunPixelCountInvalid`、`RunEndOutOfBounds` を lower step error kind に追加する。
- `stdlib/alloc/gui/render2d/row_tile_rle_storage.nepl` に `GuiRgba8888RowTileRleWriteCursorOwner`、start error、step status、step error を追加する。
- `gui_rgba8888_row_tile_rle_write_cursor_start` は encoded byte count / total run count / `total_run_count * 12` を再検査し、成功時だけ storage owner を writer owner へ移す。
- `gui_rgba8888_row_tile_rle_write_cursor_step_one` は stored counts / written counts、completion、`written_byte_count + 12`、`written_run_count + 1` を検査し、`peek_run`、12 byte write、`advance_by_run` の順に進む。
- record layout は `pixel_offset i32 LE`、`pixel_count i32 LE`、`Rgba8888 r,g,b,a` とする。
- `region_ptr_at` と `store_u8` は byte projection / byte store helper に閉じ込める。
- `tests/stdlib/gui_render2d_row_tile_rle_storage.n.md` は timeout を避けるため import smoke と source policy labels に絞る。writer の詳細契約は source policy で固定する。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5ck source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- import smoke、row tile RLE storage module doctest、source policy、`git diff --check` が通る。
- implementation review で consuming `cursor_next_run` を使わず、write helper だけが raw byte storage に触れ、reader / payload read / host present / platform / fallback へ進んでいないことを確認する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_storage.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_storage_f5ck.json -j 1
git diff --check
```

## Phase F5cl: render2d row tile RLE sealed encoded owner

目的:

- F5ck の `GuiRgba8888RowTileRleWriteCursorOwner` を、formal tile / bitmap transport 前の sealed encoded owner へ昇格する。
- partial writer cursor や lower cursor `Ready` を host-visible payload として扱わない。
- encoded byte reader、raw storage accessor、host present、video memory import、platform API、fallback には進まない。

plan review:

- Descartes plan review は `PLAN_APPROVED`。F5cl は F5ck の後、tile transport / host present ABI の前に置く正しい boundary と確認された。
- required: written count は `written_run_count >= 0`、`written_run_count <= total_run_count`、`written_byte_count >= 0`、`written_byte_count <= encoded_byte_count`、checked `written_run_count * 12 == written_byte_count` の順に検査する。
- required: otherwise valid な count が total / encoded completion に届かない場合は `WriterNotComplete`、lower cursor `Ready` は `CursorNotComplete` とする。
- required: lower cursor status は count invariant が通った後でだけ検査する。

実装:

- `stdlib/alloc/gui/render2d/row_tile_rle_encoded.nepl` を追加する。
- `GuiRgba8888RowTileRleEncodedOwner`、`GuiRgba8888RowTileRleEncodedSealErrorKind`、`GuiRgba8888RowTileRleEncodedSealError`、finish error を追加する。
- `gui_rgba8888_row_tile_rle_encoded_seal` は count invariants と cursor completion を検査し、成功時だけ cursor / storage を sealed owner へ move する。
- failure path は original `GuiRgba8888RowTileRleWriteCursorOwner` を owner-bearing error に保持する。
- metadata accessor は total run count、encoded byte count、cursor next pixel index、cursor pixel count に限定する。
- `finish_cursor` / `owner_free` は storage dealloc と lower cursor free だけを行い、byte reader を追加しない。
- `stdlib/alloc/gui/render2d.nepl` facade から row tile RLE sealed encoded owner を再公開する。
- `tests/stdlib/gui_render2d_row_tile_rle_encoded.n.md` は import smoke と source policy labels に絞る。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5cl source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- import smoke、source policy、`git diff --check` が通る。
- implementation review で count invariant、owner-bearing error、no byte reader、no host present、no fallback を確認する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_encoded.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_encoded_f5cl.json -j 1
git diff --check
```

## Phase F5cm: render2d row tile RLE packet owner

目的:

- F5cl の `GuiRgba8888RowTileRleEncodedOwner` を、formal tile / bitmap transport の直前で使う packet owner へ昇格する。
- packet descriptor に frame / plan row / tile geometry と encoded RLE metadata を閉じ込め、host ABI 側が private cursor layout を再解釈しないようにする。
- byte reader、raw storage accessor、host present、video memory import、platform API、fallback には進まない。

plan review:

- Descartes plan review は `PLAN_APPROVED`。条件は、payload descriptor を plan から再計算して authority を検証すること、validation order を encoded counts、cursor completion、descriptor authority、pixel byte count、plan shape、tile metadata の順に source policy で固定すること、すべての descriptor arithmetic を checked にすること。
- required: prepare failure は original sealed encoded owner を owner-bearing error に保持し、validation success 後だけ packet owner に move する。

実装:

- `stdlib/alloc/gui/render2d/row_tile_payload.nepl` に metadata-only descriptor authority helper を追加する。
- `stdlib/alloc/gui/render2d/row_tile_rle.nepl` と `row_tile_rle_encoded.nepl` に checked descriptor / plan metadata helper を追加する。
- `stdlib/alloc/gui/render2d/row_tile_rle_packet.nepl` を追加し、`GuiRgba8888RowTileRlePacketDescriptor`、`GuiRgba8888RowTileRlePacketOwner`、`GuiRgba8888RowTileRlePacketPrepareErrorKind`、owner-bearing prepare error を定義する。
- F5cn の std present validation で tile count を再導出するため、packet descriptor は tile 自身の row range に加えて `plan_row_start` / `plan_row_count` を持つ。
- descriptor authority failure は `PayloadDescriptorInvalid` として lower authority error を包む。
- `gui_rgba8888_row_tile_rle_packet_prepare` は encoded count、cursor completion、payload descriptor authority、descriptor byte count、plan shape、tile metadata を検査し、成功時だけ sealed owner を packet owner に move する。
- `stdlib/alloc/gui/render2d.nepl` facade から packet owner を再公開する。
- `tests/stdlib/gui_render2d_row_tile_rle_packet.n.md` は import smoke と source policy labels に絞る。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5cm source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- import smoke、source policy、`git diff --check` が通る。
- implementation review で descriptor authority、checked arithmetic、owner recovery、no byte reader、no host present、no fallback を確認する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_packet.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_packet_f5cm.json -j 1
git diff --check
```

## Phase F5cn: std row tile RLE present-frame owner

目的:

- F5cm の `GuiRgba8888RowTileRlePacketOwner` を、host import の直前で使う std layer row tile RLE present-frame owner へ昇格する。
- `SurfaceId` / `FrameId` と packet descriptor の対応を std layer で検査し、Web、native、headless、bare presenter が同じ owner を消費できるようにする。
- `GuiSurfacePresentCommand`、`PresentPixelFrame`、`GuiPixelBufferDescriptor` には接続せず、host import、platform API、byte reader、video memory、fallback には進まない。

plan review:

- Descartes plan review は `PLAN_APPROVED`。条件は、既存 `GuiSurfacePresentCommand` を拡張しないこと、platform/host module を import しないこと、source policy で `GuiSurfacePresentCommand` / `PresentPixelFrame` / `GuiPixelBufferDescriptor` / host import / video memory / raw bytes / fallback を禁止すること。
- validation は `surface_id_raw` / `frame_id_raw`、packet frame id mismatch、positive geometry、`row_count * width == pixel_count`、`total_run_count * 12 == encoded_byte_count`、`width * 4 == stride_bytes`、positive row/tile/run/byte counts、derived tile count before owner move を固定する。
- packet descriptor は F5cm 時点で plan row range を持っていなかったため、F5cn では `plan_row_start` / `plan_row_count` を descriptor へ追加し、std layer で `tile_count` を再導出できるようにする。

実装:

- `stdlib/std/gui/tile_present.nepl` を追加し、`GuiRgba8888RowTileRlePresentDescriptor`、`GuiRgba8888RowTileRlePresentFrameOwner`、`GuiRgba8888RowTileRlePresentFramePrepareErrorKind`、owner-bearing prepare error を定義する。
- `gui_rgba8888_row_tile_rle_present_frame_prepare` は packet descriptor を借用で読み、すべての validation が成功した後だけ packet owner を present-frame owner に move する。
- failure は `GuiRgba8888RowTileRlePresentFramePrepareError` に original packet owner を保持し、free / recovery helper で閉じられるようにする。
- `stdlib/std/gui.nepl` facade から tile present boundary を再公開する。
- `tests/stdlib/gui_std_tile_present.n.md` は import smoke と source policy labels に絞る。
- `nodesrc/test_web_gui_font_rendering_contract.js` に F5cn source policy を追加する。
- `doc/neplg2/gui_font_rendering_spec.md`、`doc/neplg2/gui_font_rendering_detailed_design.md`、`doc/neplg2/gui_standard_library_spec.md`、`note.n.md`、`todo.md` を更新する。

完了条件:

- import smoke、source policy、`git diff --check` が通る。
- implementation review で existing surface command へ接続していないこと、host import / byte reader / platform API / fallback に進んでいないこと、owner recovery と checked arithmetic が保たれていることを確認する。

検証:

```text
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present.n.md --no-tree -o tmp_gui_std_tile_present_f5cn.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_packet.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_packet_f5cn_regression.json -j 1
git diff --check
```

## Phase F5be: sfnt simple glyph raster coverage scan converter

目的:

- F5bd の coverage mask writer owner を authority とし、line / quadratic typed raster edge から cell coverage を計算して `push_cell` boundary へ接続する。
- scan conversion を再開可能な owner と budgeted drain として実装し、GUI scheduler / headless test / native backend が同じ処理を time slice で進められるようにする。
- quadratic は `quadratic_segment_count` を持つ明示 config による deterministic flattening として扱い、0 coverage fallback や platform fallback を行わない。
- F5be は coverage scan converter で止め、packed mask conversion、render2d command、platform API、font fallback へ進まない。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。
- F5bd coverage shape の再検証が不足し、malformed shape が `%`、`/`、sample loop、coverage range へ到達しうると指摘された。
- `cell_index > shape.cell_count` が typed error ではなく coordinate derivation へ進みうるため、drain/step の cell index bounds を completion/budget より前に固定する必要があると指摘された。
- revised plan では shape invariant validation と cell-index bounds error を追加する。
- Tesla revised plan review は `PLAN_APPROVED`。shape revalidation before math/push、cell-index bounds before completion/budget/scan、explicit quadratic segment config、typed `StepBudgetExhausted` terminal は実装開始条件を満たす。

変更:

- `GuiSfntSimpleGlyphRasterCoverageScanConfig` を追加する。value-only record とし、`Clone` / `Copy` を実装する。
- `GuiSfntSimpleGlyphRasterCoverageScanOwner` を module-private owner として追加する。F5bd writer owner、scan config、cell_index を保持し、`Clone` / `Copy` は実装しない。
- `RasterCoverageScanStartErrorKind` / `RasterCoverageScanStartError` を追加する。start error は original F5bd writer owner を必ず保持し、recovery helper で回収できる。
- start は `quadratic_segment_count > 0`、coverage shape invariant、writer written/cell Vec state、edge owner count、typed edge Vec len/cap を再検査する。
- coverage shape invariant は `width_px > 0`、`height_px > 0`、`sample_scale > 0`、`coverage_max == sample_scale * sample_scale`、`cell_count == width_px * height_px` を検査し、通過前に cell coordinate math、sample loop、edge scan、`push_cell` へ進まない。
- edge read helper を追加する。negative index、out of range、edge Vec len/cap mismatch、`vec::get None` を typed error にする。
- i64 scaled coordinate helper を追加し、sample coordinate と edge coordinate を同じ `x2 * sample_scale` 空間へ変換する。
- line crossing helper を追加する。even-odd rule、strict y-range active check、i64 cross product、dy sign に応じた比較を固定する。
- quadratic crossing helper を追加する。`quadratic_segment_count` による deterministic segment endpoints を作り、line crossing helper だけへ渡す。
- sample coverage helper を追加し、sample point ごとの crossing count parity を coverage increment に変換する。
- cell coverage helper を追加し、`sample_scale * sample_scale` sample を走査して `0 <= coverage <= coverage_max` を計算する。
- step helper を追加し、1 cell coverage を計算して F5bd `push_cell` へ渡し、push failure では recovered writer を保持した scan owner を返す。
- bounded drain terminal を追加する。`Completed` と `StepBudgetExhausted` を success enum で分け、completion は F5bd writer completion の exact completed branch だけを成功とする。
- drain / step は completion と budget より前に `0 <= cell_index <= shape.cell_count` を検査し、`CellIndexNegative` / `CellIndexExceedsCellCount` は owner-bearing typed error として返す。
- step 成功後に `cell_index + 1` と `written_cell_count + 1` の hard progress guard を入れる。
- scan owner / terminal / error の free helper を追加し、writer owner を exactly once close できるようにする。

完了条件:

- source policy が docs、plan review approval、scan config `Clone` / `Copy`、private scan owner、owner no `Clone` / `Copy`、start error writer recovery、scan error owner recovery、start validation order、coverage shape revalidation before math/push、cell-index bounds before completion/budget/scan、edge storage revalidation、edge read typed error、scaled sample coordinate、line crossing i64 cross product、quadratic explicit segment config、cell coverage sampling、push_cell integration、bounded drain terminal、hard progress guard、free functions、forbidden byte-backed / old traversal / zero-fill / packed mask / render / platform / fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_coverage_scan_converter.n.md` に start validation、line crossing、quadratic segment policy、cell coverage sampling、push integration、budgeted terminal、no fallback/no render policy の coverage label を追加する。
- implementation review で writer owner が start/step/completion failure から必ず回収可能であること、`StepBudgetExhausted` が success terminal であり zero-fill completion ではないこと、quadratic が明示 config で処理されていることを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の packed / render2d mask boundary phase へ進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_coverage_scan_converter.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_raster_coverage_scan_converter_f5be.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_coverage_mask_writer.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_raster_coverage_mask_writer_f5be_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5be.json -j 1
git diff --check
```

## Phase F5bd: sfnt simple glyph raster coverage mask writer owner

目的:

- F5bc の completed raster edge owner を authority とし、scan conversion が後続 phase で埋める coverage cell buffer の owner boundary を追加する。
- coverage 寸法は edge から暗黙推測せず、`RasterCoverageConfig` を検査して `RasterCoverageShape` に固定する。
- `coverage_max = sample_scale * sample_scale`、`cell_count = width_px * height_px` を overflow guard 付きで計算する。
- F5bd は coverage buffer writer で止め、edge scan conversion、coverage computation、packed mask conversion、2D render command、platform API、font fallback へ進まない。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。
- completed edge owner の count だけでは typed edge Vec の len/cap 不整合を検出できず、新しい allocation boundary として弱いと指摘された。
- revised plan では `EdgeStorageLenMismatch` / `EdgeStorageCapacityMismatch` を追加し、coverage cell Vec allocation 前に typed edge Vec len/cap を再検査する。
- Tesla revised plan review は `PLAN_APPROVED`。

変更:

- `GuiSfntSimpleGlyphRasterCoverageConfig` と `GuiSfntSimpleGlyphRasterCoverageShape` を追加する。どちらも scalar-only value record とし、`Clone` / `Copy` を実装する。
- coverage shape derive helper を追加し、width、height、sample scale、max cell count、coverage max overflow、cell count overflow、cell count limit を typed error として返す。
- `GuiSfntSimpleGlyphRasterCoverageMaskWriterOwner` を module-private owner として追加する。completed F5bc edge owner、coverage shape、coverage cell Vec、written_cell_count を保持する。
- `RasterCoverageStartErrorKind` / `RasterCoverageStartError` を追加する。start error は original edge owner を必ず保持し、`raster_coverage_start_error_edge_owner` で回収できる。
- start は config shape derivation、F5bc edge owner revalidation、coverage cell Vec allocation の順に検査する。
- edge owner revalidation は edge_count、line_edge_count、quadratic_edge_count、capacity.raster_edge_capacity、plan.line_to_count、plan.quadratic_to_count、typed edge Vec len、typed edge Vec cap を照合する。
- `RasterCoveragePushErrorKind` / `RasterCoveragePushError` を追加する。push error は unchanged writer owner と rejected coverage value を保持し、lower `StdErrorKind` は `Option` として分離する。
- `raster_coverage_mask_writer_owner_push_cell` を追加し、Vec len/cap、full check、coverage value range、`vec::push` recovery order、written count advance を固定する。
- completed owner、completion terminal、free 関数を追加し、full mask 以外を completed としない。

完了条件:

- source policy が docs、plan review approval、config/shape value type `Clone` / `Copy`、private writer/completed owner、owner no `Clone` / `Copy`、shape derivation validation order、edge owner revalidation、typed edge Vec len/cap revalidation、start error owner recovery、push error owner recovery、push validation order、Vec push recovery order、exact completion、free functions、forbidden byte-backed / old traversal / scan conversion / coverage computation / render / platform / fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_coverage_mask_writer.n.md` に config shape、start validation、owner recovery、push validation、completion/free、no fallback/no scan/no render policy の coverage label を追加する。
- implementation review で F5bc edge owner が start/push failure から必ず回収可能であること、coverage buffer が partial completion や zero-fill fallback を持たないことを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の edge scan conversion / coverage computation phase へ進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_coverage_mask_writer.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_raster_coverage_mask_writer_f5bd.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_edge_owner.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_raster_edge_owner_f5bd_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5bd.json -j 1
git diff --check
```

## Phase F5bc: sfnt simple glyph outline point stream item collection raster edge owner

目的:

- F5bb の completed `RasterMaskWriterOwner` を authority とし、raster mask scalar Vec を typed raster edge Vec へ変換する。
- `RasterMaskWriterOwner` は private transition owner なので、F5bc も private boundary に留め、public constructor や forged start point を作らない。
- scalar stream は tag 2 line record と tag 3 quadratic record だけを受け付け、tag 1 / tag 4 や未知 tag は typed error として返す。
- F5bc は typed edge owner で止め、scan conversion、pixel coverage、2D render command、platform API、font fallback へ進まない。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。
- typed edge Vec の storage representation、allocation cleanup、drain owner / completed owner の分離、scalar read contract、F5bb owner が private であることへの配慮が不足していると指摘された。
- Tesla revised plan review 2 は `PLAN_BLOCKED`。
- start error の F5bb writer recovery、budgeted drain の hard progress guard、`vec::get None` の `ScalarSlotMissing` 化、F5bb complete check の固定が不足していると指摘された。
- Tesla revised plan 3 review は `PLAN_APPROVED`。

変更:

- `GuiSfntSimpleGlyphRasterLineEdge`、`GuiSfntSimpleGlyphRasterQuadraticEdge`、`GuiSfntSimpleGlyphRasterEdge` を追加する。これらは scalar だけを持つ value-only edge record / enum とし、`Clone` / `Copy` を実装する。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionRasterEdgeDrainOwner` を private owner として追加する。F5bb writer owner、typed edge Vec、scalar_index、edge_count、line_edge_count、quadratic_edge_count を保持する。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionRasterEdgeOwner` を private completed owner として追加する。scalar stream と edge count が完全一致した場合だけ生成する。
- `RasterEdgeStartErrorKind` / `RasterEdgeStartError` を追加する。start error は original F5bb writer を必ず保持し、`raster_edge_start_error_writer` で回収できる。
- start は F5az capacity derivation、stored capacity、path sink cap、raster mask cap、path sink len、raster mask len、F5ba inner completed progress、F5bb outer completed progress、expected edge capacity、typed edge Vec allocation の順に検査する。
- `RasterEdgeScalarReadErrorKind` を追加する。private scalar read helper は F5bb writer を消費せず、negative index、out of range、storage length/capacity mismatch、`vec::get None` を `ScalarSlotMissing` として返す。
- `RasterEdgeDrainErrorKind` / `RasterEdgeDrainError` を追加する。drain error は exactly one drain owner を保持し、`raster_edge_drain_error_owner` で回収できる。
- budgeted drain は complete terminal を先に検査し、非 terminal かつ budget 0 以下では `StepBudgetExhausted` として owner を返す。step 成功後は line なら scalar_index +5 / edge_count +1 / line_edge_count +1、quadratic なら scalar_index +7 / edge_count +1 / quadratic_edge_count +1 を hard progress guard で検査する。
- scalar record format は line: `tag, start_x2, start_y2, end_x2, end_y2`、quadratic: `tag, start_x2, start_y2, control_x2, control_y2, end_x2, end_y2` とする。
- `vec::push` failure では `vec_push_error_kind &e` を先に読み、`vec_push_error_vec e` で Vec を回収し、unchanged drain owner と lower storage error を返す。
- `raster_edge_drain_owner_free` と `raster_edge_owner_free` を追加し、typed edge Vec と F5bb writer owner を exactly once free する。

完了条件:

- source policy が docs、value-only edge type `Clone` / `Copy`、private drain/completed owner、owner no `Clone` / `Copy`、start error writer recovery、drain error owner recovery、start validation order、non-consuming scalar read、`ScalarSlotMissing`、tag 2 / tag 3 record parsing、tag 1 / tag 4 typed rejection、truncated record error、budgeted terminal、hard progress guard、push failure recovery order、free functions、forbidden byte-backed / old traversal / scan conversion / render / platform / fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_edge_owner.n.md` に edge owner types、start validation order、error recovery、scalar read contract、record parsing、budget/progress guard、push failure recovery、free contract、no fallback/no byte-backed/no traversal/no render policy の coverage label を追加する。
- implementation review で plan review 指摘がすべて反映されていること、F5bb writer が error から必ず回収可能であること、drain owner と completed owner が混同されていないことを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の raster edge scan conversion / mask coverage boundary phase へ進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_edge_owner.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_raster_edge_owner_f5bc.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_mask_writer.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_raster_mask_writer_f5bc_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5bc.json -j 1
git diff --check
```

## Phase F5bb: sfnt simple glyph outline point stream item collection raster mask writer

目的:

- F5ba の completed `PathCommandStreamSinkWriterOwner` を authority とし、`LineTo` / `QuadraticTo` だけを raster mask scalar Vec へ書く内部 writer を追加する。
- current point は raster edge start point の authority になるため、F5bb owner は module-private transition-only owner とし、public constructor、`Clone`、`Copy` を持たせない。
- `MoveTo` と `SkipNoSegment` はどちらも scalar を出さないが、別々の progress count として保持し、forged progress が kind count を隠せないようにする。
- F5bb は scalar stream writer で止め、path object materialization、rasterization、render2d、platform API、font fallback へ進まない。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。
- `skip_without_mask_count` では `MoveTo` と `SkipNoSegment` の forged progress を隠せるため、kind 別 count を分ける必要があると指摘された。
- current point を public owner state として信頼すると forged mid-state から任意 start point を作れるため、private transition-only owner contract または再導出境界が必要だと指摘された。
- stable tag scalar は F5au helper を通すことを source policy に固定する必要があると指摘された。
- revised plan では private owner、private constructor、start / advance / recovery だけの生成、kind 別 progress count、F5au tag scalar helper を明示した。
- Tesla revised plan review は `PLAN_APPROVED`。

変更:

- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionRasterMaskWriterOwner` を module-private struct として追加する。inner F5ba writer owner、written_count、raster_mask_scalar_count、move_to_count、line_to_count、quadratic_to_count、skip_no_segment_count、last_path_command_index、has_current_point、current_x2、current_y2 を保持する。
- F5bb owner constructor は private function に限定し、public constructor を作らない。owner / step / error は Vec owner を含むため `Clone` / `Copy` は実装しない。
- `RasterMaskWriterStartErrorKind` / `RasterMaskWriterStartError` を追加する。start は F5az plan/capacity、path sink Vec cap/len、raster mask Vec cap/len、inner F5ba completed progress を検査する。
- `RasterMaskWriterPushErrorKind` / `RasterMaskWriterPushError` を追加する。push error は current or reconstructed F5bb owner、rejected `PathCommandValue`、capacity error option、rejected scalar option、storage error option を保持する。
- `raster_mask_writer_owner_validate_for_push` を追加する。F5az plan/capacity 再検査、path sink complete state、raster mask len/count、inner F5ba complete progress、F5bb kind progress、aggregate progress、path command index、stored/source/command tag consistency を検査する。
- `raster_mask_writer_owner_push_scalar` を追加する。`vec::push` failure では `vec_push_error_kind &e` を先に読み、`vec_push_error_vec e` で Vec を回収し、F5az owner、F5ba writer owner、F5bb writer owner を unchanged progress/current point で復元する。
- `MoveTo` push は scalar を書かず、current point を move x2/y2 に更新し、move_to_count を進める。
- `LineTo` push は current point を要求し、F5au stable tag scalar 2、start_x2、start_y2、end_x2、end_y2 の順に 5 scalar を書き、line_to_count と raster_mask_scalar_count を進める。
- `QuadraticTo` push は current point を要求し、F5au stable tag scalar 3、start_x2、start_y2、control_x2、control_y2、end_x2、end_y2 の順に 7 scalar を書き、quadratic_to_count と raster_mask_scalar_count を進める。
- `SkipNoSegment` push は scalar を書かず、current point を維持し、skip_no_segment_count を進める。
- `raster_mask_writer_owner_free` を追加し、inner F5ba writer owner free に委譲する。

完了条件:

- source policy が docs、private owner、no public constructor、no Clone/Copy、start/push error kind、start validation order、push validation order、inner complete checks、separate kind progress counts、F5au stable tag scalar helper、LineTo 5 scalar order、QuadraticTo 7 scalar order、MoveTo / SkipNoSegment no push、current point missing typed error、push failure recovery order、partial append fail-closed、forbidden F5av/F5aw / byte-backed / old traversal / path object / raster / render / platform / fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_mask_writer.n.md` に types、private owner、start validation、push validation、inner complete checks、kind progress bounds、stable scalar order、current point behavior、push recovery、partial failure fail-closed、no fallback/no byte-backed/no traversal/no render policy の coverage label を追加する。
- implementation review で plan review 指摘がすべて反映されていること、current point が public forged authority になっていないこと、multi scalar failure の recovery が F5ba と同じ順序であることを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の raster mask finalization / path object boundary phase へ進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_raster_mask_writer.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_raster_mask_writer_f5bb.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_stream_sink_writer.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_command_stream_sink_writer_f5bb_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5bb.json -j 1
git diff --check
```

## Phase F5ba: sfnt simple glyph outline point stream item collection path command stream sink writer

目的:

- F5az の `PathCommandStreamSinkOwner` を authority とし、F5aw の `PathCommandValue` を path sink scalar Vec へ書く real writer を追加する。
- writer は `PathCommandValue` を直接受け取り、F5av lookup、F5aw step / drain、byte-backed lookup、old path sink traversal へ戻らない。
- `PathCommandValue` は public value として forged 可能なので、stored tag、source tag、command payload tag を再検査してから push する。
- `SkipNoSegment` は explicit step として progress だけを進め、scalar は一切 push しない。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。
- 成功 result 型、owner-bearing error payload、start validation order、multi scalar push failure atomicity、remaining capacity check が不足していると指摘された。
- revised plan では `WriterOwner`、`WriterStep`、`WriterStartError`、`WriterPushError` を明示し、start は `Result WriterOwner WriterStartError`、push は `Result WriterStep WriterPushError` とする。
- Tesla plan review 2 は `PLAN_BLOCKED`。
- public `PathCommandValue` の stored/source tag と command-derived tag の一致検査、stable tag scalar 値、success progress 更新、push 前 writer 再検査、partial append failure owner の fail-closed 化を source policy に固定する必要があると指摘された。
- Tesla final plan review は `PLAN_APPROVED`。特に `path_sink_scalars_len == path_sink_scalar_count`、`raster_mask_scalars_len == 0`、push failure recovery order、variant ごとの progress 更新、`SkipNoSegment` no-push を固定する。

変更:

- `PathCommandValue` に `stored_tag` と `source_tag` の public accessor を追加する。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner` を追加する。F5az owner、written_count、path_sink_scalar_count、move_to_count、line_to_count、quadratic_to_count、skip_no_segment_count、last_path_command_index を保持し、`Clone` / `Copy` は実装しない。
- F5az `SinkOwnerCapacity` の private equality helper を追加し、stored capacity と derived capacity を field-by-field で比較する。
- `WriterStartErrorKind` と owner-bearing `WriterStartError` を追加する。start error は original F5az owner、capacity derivation error option、derived capacity option、stored capacity、observed len/cap を保持する。
- `writer_owner_start` を追加する。plan、capacity_from_plan、stored capacity equality、path cap、raster cap、path len 0、raster len 0 の順に検査し、progress all zero / last index -1 の writer owner を返す。
- `WriterStep` を追加する。`WrittenMoveTo`、`WrittenLineTo`、`WrittenQuadraticTo`、`SkippedNoSegment` が next writer owner を保持する。
- `WriterPushErrorKind` と owner-bearing `WriterPushError` を追加する。push error は current or reconstructed writer owner、rejected `PathCommandValue`、capacity error option、rejected scalar option、storage error option を保持する。
- `gui_sfnt_simple_glyph_path_command_tag_from_command` を追加し、command payload から expected tag を導く。
- `writer_owner_validate_for_push` を追加する。F5az plan/capacity 再検査、Vec cap equality、`path_sink_scalars_len == path_sink_scalar_count`、`raster_mask_scalars_len == 0`、written count、kind 別 progress count の非負 / plan 上限、aggregate progress、path command index、stored/source/command tag consistency を検査する。
- `writer_owner_push_scalar` を追加する。`vec::push` failure では `vec_push_error_kind &e` を先に読み、その後 `vec_push_error_vec e` で Vec を回収し、F5az owner と writer owner を復元する。
- MoveTo / LineTo writer は stable tag scalar 1 / 2、x2、y2 の順に 3 scalar を push し、成功時に progress を 1 command / 3 scalar だけ進める。
- QuadraticTo writer は stable tag scalar 3、control_x2、control_y2、end_x2、end_y2 の順に 5 scalar を push し、成功時に progress を 1 command / 5 scalar だけ進める。
- SkipNoSegment writer は scalar を push せず、success step として `written_count`、`skip_no_segment_count`、`last_path_command_index` だけ進める。
- `writer_owner_free` を追加し、inner F5az owner free に委譲する。

完了条件:

- source policy が docs、PathCommandValue tag accessors、writer owner type、writer owner no Clone/Copy、start/push error kind、owner-bearing error payload、start validation order、push prevalidation、stored/source/command tag checks、stable tag scalar order、remaining scalar capacity check、push failure recovery order、partial append owner fail-closed guard、success progress update、SkipNoSegment no push、forbidden F5av/F5aw / byte-backed / old traversal / raster / render / platform / fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_stream_sink_writer.n.md` に types、PathCommandValue tag accessors、start validation、push validation、tag scalar order、progress update、push failure recovery、partial failure fail-closed、SkipNoSegment no push、no fallback/no byte-backed/no traversal/no raster の coverage label を追加する。
- implementation review で plan review 指摘がすべて反映されていること、multi scalar failure の owner recovery と progress 未更新が一致することを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の raster mask writer または batch append repair helper phase へ進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_stream_sink_writer.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_command_stream_sink_writer_f5ba.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_stream_sink_owner.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_command_stream_sink_owner_f5ba_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ba.json -j 1
git diff --check
```

## Phase F5az: sfnt simple glyph outline point stream item collection path command stream sink owner

目的:

- F5ay の completed `PathCommandStreamSinkPlan` を authority として、後続 explicit command sink writer と raster mask writer が使う scalar storage owner を確保する。
- F5az は allocation owner boundary までで止め、real writer、raster mask writer、render2d command emission、platform API へ進まない。
- public `SinkPlan` は forged value の可能性があるため、全 count / capacity / derived invariant を再検査する。
- `SkipNoSegment` だけの completed plan は 0 容量 Vec owner を持つ valid success として扱い、silent no-op や `NoCommandsPrepared` にしない。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。
- coarse `InvalidPlan` ではなく、negative count、empty total、last index invalid、prepared / emitted / draw / path / raster mismatch、overflow を typed error kind として分離する必要があると指摘された。
- allocation error は `capacity Option` だけでなく lower `StdErrorKind` を `storage_error Option` として保持する必要があると指摘された。
- raster Vec allocation failure では lower error を保持してから path sink Vec owner を 1 回だけ free し、path sink Vec allocation failure では free しないことを source policy で固定する必要があると指摘された。
- revised plan では error kind を細分化し、`SinkOwnerAllocError` を `kind / plan / capacity Option / storage_error Option` の value-only payload とし、cleanup order を source policy に入れる。
- Tesla revised plan review は `PLAN_APPROVED`。追加条件として、`SkipNoSegment` only completed plan の 0 capacity allocation を valid owner として明文化する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に F5ay plan の不足 accessor を追加する。`emitted_count`、`move_to_count`、`line_to_count`、`quadratic_to_count`、`skip_no_segment_count`、`last_path_command_index` を public accessor で読む。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkOwnerCapacity` を追加する。`path_sink_scalar_capacity`、`raster_mask_scalar_capacity`、`path_segment_capacity`、`raster_edge_capacity` を持つ value-only record とし、`Clone` / `Copy` を実装する。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkOwnerAllocErrorKind` を追加する。negative count / capacity、last index invalid、prepared / emitted / draw / path / raster mismatch、count overflow、path sink scalar storage allocation failure、raster mask scalar storage allocation failure を enum variant で表す。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkOwnerAllocError` を追加する。`kind`、`plan`、`capacity Option`、`storage_error Option StdErrorKind` を保持する value-only record とする。
- plan shape guard は public `SinkPlan` accessor だけを使い、forged direct field access を避ける。
- checked add helper は `2147483647 - left` を先に計算し、`right` が残余を超える場合に `CountOverflow` を返す。
- checked multiply helper は `2147483647 / factor` を先に計算し、`count` が許容値を超える場合に `CountOverflow` を返す。
- scalar format constant は MoveTo path sink 3、LineTo path sink 3、QuadraticTo path sink 5、LineTo raster mask 5、QuadraticTo raster mask 7 とする。
- `sink_owner_capacity_from_plan` は `path_segment_capacity == move + line + quadratic`、`prepared_count == path_segment_capacity + skip`、`emitted_count == total_count`、`raster_edge_capacity == line + quadratic`、`draw_count == raster_edge_capacity` を検査してから scalar capacity を返す。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkOwner` を追加する。plan、capacity、path sink scalar Vec、raster mask scalar Vec を持つ owner record とし、`Clone` / `Copy` を実装しない。
- owner の plan / capacity accessor、path sink scalar Vec len/cap accessor、raster mask scalar Vec len/cap accessor、owner free を追加する。
- `sink_owner_alloc` は capacity derivation 後、path sink scalar Vec、raster mask scalar Vec の順に `vec::with_capacity` で確保する。1 本目の失敗では free しない。2 本目の失敗では lower error を保持し、1 本目の Vec を 1 回だけ free してから error を返す。

完了条件:

- source policy が docs、types、F5ay missing accessors、capacity Clone/Copy、owner no Clone/Copy、error kind variants、alloc error payload、plan shape guard、checked add、checked multiply、scalar width constants、capacity derivation invariant、0 capacity owner success contract、allocation order、path allocation failure no-free、raster allocation failure exactly-one-free、owner len/cap accessors、owner free、forbidden F5ax/F5aw/F5av / byte-backed / old traversal / `vec::push` / path object / raster / render / platform / fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_stream_sink_owner.n.md` に types、plan accessors、precise validation kinds、checked add/mul、capacity derivation、skip-only zero capacity success、allocation order、second allocation cleanup、no fallback/no byte-backed/no traversal/no push/no render の coverage label を追加する。
- implementation review で docs / source policy / doctest / note / todo が揃っていること、粗い `InvalidPlan` がないこと、storage allocation lower error を保持していること、raster allocation failure の cleanup order が正しいことを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の real command sink writer / raster mask writer phase へ進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_stream_sink_owner.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_command_stream_sink_owner_f5az.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_stream_sink_plan.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_command_stream_sink_plan_f5az_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5az.json -j 1
git diff --check
```

## Phase F5ay: sfnt simple glyph outline point stream item collection path command stream sink plan

目的:

- F5ax の `PrepareDrainTerminal::Completed` だけを authority として、後続 explicit command sink / raster mask writer の capacity plan を value-only に固定する。
- `PrepareSummary` 単体や `StepBudgetExhausted` partial terminal を final plan として扱わない。
- command list、real sink、raster mask、render2d command emission、platform API へ進まない。
- count の非負性、completed terminal、emitted count 一致、prepared count 一致、checked addition overflow guard を固定する。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。
- `PrepareSummary` 単体を入力にすると `StepBudgetExhausted` partial summary を completed capacity と誤認できるため、F5ax `PrepareDrainTerminal` か completed-only value が必要と指摘された。
- summary count はすべて非負検査し、`move + line + quadratic + skip`、`line + quadratic`、`move + line + quadratic` は raw addition ではなく checked / guarded arithmetic にする必要があると指摘された。
- source policy は partial terminal を plan 化しないこと、F5ax drain / step や F5aw / F5av を直接呼ばないことを固定する必要がある。
- 修正版では public input を `PrepareDrainTerminal` とし、`Completed` だけを成功 path、`StepBudgetExhausted` を `PrepareNotCompleted` error とする。
- Tesla revised plan review は `PLAN_APPROVED`。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkPlan` を追加する。`total_count`、`emitted_count`、`draw_count`、`move_to_count`、`line_to_count`、`quadratic_to_count`、`skip_no_segment_count`、`path_segment_capacity`、`raster_edge_capacity`、`last_path_command_index` を持ち、`Clone` / `Copy` を実装する。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkPlanErrorKind` と `SinkPlanError` を追加する。error は terminal と extracted count context を保持する value-only record とする。
- public `sink_plan_from_prepare_drain_terminal` は `PrepareDrainTerminal::StepBudgetExhausted` を `PrepareNotCompleted` で拒否し、`Completed` branch だけで summary accessors から count を読む。
- count guard は `total_count`、`move_to_count`、`line_to_count`、`quadratic_to_count`、`skip_no_segment_count`、`emitted_count` の非負性、`total_count > 0`、`last_path_command_index >= 0` を検査する。
- private checked add helper は `2147483647 - left` を計算し、`right` が残余を超える場合に `CountOverflow` を返す。overflow guard を通した後だけ `add left right` を使う。
- `move + line`、`move + line + quadratic`、`move + line + quadratic + skip`、`line + quadratic` を checked add で求める。
- `prepared_count == total_count`、`emitted_count == total_count`、`draw_count == raster_edge_capacity` を検査してから plan を返す。

完了条件:

- source policy が docs、types、SinkPlan / ErrorKind / Error value-only Clone/Copy、public input が `PrepareDrainTerminal` であること、`StepBudgetExhausted` が `PrepareNotCompleted` になること、各 count の非負検査、checked add helper の `2147483647 - left` guard、prepared / emitted / draw count invariant、forbidden F5ax drain / F5ax step / F5aw step / F5av lookup / byte-backed / old traversal / Vec / path object / raster / render / platform / fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_stream_sink_plan.n.md` に types、completed terminal authority、budget exhausted rejection、non-negative count guard、checked add guard、capacity derivation、count invariants、no fallback/no byte-backed/no traversal/no Vec/no raster の coverage label を追加する。
- implementation review で docs / source policy / doctest / note / todo が揃っていること、partial terminal が成功 path へ進まないこと、F5ax/F5aw/F5av や実描画 API へ進んでいないことを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の real command sink writer / raster mask writer boundary へ進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_stream_sink_plan.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_command_stream_sink_plan_f5ay.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_stream_prepare.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_command_stream_prepare_f5ay_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ay.json -j 1
git diff --check
```

## Phase F5ax: sfnt simple glyph outline point stream item collection path command stream prepare

目的:

- F5aw の `PathCommandStreamStep` を authority として、path command stream を value-only prepare summary に畳む。
- 後続 command sink / raster mask / render2d command emission の前段階として、command 種別 count と last path command index だけを保持する。
- full path object construction、`Vec` allocation、sink mutation、raster / render / platform / host API へ進まない。
- F5av lookup を直接呼ばず、command acquisition は F5aw step helper だけを通す。

plan review:

- Tesla plan review は `PLAN_APPROVED`。
- F5ax は sink ではなく prepare summary boundary として扱い、counts と `last_path_command_index` の value-only summary を最初の render preparation contract にする方針で承認された。
- F5ax prepare drain は F5aw step を直接呼ばず、F5ax prepare step helper だけを呼ぶ。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamPrepareSummary` を追加する。`total_count`、`move_to_count`、`line_to_count`、`quadratic_to_count`、`skip_no_segment_count`、`last_path_command_index` を持ち、`Clone` / `Copy` を実装する。
- initial summary helper は count をすべて `0`、`last_path_command_index` を `-1` とする。
- `PathCommandValue` の `path_command_index` accessor を追加し、F5ax は field を直接読まない。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamPrepareAction` を追加する。`CountedMoveTo`、`CountedLineTo`、`CountedQuadraticTo`、`CountedSkipNoSegment` の enum とし、`Clone` / `Copy` を実装する。
- private summary increment helper は `PathCommandValue` の command payload を 1 回だけ読み、`GuiSfntSimpleGlyphPathCommand` を `match` して 1 種類の count だけを増やす。`total_count` は常に 1 増やし、`last_path_command_index` は accessor で読んだ path command index へ更新する。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamPrepareStep` を `Prepared action summary next_cursor` / `Completed summary cursor` の enum として追加する。completed branch は dummy action / dummy command value を持たない。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamPrepareStepError` は current summary、cursor、lower F5aw step error を `Option` で保持する。
- public prepare step は F5aw `path_command_stream_step` を exactly once 呼ぶ。lower completed は summary を変えず `Completed`、lower emitted は summary increment helper を 1 回呼び `Prepared` を返す。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamPrepareDrainTerminal` を `Completed summary cursor emitted_count` / `StepBudgetExhausted summary cursor emitted_count` の enum として追加する。
- public prepare drain は `remaining_steps <= 0` では prepare step も F5aw step も呼ばず `StepBudgetExhausted summary cursor 0` を返す。positive budget では F5ax prepare step helper だけを呼ぶ。

完了条件:

- source policy が docs、types、Summary / Action / Update / Step / StepError / DrainTerminal value-only Clone/Copy、initial summary、PathCommandValue path_command_index accessor、command accessor exactly once、command variant match、single counter increment、completed no dummy action/value、prepare step F5aw step exactly once / no F5av lookup、prepare drain prepare-step-only / budget exhausted no step、forbidden byte-backed / old traversal / Vec / path object / raster / render / platform / fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_stream_prepare.n.md` に types、initial summary、path command value accessor、single command classification、prepare step completed no dummy、prepare step uses F5aw once、prepare drain terminal variants、budget exhausted no step、no fallback/no byte-backed/no traversal/no Vec/no raster の coverage label を追加する。
- implementation review で docs / source policy / doctest / note / todo が揃っていること、drain から F5aw step / F5av lookup を直接呼んでいないこと、実描画 API へ進んでいないことを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の explicit command sink / raster mask preparation boundary へ進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_stream_prepare.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_command_stream_prepare_f5ax.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_path_command_stream_cursor.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_path_command_stream_cursor_f5ax_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ax.json -j 1
git diff --check
```

## Phase F5aw: sfnt simple glyph outline point stream item collection path command stream cursor

目的:

- F5av の `PathCommandValue` lookup を順序付きに読む bounded cursor / stream preparation boundary を実装する。
- full stream object construction、`Vec` allocation、path object materialization、raster / render / platform / host API へ進まない。
- `PathCommandTagCompleteOwner` は borrow し、storage owner を消費しない。
- scheduler / timeslice 層へ渡せるように、1 step と bounded drain terminal を typed enum として分離する。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。
- dummy `PathCommandValue` を必要とする step struct ではなく、`Emitted value next_cursor` / `Completed cursor` の enum にする必要がある。
- drain terminal は `{ next_cursor, emitted_count }` だけではなく、`Completed cursor emitted_count` と `StepBudgetExhausted cursor emitted_count` の enum にする必要がある。
- bounded drain は F5av lookup を直接呼ばず、F5aw step helper だけを呼ぶ必要がある。
- empty stream success を別に作らず、既存 capacity-shape contract で forged empty / malformed capacity を拒否する必要がある。
- Tesla revised plan review は `PLAN_APPROVED`。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandStreamCursor` を追加する。`next_index` と `end_index` の value-only cursor で、`Clone` / `Copy` を実装する。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandStreamCursorErrorKind` と `CursorError` を追加する。capacity / index context を保持し、owner payload は持たない。
- cursor creation は summary capacity、non-consuming owner storage capacity、collection capacity、capacity shape、start index を固定順で検査し、`end_index = path_command_count` とする。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandStreamStep` は `Emitted PathCommandValue PathCommandStreamCursor` / `Completed PathCommandStreamCursor` の enum とする。
- step は capacity / cursor authority を検査し、`next_index >= end_index` なら F5av lookup を呼ばず `Completed` を返す。未完了 branch だけで F5av lookup を exactly once 呼び、成功時に cursor を 1 つ進める。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandStreamDrainTerminal` は `Completed cursor emitted_count` / `StepBudgetExhausted cursor emitted_count` の enum とする。
- bounded drain は `remaining_steps <= 0` では step helper も F5av lookup も呼ばず `StepBudgetExhausted cursor 0` を返す。positive budget では F5aw step helper だけを呼ぶ。

完了条件:

- source policy が docs、types、Cursor / CursorError / Step / StepError / DrainTerminal value-only Clone/Copy、capacity authority order、owner storage non-consuming、cursor validation、completed branch no dummy value / no F5av lookup、step non-terminal F5av lookup exactly once、drain calls step helper only、budget exhausted no step / no F5av、explicit Completed / StepBudgetExhausted terminal、forbidden byte-backed / old traversal / Vec / path object / raster / render / platform / fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_path_command_stream_cursor.n.md` に types、authority checks、cursor validation、step completed no value/no lookup、step emits via F5av exactly once、drain terminal variants、budget exhausted no step/no lookup、no fallback/no byte-backed/no traversal/no Vec/no raster の coverage label を追加する。
- implementation review で docs / source policy / doctest / note / todo が揃っていること、drain から F5av lookup を直接呼んでいないこと、owner を消費していないことを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の path command stream consumer / render preparation boundary へ進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_path_command_stream_cursor.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_path_command_stream_cursor_f5aw.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_path_command_value.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_path_command_value_f5aw_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5aw.json -j 1
git diff --check
```

## Phase F5av: sfnt simple glyph outline point stream item collection path command value lookup

目的:

- SFNT simple glyph outline point stream item collection path command value lookup を F5au 後続の read-only boundary として実装する。
- F5au の `PathCommandTagCompleteOwner` を authority とし、owner storage の PathCommandTag scalar と collection-backed source event を照合して 1 logical path command value を返す。
- `PathCommandTagCompleteOwner` は borrow し、storage owner を消費しない。成功時も失敗時も owner recovery は不要である。
- `SkipNoSegment` reason は scalar から推測せず、collection-backed source event payload から再導出する。
- full stream construction、raster/render/platform/host API、font fallback、byte-backed lookup、old sink traversal へ進まない。

plan review:

- Tesla plan review は `PLAN_APPROVED`。
- F5av は read-only value lookup boundary とし、stream construction / raster / render / platform には進めない。
- source event は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at` を span validation 後に exactly once 呼び、kind は返った event から導出する。`path_sink_event_kind_at` を別に呼んで payload と tag source を二重化しない。
- stored tag と source tag が一致しない場合は `TagMismatch` を返し、fallback / no-op / inferred replacement command にしない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_path_command_tag_from_scalar_value` と `gui_sfnt_simple_glyph_path_command_tag_eq` を追加する。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePathCommandTagReadErrorKind`、`GuiSfntSimpleGlyphOutlinePathCommandTagReadError`、`gui_sfnt_simple_glyph_outline_storage_read_path_command_tag` を追加する。
- storage-level PathCommandTag read helper は capacity shape、scalar slot count、scalar storage capacity、path command index range、PathCommandTag region readiness、slot presence、known scalar value を検査し、unknown scalar は `PathCommandTagScalarUnknown` として返す。storage owner は消費しない。
- `alloc/gui/font/sfnt/glyf.nepl` に CompleteOwner の non-consuming storage capacity accessor、private read path command tag helper、private read edge owner helper を追加する。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandValue` を追加する。value は path command index、edge index、contour index、contour edge index、event slot、stored tag、source tag、command payload を保持し、value-only なので `Clone` / `Copy` を実装する。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandValueErrorKind` と `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandValueError` を追加する。error は owner を含まず、typed context を `Option` で保持する。
- public `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_complete_owner_path_command_value` を追加する。
- public function は summary capacity、non-consuming owner storage capacity、collection capacity、path command index を固定順で検査してから storage tag read / Edge owner read / collection span / source event へ進む。

完了条件:

- source policy が docs、types、PathCommandTagReadError value-only Clone/Copy、PathCommandValue / Error value-only Clone/Copy、CompleteOwner non-consuming capacity accessor、CompleteOwner read helper non-consuming、authority order、index mapping、PathCommandTag scalar read validation、source event exactly once、source kind derived from event、tag mismatch branch、SkipNoSegment reason source payload recovery、forbidden byte-backed / traversal / stream / render / raster / platform / fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_path_command_value.n.md` に types、authority checks、scalar read checks、source event exactly once、tag mismatch、SkipNoSegment reason rederive、no fallback/no byte-backed/no stream/no raster の coverage label を追加する。
- implementation review で value lookup が owner を消費しないこと、storage mutation が無いこと、event source が二重化していないこと、source policy / doctest / note / todo が揃っていることを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の path command stream preparation / bounded stream cursor boundary に進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_path_command_value.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_path_command_value_f5av.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_path_command_tag_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_path_command_tag_drain_f5av_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5av.json -j 1
git diff --check
```

Subagent review:

- 実装前に文書レビューを受ける。
- 実装後に core が alloc/std/platform を import していないこと、fallback sentinel がないこと、invalid case が Result で返ることを確認させる。

## Phase F2: std font resource boundary

目的:

- Font bytes loading を app-facing raw path 文字列ではなく typed request として std layer に置く。
- Web VFS / native filesystem / bare embedded blob の差を `std/gui` と platform provider の境界へ押し出す。

変更:

- `stdlib/std/gui/font_resource.nepl` を追加する。
- `GuiFontDecodePolicy`、`GuiFontResourceSource`、`GuiFontResourcePath`、`GuiResourceHash`、`GuiFontResourceRequest` を追加する。
- `gui_font_resource_request` は typed path、face index、expected hash、decode policy を保持する。
- F2 は request shape だけを検査する。`face_index` が `Some n` で `n < 0` の場合は `GuiError::InvalidCommand` とする。Collection font の `face_count` が必要な検査は F4 へ送る。
- `std/gui.nepl` facade から公開する。
- `tests/stdlib/gui_std.n.md` に doctest を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` を追加し、標準 API に DOM / Canvas / FontFace / CoreText / DirectWrite / fontconfig handle が入らないことを固定する。
- 同 source policy で formal font renderer / font resource contract が `MockTextMeasurer`、`HostTextMeasurer`、`host_text_measurer_fixed` に依存しないことを固定する。

検証:

```powershell
node nodesrc/tests.js -i tests/stdlib/gui_std.n.md --no-tree -o tmp_gui_std_font.json -j 1
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/run_source_policy_regressions.js --warn-only
git diff --check
```

## Phase F3: bundled HackGen fixture routing

目的:

- `web/src/fonts/HackGenConsoleNF-Regular.ttf` を formal fixture として `fonts/HackGenConsoleNF-Regular.ttf` に mapping する。
- HackGen 専用 API を作らず、任意 font resource を登録できる経路を保つ。

変更:

- Web VFS manifest に canonical resource path `fonts/HackGenConsoleNF-Regular.ttf` と `fonts/HackGen-LICENSE.txt` を登録する。
- Web VFS 内部 path は `/fonts/...` とし、canonical path とは別の transport 表現として扱う。
- `web/src/gui-font/font-resource-vfs.ts` を追加し、bundled resource manifest、path normalization、VFS mount、typed mount error を持たせる。
- Web Playground startup で mount promise を開始し、`neplg2 run` の直前に完了を待つ。失敗時は typed error を terminal に表示して実行を開始しない。
- Compile-only path は runtime font bytes を要求しないため mount を待たない。
- Native resource root の探索 contract を doc と source policy に追加する。
- Bare は embedded blob provider が未設定なら unsupported を返す contract にする。
- Source policy で、HackGen 専用 API、suffix match、silent success、binary/read-only file の compile overlay 混入を禁止する。

検証:

```powershell
npm --prefix web run build:ts
node nodesrc/test_web_gui_font_rendering_contract.js
git diff --check
```

完了条件:

- `web/src/fonts/HackGenConsoleNF-Regular.ttf` が `/fonts/HackGenConsoleNF-Regular.ttf` として VFS に read-only mount される。
- `VFS.serializeForCompile()` が font binary と read-only license text を compiler overlay へ含めない。
- `FetchUnavailable`、`InvalidResourcePath`、`NetworkError`、`HttpError`、`InvalidBytes`、`InvalidText`、`VfsWriteFailed` のいずれも typed error として扱われる。
- Native / Bare / Headless の resource provider contract が doc と source policy で検査される。

## Phase F4a: sfnt directory and numeric metrics parser

目的:

- TTF / OTF / TTC / OTC の table directory と numeric basic metrics を decode する。

変更:

- `alloc/gui/font/sfnt.nepl` と basic table parser を追加する。
- Invalid table directory、invalid table offset、unsupported container、collection face index error を typed error として扱う。
- 未解析の extra table は error にせず無視する。error にするのは unsupported container、必須 numeric table の欠落、範囲外 offset、face selection の不整合だけである。
- Headless/offscreen tests が explicit fixture bytes を使えるようにする。

完了条件:

- explicit fixture bytes から container kind、face count、face index、units per em、ascent、descent、line gap、num glyphs を取得できる。
- Missing `head` / `hhea` / `maxp` や invalid face index は代替成功させず error になる。

## Phase F4b: sfnt name table policy

目的:

- font family、subfamily、full name などの代表値を name table から decode するための encoding policy を固定する。
- name parser を numeric metrics parser から分け、metadata parse の成功条件に name decode を混ぜない。

変更:

- `alloc/gui/font/sfnt.nepl` を facade にし、F4a 実装を `alloc/gui/font/sfnt/metadata.nepl` へ置く。
- `alloc/gui/font/sfnt/name.nepl` を追加する。
- `GuiSfntNameEncodingKind`、`GuiSfntNameRecord`、`GuiSfntNameSelection`、`GuiSfntNames` を追加する。
- name ID 1 / 2 / 4 を family / subfamily / full name として扱う。
- 代表 record の順位は、Windows platform 3 encoding 1 language 0x0409、Windows platform 3 のその他、Macintosh platform 1 encoding 0 language 0、Macintosh platform 1 のその他、の順にする。
- Windows platform 3 encoding 1 language 0x0409 は UTF-16BE ASCII subset として decode する。
- Macintosh platform 1 encoding 0 language 0 は Roman ASCII subset として decode する。
- higher-ranked candidate が未対応 encoding の場合は、lower-ranked candidate へ暗黙に切り替えず `UnsupportedNameEncoding` を返す。
- `name` table 欠落は `MissingTable`、format 0 以外は `UnsupportedNameTableFormat`、record / string range 不正や empty selected string は `MalformedNameRecord`、ASCII subset 外文字は `UnsupportedNameCharacter` とする。
- name ID 1 / 2 / 4 の candidate が存在しない場合、その field は `Option::None` とする。
- Source policy で `gui_sfnt_parse_metadata` が `gui_sfnt_parse_names` を呼ばないこと、SFNT parser が platform / host font API / path display-name authority を持たないことを固定する。

完了条件:

- explicit fixture bytes から `Demo` / `Regular` / `Demo Regular` を取得できる。
- `name` table がない fixture は `MissingTable` になる。
- unsupported selected record は `UnsupportedNameEncoding`、UTF-16BE の奇数 byte length は `MalformedNameRecord`、ASCII subset 外文字は `UnsupportedNameCharacter` になる。
- `gui_sfnt_parse_metadata` の existing F4a doctest は name table の有無に依存せず通る。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt.n.md --no-tree -o tmp_gui_font_sfnt.json -j 1
git diff --check
```

## Phase F4c: sfnt cmap glyph lookup

目的:

- Unicode BMP code point から `GuiGlyphId` を取得する最初の `cmap` parser を追加する。
- glyph mapping を host font API、browser text API、path / family name、暗黙置換に依存させない。

変更:

- `alloc/gui/font/sfnt/cmap.nepl` を追加する。
- `alloc/gui/font/sfnt.nepl` facade から metadata、name、cmap を再公開する。
- `GuiSfntDirectory` に optional `cmap` table record を追加し、`gui_sfnt_directory_cmap` を公開する。
- `GuiSfntParseErrorKind` に `UnsupportedCmapEncoding`、`UnsupportedCmapTableFormat`、`MalformedCmapRecord`、`MissingGlyphMapping` を追加する。
- `gui_sfnt_lookup_glyph_id` は `Result GuiGlyphId GuiSfntParseError` を返し、raw `i32` を public glyph id として返さない。
- F4c の subtable selection は platformID 3 / encodingID 1 の最初の record だけを選ぶ。対象 record がなければ `UnsupportedCmapEncoding`、選択 record が format 4 でなければ `UnsupportedCmapTableFormat` とする。
- BMP 外 code point は `UnsupportedCmapEncoding`、BMP 内で segment がない、glyphIdArray entry が 0、computed glyph id が 0 の場合は `MissingGlyphMapping` とする。
- Format 4 の declared table header、encoding record array overlap、`length`、`segCountX2`、`reservedPad`、segment array bounds、idRangeOffset target bounds を検査し、不正なら `MalformedCmapRecord` とする。
- Source policy で `gui_sfnt_parse_metadata` が `gui_sfnt_lookup_glyph_id` を呼ばないこと、SFNT facade が `metadata` / `name` / `cmap` を公開すること、`cmap` parser が platform / host font API / 暗黙置換 / path authority を持たないことを固定する。

完了条件:

- explicit fixture bytes から ASCII `A` の glyph id 36 を `GuiGlyphId` として取得できる。
- `cmap` table がない fixture は `MissingTable` になる。
- platformID 3 / encodingID 1 がない fixture は `UnsupportedCmapEncoding` になる。
- selected record が format 4 以外の場合は `UnsupportedCmapTableFormat` になる。
- glyph 0、missing segment、壊れた format 4 array、encoding record array を指す selected subtable offset、短い declared table header は typed error になる。
- unsupported selected record と別の plausible record が同居しても別 record に切り替えない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt.n.md --no-tree -o tmp_gui_font_sfnt.json -j 1
git diff --check
```

## Phase F4d: sfnt hmtx horizontal metrics lookup

目的:

- `GuiGlyphId` から horizontal advance width と left side bearing を取得する最初の `hmtx` parser を追加する。
- layout engine が host text measurement や fixed-cell utility に逃げず、font bytes の metrics table を authority として使えるようにする。

変更:

- `alloc/gui/font/sfnt/hmtx.nepl` を追加する。
- `alloc/gui/font/sfnt.nepl` facade から metadata、name、cmap、hmtx を再公開する。
- `GuiSfntDirectory` に optional `hmtx` table record を追加し、`gui_sfnt_directory_hmtx` を公開する。
- `GuiSfntParseErrorKind` に `MalformedHmtxRecord` と `MissingGlyphMetric` を追加する。
- `GuiSfntHorizontalMetric` を追加し、glyph、advance_width、left_side_bearing を typed value として返す。
- `gui_sfnt_lookup_horizontal_metric` は `Result GuiSfntHorizontalMetric GuiSfntParseError` を返す。
- `hhea.numberOfHMetrics` は `hhea.offset + 34` の u16 として読む。このため `hhea.length >= 36` は `hmtx` lookup 専用の要件とし、F4a metadata parser の `hhea.length >= 10` は変更しない。
- `numberOfHMetrics <= 0`、`numberOfHMetrics > maxp.numGlyphs`、`glyphRaw <= 0`、`glyphRaw >= maxp.numGlyphs`、declared `hmtx.length` 不足は typed error とする。
- `hmtx.length` は `numberOfHMetrics * 4 + (numGlyphs - numberOfHMetrics) * 2` 以上でなければならない。file 末尾に余分な byte があっても declared table length を越えて読まない。
- `glyphRaw < numberOfHMetrics` は `longHorMetric[glyphRaw]` を読む。`glyphRaw >= numberOfHMetrics` は最後の longHorMetric の advance width と leftSideBearing array を読む。
- Source policy で `gui_sfnt_parse_metadata` が `gui_sfnt_lookup_horizontal_metric` を呼ばないこと、`hmtx` parser が platform / host font API / path authority / fixed-cell fallback / name or cmap 代替を持たないことを固定する。

完了条件:

- explicit fixture bytes から glyph 1 の longHorMetric advance width と left side bearing を取得できる。
- explicit fixture bytes から glyph 3 の last advance width と leftSideBearing array entry を取得できる。
- `hmtx` table がない fixture は `MissingTable` になる。
- `hhea` が `numberOfHMetrics` を読めない fixture、invalid `numberOfHMetrics`、glyph range 外、declared `hmtx.length` 不足は typed error になる。
- `gui_sfnt_parse_metadata` の existing F4a doctest は `hmtx` table の有無に依存せず通る。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt.n.md --no-tree -o tmp_gui_font_sfnt.json -j 1
git diff --check
```

## Phase F4e: sfnt loca/glyf glyph header bounds lookup

目的:

- `GuiGlyphId` から glyph header の x/y bounds を取得する最初の `loca` / `glyf` parser を追加する。
- layout engine が rendered bounds を扱う前段として、host text measurement や fixed-cell utility に逃げず、font bytes の outline table header を authority として使えるようにする。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` を追加する。
- `alloc/gui/font/sfnt.nepl` facade から metadata、name、cmap、hmtx、glyf を再公開する。
- `GuiSfntDirectory` に optional `loca` / `glyf` table record を追加し、`gui_sfnt_directory_head`、`gui_sfnt_directory_loca`、`gui_sfnt_directory_glyf` を公開する。
- `GuiSfntParseErrorKind` に `UnsupportedLocaFormat`、`MalformedGlyfRecord`、`MissingGlyphOutline` を追加する。
- `GuiSfntGlyphBounds` を追加し、glyph、x_min、y_min、x_max、y_max を typed value として返す。
- `gui_sfnt_lookup_glyph_bounds` は `Result GuiSfntGlyphBounds GuiSfntParseError` を返す。
- `head.indexToLocFormat` は `head.offset + 50` の i16 として読む。このため `head.length >= 52` は `glyf` lookup 専用の要件とし、F4a metadata parser の `head.length >= 20` は変更しない。
- `indexToLocFormat == 0` は short loca offset として u16 value を 2 倍する。`indexToLocFormat == 1` は long loca offset として u32 value を読む。u32 value が i32 範囲外なら `MalformedGlyfRecord` とする。
- `indexToLocFormat` が 0 / 1 以外なら `UnsupportedLocaFormat` とする。
- `loca.length` は format 0 で `(numGlyphs + 1) * 2`、format 1 で `(numGlyphs + 1) * 4` 以上でなければならない。file 末尾に余分な byte があっても declared table length を越えて読まない。
- `glyphRaw <= 0`、`glyphRaw >= maxp.numGlyphs`、empty glyph range は `MissingGlyphOutline` とする。
- `start > end`、`end > glyf.length`、glyph header 10 byte 未満、inverted x/y bounds は `MalformedGlyfRecord` とする。
- Source policy で `gui_sfnt_parse_metadata` が `gui_sfnt_lookup_glyph_bounds` を呼ばないこと、`glyf` parser が platform / host font API / path authority / fixed-cell fallback / name or cmap or hmtx 代替を持たないことを固定する。

完了条件:

- explicit fixture bytes から glyph 1 の negative x/y min を含む bounds を取得できる。
- format 1 loca fixture から glyph bounds を取得できる。
- `loca` / `glyf` table がない fixture は `MissingTable` になる。
- `head` が `indexToLocFormat` を読めない fixture、unsupported format、long loca high-bit u32 offset、declared `loca.length` 不足、decreasing offset、empty glyph、short glyph header、inverted bounds は typed error になる。
- `gui_sfnt_parse_metadata` の existing F4a doctest は `loca` / `glyf` table の有無に依存せず通る。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt.n.md --no-tree -o tmp_gui_font_sfnt.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4f: sfnt simple glyph topology lookup

目的:

- full outline / rasterization の前段として、simple glyph の contour endpoint array、instruction length、point data range を typed value として取得する。
- 後続の flags / coordinate decode が host font API や fallback に逃げず、font bytes 内の checked topology から始められるようにする。

変更:

- `GuiSfntParseErrorKind` に `UnsupportedGlyphOutlineFormat` を追加する。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphTopology` と `gui_sfnt_lookup_simple_glyph_topology` を追加する。
- `GuiSfntSimpleGlyphTopology` は glyph、bounds、contour_count、point_count、instruction_length、point_data_offset、point_data_length を持つ。
- `point_data_offset` は file absolute offset ではなく `glyf` table-relative offset とする。
- `numberOfContours < 0` は composite glyph / unsupported outline format として `UnsupportedGlyphOutlineFormat` を返す。
- `numberOfContours == 0` は renderable outline がないため `MissingGlyphOutline` を返す。
- endpoint array 全体、instructionLength、instructions、point data range は selected glyph range 内に閉じる。
- endpoint は strict increasing とし、`point_count = last_endpoint + 1` とする。overflow や `point_count <= 0` は `MalformedGlyfRecord` とする。
- `numberOfContours > 0` かつ `point_count > 0` で `point_data_length == 0` なら `MalformedGlyfRecord` とする。
- Source policy で simple topology API、typed error、declared range validation、metadata / name / cmap / hmtx / platform API 非依存を固定する。

完了条件:

- explicit fixture bytes から glyph 1 の contour count、point count、instruction length、point data offset、point data length を取得できる。
- composite glyph、zero contour、non-increasing endpoint、short endpoint array、short instruction length、instruction overrun、missing point data は typed error になる。
- F4e の glyph bounds doctest と `glyf.nepl` module doctest は引き続き通る。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4g: sfnt simple glyph point stream range lookup

目的:

- simple glyph の flags repeat 展開と x/y coordinate byte range を検査する。
- coordinate value や point `Vec` をまだ作らず、後続 decoder が読む raw byte range を typed value として返す。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphPointStream` と `gui_sfnt_lookup_simple_glyph_point_stream` を追加する。
- `GuiSfntSimpleGlyphPointStream` は topology、flag_data_offset、flag_data_length、x_data_offset、x_data_length、y_data_offset、y_data_length、trailing_data_offset、trailing_data_length を持つ。
- すべての offset は `glyf` table-relative とする。
- `flag_data_offset = topology.point_data_offset` とする。
- `flag_data_length` は expanded logical flag count ではなく、repeat count byte を含む raw consumed flag stream length とする。
- repeat flag byte 自身は 1 point 分であり、repeat count byte は追加 point 数である。`repeat_count = 0` は current flag 1 個だけを意味する。
- flags scan はちょうど `point_count` 個の logical flags を満たす。point count に届かない、repeat byte 欠落、repeat run overrun は `MalformedGlyfRecord` とする。
- x/y coordinate byte length は short bit と same bit だけから計算する。short bit が立つ場合、same / positive bit は sign であり byte length には影響しない。
- `x_data_offset = flag_data_offset + flag_data_length`、`y_data_offset = x_data_offset + x_data_length`、`trailing_data_offset = y_data_offset + y_data_length`、`trailing_data_length = glyph_end - trailing_data_offset` とする。
- `trailing_data_length < 0` は `MalformedGlyfRecord`。`trailing_data_length >= 0` は success として明示値で返す。
- Source policy で raw flag length、repeat semantics、x/y length formula、trailing data policy、metadata / name / cmap / hmtx / platform API 非依存を固定する。

完了条件:

- explicit fixture bytes から no-repeat point stream の flag/x/y/trailing ranges を取得できる。
- repeat run を含む fixture で raw flag length と coordinate ranges を取得できる。
- `repeat_count = 0` を current flag 1 個として扱う fixture が成功する。
- short=1、short=0 same=1、short=0 same=0 の x/y byte length 分岐を doctest で固定する。
- repeat overrun、missing repeat byte、x coordinate overrun、y coordinate overrun は typed `MalformedGlyfRecord` になる。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4h: sfnt simple glyph single point decode

目的:

- checked point stream range から 1 logical point の coordinate、on-curve、contour end state を復元する。
- full point `Vec` / outline builder は allocation failure と owner recovery の contract を設計してから後続 phase で実装する。
- F4h は allocation なしで動作し、F4g の range validation を必ず通る。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphPoint` と `gui_sfnt_lookup_simple_glyph_point` を追加する。
- `GuiSfntSimpleGlyphPoint` は glyph、point_index、x、y、on_curve、end_of_contour を持つ。
- `point_index < 0` または `point_index >= topology.point_count` は `MissingGlyphOutline` とする。
- flag / coordinate / endpoint の byte 構造不整合は `MalformedGlyfRecord` とする。
- `gui_sfnt_lookup_simple_glyph_point` は `gui_sfnt_glyf_simple_point_stream_with_tables` を通り、F4g-derived `flag_data` / `x_data` / `y_data` range 内だけを読む。
- flag bit 0 を `on_curve` とする。
- x delta は xShort / xPositive / xSame から `+u8`、`-u8`、`0`、`i16be` に復元する。
- y delta は yShort / yPositive / ySame から `+u8`、`-u8`、`0`、`i16be` に復元する。
- coordinate は point 0 から `point_index` まで累積する。target が repeat run の途中にある場合も、target より前の repeated point の delta は消費・累積する。
- `end_of_contour` は topology から endpoint array offset を復元し、endpoint value と point_index の一致で判定する。
- F4h は `trailing_data_length` を読まず、zero padding も要求しない。
- Source policy で single point API、no Vec allocation、F4g validation reuse、cumulative coordinate semantics、out-of-range error kind、platform / fallback 非依存を固定する。

完了条件:

- no-repeat fixture で point 0 と endpoint point を decode できる。
- repeat run fixture で target が repeat run 内にある場合でも、前の repeated point の delta が累積される。
- signed long coordinate と negative short coordinate を decode できる。
- `repeat_count = 0` fixture で x/y 0、contour end を decode できる。
- `point_index = -1` と `point_index = point_count` は `MissingGlyphOutline` になる。
- coordinate overrun 系 fixture を point lookup 経由でも `MalformedGlyfRecord` として扱える。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4i: sfnt simple glyph contour span lookup

目的:

- checked simple glyph topology から、1 contour の inclusive logical point range を返す。
- full outline `Vec` / curve segment builder / mask rasterizer は allocation failure と owner recovery の contract を設計してから後続 phase で実装する。
- F4i は allocation なしで動作し、F4f の topology validation だけに依存する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphContourSpan` と `gui_sfnt_lookup_simple_glyph_contour_span` を追加する。
- `GuiSfntSimpleGlyphContourSpan` は glyph、contour_index、start_point_index、end_point_index、point_count を持つ。
- `end_point_index` は inclusive endpoint とし、`point_count = end_point_index - start_point_index + 1` とする。
- `contour_index < 0` または `contour_index >= topology.contour_count` は `MissingGlyphOutline` とする。
- endpoint array read failure や F4f topology validation で観測された endpoint 不整合は `MalformedGlyfRecord` とする。
- contour 0 の start は 0、contour n の start は contour n-1 の endpoint + 1 とする。
- `gui_sfnt_lookup_simple_glyph_contour_span` は `gui_sfnt_glyf_simple_topology_with_tables` を通る。
- F4i は `gui_sfnt_glyf_simple_point_stream_with_tables`、`gui_sfnt_lookup_simple_glyph_point_stream`、`gui_sfnt_lookup_simple_glyph_point` を呼ばない。
- Source policy で contour span API、F4f validation reuse、F4g/F4h 非依存、metadata 非依存、no Vec allocation を固定する。

完了条件:

- two-contour fixture の contour 0 が start 0、end 1、point_count 2 を返す。
- two-contour fixture の contour 1 が start 2、end 3、point_count 2 を返す。
- one-contour signed coordinate fixture の contour 0 が start 0、end 2、point_count 3 を返す。
- `contour_index = -1` と `contour_index = contour_count` は `MissingGlyphOutline` になる。
- malformed endpoint fixture を contour span lookup 経由でも `MalformedGlyfRecord` として観測できる。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4j: sfnt simple glyph contour-local point lookup

目的:

- F4i の contour span と F4h の single point decode を合成し、contour-local point index から 1 点だけを復元する。
- full point `Vec` / full contour `Vec` / curve segment builder / rasterizer は後続 phase で実装する。
- F4j は allocation なしで動作し、streaming contour sink の前段になる typed boundary を提供する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphContourPoint` と `gui_sfnt_lookup_simple_glyph_contour_point` を追加する。
- `GuiSfntSimpleGlyphContourPoint` は `span GuiSfntSimpleGlyphContourSpan`、`contour_point_index i32`、`point GuiSfntSimpleGlyphPoint` を持つ。
- `contour_point_index` は contour-local index、nested `point.point_index` は glyph absolute point index とする。
- `absolute_point_index = span.start_point_index + contour_point_index` とする。
- 処理順序は `contour span lookup -> validate contour_point_index -> compute absolute_point_index -> point decode` とし、local index validation を point decode より先に行う。
- `contour_point_index < 0` または `contour_point_index >= span.point_count` は `MissingGlyphOutline` とする。
- F4i / F4h から返る `MalformedGlyfRecord` などの typed error は伝播する。
- `gui_sfnt_glyf_simple_contour_point_with_tables` は public wrapper ではなく `gui_sfnt_glyf_simple_contour_span_with_tables` と `gui_sfnt_glyf_simple_point_with_tables` を通る。
- Source policy で contour point API、internal table helper reuse、local-before-point validation、absolute point index formula、metadata 非依存、no Vec allocation を固定する。

完了条件:

- two-contour fixture の contour 0 local 0 が absolute point 0、x 0、y 0、not contour end を返す。
- two-contour fixture の contour 1 local 1 が absolute point 3、contour end true を返す。
- one-contour signed coordinate fixture の local 1 が absolute point 1、x 2、y -6、on_curve true を返す。
- local index `-1` と `span.point_count` は `MissingGlyphOutline` になる。
- coordinate overrun fixture を contour point lookup 経由でも `MalformedGlyfRecord` として扱える。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4k: sfnt simple glyph contour edge lookup

目的:

- contour-local edge index から、contour topology 上で隣接する start / end point pair を 1 つだけ復元する。
- edge は描画線分ではなく topology pair であり、quadratic curve classification、implied on-curve point、winding、rasterization は後続 phase で実装する。
- full edge `Vec` / full contour `Vec` / curve segment builder は作らず、allocation なしの lookup boundary を提供する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphContourEdge` と `gui_sfnt_lookup_simple_glyph_contour_edge` を追加する。
- `GuiSfntSimpleGlyphContourEdge` は `start GuiSfntSimpleGlyphContourPoint`、`end GuiSfntSimpleGlyphContourPoint`、`edge_index i32`、`next_contour_point_index i32` を持つ。
- `edge_index` は contour-local edge start index、`next_contour_point_index` は wrap 後の contour-local end index とする。
- `start.contour_point_index == edge_index`、`end.contour_point_index == next_contour_point_index` を不変条件とする。
- nested `start.point.point_index` と `end.point.point_index` は glyph absolute point index のままとする。
- 処理順序は `contour span lookup -> validate edge_index -> compute next_contour_point_index -> decode start contour point -> decode end contour point` とし、edge index validation を endpoint decode より先に行う。
- `edge_index < 0` または `edge_index >= span.point_count` は `MissingGlyphOutline` とする。
- `edge_index + 1 == span.point_count` の場合、`next_contour_point_index = 0` として contour end から contour start へ wrap する。
- `span.point_count == 1` の場合、`edge_index = 0` だけを成功させ、start と end が同じ point を参照する topology self-wrap とする。
- `gui_sfnt_glyf_simple_contour_edge_with_tables` は public wrapper ではなく `gui_sfnt_glyf_simple_contour_span_with_tables` と `gui_sfnt_glyf_simple_contour_point_with_tables` を通る。
- Source policy で contour edge API、internal table helper reuse、edge-before-endpoint validation、wrap formula、metadata 非依存、no Vec allocation を固定する。

完了条件:

- two-contour fixture の contour 0 edge 0 が start local 0 / absolute 0、end local 1 / absolute 1、wrap なしを返す。
- two-contour fixture の contour 1 last edge が start local 1 / absolute 3、next local 0、end absolute 2 を返す。
- one-point contour fixture の edge 0 が next local 0、start/end absolute point equal の self-wrap を返す。
- signed coordinate fixture の edge 1 が start absolute point 1、x 2、y -6 を返す。
- edge index `-1` と `span.point_count` は `MissingGlyphOutline` になる。
- coordinate overrun fixture を contour edge lookup 経由でも `MalformedGlyfRecord` として扱える。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4l: sfnt simple glyph curve segment classification

目的:

- F4k の contour topology edge から、line / quadratic / no-segment を enum payload として 1 つだけ分類する。
- TrueType simple glyph の implied on-curve midpoint を exact に表すため、coordinate は font unit の 2 倍である `x2` / `y2` として保持する。
- full segment `Vec` / full outline `Vec` / streaming contour sink / rasterizer は作らず、allocation なしの classifier boundary を提供する。
- valid topology だが現在 edge start から drawable segment を出さない状態を `NoSegment` の成功値として返し、parse error と混同しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphCurveNoSegmentReason`
  - `GuiSfntSimpleGlyphCurveNoSegment`
  - `GuiSfntSimpleGlyphLineSegment`
  - `GuiSfntSimpleGlyphQuadraticSegment`
  - `GuiSfntSimpleGlyphCurveSegment`
  - `gui_sfnt_classify_simple_glyph_curve_segment`
  - `gui_sfnt_lookup_simple_glyph_curve_segment`
- `GuiSfntSimpleGlyphCurveSegment` は `NoSegment` / `Line` / `Quadratic` の payload 付き enum とし、inactive field を持つ shared struct にはしない。
- `Line` は edge.start / edge.end が両方 on-curve の場合だけ返す。
- `Quadratic` は edge.start が on-curve、edge.end が off-curve の場合だけ返す。edge.end は control point とする。
- quadratic end が explicit on-curve の場合、`end_x2 = lookahead.x * 2`、`end_y2 = lookahead.y * 2` とする。
- quadratic end が implied midpoint の場合、`end_x2 = control.x + lookahead.x`、`end_y2 = control.y + lookahead.y` とする。`div_s ... 2` や丸めは使わない。
- `span.point_count == 1` は `NoSegment SinglePointContour` の成功値とする。
- edge.start が off-curve の場合は `NoSegment OffCurveStart` の成功値とする。F4l は implied contour start を合成しない。
- pure classifier で off-curve end に `lookahead = None` が渡された場合は `NoSegment MissingLookahead` とし、byte lookup 側ではこの状態を出さないように必要な時だけ lookahead を読む。
- `gui_sfnt_glyf_simple_curve_segment_with_tables` は public wrapper ではなく `gui_sfnt_glyf_simple_contour_edge_with_tables` と `gui_sfnt_glyf_simple_contour_point_with_tables` を通る。
- Source policy で curve segment API、payload enum、doubled coordinate field、no integer midpoint division、conditional lookahead decode、internal helper reuse、metadata 非依存、no curve segment `Vec` allocation を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_curve.n.md` を追加し、巨大化した `tests/stdlib/gui_font_sfnt_glyf.n.md` とは別に分類規則の doctest を保持する。
- `tests/stdlib/gui_font_sfnt_glyf_curve_lookup.n.md` を追加し、public `gui_sfnt_lookup_simple_glyph_curve_segment` が最小 SFNT byte fixture から odd implied midpoint の `Quadratic` へ到達する smoke を保持する。
- 現時点の compiler では `alloc/gui/font/sfnt/glyf` import の resource static check が 60 秒制限に近いため、public lookup smoke は `skip` 付きの仕様化 doctest とし、source policy で fixture、public lookup 呼び出し、`ByteBuilder` binary construction、`io_bytebuf_from_str_result` 禁止を固定する。

完了条件:

- on-curve -> on-curve edge が `Line` になり、start/end doubled coordinate を返す。
- on-curve -> off-curve -> on-curve が `Quadratic` になり、control doubled coordinate と explicit end doubled coordinate を返す。
- on-curve -> off-curve -> off-curve が `Quadratic` になり、`end_is_implied = true`、odd midpoint を `end_x2` / `end_y2` で丸めず返す。
- 1 point contour が `Result::Ok (NoSegment SinglePointContour)` 相当の typed success になる。
- off-curve start が `NoSegment OffCurveStart` の typed success になる。
- `edge_index` 範囲外や malformed bytes は引き続き `Result::Err GuiSfntParseError` になる。
- classifier helper と byte lookup helper は full outline allocation、`Vec GuiSfntSimpleGlyphCurveSegment`、rasterizer、platform API、fallback rendering path を使わない。
- public lookup smoke は UTF-8 text conversion ではなく `ByteBuilder` で binary SFNT bytes を組み立てる。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
# skip policy check: current compiler exceeds the normal 60s timeout for this byte-level public lookup smoke.
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve_lookup.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve_lookup.json -j 1
# executable smoke check until the resource static check is made faster:
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve_lookup.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve_lookup_long.json -j 1; Remove-Item Env:NEPL_TEST_CASE_TIMEOUT_MS
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf.n.md --no-tree -o tmp_gui_font_sfnt_glyf.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4m: sfnt simple glyph path command projection

目的:

- F4l の `GuiSfntSimpleGlyphCurveSegment` を、後続の outline / path sink が読む明示的な move command / draw command へ写す。
- full outline `Vec` / streaming sink trait / winding / fill rule / rasterizer / render2d command はまだ作らない。
- `NoSegment` を parse error や silent no-op にせず、`SkipNoSegment` command として明示的に保持する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathMoveTo`
  - `GuiSfntSimpleGlyphPathLineTo`
  - `GuiSfntSimpleGlyphPathQuadraticTo`
  - `GuiSfntSimpleGlyphPathSkipNoSegment`
  - `GuiSfntSimpleGlyphPathCommand`
  - `gui_sfnt_simple_glyph_curve_segment_move_to_command`
  - `gui_sfnt_simple_glyph_curve_segment_draw_command`
- `GuiSfntSimpleGlyphPathCommand` は `MoveTo` / `LineTo` / `QuadraticTo` / `SkipNoSegment` の payload 付き enum とし、inactive field を持つ shared struct にはしない。
- Path command payload は full edge / line / quadratic / no-segment value を再保持せず、source contour/edge index、doubled coordinate、no-segment reason の小さな値へ射影する。
- `Line` は `move_to_command` で `MoveTo`、`draw_command` で `LineTo` を返す。
- `Quadratic` は `move_to_command` で `MoveTo`、`draw_command` で `QuadraticTo` を返す。
- `NoSegment` はどちらの関数でも `SkipNoSegment` を返す。
- command index を受け取らず、`Option` / `Result` も返さない。
- `MoveTo`、`LineTo`、`QuadraticTo` は F4l の doubled coordinate をそのまま使い、integer midpoint division や coordinate fallback を行わない。
- Source policy で path command API、payload enum、no command index / no `Option` / no `Result` contract、`SkipNoSegment`、no `Vec GuiSfntSimpleGlyphPathCommand` allocation、no metadata parse、no render2d/backend/platform import、no rasterizer を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` を追加し、typed value から line / quadratic / no-segment projection を検査する。

完了条件:

- `Line` segment が `MoveTo` と `LineTo` を明示的な関数で返す。
- `Quadratic` segment が control / end doubled coordinate と `end_is_implied` を保持する `QuadraticTo` を返す。
- `NoSegment` が `SkipNoSegment` と reason を保持する。
- path command projection は full outline allocation、`Vec GuiSfntSimpleGlyphPathCommand`、rasterizer、platform API、render2d command、fallback rendering path を使わない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4n: sfnt simple glyph path command public lookup

目的:

- SFNT byte input から contour-local edge の move / draw path command を public API として取得できるようにする。
- F4l の byte-backed curve segment lookup と F4m の path command projection を合成するだけに限定する。
- full outline `Vec` / command list / sink trait / winding / fill rule / rasterizer / render2d command はまだ作らない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_lookup_simple_glyph_move_to_command`
  - `gui_sfnt_lookup_simple_glyph_draw_command`
- 両関数は `gui_sfnt_lookup_simple_glyph_curve_segment` を呼び、`Result::Err` は同じ `GuiSfntParseError` として伝播する。
- `Result::Ok segment` の場合、move helper は `gui_sfnt_simple_glyph_curve_segment_move_to_command`、draw helper は `gui_sfnt_simple_glyph_curve_segment_draw_command` を呼び、`Result::Ok GuiSfntSimpleGlyphPathCommand` を返す。
- F4n では `gui_sfnt_parse_metadata`、`*_with_tables` helper、point / contour table helper、curve classification logic を直接呼ばない。
- `NoSegment` は `Result::Ok SkipNoSegment` として保持し、`Result::Err`、`Option::None`、empty command、silent no-op、fallback rendering path にしない。
- Source policy で public signatures、F4l/F4m composition、no metadata unwrap / no table-helper bypass / no `Vec GuiSfntSimpleGlyphPathCommand` / no render2d/backend/platform import を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に `NoSegment -> move_to_command -> SkipNoSegment` の cheap typed doctest assertion を追加する。

完了条件:

- move lookup と draw lookup が `Result GuiSfntSimpleGlyphPathCommand GuiSfntParseError` を返す。
- move lookup は byte-backed curve segment lookup の成功値を F4m move projection に渡す。
- draw lookup は byte-backed curve segment lookup の成功値を F4m draw projection に渡す。
- F4n は full outline allocation、command list、rasterizer、platform API、render2d command、metadata unwrap bypass を使わない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4o: sfnt simple glyph path command pair lookup

目的:

- 同じ contour-local edge の move command と draw command を 1 つの pair value として取得できるようにする。
- F4n の move lookup と draw lookup を別々に呼ぶことで同じ SFNT edge decode が 2 回走る問題を避ける。
- contour stream、command sequence、full outline `Vec`、sink trait、winding、fill rule、rasterizer、render2d command はまだ作らない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathCommandPair`
  - `gui_sfnt_simple_glyph_path_command_pair`
  - `gui_sfnt_simple_glyph_path_command_pair_move_command`
  - `gui_sfnt_simple_glyph_path_command_pair_draw_command`
  - `gui_sfnt_simple_glyph_curve_segment_path_command_pair`
  - `gui_sfnt_lookup_simple_glyph_path_command_pair`
- `GuiSfntSimpleGlyphPathCommandPair` は ordered list ではなく、`move_command` と `draw_command` だけを持つ O(1) value とする。
- pure helper は F4m の `move_to_command` と `draw_command` を同じ segment に適用して pair を返す。
- byte-backed helper は `gui_sfnt_lookup_simple_glyph_curve_segment` を 1 回だけ呼び、`Result::Err` は同じ `GuiSfntParseError` として伝播する。
- `Result::Ok segment` の場合、`gui_sfnt_simple_glyph_curve_segment_path_command_pair` を呼び、`Result::Ok GuiSfntSimpleGlyphPathCommandPair` を返す。
- `NoSegment` は pair 内の move / draw の両方で `SkipNoSegment` の成功値として保持する。
- F4o では command index、count、next、current point state、`Vec GuiSfntSimpleGlyphPathCommand`、`push` を導入しない。
- F4o public helper では `gui_sfnt_parse_metadata`、`*_with_tables` helper、lower public lookup helper、curve classifier、render2d/backend/platform、rasterizer、host text API を使わない。
- Source policy で pair API、curve lookup 1 回、pair helper composition、no list / no sink / no metadata unwrap / no table-helper bypass を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に line pair、implied quadratic pair、NoSegment pair の typed doctest assertion を追加する。

完了条件:

- line segment pair が `MoveTo` と `LineTo` を保持する。
- implied quadratic segment pair が `MoveTo` と `QuadraticTo` を保持し、doubled coordinate と `end_is_implied` を落とさない。
- NoSegment pair が move / draw の両方で `SkipNoSegment` と reason を保持する。
- byte-backed public lookup が curve segment lookup を 1 回だけ呼ぶ thin composition になっている。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4p: sfnt simple glyph path sink event adapter

目的:

- F4o の `GuiSfntSimpleGlyphPathCommandPair` を、後続の contour/path sink が読む single-edge event pair へ写す。
- full contour stream、command sequence、sink trait、ownership / allocation boundary、winding、fill rule、rasterizer、render2d command はまだ作らない。
- `SkipNoSegment` を empty event にせず、既存の typed path command を event として保持する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkEvent`
  - `GuiSfntSimpleGlyphPathSinkEventPair`
  - `gui_sfnt_simple_glyph_path_command_sink_event`
  - `gui_sfnt_simple_glyph_path_sink_event_command`
  - `gui_sfnt_simple_glyph_path_sink_event_pair`
  - `gui_sfnt_simple_glyph_path_sink_event_pair_first_event`
  - `gui_sfnt_simple_glyph_path_sink_event_pair_second_event`
  - `gui_sfnt_simple_glyph_path_command_pair_sink_event_pair`
- `GuiSfntSimpleGlyphPathSinkEvent` は `Command GuiSfntSimpleGlyphPathCommand` の thin wrapper とし、`MoveTo` / `LineTo` / `QuadraticTo` / `SkipNoSegment` payload を再定義しない。
- `GuiSfntSimpleGlyphPathSinkEventPair` は `first_event` と `second_event` だけを持つ O(1) value とする。
- pure helper は `gui_sfnt_simple_glyph_path_command_pair_move_command` と `gui_sfnt_simple_glyph_path_command_pair_draw_command` だけを読み、first / second event を作る。
- F4p では `Option` / `Result`、command index、count、next、current point state、contour closure、off-curve contour-start synthesis、`Vec GuiSfntSimpleGlyphPathSinkEvent`、`push` を導入しない。
- F4p の pure helper では byte-backed lookup、metadata parser、`*_with_tables` helper、lower point / contour helper、curve classifier、render2d/backend/platform、rasterizer、host text API を使わない。
- Source policy で pair-to-sink-event adapter、thin wrapper、event pair accessors、no duplicate payload enum、no lookup/parser/helper bypass、no allocation/stream state を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に direct path command から sink event / event pair を作る cheap typed doctest assertion を追加する。line / quadratic / NoSegment の payload behavior は既存 F4m/F4o doctest と F4p source policy で固定し、既存の重い executable case へ nested event match は足さない。

完了条件:

- direct `MoveTo` / `LineTo` command pair が first event `MoveTo`、second event `LineTo` として読める。
- direct `SkipNoSegment` command が `GuiSfntSimpleGlyphPathSinkEvent::Command` の内側で `SkipNoSegment` として読める。
- implied quadratic pair と NoSegment pair の payload preservation は F4m/F4o の executable doctest と F4p source policy で固定される。
- pure adapter が lookup / parser / table helper / renderer / platform API に依存しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4q: sfnt simple glyph path sink event kind classification

目的:

- F4p の `GuiSfntSimpleGlyphPathSinkEvent` を、後続 sink の dispatch 用 kind へ写す。
- kind は path command payload の軽量版ではなく、座標や contour/edge の authority は既存 event command payload に残す。
- real sink trait、ownership / allocation boundary、contour traversal、winding、fill、rasterizer、render2d command はまだ作らない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkEventKind`
  - `GuiSfntSimpleGlyphPathSinkEventKindPair`
  - `gui_sfnt_simple_glyph_path_sink_event_kind`
  - `gui_sfnt_simple_glyph_path_sink_event_kind_pair`
  - `gui_sfnt_simple_glyph_path_sink_event_kind_pair_first_kind`
  - `gui_sfnt_simple_glyph_path_sink_event_kind_pair_second_kind`
  - `gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair`
- `GuiSfntSimpleGlyphPathSinkEventKind` は `MoveTo`、`LineTo`、`QuadraticTo`、`SkipNoSegment GuiSfntSimpleGlyphCurveNoSegmentReason` だけを持つ。
- `SkipNoSegment` kind の reason は diagnostics / skip counting / branch selection 用であり、source contour / edge 復元用ではない。
- kind には `contour_index`、`edge_index`、`x2`、`y2`、`control_x2`、`end_x2` などを入れない。
- `gui_sfnt_simple_glyph_path_sink_event_kind` は `gui_sfnt_simple_glyph_path_sink_event_command` で command を読み、全 variant を明示的に `match` する。catch-all arm は使わない。
- `gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair` は F4p event pair accessors と `gui_sfnt_simple_glyph_path_sink_event_kind` だけを使う。
- F4q では `Option` / `Result`、`Vec GuiSfntSimpleGlyphPathSinkEventKind`、`push`、command index、count、next、current point state、contour closure、off-curve contour-start synthesis、byte-backed lookup、metadata parser、`*_with_tables` helper、lower point / contour helper、curve classifier、render2d/backend/platform、rasterizer、host text API を使わない。
- Source policy で kind の dispatch 専用性、no duplicate payload、no coordinate/source index fields、no allocation/stream state、no lookup/parser/helper bypass を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の direct sink event doctest に、`MoveTo` / `LineTo` kind pair と `SkipNoSegment` reason kind を確認する cheap typed assertion を追加する。

完了条件:

- direct `MoveTo` event が `GuiSfntSimpleGlyphPathSinkEventKind::MoveTo` として分類される。
- direct `LineTo` event が `GuiSfntSimpleGlyphPathSinkEventKind::LineTo` として分類される。
- direct `SkipNoSegment` event が reason を保持した `GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment` として分類される。
- kind helper が lookup / parser / table helper / renderer / platform API に依存しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4r: sfnt simple glyph path sink event indexed selection

目的:

- F4p/F4q の two-slot pair から、後続 sink が first / second event または kind を O(1) に選択できる typed boundary を追加する。
- numeric index ではなく enum slot を使い、不正 event index を型として表現不能にする。
- contour traversal、iterator、command count、current point state、rasterizer、render2d command はまだ作らない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkEventSlot`
  - `gui_sfnt_simple_glyph_path_sink_event_pair_event_at`
  - `gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at`
  - `gui_sfnt_simple_glyph_path_sink_event_pair_kind_at`
- `GuiSfntSimpleGlyphPathSinkEventSlot` は `First` と `Second` だけを持ち、`Clone` / `Copy` を実装する。
- `event_at` は slot を明示 `match` し、`First` なら `gui_sfnt_simple_glyph_path_sink_event_pair_first_event`、`Second` なら `gui_sfnt_simple_glyph_path_sink_event_pair_second_event` だけを使う。catch-all arm は使わない。
- `kind_pair_kind_at` は slot を明示 `match` し、kind pair の first / second accessor だけを使う。catch-all arm は使わない。
- `event_pair_kind_at` は `gui_sfnt_simple_glyph_path_sink_event_pair_event_at` と `gui_sfnt_simple_glyph_path_sink_event_kind` の合成だけで実装する。kind classification logic を重複させない。
- F4r では `i32` event index、`Option` / `Result`、`Vec`、`push`、command index、count、next、current point state、contour traversal、contour closure、off-curve contour-start synthesis、byte-backed lookup、metadata parser、`*_with_tables` helper、curve classifier、render2d/backend/platform、rasterizer、host text API を使わない。
- Source policy で slot enum、no numeric index、total selection、event/kind accessor composition、no allocation/stream state、no lookup/parser/helper bypass を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の direct sink event doctest に、`First` / `Second` slot で event と kind を取得できる cheap typed assertion を追加する。

完了条件:

- `First` slot が first event / first kind を返す。
- `Second` slot が second event / second kind を返す。
- event pair から single slot kind を読む helper が event selection と F4q kind helper の合成だけで動く。
- F4r は numeric index、full outline allocation、stream state、rasterizer、platform API、metadata unwrap bypass を使わない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4s: sfnt simple glyph path contour traversal step

目的:

- F4r の typed slot selection を、1 contour 内の 1 event step traversal に接続する。
- cursor / next / step を enum と struct で表し、`Option` や numeric index で終端や slot を表さない。
- public lookup は range / parse error を `Result` で返し、contour end は成功値 `GuiSfntSimpleGlyphPathContourNext::EndContour` として返す。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathContourCursor`
  - `GuiSfntSimpleGlyphPathContourNext`
  - `GuiSfntSimpleGlyphPathContourStep`
  - cursor / step constructor と accessor
  - private `gui_sfnt_simple_glyph_path_contour_next_from_cursor`
  - public `gui_sfnt_lookup_simple_glyph_path_contour_step`
- `GuiSfntSimpleGlyphPathContourCursor` / `GuiSfntSimpleGlyphPathContourNext` / `GuiSfntSimpleGlyphPathContourStep` は `Clone` / `Copy` を実装する。
- private next helper は、public lookup が `span_point_count > 0` と `0 <= edge_index < span_point_count` を検証した後だけ呼ぶ。public total helper にしない。
- next helper は slot を明示 `match` し、`First` なら same edge `Second`、`Second` なら `edge + 1` の `First` または `EndContour` を返す。catch-all arm は使わない。
- public lookup は `gui_sfnt_lookup_simple_glyph_contour_span` で contour span / point count を検証し、`gui_sfnt_lookup_simple_glyph_path_command_pair` で edge を path command pair に変換する。
- public lookup は `gui_sfnt_simple_glyph_path_command_pair_sink_event_pair`、`gui_sfnt_simple_glyph_path_sink_event_pair_event_at`、`gui_sfnt_simple_glyph_path_sink_event_kind`、private next helper を合成する。
- F4s は `Vec`、`push`、command list、full outline allocation、rasterizer、render2d/backend/platform、font fallback、metadata unwrap bypass を使わない。
- off-curve contour-start synthesis と contour closure insertion は F4s では行わず、既存 `SkipNoSegment OffCurveStart` を typed event として保持する。
- Source policy で cursor / next / step 型、Clone / Copy、private next helper、public lookup composition、no fallback/no allocation/no renderer/no platform を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に constructor/accessor の cheap typed assertion と、最小 SFNT fixture を使う public `gui_sfnt_lookup_simple_glyph_path_contour_step` doctest を追加する。
- public lookup doctest は `First -> Second`、`Second -> next edge First`、final `Second -> EndContour`、out-of-range edge の `GuiSfntParseErrorKind::MissingGlyphOutline` を直接検査する。
- 現行 doctest runner では public glyf lookup fixture の compile が 60 秒制限を超えるため、public lookup fixture は `skip` とし、`nodesrc/test_web_gui_font_rendering_contract.js` で doctest 名、public call、typed error branch の存在を固定する。

完了条件:

- cursor は glyph / contour / edge / slot を保持し、accessor で読める。
- step は cursor / event / kind / next を保持し、accessor で読める。
- `First` は同じ edge の `Second` に進む。
- final ではない `Second` は次 edge の `First` に進む。
- final edge の `Second` は `EndContour` を返す。
- public lookup は parse/range 不正だけ `Result::Err` にし、contour end は `Result::Ok step` の `EndContour` として返す。
- F4s は full outline allocation、renderer、platform API、font fallback、off-curve contour-start synthesis を導入しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4t: sfnt simple glyph allocation-free path sink ownership boundary

目的:

- F4s の `GuiSfntSimpleGlyphPathContourStep` を、real sink trait へ進む前の allocation-free sink decision に写す。
- off-curve contour-start synthesis と contour closure insertion を、別々の typed policy として分離する。
- policy reject を `GuiSfntParseError` に混ぜず、success payload 内の enum decision として保持する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathOffCurveStartPolicy`
  - `GuiSfntSimpleGlyphPathClosurePolicy`
  - `GuiSfntSimpleGlyphPathSinkPolicy`
  - `GuiSfntSimpleGlyphPathSinkRejectReason`
  - `GuiSfntSimpleGlyphPathSinkPrimaryAction`
  - `GuiSfntSimpleGlyphPathContourClose`
  - `GuiSfntSimpleGlyphPathSinkTailAction`
  - `GuiSfntSimpleGlyphPathSinkStep`
  - constructor / accessor
  - `gui_sfnt_simple_glyph_path_sink_primary_action_from_contour_step`
  - `gui_sfnt_simple_glyph_path_sink_tail_action_from_contour_step`
  - `gui_sfnt_simple_glyph_path_sink_step_from_contour_step`
  - public `gui_sfnt_lookup_simple_glyph_path_sink_step`
- `GuiSfntSimpleGlyphPathOffCurveStartPolicy` は `KeepTypedSkip` / `RejectUnsupported` を持つ。
- `GuiSfntSimpleGlyphPathClosurePolicy` は `KeepOpen` / `EmitCloseAfterFinalEvent` を持つ。
- `RejectUnsupported` は `SkipNoSegment OffCurveStart` だけを `Reject UnsupportedOffCurveStart` に写す。`SinglePointContour` と `MissingLookahead` は emit する。
- `GuiSfntSimpleGlyphPathSinkPrimaryAction` は `EmitEvent` / `Reject` を持ち、reject reason は dedicated enum にする。
- `GuiSfntSimpleGlyphPathSinkTailAction` は `NoTailAction` / `CloseContour` を持つ。
- tail action は次の規則にする。
  - `Reject` なら常に `NoTailAction`
  - `Continue` なら常に `NoTailAction`
  - `EndContour` かつ `KeepOpen` なら `NoTailAction`
  - `EndContour` かつ `EmitCloseAfterFinalEvent` かつ primary が emit なら `CloseContour`
- `CloseContour` は source cursor の glyph / contour index だけを持つ marker とし、renderer command にはしない。
- byte-backed public helper は `gui_sfnt_lookup_simple_glyph_path_contour_step` を呼び、成功値を pure sink-step helper に渡すだけにする。
- F4t は `Vec`、`push`、command list、full outline allocation、rasterizer、render2d/backend/platform、font fallback、metadata unwrap bypass を使わない。
- Source policy で F4t の type set、reject/close 排他、OffCurveStart 限定、EndContour 限定 close、F4s lookup 委譲を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に cheap typed assertion を追加する。
  - keep policy は off-curve skip を emit し、final step だけ close marker を出す。
  - reject policy は off-curve start を reject にし、final step でも close marker を出さない。
  - `Continue` step は close marker を出さない。
  - `RejectUnsupported` でも `SinglePointContour` は emit される。
- F4s の skipped public lookup fixture に `gui_sfnt_lookup_simple_glyph_path_sink_step` の call を含め、source policy で byte-backed helper の存在を固定する。

完了条件:

- policy、primary action、tail action、sink step はすべて enum/struct payload として表現される。
- policy reject は `Result::Err` ではなく `GuiSfntSimpleGlyphPathSinkPrimaryAction::Reject` になる。
- reject と close contour は同時に発生しない。
- close contour は primary が emit で、かつ `step.next = EndContour` の場合だけ発生し得る。
- off-curve policy は `OffCurveStart` だけに作用する。
- F4t は full outline allocation、renderer、platform API、font fallback、off-curve start synthesis を導入しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_curve.n.md --no-tree -o tmp_gui_font_sfnt_glyf_curve.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4u: sfnt simple glyph path sink action selection projection

目的:

- F4t の `GuiSfntSimpleGlyphPathSinkStep` から、future sink が順に処理する action を enum slot で選べるようにする。
- `Primary` / `Tail` の action 選択を、F4r/F4s の `First` / `Second` event slot から明確に分離する。
- `NoTailAction` を明示的な `NoAction` に写し、fallback や silent no-op とは別の typed state として扱う。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionSlot`
  - `GuiSfntSimpleGlyphPathSinkAction`
  - action slot の `Clone` / `Copy`
  - action の `Clone` / `Copy`
  - `gui_sfnt_simple_glyph_path_sink_action_slot_is_primary`
  - `gui_sfnt_simple_glyph_path_sink_action_slot_is_tail`
  - `gui_sfnt_simple_glyph_path_sink_primary_action_as_action`
  - `gui_sfnt_simple_glyph_path_sink_tail_action_as_action`
  - `gui_sfnt_simple_glyph_path_sink_step_action_at`
  - public `gui_sfnt_lookup_simple_glyph_path_sink_action`
- `GuiSfntSimpleGlyphPathSinkActionSlot` は `Primary` / `Tail` だけを持つ。
- `GuiSfntSimpleGlyphPathSinkAction` は `EmitEvent` / `Reject` / `CloseContour` / `NoAction` を持つ。
- primary action projection は `EmitEvent` / `Reject` だけを返し、`NoAction` を返さない。
- tail action projection は `NoTailAction -> NoAction`、`CloseContour -> CloseContour` だけを行う。
- `gui_sfnt_simple_glyph_path_sink_step_action_at` は slot の網羅的 `match` で `Primary` または `Tail` を選ぶ。
- byte-backed public helper は `gui_sfnt_lookup_simple_glyph_path_sink_step` を 1 回だけ呼び、成功値に pure action projection を適用する。
- F4u は `Vec`、`push`、numeric action index、command list、full outline allocation、rasterizer、render2d/backend/platform、font fallback、metadata unwrap bypass、`*_with_tables` bypass を使わない。
- Source policy で F4u の type set、slot 軸の分離、primary が `NoAction` を返さないこと、tail の `NoAction` 限定、F4t lookup への 1 回委譲を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の cheap typed assertion を拡張する。
  - `Primary` slot は primary action、`Tail` slot は tail action を選ぶ。
  - `EmitEvent` / `Reject` / `CloseContour` / `NoAction` が明示的に区別される。
  - `NoAction` は tail の `NoTailAction` だけから得られる。

完了条件:

- sink action selection は enum / match で表現され、数値 index や fallback branch を持たない。
- `GuiSfntSimpleGlyphPathSinkActionSlot` は `GuiSfntSimpleGlyphPathSinkEventSlot` と混同されない。
- primary action projection は `NoAction` を返さない。
- policy reject は `Result::Err` ではなく `GuiSfntSimpleGlyphPathSinkAction::Reject` として保持される。
- byte-backed helper は F4t lookup にだけ委譲し、下位 glyph/contour/curve helper を直接呼ばない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4v: sfnt simple glyph path sink action traversal step

目的:

- F4u の single action projection を、contour 内で順に読める typed traversal step へ拡張する。
- future sink が `Primary -> Tail -> F4s source next` の順に action を読むための cursor / next / step を追加する。
- real sink、callback、`Vec` command stream、full outline allocation、renderer、rasterizer、platform API はまだ導入しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionCursor`
  - `GuiSfntSimpleGlyphPathSinkActionNext`
  - `GuiSfntSimpleGlyphPathSinkActionStep`
  - constructor / accessor
  - `Clone` / `Copy`
  - `gui_sfnt_simple_glyph_path_sink_action_next_from_step`
  - `gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step`
  - public `gui_sfnt_lookup_simple_glyph_path_sink_action_step`
- `GuiSfntSimpleGlyphPathSinkActionCursor` は checked `GuiSfntSimpleGlyphPathContourCursor` と `GuiSfntSimpleGlyphPathSinkActionSlot` を持つ。
- 新しい numeric action index、command index、loop index、count field、ad-hoc traversal counter は追加しない。既存 contour cursor 内の `contour_index` / `edge_index` は F4s の authority として保持する。
- `GuiSfntSimpleGlyphPathSinkActionNext` は `Continue` / `EndContour` を持つ。contour 終端を `Option::None` や error で表さない。
- next の規則は次とする。
  - `Primary` は action payload に関係なく同じ contour cursor の `Tail` へ進む。
  - `Tail` は action payload に関係なく `sink_step.source_step.next` に従う。
  - `source_step.next = Continue next_cursor` なら `next_cursor Primary` へ進む。
  - `source_step.next = EndContour` なら `EndContour` を返す。
- `gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step` は F4u の `gui_sfnt_simple_glyph_path_sink_step_action_at` を使い、primary / tail action の中身を再分類しない。
- byte-backed public helper は `gui_sfnt_lookup_simple_glyph_path_sink_step` を 1 回だけ呼び、成功値を pure action-step helper に渡すだけにする。
- F4v は `Vec`、`push`、numeric action index、command list、full outline allocation、rasterizer、render2d/backend/platform、font fallback、metadata unwrap bypass、`*_with_tables` bypass を使わない。
- Source policy で F4v の type set、payload-independent traversal、F4u action projection reuse、F4t lookup への 1 回委譲、下位 glyph/contour/curve helper へ直接入らないことを固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の cheap typed assertion を拡張する。
  - Primary は emit / reject に関係なく same contour cursor Tail へ進む。
  - Tail は `Continue next_cursor` の場合に next cursor Primary へ進む。
  - Tail は `EndContour` の場合に `EndContour` へ進む。
  - Tail の `NoAction` は traversal stop ではなく、F4s source next に従う。

完了条件:

- traversal state は enum / struct payload として表現され、numeric action index を持たない。
- action payload と next state は分離される。
- next は action payload を見ず、action slot と F4s source step next だけから決まる。
- byte-backed helper は F4t lookup にだけ委譲し、下位 helper を直接呼ばない。
- F4v は full outline allocation、renderer、platform API、font fallback、off-curve start synthesis を導入しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4w: sfnt simple glyph path sink action start cursor

目的:

- F4v の action traversal に、contour-local action stream の開始 cursor を追加する。
- 開始 cursor を `edge 0` / `First` / `Primary` として型で固定する。
- pure constructor と byte-backed validated entry point を分け、unchecked value construction と byte validation を混同しない。
- action payload lookup、sink policy、full outline allocation、renderer、rasterizer、platform API は導入しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_simple_glyph_path_sink_action_start_cursor`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor`
- pure helper は `gui_sfnt_simple_glyph_path_contour_cursor glyph contour_index 0 GuiSfntSimpleGlyphPathSinkEventSlot::First` を作り、`GuiSfntSimpleGlyphPathSinkActionSlot::Primary` と合成する。
- pure helper は unchecked value constructor であり、byte 妥当性、contour 存在、span 範囲、point count を検証しない。
- byte-backed helper は `gui_sfnt_lookup_simple_glyph_contour_span` を 1 回だけ呼び、成功した場合にだけ pure helper へ委譲する。
- byte-backed helper は F4v action-step lookup、F4t sink-step lookup、F4s contour-step lookup、point / curve / path-command helper、sink policy、renderer、rasterizer、platform font API を呼ばない。
- Source policy で F4w の doc contract、pure helper の `edge 0` / `First` / `Primary`、byte-backed helper の contour span lookup への 1 回委譲、追加 NEPL body に括弧がないことを固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の cheap typed assertion を拡張する。
  - `gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph 3` が contour `3`、edge `0`、event slot `First`、action slot `Primary` を返すことを確認する。

完了条件:

- 開始 cursor は enum slot と既存 contour cursor で表現され、numeric action index や command index を持たない。
- pure constructor は byte validation を行わないことが doc と実装で明示される。
- byte-backed helper は contour span validation にだけ委譲し、action payload や policy を読まない。
- hidden fallback、silent no-op、renderer/backend/platform dependency を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4x: sfnt simple glyph path sink action start step

目的:

- F4w の start cursor と F4v の action step lookup を接続し、contour の first action step を読む public helper を追加する。
- F4x 自体は新しい validation authority にならず、既存 action step lookup の Result 境界を再利用する。
- contour span 検証の二重実行を避けるため、byte-backed start cursor helper は呼ばない。
- real sink、full outline allocation、command list、renderer、rasterizer、platform API は導入しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_lookup_simple_glyph_path_sink_action_start_step` を追加する。
- helper は `gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index` を 1 回呼ぶ。
- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index start_cursor policy` を 1 回呼ぶ。
- helper は `Result::Err error` / `Result::Ok action_step` を明示的に `match` し、新しい判断や error 変換を行わない。
- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor`、`gui_sfnt_lookup_simple_glyph_contour_span`、`gui_sfnt_lookup_simple_glyph_path_sink_step`、F4s/F4t より下位の lookup を直接呼ばない。
- Source policy で F4x の doc contract、pure start cursor 1 回、action step lookup 1 回、禁止 helper、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の skipped byte-backed fixture に呼び出しを追加する。
  - `Result::Ok action_step` から cursor を読み、contour `0`、edge `0`、event slot `First`、action slot `Primary` を確認する。
  - `Result::Err` は false とし、typed Result branch を明示する。

完了条件:

- start step helper は `start cursor construction + existing checked action step lookup` だけに閉じる。
- parse/range error は `Result::Err` として伝播し、policy reject は `Result::Ok` action payload に残る。
- byte-backed start cursor helper と contour span lookup を直接呼ばず、検証の二重化を避ける。
- hidden fallback、silent no-op、renderer/backend/platform dependency を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4y: sfnt simple glyph path sink action step advance

目的:

- F4v の `GuiSfntSimpleGlyphPathSinkActionStep.next` を 1 段だけ進める byte-backed helper を追加する。
- `Continue cursor` は checked action step lookup で次 step に解決し、`EndContour` は成功値として返す。
- contour 終端を `Option::None` や `Result::Err` で表さない。
- loop traversal、real sink、full outline allocation、command list、renderer、rasterizer、platform API は導入しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionStepAdvance`
  - `Clone` / `Copy`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance`
- `GuiSfntSimpleGlyphPathSinkActionStepAdvance` は `Continue GuiSfntSimpleGlyphPathSinkActionStep` / `EndContour` を持つ。
- helper は `gui_sfnt_simple_glyph_path_sink_action_step_next step` を読み、`match` する。
- `Continue cursor` の場合だけ `gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index cursor policy` を 1 回呼ぶ。
- `Result::Err error` はそのまま伝播し、`Result::Ok next_step` は `GuiSfntSimpleGlyphPathSinkActionStepAdvance::Continue next_step` に包む。
- `EndContour` は `Result::Ok GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour` として返す。
- helper は action payload を見ない。`GuiSfntSimpleGlyphPathSinkAction::Reject`、`NoAction`、`CloseContour` などで traversal を変えない。
- helper は start cursor/start step helper、sink action lookup、sink step lookup、contour step lookup、F4s/F4t より下位の lookup、metadata parser、`*_with_tables`、`Vec`、`push`、renderer、rasterizer、platform API を直接呼ばない。
- Source policy で F4y enum、Clone/Copy、helper body、下位 lookup 1 回、禁止 helper、payload inspection 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` を拡張する。
  - cheap assertion で `GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour` が成功 terminal enum として `match` できることを確認する。
  - skipped byte-backed fixture で `start_step -> advance` が `Continue next_step` を返し、next step cursor が same contour/edge/event の `Tail` であることを確認する。

完了条件:

- action step advance は `Continue next_step` / `EndContour` の typed enum で表現される。
- `Result` は byte parse/range/table error の伝播にだけ使われ、contour 終端や policy reject を error にしない。
- traversal は `step.next` だけから決まり、action payload を読まない。
- hidden fallback、silent no-op、renderer/backend/platform dependency を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4z: sfnt simple glyph path sink action step item

目的:

- F4v の `GuiSfntSimpleGlyphPathSinkActionStep` と F4y の checked advance を、後続 sink consumer が読む 1 action 分の typed item として束ねる。
- 現在 action step と次状態の lookup 結果を同時に渡せるようにしつつ、contour-wide traversal や real sink mutation には進まない。
- `EndContour` は `GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour` として item 内に残し、`Option::None` や `Result::Err` に変換しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionStepItem`
  - `Clone` / `Copy`
  - `gui_sfnt_simple_glyph_path_sink_action_step_item`
  - `gui_sfnt_simple_glyph_path_sink_action_step_item_step`
  - `gui_sfnt_simple_glyph_path_sink_action_step_item_advance`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_step_item`
- `GuiSfntSimpleGlyphPathSinkActionStepItem` は `step GuiSfntSimpleGlyphPathSinkActionStep` と `advance GuiSfntSimpleGlyphPathSinkActionStepAdvance` を持つ。
- byte-backed helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance bytes face_index step policy` を 1 回だけ呼ぶ。
- `Result::Err error` はそのまま伝播する。
- `Result::Ok advance` では `let stored_step %GuiSfntSimpleGlyphPathSinkActionStep *step` により現在 step を明示コピーし、`GuiSfntSimpleGlyphPathSinkActionStepItem` を返す。
- helper は action payload を見ない。`Reject`、`NoAction`、`CloseContour` などで traversal を変えない。
- helper は start cursor/start step helper、F4v action step lookup、sink action lookup、sink step lookup、contour step lookup、F4s/F4t より下位の lookup、metadata parser、`*_with_tables`、`Vec`、`push`、renderer、rasterizer、platform API を直接呼ばない。
- Source policy で F4z struct、Clone/Copy、constructor/accessor、helper body、F4y helper 1 回、禁止 helper、payload inspection 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` を拡張する。
  - cheap assertion で synthetic action step と `EndContour` advance から item を作り、accessor で step と terminal advance を確認する。
  - skipped byte-backed fixture で `start_step -> action_step_item` が `Continue next_step` を持ち、next step cursor が same contour/edge/event の `Tail` であることを確認する。

完了条件:

- action step item は現在 step と checked advance を value として保持する。
- item helper は F4y helper だけに委譲し、lower lookup や start composition を行わない。
- `Result` は byte parse/range/table error の伝播にだけ使われ、contour 終端や policy reject を error にしない。
- hidden fallback、silent no-op、renderer/backend/platform dependency を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4aa: sfnt simple glyph path sink action start item

目的:

- F4x の first action step helper と F4z の action step item helper を接続し、contour の first action item を読む public helper を追加する。
- F4aa 自体は新しい validation authority、new item type、contour-wide traversal、real sink mutation にはならない。
- `Result::Err` は parse/range/table error の伝播にだけ使い、policy reject や contour terminal state は F4x/F4z の typed value として残す。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_item`
- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_item:
    &ByteBuf
    Option i32
    GuiGlyphId
    i32
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStepItem GuiSfntParseError
```

- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_step bytes face_index glyph contour_index policy` を 1 回だけ呼ぶ。
- start step が `Result::Err error` ならそのまま `Result::Err error` を返す。
- start step が `Result::Ok start_step` なら `gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &start_step policy` を 1 回だけ呼ぶ。
- action step item lookup の `Result::Err error` はそのまま伝播し、`Result::Ok item` はそのまま返す。
- helper は action payload を見ない。`Reject`、`NoAction`、`CloseContour`、`EndContour` などで traversal を変えない。
- helper は start cursor helper、F4v action step lookup、F4y advance helper、sink action lookup、sink step lookup、contour step lookup、F4s/F4t より下位の lookup、metadata parser、`*_with_tables`、`Vec`、`push`、renderer、rasterizer、platform API を直接呼ばない。
- Source policy で F4aa docs、helper body、F4x helper 1 回、F4z helper 1 回、禁止 helper、payload inspection 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の skipped byte-backed fixture を拡張する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_item &bytes none glyph 0 &sink_policy` を呼ぶ。
  - item 内の stored step cursor が contour `0`、edge `0`、event slot `First`、action slot `Primary` であることを確認する。
  - advance が `Continue next_step` で、next step cursor が same contour/edge/event の `Tail` であることを確認する。

完了条件:

- start item helper は F4x と F4z を value として合成し、同じ `GuiSfntSimpleGlyphPathSinkActionStepItem` を返す。
- helper body は F4x helper と F4z helper 以外の lookup / payload / renderer / platform API に依存しない。
- hidden fallback、silent no-op、new type duplication、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ab: sfnt simple glyph path sink action item next

目的:

- F4z/F4aa の `GuiSfntSimpleGlyphPathSinkActionStepItem` から、次の action item または contour terminal state を 1 段だけ取得する public helper を追加する。
- F4ab は contour-wide traversal、iterator、real sink mutation、command list、full outline allocation、renderer、rasterizer にはならない。
- `EndContour` は successful terminal state として enum payload に残し、`Result::Err`、`Option::None`、hidden no-op へ変換しない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionItemNext`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_item_next`
- enum は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionItemNext:
    Continue GuiSfntSimpleGlyphPathSinkActionStepItem
    EndContour
```

- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_item_next:
    &ByteBuf
    Option i32
    &GuiSfntSimpleGlyphPathSinkActionStepItem
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionItemNext GuiSfntParseError
```

- helper は `gui_sfnt_simple_glyph_path_sink_action_step_item_advance item` を 1 回だけ読む。
- `advance = Continue next_step` の場合だけ `gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &next_step policy` を 1 回だけ呼ぶ。
- step item lookup の `Result::Err error` はそのまま伝播し、`Result::Ok next_item` は `Result::Ok GuiSfntSimpleGlyphPathSinkActionItemNext::Continue next_item` として返す。
- `advance = EndContour` の場合は `Result::Ok GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour` を返す。
- helper は `item.step`、`GuiSfntSimpleGlyphPathSinkActionStep.next`、action payload、primary/tail action、sink policy payload を読まない。
- helper は start cursor/start step/start item helper、F4v action step lookup、F4y advance helper、sink action lookup、sink step lookup、contour step lookup、F4s/F4t より下位の lookup、metadata parser、`*_with_tables`、`Vec`、`push`、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ab docs、enum、Clone/Copy、helper body、item advance accessor 1 回、F4z helper 1 回、禁止 helper、payload inspection 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の typed doctest を拡張する。
  - synthetic item の `EndContour` advance を `GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour` として返すことを確認する。
  - byte-backed fixture で `gui_sfnt_lookup_simple_glyph_path_sink_action_start_item` から得た item を `gui_sfnt_lookup_simple_glyph_path_sink_action_item_next` に渡し、`Continue next_item` を得ることを確認する。
  - next item の stored step cursor が same contour/edge/event の `Tail` action slot であることを確認する。

完了条件:

- item next helper は F4z item の checked advance と F4z step-item lookup だけを value として合成する。
- helper body は item advance accessor と F4z helper 以外の lookup / payload / renderer / platform API に依存しない。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ac: sfnt simple glyph path sink action consumer item

目的:

- F4z/F4aa の `GuiSfntSimpleGlyphPathSinkActionStepItem` から、future sink consumer が 1 action 分として読む typed packet を追加する。
- 現在 action と F4ab の checked next state を束ね、後続 sink が hidden current state に依存しない入力境界を作る。
- F4ac は real sink、iterator、contour-wide consumer、callback、command list、full outline allocation、renderer、rasterizer にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerItem`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_item`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_item_action`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_item_next`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item`
- struct は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerItem:
    action GuiSfntSimpleGlyphPathSinkAction
    next GuiSfntSimpleGlyphPathSinkActionItemNext
```

- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item:
    &ByteBuf
    Option i32
    &GuiSfntSimpleGlyphPathSinkActionStepItem
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItem GuiSfntParseError
```

- helper は `gui_sfnt_simple_glyph_path_sink_action_step_item_step item` を 1 回だけ読み、`gui_sfnt_simple_glyph_path_sink_action_step_action &stored_step` で action を 1 回だけ読む。
- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_item_next bytes face_index item policy` を 1 回だけ呼ぶ。
- `Result::Err error` はそのまま伝播し、`Result::Ok next` なら `GuiSfntSimpleGlyphPathSinkActionConsumerItem action next` を `Result::Ok` で返す。
- helper は `EmitEvent` / `Reject` / `NoAction` / `CloseContour` payload、primary/tail action、sink policy payload を match しない。
- helper は F4z action step item lookup、F4y advance helper、F4v action step lookup、F4x/F4aa start helper、sink action lookup、sink step lookup、contour step lookup、F4s/F4t より下位の lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ac docs、struct、Clone/Copy、constructor/accessors、helper body、step accessor 1 回、action accessor 1 回、F4ab item-next helper 1 回、禁止 helper、payload inspection 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の typed doctest を拡張する。
  - byte-backed fixture で `gui_sfnt_lookup_simple_glyph_path_sink_action_start_item` から得た item を consumer item helper に渡す。
  - consumer item の `action` が current start action を保持していることを確認する。
  - consumer item の `next` が `Continue next_item` であり、next item の cursor が same contour/edge/event の `Tail` action slot であることを確認する。

完了条件:

- consumer item helper は F4z item の current action copy と F4ab next state だけを value として合成する。
- helper body は step accessor、action accessor、F4ab item-next helper 以外の lookup / payload / renderer / platform API に依存しない。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ad: sfnt simple glyph path sink action consumer item next

目的:

- F4ac の `GuiSfntSimpleGlyphPathSinkActionConsumerItem` から、次の consumer item または contour terminal state を 1 段だけ取得する public helper を追加する。
- future sink loop が hidden current state に依存せず、typed packet continuation を扱える境界を作る。
- F4ad は contour-wide traversal、iterator、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next`
- enum は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerItemNext:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    EndContour
```

- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next:
    &ByteBuf
    Option i32
    &GuiSfntSimpleGlyphPathSinkActionConsumerItem
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItemNext GuiSfntParseError
```

- helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_item_next item` を 1 回だけ読む。
- `next = Continue next_item` の場合だけ `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &next_item policy` を 1 回だけ呼ぶ。
- consumer item lookup の `Result::Err error` はそのまま伝播し、`Result::Ok next_consumer_item` は `Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::Continue next_consumer_item` として返す。
- `next = EndContour` の場合は `Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::EndContour` を返す。
- helper は current action、`EmitEvent` / `Reject` / `NoAction` / `CloseContour` payload、primary/tail action、sink policy payload を読まない。
- helper は F4ab item next lookup、F4z action step item lookup、F4y advance helper、F4v action step lookup、F4x/F4aa start helper、sink action lookup、sink step lookup、contour step lookup、F4s/F4t より下位の lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ad docs、enum、Clone/Copy、helper body、consumer item next accessor 1 回、F4ac consumer item helper 1 回、禁止 helper、payload inspection 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の typed doctest を拡張する。
  - synthetic consumer item の `EndContour` next を `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::EndContour` として返すことを確認する。
  - byte-backed fixture で start consumer item から `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` を呼び、`Continue next_consumer_item` を得ることを確認する。

完了条件:

- consumer item next helper は F4ac consumer item の checked next と F4ac consumer item lookup だけを value として合成する。
- helper body は consumer next accessor と F4ac helper 以外の lookup / payload / renderer / platform API に依存しない。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ae: sfnt simple glyph path sink action apply state

目的:

- F4ac/F4ad の consumer item が保持する `GuiSfntSimpleGlyphPathSinkAction` を 1 action だけ消費し、明示的な domain status と count state に変換する。
- `Reject`、`CloseContour`、`NoAction` を hidden fallback や silent no-op にせず、enum status として future sink に渡せる境界を作る。
- F4ae は contour-wide traversal、iterator、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionApplyStatus`
  - `GuiSfntSimpleGlyphPathSinkActionApplyState`
  - `GuiSfntSimpleGlyphPathSinkActionApplyStep`
  - constructor / accessor helper
  - `gui_sfnt_simple_glyph_path_sink_action_apply_state_new`
  - `gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action`
- enum は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionApplyStatus:
    EmittedEvent GuiSfntSimpleGlyphPathSinkEvent
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    ClosedContour GuiSfntSimpleGlyphPathContourClose
    NoAction
```

- state は次の 4 count を持つ。

```text
GuiSfntSimpleGlyphPathSinkActionApplyState:
    emitted_event_count i32
    reject_count i32
    close_contour_count i32
    no_action_count i32
```

- helper は `GuiSfntSimpleGlyphPathSinkAction` を `match` し、各 variant で対応する count だけを `add count 1` する。
- `Reject` は `Result::Err` へ変換しない。typed reject status として `Rejected reason` を返す。
- `NoAction` は silent no-op ではない。`NoAction` status と `no_action_count + 1` を返す。
- count state は diagnostic / contract 検査用であり、cursor、next state、traversal authority として使わない。
- helper は F4ad consumer next、F4ac consumer item lookup、F4ab/F4z/F4y/F4v/start/lower lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ae docs、enum / struct、Clone/Copy、constructor / accessor、apply helper body、4 variant の count 更新、禁止 helper、`Result` / `Option` / allocation / renderer 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の typed doctest を拡張する。
  - `EmitEvent`、`Reject`、`CloseContour`、`NoAction` を順に apply し、それぞれの status と count が明示的に更新されることを確認する。
  - `NoAction` が test 上でも no-op ではなく `no_action_count` を進めることを確認する。

完了条件:

- action apply helper は 1 action を 1 status に変換し、1 counter だけを更新する。
- `Rejected` と `NoAction` は成功系の domain status として保持される。
- traversal authority は F4ac/F4ad に残り、F4ae は cursor / next state を決めない。
- hidden fallback、silent no-op、new traversal loop、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4af: sfnt simple glyph path sink action consumer apply step

目的:

- F4ac の `GuiSfntSimpleGlyphPathSinkActionConsumerItem` から current action を F4ae apply state に適用し、apply result と保存済み checked continuation を同じ value として運ぶ。
- future loop / real sink が「今回の消費結果」と「次に進むための保存済み next」を同時に読める境界を作る。
- F4af は byte-backed next lookup、contour-wide traversal、iterator、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep`
  - constructor / accessor helper
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply`
- struct は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep:
    apply_step GuiSfntSimpleGlyphPathSinkActionApplyStep
    next GuiSfntSimpleGlyphPathSinkActionItemNext
```

- helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_item_action item` を 1 回だけ読む。
- helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_item_next item` を 1 回だけ読む。
- helper は `gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action state action` を 1 回だけ呼ぶ。
- helper は `apply_step` と `next` を `GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep` に束ねる。
- `next` は F4ac packet に保存されていた `GuiSfntSimpleGlyphPathSinkActionItemNext` であり、F4af が新しく決める traversal state ではない。
- helper は `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` を作らず、`gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` も呼ばない。次 consumer item への byte-backed 解決は F4ad に残す。
- helper は action payload を直接 `match` しない。payload 解釈は F4ae helper だけに委譲する。
- helper は `Result`、`Option`、F4ad/F4ac byte-backed lookup、F4ab/F4z/F4y/F4v/start/lower lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4af docs、struct、Clone/Copy、constructor / accessor、consumer item action accessor 1 回、consumer item next accessor 1 回、F4ae apply helper 1 回、F4ad next helper 禁止、payload match 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の typed doctest を拡張する。
  - synthetic consumer item を `apply` し、status / state count と保存済み `next` が同時に読めることを確認する。
  - `next` が `GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour` のまま保存され、`GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` へ変換されないことを確認する。

完了条件:

- consumer item apply helper は current action を F4ae helper へ委譲し、保存済み checked continuation をそのまま同梱する。
- helper は F4ad の next resolution を呼ばず、traversal authority を持たない。
- hidden fallback、silent no-op、payload direct match、new traversal loop、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ag: sfnt simple glyph path sink action consumer apply terminal

目的:

- F4af の `GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep` を future consumer loop が扱う typed terminal 判定に変換する。
- `Rejected`、保存済み `EndContour`、保存済み `Continue` を enum で明示し、hidden fallback や silent skip を作らない。
- F4ag は contour-wide loop、byte-backed next lookup、real sink mutation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_reject_reason`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step`
- enum は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
```

- `Rejected reason` は malformed SFNT parse error ではなく domain terminal なので、`Result::Err` にはしない。
- 保存済み `EndContour` は successful terminal なので、これも `Result::Err` にはしない。
- `NoAction` は silent no-op ではないが、それだけで terminal にしない。`NoAction + Continue` は `Continue`、`NoAction + EndContour` は `EndContour` とする。
- helper は F4af の `apply_step` と `next` だけを読む。
- helper は `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` を作らず、`gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` も呼ばない。
- helper は action payload を直接 `match` しない。reject reason の取り出しは `GuiSfntSimpleGlyphPathSinkActionApplyStatus` の分類だけに限定する。
- helper は F4ad/F4ac byte-backed lookup、F4ab/F4z/F4y/F4v/start/lower lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ag docs、enum、Clone/Copy、reject reason helper、terminal helper、F4ad next helper 禁止、payload direct match 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の typed doctest を拡張する。
  - `Rejected` status が保存済み `EndContour` より優先されることを確認する。
  - 保存済み `EndContour` が successful terminal になることを確認する。
  - `NoAction + Continue` が terminal ではなく `Continue` になることを確認する。

完了条件:

- consumer apply step は `Continue` / `Rejected` / `EndContour` の typed terminal 判定に分類される。
- `Rejected` と `EndContour` を `Result::Err` に逃がさない。
- F4ag は next consumer item lookup や traversal loop を実装しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ah: sfnt simple glyph path sink action consumer apply advance

目的:

- F4ag の terminal 判定を使い、apply 後の consumer stream を 1 step だけ進める byte-backed boundary を追加する。
- `Continue` は次 consumer item、`Rejected` は domain terminal、`EndContour` は successful terminal として enum で明示する。
- F4ah は contour-wide loop、iterator、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance`
- enum は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

- helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step step` を 1 回だけ呼ぶ。
- `Rejected reason` は `Result::Ok Rejected reason` にする。`Result::Err` にはしない。
- `EndContour` は `Result::Ok EndContour` にする。`Result::Err` にはしない。
- `Continue continue_step` では、`gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next &continue_step` を読み、保存済み `GuiSfntSimpleGlyphPathSinkActionItemNext` を authority とする。
- 保存済み next が `Continue next_item` なら `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &next_item policy` を 1 回だけ呼び、成功時は `Continue next_consumer_item` を返す。
- 保存済み next が `EndContour` なら successful terminal として `EndContour` を返す。
- helper は original `GuiSfntSimpleGlyphPathSinkActionConsumerItem` を要求しない。
- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` を呼ばない。これは F4ad direct wrapper ではなく、F4ag terminal と保存済み `ActionItemNext` から F4ac lookup へ接続する 1 step boundary である。
- helper は action payload を直接 `match` せず、F4ae apply helper も呼ばない。
- helper は F4ad/F4ab/F4z/F4y/F4v/start/lower lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ah docs、enum、Clone/Copy、terminal helper 1 回、stored next accessor 1 回、F4ac lookup 1 回、F4ad next helper 禁止、payload direct match 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に F4ah 用の contract doctest を追加する。
  - `Rejected` terminal が `Ok Rejected` になることを確認する。
  - `EndContour` terminal が `Ok EndContour` になることを確認する。
  - `Continue` branch の byte-backed lookup path は source policy で固定し、必要なら後続 byte-backed fixture で拡張する。
  - F4ah helper は F4ac byte-backed lookup を参照するため、現行 compiler の 60 秒 doctest 制限では外部 `.n.md` fixture の compile が timeout する。したがって runnable ではなく `skip` 付き contract doctest とし、source policy で terminal helper / stored next / F4ac lookup の exact call pattern を固定する。

完了条件:

- F4ah は F4ag terminal 判定から `Continue` / `Rejected` / `EndContour` の apply advance を返す。
- `Rejected` と `EndContour` を `Result::Err` に逃がさない。
- F4ah は F4ad next helper や contour-wide loop を実装しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

`tests/stdlib/gui_font_sfnt_glyf_path.n.md` 側は F4ah contract fixture を `skip` として数える。実行可能な validation は既存 F4ag terminal doctest と `stdlib/alloc/gui/font/sfnt/glyf.nepl` doctest に置き、F4ah の byte-backed composition は `nodesrc/test_web_gui_font_rendering_contract.js` で静的検査する。

## Phase F4ai: sfnt simple glyph path sink action consumer item consume once

目的:

- 1 consumer item を F4af で apply し、その apply step を F4ah で 1 step advance する境界を追加する。
- F4af の apply state / status を捨てず、advance と同じ typed value に保持する。
- F4ai は contour-wide loop、iterator、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_step`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once`
- struct は次にする。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep:
    apply_step GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    advance GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance
```

- consume-once helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply state item` を 1 回だけ呼ぶ。
- consume-once helper は得られた `apply_step` を `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance bytes face_index &apply_step policy` へ 1 回だけ渡す。
- advance が `Result::Err error` なら parse/range failure としてそのまま伝播する。
- advance が `Result::Ok advance` なら、`apply_step` と `advance` を `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` に束ねて `Result::Ok` で返す。
- helper は F4ag を直接呼ばない。terminal classification は F4ah の責務である。
- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` を呼ばない。F4ad direct wrapper に戻すと apply result preservation が曖昧になる。
- helper は action payload を直接 `match` せず、F4ae apply helper も直接呼ばない。
- helper は F4ad/F4ab/F4z/F4y/F4v/start/lower lookup、metadata parser、`*_with_tables`、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API を直接呼ばない。
- Source policy で F4ai docs、struct、Clone/Copy、constructor / accessor、F4af helper 1 回、F4ah helper 1 回、constructor 1 回、F4ag direct call 禁止、F4ad next helper 禁止、payload direct match 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` に F4ai 用の contract doctest を追加する。
  - synthetic `Rejected` case で apply status / state count と `Rejected` advance の両方が保持されることを確認する。
  - synthetic `EndContour` case で apply status / state count と `EndContour` advance の両方が保持されることを確認する。
  - F4ai helper は F4ah byte-backed lookup を参照するため、現行 compiler の 60 秒 doctest 制限で外部 `.n.md` fixture の compile が timeout する場合は `skip` 付き contract doctest とし、source policy で exact call pattern を固定する。

完了条件:

- consume-once result は apply step と advance を両方保持する。
- F4ai は F4af と F4ah の薄い合成に留まり、F4ag/F4ad/lower traversal へ直接依存しない。
- F4ai は loop、real sink、renderer、rasterizer、platform backend、font fallback を実装しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

`tests/stdlib/gui_font_sfnt_glyf_path.n.md` 側の F4ai fixture が byte-backed helper materialization で timeout する場合は、F4ah と同じく `skip` として数える。実装 body の exact call pattern は `nodesrc/test_web_gui_font_rendering_contract.js` で固定する。

## Phase F4aj: sfnt simple glyph path sink action start consumer item

目的:

- contour start から future consumer loop の初期 `GuiSfntSimpleGlyphPathSinkActionConsumerItem` を読む public helper を追加する。
- F4aa start item と F4ac consumer item を薄く合成し、新しい value type や traversal authority を作らない。
- F4aj は consume、apply、post-apply advance、consumer item next、contour-wide loop、real sink mutation、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item`
- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item:
    &ByteBuf
    Option i32
    GuiGlyphId
    i32
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItem GuiSfntParseError
```

- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_item bytes face_index glyph contour_index policy` を 1 回だけ呼ぶ。
- start item が `Result::Err error` ならそのまま `Result::Err error` を返す。
- start item が `Result::Ok item` なら `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &item policy` を 1 回だけ呼ぶ。
- consumer item lookup の `Result::Err error` はそのまま伝播し、`Result::Ok consumer_item` はそのまま返す。
- helper は `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` を作らず、`gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` と `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once` を呼ばない。
- helper は F4af apply、F4ah apply advance、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- F4ac は consumer item を作る契約上、checked `GuiSfntSimpleGlyphPathSinkActionItemNext` を内部で読む。この F4ac 内部処理は許容し、F4aj 自体の consumer item next / consume / apply / advance とは区別する。
- Source policy で F4aj docs、helper body、F4aa helper 1 回、F4ac helper 1 回、F4ad/F4ai/F4af/F4ah/direct lower helper 禁止、payload direct match 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の skipped byte-backed fixture を拡張する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item &bytes none glyph 0 &sink_policy` を呼ぶ。
  - 成功時の action が first event の `EmitEvent` であることを確認する。
  - checked next が `Continue next_item` で、next item の stored cursor が same contour/edge/event の `Tail` action slot であることを確認する。

完了条件:

- start consumer item helper は F4aa と F4ac を value として合成し、同じ `GuiSfntSimpleGlyphPathSinkActionConsumerItem` を返す。
- helper body は F4aa helper と F4ac helper 以外の lookup / payload / renderer / platform API に依存しない。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

`tests/stdlib/gui_font_sfnt_glyf_path.n.md` 側は既存 byte-backed public lookup fixture を `skip` として数える。実装 body の exact call pattern は `nodesrc/test_web_gui_font_rendering_contract.js` で固定する。

## Phase F4ak: sfnt simple glyph path sink action start consume once

目的:

- contour start から first consumer item を作り、その 1 item だけを consume する public helper を追加する。
- F4aj start consumer item と F4ai consume once を薄く合成し、F4ai の apply step / advance preservation contract をそのまま保つ。
- F4ak は contour-wide loop、iterator、real sink mutation、callback、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once`
- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once:
    &ByteBuf
    Option i32
    GuiSfntSimpleGlyphPathSinkActionApplyState
    GuiGlyphId
    i32
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntParseError
```

- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item bytes face_index glyph contour_index policy` を 1 回だけ呼ぶ。
- start consumer item が `Result::Err error` ならそのまま `Result::Err error` を返す。
- start consumer item が `Result::Ok consumer_item` なら `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state &consumer_item policy` を 1 回だけ呼ぶ。
- consume-once helper の `Result::Err error` はそのまま伝播し、`Result::Ok consume_step` はそのまま返す。
- helper は `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance` だけを返してはならない。F4ai と同じ `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` を返し、apply state / status と post-consume advance を保持する。
- helper は `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` を作らず、F4aa start item、F4ac consumer item、F4ad consumer item next、F4af apply、F4ah apply advance、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- Source policy で F4ak docs、helper body、F4aj helper 1 回、F4ai helper 1 回、F4aa/F4ac/F4ad/F4af/F4ah/direct lower helper 禁止、payload direct match 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の skipped byte-backed fixture を拡張する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once &bytes none state glyph 0 &sink_policy` を呼ぶ。
  - 成功時の consume step から apply step を読み、first event が `EmittedEvent` として status に残ることを確認する。
  - apply state の emitted event count が 1 になることを確認する。
  - advance が `Continue next_consumer` であり、next consumer の action が same edge tail の `NoAction` として保持されることを確認する。

完了条件:

- start consume-once helper は F4aj と F4ai を value として合成し、同じ `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` を返す。
- helper body は F4aj helper と F4ai helper 以外の lookup / payload / renderer / platform API に依存しない。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

`tests/stdlib/gui_font_sfnt_glyf_path.n.md` 側は既存 byte-backed public lookup fixture を `skip` として数える。実装 body の exact call pattern は `nodesrc/test_web_gui_font_rendering_contract.js` で固定する。

## Phase F4al: sfnt simple glyph path sink action consumer consume step apply summary

目的:

- `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` から consume 後 apply state と consumed action status を読む pure public helper を追加する。
- future consumer loop が F4ai/F4af の nested storage layout へ直接依存しないようにする。
- F4al は loop、iterator、real sink mutation、byte-backed lookup、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status`
- helper signature は次にする。

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state:
    &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep
    -> GuiSfntSimpleGlyphPathSinkActionApplyState

gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status:
    &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep
    -> GuiSfntSimpleGlyphPathSinkActionApplyStatus
```

- state helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_step step` を 1 回だけ呼ぶ。
- state helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_apply_step &consumer_apply_step` を 1 回だけ呼ぶ。
- state helper は `gui_sfnt_simple_glyph_path_sink_action_apply_step_state &inner_apply_step` を 1 回だけ呼ぶ。
- status helper は同じ first two calls の後、`gui_sfnt_simple_glyph_path_sink_action_apply_step_status &inner_apply_step` を 1 回だけ呼ぶ。
- helper は `advance` を読まない。traversal / terminal state は既存 `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance` の責務として分離する。
- helper は `Result`、`Option`、byte-backed lookup、consumer item next、consume-once、start helper、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- Source policy で F4al docs、helper body、exact call count、advance 禁止、lookup / payload / renderer / platform API 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ai synthetic fixture を更新する。
  - `Rejected` case と `NoAction` case で新しい state / status helpers を使い、nested layout へ直接入らないことを確認する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ak byte-backed fixture も更新する。
  - start consume-once result から新しい state / status helpers を使い、first action status / count と post-consume advance を別々に読む。

完了条件:

- consume step apply summary helper は consume step の apply side だけを読む。
- future loop が更新後 state / consumed status を nested F4af/F4ae layout へ直接依存せずに読める。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4am: sfnt simple glyph path sink action consumer consume summary value

目的:

- `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep` から、future consumer loop が直接扱う state / status / advance の flat summary value を作る。
- F4al の apply summary helper と既存 `advance` accessor を 1 value に束ね、future loop が nested F4ai/F4af/F4al storage layout へ依存しないようにする。
- F4am は contour-wide loop、iterator、real sink mutation、byte-backed lookup、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_status`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step`
- summary type は次の 3 fields を持つ。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary:
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    status GuiSfntSimpleGlyphPathSinkActionApplyStatus
    advance GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance
```

- `summary_from_step` は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state step` を 1 回だけ呼ぶ。
- `summary_from_step` は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status step` を 1 回だけ呼ぶ。
- `summary_from_step` は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance step` を 1 回だけ呼ぶ。
- F4al の apply-state/status helper は引き続き `advance` を読まない。F4am だけが full consume summary contract として既存 advance accessor を読む。
- helper は `Result`、`Option`、byte-backed lookup、consumer item next lookup、consume-once、start helper、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- Source policy で F4am docs、summary type、Clone / Copy、constructor/accessors、from-step exact call count、lookup / payload / renderer / platform API 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ai synthetic fixture を更新する。
  - `Rejected` case と `NoAction` case で summary を作り、summary accessors から state / status / advance を読む。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ak byte-backed fixture も更新する。
  - start consume-once result から summary を作り、first action status / count / post-consume advance を summary accessors から読む。

完了条件:

- consume summary は state / status / advance を 1 value として持つ。
- from-step helper は F4al state helper、F4al status helper、existing advance accessor をそれぞれ 1 回だけ読む。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4an: sfnt simple glyph path sink action consumer consume summary terminal

目的:

- `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary` に保持された post-consume advance を、future loop が読む traversal control state へ写す。
- F4am は state / status / advance を束ねるだけで advance を解釈しない。F4an は stored advance の 3 分岐を 1 回だけ読み、loop 側が lower `ApplyAdvance` storage detail に直接依存しないようにする。
- `Terminal` は名前として使うが `Continue` も含む。これは terminal-only value ではなく traversal control projection である。
- F4an は contour-wide loop、iterator、real sink mutation、byte-backed lookup、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal`
  - `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal`
- summary terminal type は次の 3 variants を持つ。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

- `summary_terminal` は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance summary` を 1 回だけ呼ぶ。
- `summary_terminal` は `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Continue item` を `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Continue item` に写す。
- `summary_terminal` は `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Rejected reason` を `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Rejected reason` に写す。
- `summary_terminal` は `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::EndContour` を `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::EndContour` に写す。
- helper は `Result`、`Option`、byte-backed lookup、consumer item next lookup、consume-once、start helper、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- Source policy で F4an docs、summary terminal enum、Clone / Copy、helper exact advance accessor call count、3 分岐の同型写像、lookup / payload / renderer / platform API 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ai synthetic fixture を更新する。
  - `Rejected` case と `NoAction` case で summary terminal helper を使い、Rejected / EndContour を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ak byte-backed fixture も更新する。
  - start consume-once result から summary terminal helper を使い、Continue 分岐と次 consumer item の action を検査する。

完了条件:

- summary terminal は Continue / Rejected / EndContour を 1 value として持つ。
- summary terminal helper は stored advance accessor を 1 回だけ読み、lower advance enum を同型写像する。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ap: sfnt simple glyph path sink action start consume summary

目的:

- F4ak start consume-once と F4am consume summary projection を薄く合成し、future consumer loop の initial summary boundary を作る。
- F4ao の consume summary advance-once が受け取る `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary` を、contour start から直接得られるようにする。
- 新しい enum は増やさず、既存 `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary` を返す。
- F4ap は contour-wide loop、iterator、real sink mutation、summary advance、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary`
- helper signature は次にする。

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary:
    &ByteBuf
    Option i32
    GuiSfntSimpleGlyphPathSinkActionApplyState
    GuiGlyphId
    i32
    &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary GuiSfntParseError
```

- helper は `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once bytes face_index state glyph contour_index policy` を 1 回だけ呼ぶ。
- `Result::Err error` はそのまま `Result::Err error` として返す。
- `Result::Ok consume_step` の場合だけ、`gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step &consume_step` を 1 回だけ呼ぶ。
- success branch は `Result::Ok summary` を返す。
- helper が直接使ってよい byte-backed lookup は start consume-once helper だけである。
- helper は start item、start consumer item、consumer item consume-once、summary advance-once、consumer item next lookup、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、full loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- Source policy で F4ap docs、helper signature、exact call count、error propagation、success conversion、lookup / payload / renderer / platform API 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ak byte-backed fixture を更新する。
  - start summary helper を直接呼び、F4ak 経由で作った summary と同じ first action status / count / terminal を確認する。

完了条件:

- start consume summary helper は F4ak と F4am を value として合成し、initial summary を返す。
- full loop、hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4ao: sfnt simple glyph path sink action consumer consume summary advance once

目的:

- F4am/F4an の summary boundary を使い、future consumer loop の 1 step advance 境界を作る。
- `Continue` の場合だけ次 consumer item を 1 つ消費し、次 summary を返す。
- `Rejected` と `EndContour` は parse error ではなく、`Result::Ok` の domain terminal として返す。
- F4ao は contour-wide loop、iterator、real sink mutation、byte-backed start traversal、command list、full outline allocation、renderer、rasterizer、platform API にはならない。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once`
- summary advance type は次の 3 variants を持つ。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

- helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state summary` を 1 回だけ呼ぶ。
- helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary` を 1 回だけ呼ぶ。
- `Continue item` branch だけが `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state &item policy` を 1 回だけ呼ぶ。
- consume-once が `Result::Err error` を返した場合は、その parse error をそのまま返す。
- consume-once が `Result::Ok consume_step` を返した場合は、`gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step &consume_step` を 1 回だけ呼び、`Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance::Continue next_summary` を返す。
- `Rejected reason` branch は `Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance::Rejected reason` を返す。
- `EndContour` branch は `Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance::EndContour` を返す。
- helper が直接使ってよい byte-backed lookup は Continue branch の consume-once helper だけである。
- helper は start helper、start consume-once、consumer item next lookup、F4ab/F4z/F4y/F4v/lower lookup、metadata parser、`*_with_tables`、action payload direct match、`Vec`、`push`、full loop、current point、renderer、rasterizer、platform API、host text API、font fallback を直接使わない。
- Source policy で F4ao docs、summary advance enum、Clone / Copy、helper exact call count、domain terminal `Result::Ok`、lookup / payload / renderer / platform API 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ai synthetic fixture を更新する。
  - `Rejected` case と `NoAction` case で summary advance-once helper を使い、Rejected / EndContour が `Result::Ok` domain terminal として返ることを検査する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ak byte-backed fixture も更新する。
  - first action summary から summary advance-once helper を使い、Continue が次 summary を返し、その次 summary が NoAction / EndContour になることを検査する。

完了条件:

- summary advance-once は full loop ではなく、1 summary から次 summary または domain terminal へ 1 step だけ進める。
- parse error と domain terminal を混同しない。
- hidden fallback、silent no-op、new traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F4aq: sfnt simple glyph path sink action consume summary drain budget

目的:

- F4ap initial summary と F4ao advance-once を使い、contour action consumer を explicit budget 内で domain terminal まで進める。
- `StepBudgetExhausted` を typed terminal として返し、unbounded traversal、silent success、hidden fallback を避ける。
- outline allocation / sink mutation / render command emission の前に、byte-backed traversal の停止点を enum として固定する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_drain_budget`
  - `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary_drain_budget`
- drain result type は次の 3 variants を持つ。

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain:
    EndContour GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    StepBudgetExhausted GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
```

- drain helper は `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary` を 1 回だけ呼ぶ。
- `Rejected reason` branch は budget を消費せず、`Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain::Rejected reason current_summary` を返す。
- `EndContour` branch は budget を消費せず、`Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain::EndContour current_summary` を返す。
- `Continue` かつ `remaining_steps <= 0` は `Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain::StepBudgetExhausted current_summary` を返す。
- `Continue` かつ `remaining_steps > 0` の場合だけ、`gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once bytes face_index summary policy` を 1 回だけ呼ぶ。
- advance-once が `Result::Err error` を返した場合は、その parse error をそのまま返す。
- advance-once が `Continue next_summary` を返した場合は、`remaining_steps - 1` で drain helper を 1 回だけ再帰呼び出しする。
- advance-once が保守上 `Rejected` / `EndContour` を返した場合は、F4ao に渡した current summary を drain result に入れる。
- start drain helper は F4ap start consume summary を 1 回だけ呼び、成功時だけ drain helper へ 1 回渡す。
- start drain helper は F4ao を直接呼ばない。
- helper は action payload direct match、`Vec`、`push`、full outline allocation、renderer、rasterizer、platform API、host text API、font fallback、lower lookup、metadata parser、`*_with_tables` を直接使わない。
- Source policy で F4aq docs、drain enum、Clone / Copy、helper exact call count、`remaining_steps == 0` / `< 0` evidence、current summary terminal payload、lookup / payload / renderer / platform API 禁止、括弧なし body を固定する。
- `tests/stdlib/gui_font_sfnt_glyf_path.n.md` の F4ak byte-backed fixture を更新する。
  - first summary から drain budget 0 と -1 が `StepBudgetExhausted` になることを検査する。
  - start drain budget 2 が `EndContour` summary を返し、emitted event count と no-action count を保持することを検査する。

完了条件:

- drain helper は bounded traversal boundary であり、unbounded traversal や command allocation にはならない。
- parse error と domain terminal と budget exhaustion を混同しない。
- hidden fallback、silent no-op、new untyped traversal counter、full outline allocation を追加しない。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_path.n.md --no-tree -o tmp_gui_font_sfnt_glyf_path.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5a: sfnt simple glyph outline storage capacity and owner recovery contract

目的:

- F4aq の bounded traversal の後に、simple glyph outline storage が必要とする capacity を allocation-free な value として計算する。
- capacity exceeded、invalid topology、command count overflow を enum branch として分離し、owner-taking allocation API の前に失敗時の owner recovery contract を固定する。
- outline allocation、sink mutation、renderer、rasterizer、platform API、host text API、font substitute へ進まない。

変更:

- 先に source policy を追加し、F5a docs、value type、helper の責務、禁止 API、括弧なし body を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlineStorageCapacity`
  - `GuiSfntSimpleGlyphOutlineStorageLimit`
  - `GuiSfntSimpleGlyphOutlineCapacityRejectReason`
  - `GuiSfntSimpleGlyphOutlineCapacityRejected`
  - `GuiSfntSimpleGlyphOutlineCapacityCheck`
  - `gui_sfnt_simple_glyph_outline_storage_capacity_from_topology`
  - `gui_sfnt_simple_glyph_outline_storage_capacity_check_limit`
- capacity fields は glyph、contour_count、point_count、edge_count、path_command_pair_count、path_command_count とする。
- `edge_count = point_count`、`path_command_pair_count = point_count`、`path_command_count = point_count * 2` とする。
- `contour_count <= 0`、`point_count <= 0`、`contour_count > point_count` は `InvalidTopology topology` とする。
- `point_count > 1073741823` は `CommandCountOverflow topology` とする。
- limit の各値は 1 以上を許可容量として扱う。0 以下は unlimited ではなく capacity exceeded とする。
- limit check は contour、point、edge、path command の順に最初の exceeded reason を返す。
- capacity exceeded は `GuiSfntSimpleGlyphOutlineCapacityRejected` として reason、capacity、limit を保持する。
- `GuiSfntSimpleGlyphOutlineCapacityRejectReason` は limit exceeded 専用であり、`InvalidTopology` と `CommandCountOverflow` は capacity が信頼できないため `GuiSfntSimpleGlyphOutlineCapacityCheck` の独立 variant とする。
- F5a helper は `Vec`、`push`、outline point list、contour list、path command list、renderer、rasterizer、platform API、host text API、font substitute、byte-backed lookup、metadata parser、`*_with_tables`、F4aq drain helper、lower contour helper、point decoder を使わない。
- doctest は synthetic topology と synthetic limit だけで分岐を検査する。byte-backed font fixture、renderer、raster、platform、host font API は使わない。

完了条件:

- valid topology から capacity が生成され、edge / path command count が仕様通りになる。
- forged invalid topology、command count overflow、各 limit exceeded が enum branch として検査される。
- F5a source policy が docs と implementation の責務逸脱を検出する。
- F4aq の `StepBudgetExhausted` が capacity success として扱われていないことを docs / policy で固定する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_capacity.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_capacity.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5b+: outline, shaping, ruby, vertical, math bridge

目的:

- glyph outline / mask、GSUB/GPOS、縦書き、ruby、math inline bridge を段階的に実装する。

注意:

- F5a の capacity / owner recovery contract を保ったまま、owner-taking storage API、outline point stream、raster mask、render2d command へ順に接続する。
- 未対応 feature は typed unsupported として返す。
- F5b 以降の outline doctest は timeout と責務混在を避けるため、phase ごとの専用ファイルに分ける。
  - F5b storage owner: `tests/stdlib/gui_font_sfnt_glyf_outline_storage.n.md`
  - F5c scalar push: `tests/stdlib/gui_font_sfnt_glyf_outline_scalar_push.n.md`
  - F5d region cursor: `tests/stdlib/gui_font_sfnt_glyf_outline_region_cursor.n.md`
  - F5e/F5f contour endpoint: `tests/stdlib/gui_font_sfnt_glyf_outline_contour_endpoint.n.md`
  - F5g PointX population: `tests/stdlib/gui_font_sfnt_glyf_outline_point_x.n.md`
  - F5h PointX reader bridge success: `tests/stdlib/gui_font_sfnt_glyf_outline_point_x_reader_success.n.md`
  - F5h PointX reader bridge read failure: `tests/stdlib/gui_font_sfnt_glyf_outline_point_x_reader_read_failure.n.md`
  - F5h PointX reader bridge push failure: `tests/stdlib/gui_font_sfnt_glyf_outline_point_x_reader_push_failure.n.md`
  - F5i/F5j PointY: `tests/stdlib/gui_font_sfnt_glyf_outline_point_y.n.md`
  - F5k coordinate read: `tests/stdlib/gui_font_sfnt_glyf_outline_point_coordinate.n.md`
  - F5l endpoint marker read: `tests/stdlib/gui_font_sfnt_glyf_outline_point_endpoint.n.md`
  - F5m point flag marker read: `tests/stdlib/gui_font_sfnt_glyf_outline_point_flag.n.md`
  - F5n full point read composition: `tests/stdlib/gui_font_sfnt_glyf_outline_point_read.n.md`

## Phase F5b: sfnt simple glyph outline scalar storage owner

目的:

- F5a の trusted capacity から、後続 outline builder が使う empty scalar slot storage owner を作る。
- forged capacity を capacity exceeded と混同せず、`InvalidCapacity` を limit rejection より前に返す。
- 複数 Vec owner の部分確保失敗を避けるため、F5b では 1 本の `Vec i32` scalar slot storage だけを確保する。
- point decode、contour decode、path command push、renderer、rasterizer、platform API、host text API、font substitute へ進まない。

変更:

- 先に source policy を追加し、F5b docs、storage owner、error enum、shape validation、scalar overflow guard、allocation/free 回数、禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に `alloc/collections/vec` を qualified import する。
- 次の型を追加する。
  - `GuiSfntSimpleGlyphOutlineStorage`
  - `GuiSfntSimpleGlyphOutlineStorageAllocErrorKind`
  - `GuiSfntSimpleGlyphOutlineStorageAllocError`
  - `GuiSfntSimpleGlyphOutlineScalarSlotCountCheck`
- `GuiSfntSimpleGlyphOutlineStorage` は `capacity`、`scalar_slots Vec i32`、`scalar_slot_count` を持つ owner であり、`Clone` / `Copy` を実装しない。
- `scalar_slot_count` は `contour_count + point_count + point_count + edge_count + path_command_count` とする。
- `gui_sfnt_simple_glyph_outline_storage_capacity_shape_is_valid` は capacity shape を検査する。`point_count <= 1073741823` は `point_count * 2` 比較より前に確認する。
- `gui_sfnt_simple_glyph_outline_storage_scalar_slot_count_check` は staged residual guard で i32 overflow を検出する。
- `gui_sfnt_simple_glyph_outline_storage_alloc` は次の順序を守る。
  - `shape_is_valid` が false なら `InvalidCapacity` と `capacity_check = none` を返す。
  - shape が valid の場合だけ `gui_sfnt_simple_glyph_outline_storage_capacity_check_limit` を呼ぶ。
  - `Rejected` は `CapacityRejected` と `capacity_check = some checked` を返す。
  - `Fits` の場合だけ scalar slot count を検査する。
  - scalar overflow は `ScalarSlotCountOverflow` と `capacity_check = some checked` を返す。
  - `vec::with_capacity` は 1 回だけ呼ぶ。
  - allocation failure は `ScalarSlotStorageAllocFailed` と `capacity_check = some checked` を返す。
- `gui_sfnt_simple_glyph_outline_storage_free` は storage owner を消費し、`vec::free` を 1 回だけ呼ぶ。
- doctest は synthetic capacity / limit だけで success、invalid forged capacity、limit rejection、scalar slot overflow を検査する。byte-backed font fixture、renderer、raster、platform、host font API は使わない。

完了条件:

- small topology から storage が確保され、`len == 0`、`cap == scalar_slot_count`、`scalar_slot_count` が formula 通りである。
- forged invalid capacity は `CapacityRejected` ではなく `InvalidCapacity` になる。
- limit exceeded は shape valid の場合だけ `CapacityRejected` になる。
- scalar slot count overflow は allocation を試みず enum branch になる。
- source policy が docs、型、allocation ordering、`Vec` 呼び出し回数、storage owner の非 Copy / 非 Clone、禁止 API を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_storage.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_storage.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5c: sfnt simple glyph outline scalar slot push owner recovery

目的:

- F5b の `GuiSfntSimpleGlyphOutlineStorage` owner を消費し、scalar slot value を 1 件追加した owner を返す。
- `Vec` push failure を `StdErrorKind` だけへ潰さず、storage owner と rejected scalar value を error payload に返す。
- slot value の意味づけ、point decode、contour endpoint population、path command tag population、renderer、rasterizer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5c docs、push error type、helper signatures、owner recovery、禁止 API、`vec::push` 呼び出し回数を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlineStoragePushError`
  - `gui_sfnt_simple_glyph_outline_storage_push_error`
  - `gui_sfnt_simple_glyph_outline_storage_push_error_kind`
  - `gui_sfnt_simple_glyph_outline_storage_push_error_scalar_value`
  - `gui_sfnt_simple_glyph_outline_storage_push_error_storage`
  - `gui_sfnt_simple_glyph_outline_storage_push_error_with`
  - `gui_sfnt_simple_glyph_outline_storage_push_scalar_slot`
- push helper は storage owner から capacity、scalar_slot_count、scalar_slots を取り出し、`vec::push scalar_slots value` を 1 回だけ呼ぶ。
- `Result::Ok next_slots` は `GuiSfntSimpleGlyphOutlineStorage capacity next_slots scalar_slot_count` を返す。
- `Result::Err e` は `vec::vec_push_error_kind &e` を先に読み、その後 `vec::vec_push_error_vec e` で returned slots を取り出し、returned storage と rejected scalar value と error kind を `GuiSfntSimpleGlyphOutlineStoragePushError` に入れて返す。
- F5c push helper は `vec::with_capacity`、`vec::free`、`vec::filled`、`vec::replace`、`vec::pop` を直接呼ばない。
- doctest は dedicated scalar push test file に success push と synthetic error recovery を追加する。real OOM は誘発しない。

完了条件:

- storage に scalar value を 2 件 push し、`len == 2`、`cap` と `scalar_slot_count` が F5b のまま保たれる。
- synthetic push error から storage owner、scalar value、error kind を取り出し、recovered storage を 1 回だけ free できる。
- source policy が F5c docs、型、helper、push の owner recovery、禁止 API、`vec::push` 1 回を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_scalar_push.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_scalar_push.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5d: sfnt simple glyph outline scalar region cursor

目的:

- F5b/F5c の single `Vec i32` storage owner に、contour endpoint、x、y、edge、path command tag の typed region cursor を追加する。
- unchecked boundary 計算を public API にせず、capacity shape と scalar slot count overflow を検査してから region start/end を計算する。
- fixed-capacity outline storage の invariant を守り、region push で Vec growth に依存しない。
- point decode、path command generation、renderer、rasterizer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5d docs、cursor type、region push result/error type、unchecked helper 非公開、validation order、owner 型の非 Clone / 非 Copy、禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlineScalarRegion`
  - `GuiSfntSimpleGlyphOutlineScalarRegionCursor`
  - `gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity`
  - non-public `gui_sfnt_simple_glyph_outline_scalar_region_cursor_from_valid_capacity`
  - `gui_sfnt_simple_glyph_outline_scalar_region_cursor_is_well_formed`
  - non-public cursor/capacity matching helper
  - `GuiSfntSimpleGlyphOutlineRegionPush`
  - `GuiSfntSimpleGlyphOutlineRegionPushErrorKind`
  - `GuiSfntSimpleGlyphOutlineRegionPushError`
  - `gui_sfnt_simple_glyph_outline_storage_push_region_scalar`
- `try_from_capacity` は `shape_is_valid` と `scalar_slot_count_check` が成功した後でだけ raw boundary helper を呼ぶ。
- `push_region_scalar` は capacity、`scalar_slot_count`、`scalar_slots_len`、`scalar_slots_cap` を先に読み、次の順序で検査する。
  - capacity shape
  - scalar slot count `Fits`
  - `scalar_slot_count == expected`
  - `scalar_slots_cap == scalar_slot_count`
  - cursor well-formed
  - cursor region/start/end match
  - `scalar_slots_len == cursor.next_index`
  - `cursor.next_index < cursor.end`
  - F5c `gui_sfnt_simple_glyph_outline_storage_push_scalar_slot` を 1 回だけ呼ぶ
- `scalar_slots_len == cursor.next_index` は `RegionFull` より前に検査する。
- `GuiSfntSimpleGlyphOutlineRegionPush` と `GuiSfntSimpleGlyphOutlineRegionPushError` は storage owner を持つため `Clone` / `Copy` を実装しない。
- doctest は cursor boundary、region push success、region full、storage cursor mismatch を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。

完了条件:

- 2 contours / 4 points の capacity から、region cursor が `0..2`、`2..6`、`6..10`、`10..14`、`14..22` を返す。
- contour endpoint region に 2 件 push でき、storage len と cursor next index が 2 になる。
- full region への追加は storage owner と rejected scalar value を保持した `RegionFull` になる。
- empty storage に full cursor を渡す forged case は `StorageCursorMismatch` になる。
- source policy が unchecked public helper、validation order、fixed Vec cap invariant、F5c push 呼び出し回数、禁止 API、owner 型の非 Clone / 非 Copy を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_region_cursor.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_region_cursor.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5e: sfnt simple glyph contour endpoint population

目的:

- F5d の contour endpoint region cursor を使い、typed contour endpoint slot を owner-preserving に storage へ追加する。
- byte-backed endpoint array reading、point flag decode、x/y coordinate decode、edge/path command generation、rasterizer、renderer、platform API、host text API へ進まない。
- capacity、cursor、endpoint sequence の validation order を source policy と doctest で固定する。

変更:

- 先に source policy を追加し、F5e docs、endpoint slot type、success/error owner payload、validation order、owner 型の非 Clone / 非 Copy、禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphContourEndpointSlot`
  - `GuiSfntSimpleGlyphContourEndpointPush`
  - `GuiSfntSimpleGlyphContourEndpointPushErrorKind`
  - `GuiSfntSimpleGlyphContourEndpointPushError`
  - `gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint`
- public helper は storage capacity を検査してから `contour_count` / `point_count` を読む。
- cursor well-formed validation は `cursor.next_index` を読む前に行う。
- endpoint contour index range は final/non-final classification より前に検査する。
- previous endpoint range は `end_point_index > previous` より前に検査する。
- commit helper は F5d `gui_sfnt_simple_glyph_outline_storage_push_region_scalar` を 1 回だけ呼び、F5d error を `RegionPushFailed` に owner-preserving に包む。
- doctest は success、non-final endpoint at last point、final endpoint mismatch、forged PointX cursor region mismatch を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。

完了条件:

- 2 contours / 4 points の synthetic endpoint 1, 3 を追加でき、storage len と cursor next index が 2、previous endpoint が 3 になる。
- non-final contour が final point を endpoint にした場合は `EndpointOutOfRange` になる。
- final contour endpoint が `point_count - 1` でない場合は `FinalEndpointMismatch` になる。
- PointX cursor を渡した場合は `CursorRegionMismatch` になり、storage cursor mismatch など下位 error に落ちない。
- source policy が capacity/cursor/endpoint validation order、F5d region push 呼び出し回数、direct `vec::` 禁止、byte/render/raster/platform/host API 禁止、owner 型の非 Clone / 非 Copy を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_contour_endpoint.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_contour_endpoint.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5f: sfnt simple glyph contour endpoint byte reader bridge

目的:

- 既存の checked `gui_sfnt_glyf_read_contour_endpoint` と F5e の `gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint` を接続する。
- byte-backed endpoint array reading と owner-preserving storage mutation の error domain を分ける。
- x/y coordinate decode、flag decode、edge/path command generation、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5f docs、read-before-mutate ordering、read failure と push failure の分離、owner 型の非 Clone / 非 Copy、禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphContourEndpointReadPush`
  - `GuiSfntSimpleGlyphContourEndpointReadPushErrorKind`
  - `GuiSfntSimpleGlyphContourEndpointReadPushError`
  - `gui_sfnt_glyf_read_push_contour_endpoint`
- public helper は `gui_sfnt_glyf_read_contour_endpoint` を 1 回だけ呼び、read failure では F5e push を呼ばない。
- read success では `GuiSfntSimpleGlyphContourEndpointSlot` を作り、F5e `gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint` を 1 回だけ呼ぶ。
- F5e push failure では endpoint、F5e error kind、F5d region error kind、F5c storage push error kind を owner 消費前に読む。
- doctest は byte-backed success、read failure owner recovery、push failure endpoint preservation を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。

完了条件:

- synthetic endpoint bytes から 2 contours / 4 points の endpoint 1, 3 を読み、storage len と cursor next index が 2、previous endpoint が 3 になる。
- endpoint byte range が table 外なら `ReadFailed` になり、parse error が `Some`、endpoint が `None`、storage len が 0 のまま回収できる。
- valid bytes だが F5e validation が失敗する場合は `PushFailed` になり、parse error が `None`、endpoint が `Some`、lower F5e error kind が `Some` になる。
- source policy が read-before-mutate、F5e push 呼び出し回数、lower error metadata の owner 消費前読み取り、direct `vec::` 禁止、point decode/render/raster/platform/host API 禁止、owner 型の非 Clone / 非 Copy を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_contour_endpoint.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_contour_endpoint.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5g: sfnt simple glyph point x coordinate population

目的:

- F5d の `PointX` region cursor を使い、typed x coordinate slot を owner-preserving に storage へ追加する。
- scalar storage index と glyph logical point index を混同しない validation order を固定する。
- byte-backed x decode、point flag decode、y coordinate、edge/path command generation、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5g docs、PointX slot type、success/error owner payload、validation order、owner 型の非 Clone / 非 Copy、禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPointXSlot`
  - `GuiSfntSimpleGlyphPointXPush`
  - `GuiSfntSimpleGlyphPointXPushErrorKind`
  - `GuiSfntSimpleGlyphPointXPushError`
  - `gui_sfnt_simple_glyph_outline_storage_push_point_x`
- public helper は capacity shape と scalar slot count `Fits` を検査してから `point_count` を読む。
- cursor well-formed validation と cursor/capacity boundary match は `logical_point_index = cursor.next_index - cursor.start` より前に行う。
- `PointX` region であることを確認し、`point.point_index == logical_point_index`、`0 <= point.point_index < point_count` を検査する。
- commit helper は F5d `gui_sfnt_simple_glyph_outline_storage_push_region_scalar` を 1 回だけ呼び、F5d error を `RegionPushFailed` に owner-preserving に包む。
- doctest は endpoint region を先に埋めてから PointX success、point index mismatch、wrong region を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。

完了条件:

- 2 contours / 4 points の storage に contour endpoint 1, 3 を追加した後、PointX point 0 と point 1 を追加でき、storage len が 4、cursor next index が 4 になる。
- PointX cursor が logical point 0 を指す状態で slot point_index 1 を渡すと `PointIndexMismatch` になり、storage len が 2 のまま回収できる。
- PointY cursor を渡した場合は `CursorRegionMismatch` になり、storage len が 2 のまま回収できる。
- source policy が capacity/cursor/point validation order、F5d region push 呼び出し回数、direct `vec::` 禁止、`gui_sfnt_glyf_` / point decode / render / raster / platform / host API 禁止、owner 型の非 Clone / 非 Copy を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_x.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_point_x.json -j 1
node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf.json -j 1
git diff --check
```

## Phase F5h: sfnt simple glyph point x byte reader bridge

目的:

- checked `GuiSfntSimpleGlyphPointStream` から 1 logical point の x coordinate だけを読み、F5g の `PointX` storage helper へ owner-preserving に接続する。
- byte-backed x read failure と F5g push failure の error domain を enum で分離する。
- y coordinate、endpoint array、contour span、edge/path command generation、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5h docs、read-before-mutate ordering、read failure と push failure の分離、owner 型の非 Clone / 非 Copy、x-only allowlist、full point / endpoint / render / platform 禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPointXReadPush`
  - `GuiSfntSimpleGlyphPointXReadPushErrorKind`
  - `GuiSfntSimpleGlyphPointXReadPushError`
  - `gui_sfnt_glyf_read_push_point_x`
- `GuiSfntSimpleGlyphPointXReadPush` と `GuiSfntSimpleGlyphPointXReadPushError` は storage owner を持つため `Clone` / `Copy` を実装しない。
- success payload には cursor accessor と storage owner accessor を追加する。
- x-only internal helper は bounded flag reads と `gui_sfnt_glyf_decode_x_delta` だけを使う。
- `gui_sfnt_glyf_decode_y_delta`、full point decode state、endpoint read、contour span helper、public lookup wrapper、direct `Vec`、render/raster/platform/host API は使わない。
- forged bad y range は F5h では検査しない。PointY / full point phase の責務として document する。
- read failure では F5g push を呼ばず、point は `None`、parse error は `Some`、storage len は変更しない。
- read success では `GuiSfntSimpleGlyphPointXSlot` を作り、F5g `gui_sfnt_simple_glyph_outline_storage_push_point_x` を 1 回だけ呼ぶ。
- F5g push failure では rejected point、F5g error kind、F5d region error kind、F5c storage push error kind を owner 消費前に読む。
- doctest は endpoint region を先に埋めてから PointX read/push success、read failure owner recovery、push failure endpoint preservation を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。

完了条件:

- 2 contours / 4 points の storage に contour endpoint 1, 3 を追加した後、byte-backed x reader から PointX point 0 と point 1 を追加でき、storage len が 4、cursor next index が 4 になる。
- x byte range が壊れた stream では `ReadFailed` になり、point は `None`、parse error は `Some`、storage len が 2 のまま回収できる。
- valid x read だが cursor が logical point 0 を指す状態で point_index 1 を push すると `PushFailed` になり、point は `Some`、lower F5g error kind が `Some PointIndexMismatch` になる。
- source policy が x-only allowlist、read-before-mutate、F5g push 呼び出し回数、lower error metadata の owner 消費前読み取り、direct `vec::` 禁止、full point / endpoint / render / raster / platform / host API 禁止、owner 型の非 Clone / 非 Copy を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_x_reader_success.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_point_x_reader_success_f5h.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_x_reader_read_failure.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_point_x_reader_read_failure_f5h.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_x_reader_push_failure.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_point_x_reader_push_failure_f5h.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5h.json -j 1
git diff --check
```

## Phase F5i: sfnt simple glyph point y coordinate population

目的:

- F5d の `PointY` region cursor を使い、typed y coordinate slot を owner-preserving に storage へ追加する。
- `PointY` region は endpoint と全 `PointX` slot の後ろにあるため、scalar storage index と glyph logical point index の混同を防ぐ。
- byte-backed y decode、point flag decode、x coordinate、edge/path command generation、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5i docs、PointY slot type、success/error owner payload、validation order、owner 型の非 Clone / 非 Copy、禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPointYSlot`
  - `GuiSfntSimpleGlyphPointYPush`
  - `GuiSfntSimpleGlyphPointYPushErrorKind`
  - `GuiSfntSimpleGlyphPointYPushError`
  - `gui_sfnt_simple_glyph_outline_storage_push_point_y`
- public helper は capacity shape と scalar slot count `Fits` を検査してから `point_count` を読む。
- cursor well-formed validation と cursor/capacity boundary match は `logical_point_index = cursor.next_index - cursor.start` より前に行う。
- `PointY` region であることを確認し、`point.point_index == logical_point_index`、`0 <= point.point_index < point_count` を検査する。
- commit helper は F5d `gui_sfnt_simple_glyph_outline_storage_push_region_scalar` を 1 回だけ呼び、F5d error を `RegionPushFailed` に owner-preserving に包む。
- doctest は endpoint 2 slots と PointX 4 slots を先に埋めてから PointY success、point index mismatch、wrong region を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。

完了条件:

- 2 contours / 4 points の storage に endpoint 2 slots と PointX 4 slots を追加した後、PointY point 0 と point 1 を追加でき、storage len が 8、cursor next index が 8 になる。
- PointY cursor が logical point 0 を指す状態で slot point_index 1 を渡すと `PointIndexMismatch` になり、storage len が 6 のまま回収できる。
- PointX cursor など wrong region を渡した場合は `CursorRegionMismatch` になり、storage len が 6 のまま回収できる。
- source policy が capacity/cursor/point validation order、F5d region push 呼び出し回数、direct `vec::` 禁止、`gui_sfnt_glyf_` / point decode / render / raster / platform / host API 禁止、owner 型の非 Clone / 非 Copy、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_y.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_point_y_f5i.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5j.json -j 1
git diff --check
```

## Phase F5j: sfnt simple glyph point y byte reader bridge

目的:

- checked `GuiSfntSimpleGlyphPointStream` から 1 logical point の y coordinate だけを読み、F5i の `PointY` storage helper へ owner-preserving に接続する。
- byte-backed y read failure と F5i push failure の error domain を enum で分離する。
- x coordinate、endpoint array、contour span、edge/path command generation、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5j docs、read-before-mutate ordering、read failure と push failure の分離、owner 型の非 Clone / 非 Copy、y-only allowlist、full point / endpoint / render / platform 禁止 API を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPointYReadPush`
  - `GuiSfntSimpleGlyphPointYReadPushErrorKind`
  - `GuiSfntSimpleGlyphPointYReadPushError`
  - `gui_sfnt_glyf_read_push_point_y`
- `GuiSfntSimpleGlyphPointYReadPush` と `GuiSfntSimpleGlyphPointYReadPushError` は storage owner を持つため `Clone` / `Copy` を実装しない。
- success payload には cursor accessor と storage owner accessor を追加する。
- y-only internal helper は bounded flag reads と `gui_sfnt_glyf_decode_y_delta` だけを使う。
- `gui_sfnt_glyf_decode_x_delta`、full point decode state、endpoint read、contour span helper、public lookup wrapper、direct `Vec`、render/raster/platform/host API は使わない。
- forged bad x range は F5j では検査しない。PointX / full point phase の責務として document する。
- read failure では F5i push を呼ばず、point は `None`、parse error は `Some`、storage len は変更しない。
- read success では `GuiSfntSimpleGlyphPointYSlot` を作り、F5i `gui_sfnt_simple_glyph_outline_storage_push_point_y` を 1 回だけ呼ぶ。
- F5i push failure では rejected point、F5i error kind、F5d region error kind、F5c storage push error kind を owner 消費前に読む。
- doctest は endpoint 2 slots と PointX 4 slots を先に埋めてから PointY read/push success、read failure owner recovery、push failure point preservation を追加する。
- 実装後に subagent review を受け、指摘があれば修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- bad x range だが valid y range を持つ forged stream から PointY point 0 と point 1 を追加でき、storage len が 8、cursor next index が 8 になる。
- y byte range が壊れた stream では `ReadFailed` になり、point は `None`、parse error は `Some`、storage len が 6 のまま回収できる。
- valid y read だが cursor が logical point 0 を指す状態で point_index 1 を push すると `PushFailed` になり、point は `Some`、lower F5i error kind が `Some PointIndexMismatch` になる。
- source policy が y-only allowlist、read-before-mutate、F5i push 呼び出し回数、lower error metadata の owner 消費前読み取り、direct `vec::` 禁止、full point / endpoint / render / raster / platform / host API 禁止、owner 型の非 Clone / 非 Copy、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_y.n.md --no-tree -o tmp_gui_font_sfnt_glyf_outline_point_y_f5j.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5j.json -j 1
git diff --check
```

## Phase F5k: sfnt simple glyph outline point coordinate read

目的:

- F5b-F5j で population 済みの `PointX` / `PointY` scalar slot から、1 logical point の coordinate pair を read-only に取得する。
- `GuiSfntSimpleGlyphPoint` の `on_curve` / `end_of_contour` はまだ F5 storage に存在しないため、この phase では full point value を返さない。
- storage readiness、slot boundary、typed error を固定し、fallback coordinate、byte decode 再実行、renderer/rasterizer/platform 依存へ進まない。

変更:

- 先に source policy を追加し、F5k docs、private scalar getter、coordinate value、typed read error、validation order、禁止 API、括弧なし prefix style を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointCoordinate`
  - `GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointCoordinateReadError`
  - private `gui_sfnt_simple_glyph_outline_storage_scalar_slot_get`
  - `gui_sfnt_simple_glyph_outline_storage_read_point_coordinate`
- raw scalar slot getter は private にし、`vec::get` をここへ閉じ込める。unchecked public slot accessor は作らない。
- public read helper は storage owner を borrow し、storage を mutate しない。
- validation は次の順序で行う。
  - capacity shape
  - scalar slot count `Fits`
  - `storage.scalar_slot_count == expected`
  - `scalar_slots_cap == storage.scalar_slot_count`
  - `0 <= point_index < point_count`
  - `scalar_slots_len > y_slot_index`
  - private getter で x slot と y slot を読む
- `scalar_slots_len <= y_slot_index` は `CoordinateNotReady` として扱う。`scalar_slots_len > y_slot_index` で readiness が確認された後に private getter が `None` を返した場合は `ScalarSlotMissing` とする。
- `GuiSfntSimpleGlyphOutlinePointCoordinate` と read error は value-only なので `Clone` / `Copy` を実装してよい。
- doctest は既存 owner-preserving push API で endpoint、PointX、PointY を順に埋め、success、out-of-range、missing PointY readiness を検査する。
- 実装前 plan review と実装後 implementation review を subagent で受け、指摘があれば修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- 2 contours / 4 points の storage に endpoint 2 slots、PointX 4 slots、PointY 4 slots を追加した後、point 0 と point 1 の coordinate pair を読める。
- `point_index == point_count` は `PointIndexOutOfRange` になる。
- endpoint と PointX までしか埋まっていない storage では `CoordinateNotReady` になり、zero coordinate や byte decode fallback を返さない。
- source policy が F5k docs、value/error 型、private `vec::get` helper、public helper validation order、direct `vec::` 禁止、byte/full point/endpoint/path/render/raster/platform/host API 禁止、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_coordinate.n.md --no-tree -o tmp_gui_font_outline_point_coordinate_f5k.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5k.json -j 1
git diff --check
```

## Phase F5l: sfnt simple glyph outline point endpoint marker read

目的:

- F5e/F5f で population 済みの `ContourEndpoint` scalar region から、1 logical point が属する contour と end-of-contour marker を read-only に取得する。
- endpoint topology 全体を検査してから成功し、partial success や hidden fallback を作らない。
- flag byte、x/y coordinate、full point value、edge/path、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5l docs、endpoint marker value、typed read error、全 endpoint scan、final endpoint check、禁止 API、括弧なし prefix style を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointEndpointMarker`
  - `GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadError`
  - private scan helper
  - `gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker`
- public read helper は storage owner を borrow し、storage を mutate しない。
- validation は次の順序で行う。
  - capacity shape
  - scalar slot count `Fits`
  - `storage.scalar_slot_count == expected`
  - `scalar_slots_cap == storage.scalar_slot_count`
  - `0 <= point_index < point_count`
  - `scalar_slots_len >= contour_count`
  - private getter で endpoint slot を contour 0 から final contour まで順に読む
- scan helper は `found` state を持ち、最初に `point_index <= endpoint` になった contour / end flag を記録する。ただしそこで成功を返さず、final contour まで endpoint range、strict increase、final endpoint `point_count - 1` を検査する。
- read helper は direct `Vec` API を呼ばず、F5k の private scalar slot getter を再利用する。
- `GuiSfntSimpleGlyphOutlinePointEndpointMarker` と read error は value-only なので `Clone` / `Copy` を実装してよい。
- doctest は既存 owner-preserving endpoint push API で success、out-of-range、not-ready を検査し、direct region push で forged `[1, 2]` endpoint topology を作って `EndpointTopologyInvalid` を検査する。
- 実装前 plan review と実装後 implementation review を subagent で受け、指摘があれば修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- endpoint `[1, 3]` の 2 contours / 4 points storage から、point 0/1/2/3 の contour index と end-of-contour marker を読める。
- `point_index == point_count` は `PointIndexOutOfRange` になる。
- endpoint region が空の storage では `EndpointNotReady` になる。
- forged endpoint `[1, 2]` では point 0 でも success にならず、`EndpointTopologyInvalid` になる。
- source policy が F5l docs、value/error 型、全 endpoint scan、final endpoint `point_count - 1` before success、direct `vec::` 禁止、byte/full point/coordinate/path/render/raster/platform/host API 禁止、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_endpoint.n.md --no-tree -o tmp_gui_font_outline_point_endpoint_f5l.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5l.json -j 1
git diff --check
```

## Phase F5m: sfnt simple glyph point flag marker read

目的:

- checked `GuiSfntSimpleGlyphPointStream` の flag range だけから、1 logical point の raw flag と on-curve marker を read-only に取得する。
- F5 storage scalar layout には新しい `PointFlag` region を追加しない。既存 region boundary をこの phase で動かすと F5b から F5l の slot contract が崩れるためである。
- x/y coordinate decode、full point decode、endpoint read、coordinate storage read、edge/path、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5m docs、flag marker value、repeat run overrun before success、禁止 API、括弧なし prefix style を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphPointFlagMarker`
  - `gui_sfnt_glyf_read_point_flag_from_stream`
  - private flag run scan helper
- read helper は storage owner を持たず、`Result GuiSfntSimpleGlyphPointFlagMarker GuiSfntParseError` を返す。
- error は既存 parse error を使う。
  - `point_index` out of range は `MissingGlyphOutline`
  - missing repeat byte、repeat overrun、scan exhaustion は `MalformedGlyfRecord`
- validation / scan は次の順序で行う。
  - stream topology から `point_count` と glyph を読む
  - `0 <= point_index < point_count`
  - `flag_cursor = flag_data_offset`、`logical_index = 0`
  - flag byte を `gui_sfnt_glyf_read_u8_in_stream_range` で読む
  - repeat bit 8 がある場合は repeat count byte を同じ range helper で読む
  - `run_count = repeat_count + 1` または `1`
  - `logical_index + run_count <= point_count` を検査する
  - overrun ではない場合だけ target が run 内かを判定する
  - target が run 内なら raw flag と `gui_sfnt_glyf_flag_has_bit flag 1` を marker として返す
- doctest は no-repeat on/off curve、repeat run、out-of-range、repeat overrun、missing repeat byte を検査する。
- 実装前 plan review と実装後 implementation review を subagent で受け、指摘があれば修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- no-repeat flag stream から on-curve / off-curve marker を読める。
- repeat bit 8 の run 内 target が同じ raw flag と on-curve marker を返す。
- `point_index == point_count` は `MissingGlyphOutline` になる。
- repeat run が `point_count` を越える場合、target が run 内にあっても `MalformedGlyfRecord` になる。
- repeat bit があるのに repeat count byte が range 外なら `MalformedGlyfRecord` になる。
- source policy が F5m docs、value 型、repeat overrun before marker success、x/y decode/full point/endpoint/coordinate storage/path/render/raster/platform/host API 禁止、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_flag.n.md --no-tree -o tmp_gui_font_outline_point_flag_f5m.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5m.json -j 1
git diff --check
```

## Phase F5n: sfnt simple glyph outline point read composition

目的:

- F5k coordinate、F5l endpoint marker、F5m flag marker を合成し、既存 `GuiSfntSimpleGlyphPoint` を read-only に作る。
- storage と stream の shared precondition を component read より前に検査し、要求範囲の失敗を component error に潰さない。
- edge/path storage、outline stream、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5n docs、error kind、shared precondition order、F5k -> F5l -> F5m の exact one-call order、禁止 API、括弧なし prefix style を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointReadErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointReadError`
  - `gui_sfnt_simple_glyph_outline_storage_read_point`
- error kind は次を持つ。
  - `StorageCapacityInvalid`
  - `StorageStreamGlyphMismatch`
  - `StorageStreamContourCountMismatch`
  - `StorageStreamPointCountMismatch`
  - `PointIndexOutOfRange`
  - `CoordinateReadFailed`
  - `EndpointMarkerReadFailed`
  - `FlagReadFailed`
  - `ComponentGlyphMismatch`
  - `ComponentPointIndexMismatch`
- error payload は requested `point_index`、storage capacity、stream topology、coordinate / endpoint / flag の optional sub-error を保持する。
- validation / compose は次の順序で行う。
  - storage capacity と stream topology を読む
  - capacity shape を検査する
  - glyph、contour_count、point_count が一致することを検査する
  - `0 <= point_index < shared_point_count` を検査する
  - F5k coordinate read を 1 回だけ呼ぶ
  - F5l endpoint marker read を 1 回だけ呼ぶ
  - F5m flag marker read を 1 回だけ呼ぶ
  - component glyph / point_index を fail-closed に再検査する
  - coordinate.x/y、flag.on_curve、endpoint.end_of_contour から `GuiSfntSimpleGlyphPoint` を作る
- doctest は success、storage/stream glyph mismatch、top-level point-index out-of-range、coordinate not-ready wrapping、endpoint topology invalid wrapping、flag repeat-overrun wrapping を検査する。
- 実装前 plan review と実装後 implementation review を subagent で受け、指摘があれば修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- valid storage + stream から x/y、on-curve、end-of-contour を含む `GuiSfntSimpleGlyphPoint` を返せる。
- storage と stream の glyph / contour_count / point_count mismatch は component read 前に top-level error になる。
- `point_index == point_count` は `CoordinateReadFailed` ではなく `PointIndexOutOfRange` になる。
- coordinate read failure、endpoint marker failure、flag read failure はそれぞれ別 error kind と optional sub-error で保持される。
- source policy が F5n docs、error 型、shared precondition before component reads、F5k -> F5l -> F5m の exact one-call order、direct `vec::` / scalar getter / lower loop / x/y decode / endpoint scan / flag scan / path / render / raster / platform / host API 禁止、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_read.n.md --no-tree -o tmp_gui_font_outline_point_read_f5n.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5n.json -j 1
git diff --check
```

## Phase F5o: sfnt simple glyph outline point read step

目的:

- F5n の single point read を、allocation なしの cursor step として反復できるようにする。
- `cursor.next_point_index == point_count` を正常終端 `End` として表し、`point_count` を越える cursor は `CursorOutOfRange` として返す。
- 終端成功を返す前に storage / stream の shared precondition を検査し、forged mismatch を終端として隠さない。
- Vec、edge/path storage、outline stream、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5o docs、cursor / status / step / error 型、shared precondition order、terminal branch before F5n、F5n exact one-call、禁止 API、括弧なし prefix style を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointReadCursor`
  - `GuiSfntSimpleGlyphOutlinePointReadStepStatus`
  - `GuiSfntSimpleGlyphOutlinePointReadStep`
  - `GuiSfntSimpleGlyphOutlinePointReadStepErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointReadStepError`
  - `gui_sfnt_simple_glyph_outline_storage_read_point_step`
- error kind は次を持つ。
  - `StorageCapacityInvalid`
  - `StorageStreamGlyphMismatch`
  - `StorageStreamContourCountMismatch`
  - `StorageStreamPointCountMismatch`
  - `CursorOutOfRange`
  - `PointReadFailed`
- error payload は requested cursor、storage capacity、stream topology、optional F5n point error を保持する。
- validation / step は次の順序で行う。
  - storage capacity と stream topology を読む
  - capacity shape を検査する
  - glyph、contour_count、point_count が一致することを検査する
  - `point_index = cursor.next_point_index` を読む
  - `point_index < 0` または `point_index > shared_point_count` なら `CursorOutOfRange`
  - `point_index == shared_point_count` なら `point None` の `End` step を返す
  - `0 <= point_index < shared_point_count` の場合だけ F5n point read を 1 回だけ呼ぶ
  - F5n の失敗は `PointReadFailed` と optional point error で保持する
  - F5n の成功値を `point Some` に入れ、`next_cursor = point_index + 1` の `Point` step を返す
- doctest は first point success、last point success with `next_cursor == point_count`、terminal End with point None、cursor too far、F5n flag failure wrapping を検査する。
- 実装前 plan review と実装後 implementation review を subagent で受け、指摘があれば修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- valid storage + stream + cursor から `Point` step と `End` step を区別して返せる。
- `Point` step は `Some GuiSfntSimpleGlyphPoint`、`End` step は `None` を保持する。
- `point_index == point_count` は F5n の `PointIndexOutOfRange` に落とさず、F5o の正常終端として返す。
- `point_index > point_count` と負の cursor は `CursorOutOfRange` になる。
- storage と stream の glyph / contour_count / point_count mismatch は終端判定より前に top-level error になる。
- F5n の失敗は `PointReadFailed` と optional sub-error で保持される。
- source policy が F5o docs、error 型、shared precondition before terminal, terminal branch before F5n, F5n exact one-call、direct F5k/F5l/F5m / lower loop / `vec::` / path / render / raster / platform / host API 禁止、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_step.n.md --no-tree -o tmp_gui_font_outline_point_step_f5o.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5o.json -j 1
git diff --check
```

## Phase F5p: sfnt simple glyph outline point read drain budget

目的:

- F5o の point step を明示 budget 内で正常終端まで進める no-allocation drain boundary を追加する。
- `End` と `StepBudgetExhausted` を success enum で分け、budget exhaustion を silent success や error にしない。
- terminal check を budget check より前に置き、terminal cursor は budget 0 でも `End` にする。
- non-terminal かつ budget exhausted の場合は F5o を呼ばず、hidden point read work をしない。
- Vec、edge/path storage、outline stream、rasterizer、renderer、platform API、host text API へ進まない。

変更:

- 先に source policy を追加し、F5p docs、summary / drain / error 型、terminal-before-budget、budget-before-F5o、F5o exact one-call、Point Some before count increment、invariant failure、禁止 API、括弧なし prefix style を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointReadDrainSummary`
  - `GuiSfntSimpleGlyphOutlinePointReadDrain`
  - `GuiSfntSimpleGlyphOutlinePointReadDrainErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointReadDrainError`
  - private validation context / validation helper
  - `gui_sfnt_simple_glyph_outline_storage_read_point_drain_budget`
- error kind は次を持つ。
  - `StorageCapacityInvalid`
  - `StorageStreamGlyphMismatch`
  - `StorageStreamContourCountMismatch`
  - `StorageStreamPointCountMismatch`
  - `CursorOutOfRange`
  - `StepReadFailed`
  - `StepInvariantInvalid`
- error payload は cursor、storage capacity、stream topology、optional F5o step error、optional F5o step value を保持する。
- validation / drain は次の順序で行う。
  - storage capacity と stream topology を読む
  - capacity shape を検査する
  - glyph、contour_count、point_count が一致することを検査する
  - `point_index = cursor.next_point_index` を読む
  - `point_index < 0` または `point_index > shared_point_count` なら `CursorOutOfRange`
  - `point_index == shared_point_count` なら `End summary` を返す
  - non-terminal かつ `remaining_steps <= 0` なら `StepBudgetExhausted summary` を返す
  - non-terminal かつ budget positive の場合だけ F5o point step を 1 回だけ呼ぶ
  - F5o `Err` は `StepReadFailed` と optional step error で保持する
  - F5o `Ok Point` かつ `point Some` で、next cursor が現在 cursor から 1 点分だけ前進した場合だけ `points_read + 1`、`last_point Some point` へ進める
  - F5o `Ok Point` かつ `point None`、next cursor が `current + 1` ではない Point、または F5o `Ok End` は `StepInvariantInvalid` と optional step value で fail-closed にする
- 実装は recursive helper ではなく、local mutable state を持つ bounded `while` body にする。これは current NEPLg2.1 codegen で runtime doctest timeout を起こさず、将来の time-slice scheduling とも合わせやすい。
- doctest は full drain End、partial budget exhausted、zero budget non-terminal、zero budget terminal、cursor too far、F5o/F5n flag repeat-overrun wrapping を検査する。
- 実装前 plan review と実装後 implementation review を subagent で受け、指摘があれば修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- valid storage + stream + cursor + sufficient budget から `End summary` を返し、cursor、points_read、last_point を確認できる。
- non-terminal budget exhaustion は F5o を呼ばず `StepBudgetExhausted summary` を返す。
- terminal cursor は budget 0 でも `End summary` を返す。
- F5o read failure は `StepReadFailed` で保持される。
- impossible F5o success shape は `StepInvariantInvalid` で fail-closed になる。
- F5o `Point` が cursor を 1 点分だけ前進させない場合も `StepInvariantInvalid` で fail-closed になる。
- source policy が F5p docs、error 型、terminal-before-budget、budget-before-F5o、F5o exact one-call、direct F5n/F5k/F5l/F5m / lower loop / `vec::` / path / render / raster / platform / host API 禁止、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_drain.n.md --no-tree -o tmp_gui_font_outline_point_drain_f5p.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5p.json -j 1
git diff --check
```

## Phase F5q: sfnt simple glyph outline point stream item classification

目的:

- F5p で読める full point を、後続 outline stream / contour / path phase が O(1) value として読むための no-allocation item boundary を追加する。
- `on_curve` と `end_of_contour` の組み合わせを `GuiSfntSimpleGlyphOutlinePointStreamItemKind` に分類し、後段が bool field を重複解釈しないようにする。
- `EndOnCurve` / `EndOffCurve` を top-level variant として持ち、contour endpoint を silent flag にしない。
- ByteBuf、SFNT lookup、storage、F5p drain loop、Vec、path、raster、render、platform、host text API へ進まない。

変更:

- 先に source policy を追加し、F5q docs、item kind / item 型、classification order、constructor exact one classification、禁止 API、括弧なし prefix style を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointStreamItemKind`
  - `GuiSfntSimpleGlyphOutlinePointStreamItem`
  - `gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point`
  - `gui_sfnt_simple_glyph_outline_point_stream_item`
  - `gui_sfnt_simple_glyph_outline_point_stream_item_point`
  - `gui_sfnt_simple_glyph_outline_point_stream_item_kind`
- item kind は次を持つ。
  - `OnCurve`
  - `OffCurve`
  - `EndOnCurve`
  - `EndOffCurve`
- constructor は外部 kind を受け取らず、`gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point` を 1 回だけ呼んで kind を導く。
- classification は次の順序で行う。
  - `on_curve = gui_sfnt_simple_glyph_point_on_curve point`
  - `end_of_contour = gui_sfnt_simple_glyph_point_end_of_contour point`
  - `end_of_contour` が true なら `EndOnCurve` / `EndOffCurve`
  - `end_of_contour` が false なら `OnCurve` / `OffCurve`
- doctest は synthetic `GuiSfntSimpleGlyphPoint` を使い、4 分類と accessors を検査する。
- 実装前 plan review と実装後 implementation review を subagent review として受け、指摘があれば修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- 4 種類の `on_curve` / `end_of_contour` combination が enum variant に分類される。
- endpoint point は通常 `OnCurve` / `OffCurve` ではなく `EndOnCurve` / `EndOffCurve` になる。
- item constructor は point と分類済み kind を同時に保持し、外部 kind の不整合を受け付けない。
- source policy が F5q docs、API、classification order、constructor exact one classification、ByteBuf / `GuiSfntSimpleGlyphPointStream` / storage / drain / `gui_sfnt_glyf_` / `gui_sfnt_lookup_` / `Vec` / path / render / raster / platform / host API 禁止、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_f5q.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5q.json -j 1
git diff --check
```

## Phase F5r: sfnt simple glyph outline point stream item step from point step

目的:

- F5o の `GuiSfntSimpleGlyphOutlinePointReadStep` を、F5q の `GuiSfntSimpleGlyphOutlinePointStreamItem` を持つ no-allocation step value に変換する。
- `Point` / `End` の成功 shape を再検査し、公開 constructor で作れる不正 step を `PointStepInvariantInvalid` で fail-closed にする。
- kind classification は F5q constructor に閉じ込め、F5r では分類を重複実装しない。
- ByteBuf、SFNT lookup、storage、F5p drain loop、Vec、path、raster、render、platform、host text API へ進まない。

変更:

- 先に source policy を追加し、F5r docs、item step 型、error 型、visible cursor invariant、F5q constructor exactly once、F5q kind helper 直接呼び出し禁止、括弧なし prefix style を固定する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemStep`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemStepErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemStepError`
  - `gui_sfnt_simple_glyph_outline_point_stream_item_step`
  - `gui_sfnt_simple_glyph_outline_point_stream_item_step_*` accessors
  - `gui_sfnt_simple_glyph_outline_point_stream_item_step_from_point_step`
- `status = Point` の変換は次だけを受け付ける。
  - `point = Some point`
  - `next_cursor.next_point_index == cursor.next_point_index + 1`
  - F5q constructor `gui_sfnt_simple_glyph_outline_point_stream_item` を 1 回だけ呼ぶ
  - `Item` step と `Some item` を返す
- `status = End` の変換は次だけを受け付ける。
  - `point = None`
  - `next_cursor.next_point_index == cursor.next_point_index`
  - `End` step と `None` を返す
- 上記以外の `Point` / `End` shape は `PointStepInvariantInvalid` を返す。
- doctest は byte fixture を使わず、synthetic `GuiSfntSimpleGlyphOutlinePointReadStep` で normal Point、normal End、Point None、End Some、Point bad cursor、End bad cursor を検査する。
- Tesla plan review は 1 回目 `PLAN_BLOCKED`。F5r が F5o step の visible invariant を再検査すること、F5q kind helper を直接呼ばず constructor だけを使うことが blocker として指摘された。
- 計画を修正し、`Point` / `End` の cursor invariant と `point` option invariant を `PointStepInvariantInvalid` で検査する方針にした。
- 実装後は subagent implementation review を受け、指摘があれば source policy、stdlib、doctest、文書を修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- `Point + Some point + next = cursor + 1` だけが `Item + Some item` になる。
- `End + None + next = cursor` だけが `End + None` になる。
- `Point + None`、`End + Some`、`Point` の bad cursor、`End` の bad cursor は `PointStepInvariantInvalid` になる。
- F5r helper は `gui_sfnt_simple_glyph_outline_point_stream_item` を successful Point branch で exactly once 呼び、`gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point` を直接呼ばない。
- source policy が F5r docs、API、cursor invariant、F5q constructor exact one-call、F5q kind helper 直接呼び出し禁止、ByteBuf / `GuiSfntSimpleGlyphPointStream` / storage / drain / `gui_sfnt_glyf_` / `gui_sfnt_lookup_` / `Vec` / path / render / raster / platform / host API 禁止、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_step.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_step_f5r.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5r.json -j 1
git diff --check
```

## Phase F5s: sfnt simple glyph outline point stream item drain

目的:

- F5o の point step と F5r の classified item step conversion を、明示的な step budget 内で進める no-allocation drain boundary を追加する。
- F5p と同じ shared cursor precondition を使うが、F5s は F5p public drain や F5p error conversion に依存しない。
- terminal-before-budget、budget-before-F5o、F5o exactly once、F5r exactly once の順序を source policy と doctest で固定する。
- F5o read failure、F5r conversion failure、F5s defensive invariant failure を別の error kind として保持する。
- full point `Vec`、item list、sink mutation、path、raster、render、platform、host text API へ進まない。

変更:

- 先に source policy と実装計画を subagent にレビューさせた。
- Tesla plan review は 1 回目 `PLAN_BLOCKED`。F5s が F5p private validation に直接依存する案は phase coupling として不適切であり、F5p/F5s 共通の neutral validation helper が必要と指摘された。
- 計画を修正し、`GuiSfntSimpleGlyphOutlinePointReadCursorValidation`、`GuiSfntSimpleGlyphOutlinePointReadCursorValidationRejectKind`、`GuiSfntSimpleGlyphOutlinePointReadCursorValidationReject`、`gui_sfnt_simple_glyph_outline_point_read_cursor_validate` を private shared helper として追加する。
- 修正後の計画は Tesla review で `PLAN_APPROVED`。neutral helper は private で byte/path/render-free とし、F5p/F5s はそれぞれ phase-specific error へ変換する方針で実装を開始する。
- F5p の `gui_sfnt_simple_glyph_outline_point_read_drain_validate` は neutral helper の reject を F5p error kind へ変換し、既存 public drain behavior を維持する。
- F5s は neutral helper の reject を F5s error kind へ変換し、F5p public drain を呼ばない。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointStreamItemDrainSummary`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemDrain`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemDrainErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemDrainError`
  - `gui_sfnt_simple_glyph_outline_point_stream_item_drain_summary`
  - `gui_sfnt_simple_glyph_outline_point_stream_item_drain_summary_*` accessors
  - `gui_sfnt_simple_glyph_outline_point_stream_item_drain_error`
  - `gui_sfnt_simple_glyph_outline_point_stream_item_drain_error_*` accessors
  - `gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget`
- doctest は full End、partial budget exhausted、zero budget non-terminal、zero budget terminal、cursor out of range、wrapped F5o read failure を検査する。
- defensive branch の `ItemStepConvertFailed` と `ItemStepInvariantInvalid` は削らない。前者は F5r が拒否した sub-error、後者は F5r 成功値を F5s が再検査した fail-closed branch である。
- 実装後は subagent implementation review を受け、指摘があれば source policy、stdlib、doctest、文書を修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- F5p/F5s が shared cursor validation helper を共有し、F5s は F5p public drain を呼ばない。
- terminal cursor は budget 0 でも `End` になる。
- non-terminal かつ budget 0 は F5o/F5r を呼ばず `StepBudgetExhausted` になる。
- non-terminal かつ budget positive では F5o point step を exactly once 呼び、その成功値を F5r conversion に exactly once 渡す。
- `F5o Err` は `PointStepReadFailed`、`F5r Err` は `ItemStepConvertFailed`、F5r success shape の defensive mismatch は `ItemStepInvariantInvalid` になる。
- source policy が F5s docs、neutral validation reuse、F5p public drain 非依存、F5o/F5r exact one-call、F5q kind helper 直接呼び出し禁止、direct Vec/path/render/raster/platform/host API 禁止、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_drain_f5s.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_drain.n.md --no-tree -o tmp_gui_font_outline_point_drain_f5s_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5s.json -j 1
git diff --check
```

## Phase F5t: sfnt simple glyph outline point stream item collection owner

目的:

- F5s の classified item stream を後続 phase が owner として保持できる allocator-backed collection boundary を追加する。
- 今回は F5s drain-to-collection loop には進まず、empty collection allocation、single item push、single item read の contract だけを固定する。
- F5b scalar slot storage limit と item collection limit を混ぜず、F5t 専用 limit を導入する。
- push では public constructor で forged item が作れることを前提に、glyph、point index、kind を mutation 前に再検査する。
- read は `Option` ではなく typed `Result` とし、invariant failure、out-of-range、missing slot を区別する。
- path、raster、render、platform、host text API へ進まない。

変更:

- 先に source policy と実装計画を subagent にレビューさせた。
- Tesla plan review は 1 回目 `PLAN_BLOCKED`。既存 `GuiSfntSimpleGlyphOutlineStorageLimit` の流用、item kind 再検証不足、`item_at Option` による invariant failure の隠蔽、lower `StdErrorKind` の欠落、F5s/F5r/F5o/F5p 非依存の明文化不足が指摘された。
- 計画を修正し、F5t 専用 `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit`、`ItemKindMismatch`、typed read error、push error の `storage_error Option StdErrorKind`、F5s/F5r/F5o/F5p 直接呼び出し禁止を追加した。
- 修正後の計画は Tesla review で `PLAN_APPROVED`。実装開始可とされた。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollection`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocError`
  - allocation error constructors/accessors
  - collection observers/free
  - `gui_sfnt_simple_glyph_outline_point_stream_item_kind_is`
  - `gui_sfnt_simple_glyph_outline_point_stream_item_kind_matches_point`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushError`
  - push error constructors/accessors
  - `gui_sfnt_simple_glyph_outline_point_stream_item_collection_push`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadError`
  - read error constructors/accessors
  - `gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item`
- allocation order は `capacity shape`、`max_items > 0`、`point_count <= max_items`、`vec::with_capacity point_count` とする。
- free は collection owner を消費し、内部 `items` に対して `vec::free` を exactly once 呼ぶ。
- push order は `capacity shape`、`len == item_count`、`cap == point_count`、`item_count < point_count`、glyph/index/kind 検査、`vec::push` exactly once とする。
- read order は `capacity shape`、`len == item_count`、`cap == point_count`、index range、`vec::get` exactly once とする。
- doctest は alloc success、invalid capacity、invalid limit、limit reject、push/read success、glyph mismatch、index mismatch、kind mismatch、collection full、public read out-of-range を検査する。
- read length mismatch、read capacity mismatch、missing slot は owner-backed collection constructor の外部利用制限により doctest から forged owner を作らず、source policy で typed branch と実装順序を固定する。
- 実装後は subagent implementation review を受け、指摘があれば source policy、stdlib、doctest、文書を修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- F5t 専用 limit があり、F5b scalar slot storage limit を使わない。
- collection owner と push error は owner-bearing payload なので `Clone` / `Copy` を実装しない。
- `Vec` capacity は `capacity.point_count` であり、scalar slot count ではない。
- push failure では collection owner、rejected item、typed error kind、lower `StdErrorKind` option が失われない。
- `vec::vec_push_error_kind &e` を `vec::vec_push_error_vec e` より前に読む。
- item kind は item payload を信頼せず、F5q `kind_from_point` で再導出して検査する。
- public read は `Option` ではなく typed `Result` を返す。
- source policy が F5t docs、専用 limit、allocation/push/read order、owner-bearing payload 非 Clone / 非 Copy、F5s/F5r/F5o/F5p drain 非依存、lower byte reader/path/render/raster/platform/host API 禁止、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_f5t.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5t.json -j 1
git diff --check
```

## Phase F5u: sfnt simple glyph outline point stream item collection drain

目的:

- F5s の classified item stream を F5t の collection owner へ owner-preserving に commit する。
- F5s は `last_item` しか返さないため、F5u は F5s を caller budget でまとめて呼ばず、0 / 1 step budget の反復だけで進める。
- collection owner を success/error のどちらでも失わず、push failure では lower push metadata と rejected item を保持する。
- path、raster、render、platform、host text API へ進まない。

変更:

- 先に source policy と実装計画を subagent にレビューさせた。
- Tesla plan review は 1 回目 `PLAN_BLOCKED`。F5s success invariant failure 時に lower F5s success value を error payload に保持すること、F5s budget を 0 / 1 に固定すること、push error metadata を owner 回収前に読むこと、public API だけで push failure doctest が可能かを明確にすることが指摘された。
- 計画を修正し、`item_drain_result Option GuiSfntSimpleGlyphOutlinePointStreamItemDrain`、local `step_budget` 0 / 1、push error kind / storage error / rejected item の回収順序、collection capacity 1 と stream point count 4 による public `CollectionFull` doctest を追加した。
- 実装中に、terminal cursor と空 collection のような不整合が成功値にならないよう、`collection.item_count == cursor.next_point_index` の precondition と `CollectionCursorMismatch` を追加した。
- 修正後の計画は Tesla review で `PLAN_APPROVED`。F5u は F5s drain と F5t push だけを呼び、lower step/point/path/render/platform API へ進まない方針で実装を開始する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainSummary`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainError`
  - summary constructor/accessors
  - error constructor/accessors
  - `gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget`
- doctest は full End、partial budget exhausted、zero budget non-terminal、zero budget terminal、lower F5s failure wrapping、public `CollectionPushFailed` via `CollectionFull` を検査する。
- 実装後は subagent implementation review を受け、指摘があれば source policy、stdlib、doctest、文書を修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- F5u summary/error は collection owner を含むため `Clone` / `Copy` を実装しない。
- F5u は `collection.item_count == cursor.next_point_index` を F5s 呼び出し前に検査し、不一致なら `CollectionCursorMismatch` として owner を返す。
- F5u は `step_budget` を 0 / 1 だけにし、F5s に caller `remaining_steps` を直接渡さない。
- F5u public body は F5s drain を source 上 exactly once 呼び、その引数に `step_budget` を渡す。
- F5u public body は F5t collection push を source 上 exactly once 呼ぶ。
- F5s `Err` は `ItemDrainFailed` になり、collection owner と lower error が失われない。
- F5s `Ok` で `items_read` が 0 / 1 以外、budget 0 で item read、または `last_item None` の item read は `ItemDrainInvariantInvalid` になり、lower F5s success value が `item_drain_result` に残る。
- F5t push failure では `push_error_kind`、`push_storage_error`、`rejected_item` を owner 回収前に読み、`CollectionPushFailed` として collection owner を返す。
- source policy が F5u docs、budget 0 / 1、F5s exact one-call、F5t push exact one-call、push metadata before owner recovery、owner-bearing payload 非 Clone / 非 Copy、lower point/byte/path/render/raster/platform/host API 禁止、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_drain_f5u.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_f5u_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5u.json -j 1
git diff --check
```

## Phase F5v: sfnt simple glyph outline point stream item collection contour span

目的:

- F5u/F5t の classified item collection owner から、byte-backed F4 helper に戻らず `GuiSfntSimpleGlyphContourSpan` を導出する。
- partial collection、forged item、endpoint topology mismatch を typed `Result` で拒否する。
- collection-backed contour point / edge / path population の前段として、contour span の authority を collection に固定する。
- path、raster、render、platform、host text API へ進まない。

変更:

- 先に source policy と実装計画を subagent にレビューさせた。
- Tesla plan review は 1 回目 `PLAN_BLOCKED`。`observed_contour_count == contour_count` だけでは endpoint `[1, 2]`、`point_count = 4` のような forged topology で最終 point がどの contour にも属さないため、最終 endpoint が `point_count - 1` であることを検査するよう指摘された。
- 計画を修正し、`FinalContourEndMismatch`、`last_endpoint_index`、最終 endpoint check、endpoint `[1, 2]` forged topology doctest、source policy の ordered check を追加した。
- 修正後の計画は Tesla review で `PLAN_APPROVED`。F5v は collection read helper だけで item を読み、byte-backed contour helper、direct `Vec`、path/render/raster/platform/host API へ進まない方針で実装を開始する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError`
  - error constructor/accessors
  - `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span`
- F5v は `capacity shape`、`len == item_count`、`cap == point_count`、`item_count == point_count`、`contour_index` range を scan 前に検査する。
- F5v は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item` だけで item を読み、各 item の glyph、point index、kind を再検査する。
- F5v は requested contour の endpoint を見つけても scan を止めず、全 item scan 後に `target_found`、`observed_contour_count == contour_count`、`last_endpoint_index == point_count - 1`、derived span invariant を順に検査する。
- doctest は two-contour success、partial collection rejection、contour index out of range、extra endpoint count mismatch、final endpoint mismatch、missing contour end を検査する。通常の public collection owner では `collection_read_item` の lower `ItemStorageMissing` は作れないため、`ItemReadFailed` branch は source policy で固定する。
- 実装後は subagent implementation review を受け、指摘があれば source policy、stdlib、doctest、文書を修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- F5v は byte-backed `gui_sfnt_lookup_simple_glyph_contour_span` / `gui_sfnt_glyf_simple_contour_span_with_tables` / `gui_sfnt_glyf_read_contour_endpoint` を呼ばない。
- F5v は `collection_read_item` を通して item を読み、direct `vec::` を使わない。
- F5v は partial collection を `CollectionIncomplete` として拒否し、span を返さない。
- F5v は item glyph/index/kind を再検査し、forged item を typed error にする。
- F5v は endpoint count と final endpoint を両方検査し、`observed_contour_count == contour_count` だけで成功しない。
- source policy が F5v docs、public API、collection read helper、item glyph/index/kind validation、target_found、contour count、final endpoint、span invariant、禁止 API、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_contour_span.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_contour_span_f5v.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_drain_f5v_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_f5v_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5v.json -j 1
git diff --check
```

## Phase F5w: sfnt simple glyph outline point stream item collection contour point

目的:

- F5v の collection-backed contour span を authority とし、contour-local point index から `GuiSfntSimpleGlyphContourPoint` を 1 点だけ読む。
- F4 byte-backed contour point helper、F5 storage reader、drain、path/raster/render/platform/host API へ戻らない。
- 後続の collection-backed edge extraction が使う contour point contract を固定する。

変更:

- 先に source policy と実装計画を subagent にレビューさせた。
- Tesla plan review は 1 回目 `PLAN_BLOCKED`。F5v が `Ok span` を返した後でも、F5w 側で span/capacity invariant を local index 判定や item read より前に visible に再検査する必要があると指摘された。
- 計画を修正し、`span.glyph == capacity.glyph`、`span.contour_index == contour_index`、`span.start_point_index >= 0`、`span.end_point_index >= span.start_point_index`、`span.end_point_index < capacity.point_count`、`span.point_count == span.end_point_index - span.start_point_index + 1` を `ContourPointInvariantInvalid` として検査する方針にした。
- 修正後の計画は Tesla review で `PLAN_APPROVED`。F5w は F5v contour span lookup を exactly once 呼び、span invariant、local range、absolute range、collection read、item glyph/index/kind validation の順で実装する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointError`
  - error constructor/accessors
  - `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point`
- doctest は success、span failure wrapping、local index out of range、forged endpoint topology の span failure propagation を検査する。public F5v が拒否する impossible success span や public collection owner で作れない lower `ItemReadFailed` branch は source policy で固定する。
- 実装後は subagent implementation review を受け、指摘があれば source policy、stdlib、doctest、文書を修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- F5w は F5v `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span` を source 上 exactly once 呼ぶ。
- F5w は F5v success span の glyph、contour_index、start/end/count、capacity range を local index 判定より前に検査する。
- F5w は local `contour_point_index` が範囲外なら `ContourPointIndexOutOfRange` を返し、collection item を読まない。
- F5w は `absolute_point_index = span.start_point_index + contour_point_index` を使い、absolute range を再検査してから collection read へ進む。
- F5w は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item` を source 上 exactly once 呼ぶ。
- F5w は item glyph/index/kind を再検査し、forged item を typed error にする。
- source policy が F5w docs、public API、F5v exact one-call、span invariant before local range、local range before collection read、collection read exact one-call、forbidden API、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_contour_point.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_contour_point_f5w.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_contour_span.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_contour_span_f5w_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5w.json -j 1
git diff --check
```

## Phase F5x: sfnt simple glyph outline point stream item collection contour edge

目的:

- F5v の collection-backed contour span と F5w の collection-backed contour point を authority とし、contour-local edge index から `GuiSfntSimpleGlyphContourEdge` を 1 本だけ読む。
- F4 byte-backed contour edge helper、F5 storage reader、drain、path/raster/render/platform/host API へ戻らない。
- 後続の collection-backed curve classification / path tag population が使う point pair traversal contract を固定する。

変更:

- 先に source policy と実装計画を subagent にレビューさせた。
- Tesla plan review は 1 回目 `PLAN_BLOCKED`。F5x error payload が lower error を span/start/end の各 boundary ごとに保持すること、start/end の span/local/absolute invariant を edge construction 前に明示すること、1 point contour self-wrap を doctest に含めることが必須指摘だった。
- 計画を修正し、`span_error`、`start_error`、`end_error`、`start`、`end` を error payload に持たせる方針にした。
- 修正後の計画は Tesla review で `PLAN_APPROVED`。F5x は F5v contour span lookup を exactly once 呼び、span invariant、edge range、wrapped next index、F5w start/end point lookup、start/end invariant validation の順で実装する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeError`
  - error constructor/accessors
  - `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge`
- doctest は success with wrap、second contour success、one point self-wrap、span failure wrapping、edge index out of range、forged endpoint topology の span failure propagation を検査する。public F5v/F5w が拒否する impossible point success shape は source policy で固定する。
- 実装後は subagent implementation review を受け、指摘があれば source policy、stdlib、doctest、文書を修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- F5x は F5v `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span` を source 上 exactly once 呼ぶ。
- F5x は F5v success span の glyph、contour_index、start/end/count、capacity range を edge index 判定より前に検査する。
- F5x は `edge_index` が範囲外なら `EdgeIndexOutOfRange` を返し、F5w point lookup を呼ばない。
- F5x は `next_contour_point_index = edge_index + 1` を使い、contour end では 0 に wrap する。1 point contour では start/end が同じ local index 0 になる。
- F5x は F5w `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point` を source 上 exactly twice 呼ぶ。
- F5x は start/end point の span、local index、absolute point index を再検査し、forged point を typed error にする。
- source policy が F5x docs、public API、F5v exact one-call、F5w exact two-call、span invariant before edge range、edge range before point lookup、start/end invariant before edge construction、forbidden API、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_contour_edge.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_contour_edge_f5x.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_contour_point.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_contour_point_f5x_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5x.json -j 1
git diff --check
```

## Phase F5y: sfnt simple glyph outline point stream item collection curve segment

目的:

- F5x の collection-backed contour edge を authority とし、contour-local edge index から `GuiSfntSimpleGlyphCurveSegment` を 1 つだけ分類する。
- needed lookahead がある場合だけ F5w point lookup を呼び、F4 byte-backed curve helper、F5 storage reader、drain、path/raster/render/platform/host API へ戻らない。
- 後続の collection-backed path tag population が使う curve segment boundary を、owner-preserving collection API と typed `Result` に固定する。

変更:

- 先に source policy と実装計画を subagent にレビューさせた。
- Tesla plan review は 1 回目 `PLAN_BLOCKED`。F5y error payload が `kind`、`contour_index`、`edge_index`、computed local indices、capacity、lower edge/lookahead errors、accepted edge/lookahead、collection diagnostics を保持すること、F5x success 後に edge/span invariant を lookahead 判定より前に visible に再検査することが必須指摘だった。
- 計画を修正し、`ContourEdgeFailed`、`LookaheadPointFailed`、`CurveSegmentInvariantInvalid` の 3 kind とし、`edge_error`、`lookahead_error`、`edge`、`lookahead`、`next_contour_point_index`、`lookahead_contour_point_index`、`item_count`、`items_len`、`items_cap` を error payload に保持する方針にした。
- 修正後の計画は Tesla review で `PLAN_APPROVED`。F5y は F5x contour edge lookup を exactly once 呼び、edge/span invariant、on-curve 判定、必要な場合だけ F5w lookahead lookup、lookahead invariant validation、pure classifier call の順で実装する。
- `alloc/gui/font/sfnt/glyf.nepl` に次を追加する。
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentErrorKind`
  - `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError`
  - error constructor/accessors
  - `gui_sfnt_simple_glyph_outline_point_stream_item_collection_curve_segment`
- doctest は line、explicit quadratic、implied midpoint、single point no segment、off-curve start no segment、edge failure wrapping、lookahead contour-end wrap を検査する。public F5x/F5w が拒否する impossible success shape は source policy で固定する。
- 実装後は subagent implementation review を受け、指摘があれば source policy、stdlib、doctest、文書を修正する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

完了条件:

- F5y は F5x `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge` を source 上 exactly once 呼ぶ。
- F5y は F5x success edge の span glyph、contour_index、start/end/count、capacity range、edge index、wrapped next index、start/end local index、start/end absolute point index を lookahead 判定より前に再検査する。
- F5y は start が on-curve かつ end が off-curve の場合だけ F5w `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point` を source 上 exactly once 呼ぶ。
- F5y は needed lookahead lookup が失敗した場合、`Option::None` を classifier に渡さず `LookaheadPointFailed` を返す。
- F5y は lookahead の span、local index、absolute point index を再検査してから `gui_sfnt_classify_simple_glyph_curve_segment edge Option::Some lookahead` を呼ぶ。
- F5y は lookahead 不要 path では F5w を呼ばず、`lookahead_contour_point_index = -1` として `gui_sfnt_classify_simple_glyph_curve_segment edge Option::None` を呼ぶ。
- single-point contour と off-curve start は valid `NoSegment` success として返し、F5y error へ変換しない。
- source policy が F5y docs、public API、F5x exact one-call、F5w conditional exact one-call、edge invariant before lookahead 判定、lookahead invariant before classifier、forbidden API、括弧なし prefix style を検査する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_curve_segment.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_curve_segment_f5y.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_contour_edge.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_contour_edge_f5y_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5y.json -j 1
git diff --check
```

## Phase F5z: sfnt simple glyph outline point stream item collection path command pair

目的:

- F5y の collection-backed curve segment lookup を authority とし、1 edge を `GuiSfntSimpleGlyphPathCommandPair` へ写す。
- F4 byte-backed path command pair lookup、metadata parser、`*_with_tables` helper、F5x/F5w 直接呼び出し、drain、raster/render/platform/host API へ戻らない。
- 後続の collection-backed path sink event / outline traversal が使う single-edge command pair boundary を、owner-preserving collection API と typed `Result` に固定する。

変更:

- 先に source policy と実装計画を subagent にレビューさせた。
- Tesla plan review は `PLAN_APPROVED`。F5z は F5y と既存 pure path command pair projection の thin composition として scope が適切であり、新しい failure domain は不要と判断された。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair` を追加する。
- F5z は F5y `gui_sfnt_simple_glyph_outline_point_stream_item_collection_curve_segment` を source 上 exactly once 呼ぶ。
- F5y error は wrap せず、同じ `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError` として `Result::Err` で返す。
- F5y success segment は `gui_sfnt_simple_glyph_curve_segment_path_command_pair` に source 上 exactly once 渡し、`Result::Ok pair` を返す。
- NoSegment は F4o と同じく explicit `SkipNoSegment` pair として保持し、`Option::None`、silent no-op、fallback に変換しない。
- source policy は F5z docs、public API、F5y exact one-call、pure pair projection exact one-call、F5y error propagation、forbidden API、括弧なし prefix style を検査する。
- F5y 実呼び出しは現行 wasm doctest compiler で compile timeout するため、F5z focused doctest は source policy label と `skip` executable に留める。compiler 側の compile time が改善された時点で unskip する。

完了条件:

- F5z public helper が F5y を exactly once 呼ぶ。
- F5z public helper が `gui_sfnt_simple_glyph_curve_segment_path_command_pair` を exactly once 呼ぶ。
- F5z public helper は `gui_sfnt_lookup_simple_glyph_path_command_pair`、`gui_sfnt_lookup_simple_glyph_curve_segment`、metadata parser、`*_with_tables` helper、F5x/F5w、F5 drain/point-step、`Vec` / `push`、sink traversal、render/raster/platform/host API を呼ばない。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_pair.n.md` に line pair、quadratic pair、no-segment skip pair、F5y error propagation、no fallback/no Vec/no sink traversal coverage label を追加する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_pair.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_command_pair_f5z.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_curve_segment.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_curve_segment_f5z_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5z.json -j 1
git diff --check
```

## Phase F5aa: sfnt simple glyph outline point stream item collection path sink event pair

目的:

- F5z の collection-backed path command pair lookup を authority とし、1 edge を `GuiSfntSimpleGlyphPathSinkEventPair` へ写す。
- F4 byte-backed path lookup、metadata parser、`*_with_tables` helper、F5y/F5x/F5w 直接呼び出し、drain、sink traversal、raster/render/platform/host API へ戻らない。
- 後続の collection-backed path sink event kind / slot / outline traversal が使う single-edge event pair boundary を、owner-preserving collection API と typed `Result` に固定する。

変更:

- 先に source policy と実装計画を subagent にレビューさせた。
- Tesla plan review は `PLAN_APPROVED`。F5aa は F5z と既存 pure path sink event pair projection の thin composition として scope が適切であり、新しい failure domain は不要と判断された。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_pair` を追加する。
- F5aa は F5z `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair` を source 上 exactly once 呼ぶ。
- F5z error は wrap せず、同じ `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError` として `Result::Err` で返す。
- F5z success pair は `gui_sfnt_simple_glyph_path_command_pair_sink_event_pair` に source 上 exactly once 渡し、`Result::Ok event_pair` を返す。
- NoSegment は F4p と同じく explicit `SkipNoSegment` event pair として保持し、`Option::None`、silent no-op、fallback に変換しない。
- source policy は F5aa docs、public API、F5z exact one-call、pure event pair projection exact one-call、F5z error propagation、forbidden API、括弧なし prefix style を検査する。
- F5z/F5y 実呼び出しは現行 wasm doctest compiler で compile timeout するため、F5aa focused doctest は source policy label と `skip` executable に留める。compiler 側の compile time が改善された時点で unskip する。

完了条件:

- F5aa public helper が F5z を exactly once 呼ぶ。
- F5aa public helper が `gui_sfnt_simple_glyph_path_command_pair_sink_event_pair` を exactly once 呼ぶ。
- F5aa public helper は `gui_sfnt_lookup_simple_glyph_path_command_pair`、`gui_sfnt_lookup_simple_glyph_curve_segment`、metadata parser、`*_with_tables` helper、F5y/F5x/F5w、F5 drain/point-step、`Vec` / `push`、sink traversal、event consumer、render/raster/platform/host API を呼ばない。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_event_pair.n.md` に line event pair、quadratic event pair、no-segment skip event pair、F5z error propagation、no fallback/no Vec/no sink traversal coverage label を追加する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_event_pair.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_event_pair_f5aa.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_command_pair.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_command_pair_f5aa_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5aa.json -j 1
git diff --check
```

## Phase F5ab: sfnt simple glyph outline point stream item collection path sink event kind pair

目的:

- F5aa の collection-backed path sink event pair lookup を authority とし、1 edge を `GuiSfntSimpleGlyphPathSinkEventKindPair` へ写す。
- F4 byte-backed path lookup、metadata parser、`*_with_tables` helper、F5z/F5y/F5x/F5w 直接呼び出し、drain、sink traversal、event consumer/action、raster/render/platform/host API へ戻らない。
- 後続の collection-backed path sink typed slot / outline traversal が使う single-edge event kind pair boundary を、owner-preserving collection API と typed `Result` に固定する。

変更:

- 先に source policy と実装計画を subagent にレビューさせた。
- Tesla plan review は `PLAN_APPROVED`。F5ab は F5aa と既存 pure path sink event kind pair projection の thin composition として scope が適切であり、新しい failure domain は不要と判断された。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_pair` を追加する。
- F5ab は F5aa `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_pair` を source 上 exactly once 呼ぶ。
- F5aa error は wrap せず、同じ `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError` として `Result::Err` で返す。
- F5aa success event pair は `gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair` に source 上 exactly once 渡し、`Result::Ok kind_pair` を返す。
- NoSegment は F4q と同じく explicit `SkipNoSegment` kind pair として保持し、`Option::None`、silent no-op、fallback に変換しない。
- source policy は F5ab docs、public API、F5aa exact one-call、pure kind pair projection exact one-call、F5aa error propagation、forbidden API、括弧なし prefix style を検査する。
- F5aa/F5z/F5y 実呼び出しは現行 wasm doctest compiler で compile timeout するため、F5ab focused doctest は source policy label と `skip` executable に留める。compiler 側の compile time が改善された時点で unskip する。

完了条件:

- F5ab public helper が F5aa を exactly once 呼ぶ。
- F5ab public helper が `gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair` を exactly once 呼ぶ。
- F5ab public helper は `gui_sfnt_lookup_simple_glyph_path_command_pair`、`gui_sfnt_lookup_simple_glyph_curve_segment`、metadata parser、`*_with_tables` helper、F5z/F5y/F5x/F5w、F5 drain/point-step、`Vec` / `push`、sink traversal、event consumer/action、render/raster/platform/host API を呼ばない。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_event_kind_pair.n.md` に line kind pair、quadratic kind pair、no-segment skip kind pair、F5aa error propagation、no fallback/no Vec/no sink traversal coverage label を追加する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_event_kind_pair.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_event_kind_pair_f5ab.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_event_pair.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_event_pair_f5ab_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ab.json -j 1
git diff --check
```

## Phase F5ac: sfnt simple glyph outline point stream item collection path sink event kind at

目的:

- F5ab の collection-backed path sink event kind pair lookup を authority とし、typed `GuiSfntSimpleGlyphPathSinkEventSlot` で 1 kind だけを取り出す。
- F4 byte-backed path lookup、metadata parser、`*_with_tables` helper、F5aa/F5z/F5y/F5x/F5w 直接呼び出し、drain、sink traversal、event consumer/action、raster/render/platform/host API へ戻らない。
- 後続の collection-backed path contour traversal が使う single-edge typed slot lookup boundary を、owner-preserving collection API と typed `Result` に固定する。

変更:

- 先に source policy と実装計画を subagent にレビューさせた。
- Tesla plan review は `PLAN_APPROVED`。F5ac は F5ab と既存 pure typed-slot kind projection の thin composition として scope が適切であり、新しい failure domain は不要と判断された。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_at` を追加する。
- F5ac は F5ab `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_pair` を source 上 exactly once 呼ぶ。
- F5ab error は wrap せず、同じ `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError` として `Result::Err` で返す。
- F5ab success kind pair は `gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at` に source 上 exactly once 渡し、typed slot に対応する `Result::Ok kind` を返す。
- slot は enum なので invalid index を表現できない。`Option::None`、silent no-op、fallback、新しい invalid index error enum は導入しない。
- source policy は F5ac docs、public API、F5ab exact one-call、pure typed-slot projection exact one-call、F5ab error propagation、forbidden API、括弧なし prefix style を検査する。
- F5ab/F5aa/F5z/F5y 実呼び出しは現行 wasm doctest compiler で compile timeout するため、F5ac focused doctest は source policy label と `skip` executable に留める。compiler 側の compile time が改善された時点で unskip する。

完了条件:

- F5ac public helper が F5ab を exactly once 呼ぶ。
- F5ac public helper が `gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at` を exactly once 呼ぶ。
- F5ac public helper は `gui_sfnt_lookup_simple_glyph_path_command_pair`、`gui_sfnt_lookup_simple_glyph_curve_segment`、metadata parser、`*_with_tables` helper、F5aa/F5z/F5y/F5x/F5w、F5 drain/point-step、`Vec` / `push`、sink traversal、event consumer/action、render/raster/platform/host API を呼ばない。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_event_kind_at.n.md` に first slot line kind、second slot line kind、no-segment skip kind、F5ab error propagation、no fallback/no Vec/no sink traversal coverage label を追加する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_event_kind_at.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_event_kind_at_f5ac.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_event_kind_pair.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_event_kind_pair_f5ac_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ac.json -j 1
git diff --check
```

## Phase F5ad: sfnt simple glyph outline point stream item collection path sink event at

目的:

- F5aa の collection-backed path sink event pair lookup を authority とし、typed `GuiSfntSimpleGlyphPathSinkEventSlot` で 1 event だけを取り出す。
- F4 byte-backed path lookup、metadata parser、`*_with_tables` helper、F5z/F5y/F5x/F5w/F5v 直接呼び出し、drain、sink traversal、event consumer/action、raster/render/platform/host API へ戻らない。
- 後続の collection-backed path contour step が event と kind を二重導出しないように、single-edge typed event lookup boundary を固定する。

変更:

- 当初は collection-backed contour step まで進める計画だったが、Tesla plan review 1 は `PLAN_BLOCKED`。`contour_span` error と curve-segment error を混ぜられないこと、F5aa と F5ac を同時に呼ぶと同じ edge を二重に導出することを指摘された。
- revised Tesla plan review は `PLAN_APPROVED`。F5ad は F5aa と既存 pure typed-slot event projection の thin composition として scope が適切であり、新しい failure domain は不要と判断された。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at` を追加する。
- F5ad は F5aa `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_pair` を source 上 exactly once 呼ぶ。
- F5aa error は wrap せず、同じ `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError` として `Result::Err` で返す。
- F5aa success event pair は `gui_sfnt_simple_glyph_path_sink_event_pair_event_at` に source 上 exactly once 渡し、typed slot に対応する `Result::Ok event` を返す。
- slot は enum なので invalid index を表現できない。`Option::None`、silent no-op、fallback、新しい invalid index error enum は導入しない。
- source policy は F5ad docs、public API、F5aa exact one-call、pure typed-slot event projection exact one-call、F5aa error propagation、F5ab/F5ac kind helper 禁止、forbidden API、括弧なし prefix style を検査する。
- F5aa/F5z/F5y 実呼び出しは現行 wasm doctest compiler で compile timeout するため、F5ad focused doctest は source policy label と `skip` executable に留める。compiler 側の compile time が改善された時点で unskip する。

完了条件:

- F5ad public helper が F5aa を exactly once 呼ぶ。
- F5ad public helper が `gui_sfnt_simple_glyph_path_sink_event_pair_event_at` を exactly once 呼ぶ。
- F5ad public helper は F5ab/F5ac kind helper、`gui_sfnt_lookup_simple_glyph_path_command_pair`、`gui_sfnt_lookup_simple_glyph_curve_segment`、metadata parser、`*_with_tables` helper、F5z/F5y/F5x/F5w/F5v、F5 drain/point-step、`Vec` / `push`、sink traversal、event consumer/action、render/raster/platform/host API を呼ばない。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_event_at.n.md` に first slot line event、second slot line event、no-segment skip event、F5aa error propagation、no fallback/no Vec/no sink traversal coverage label を追加する。
- `note.n.md` に blocked plan review、revised plan review、実装、検証、残件を記録する。

検証:

```powershell
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_event_at.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_event_at_f5ad.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_event_kind_at.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_event_kind_at_f5ad_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ad.json -j 1
git diff --check
```

## Phase F5ae: sfnt simple glyph outline point stream item collection path contour step

目的:

- F5v collection-backed contour span lookup と F5ad collection-backed path sink event at lookup を接続し、`GuiSfntSimpleGlyphPathContourCursor` から `GuiSfntSimpleGlyphPathContourStep` を返す。
- contour span failure、cursor glyph identity failure、event lookup failure を同じ error に潰さず、F5ae 専用 typed error として分ける。
- F5ac は kind-only sibling boundary として残し、F5ae の内部では返された event から kind を導くことで edge の二重導出を避ける。

変更:

- Tesla plan review 1 は `PLAN_BLOCKED`。cursor が glyph を持つのに collection helper は collection 側の glyph を authority としており、cursor glyph と collection capacity glyph の一致を検査しないと forged cursor を成功 step に混ぜられる、という指摘を受けた。
- revised Tesla plan review は `PLAN_APPROVED`。`CursorGlyphMismatch` を専用 error kind として追加し、span 成功後かつ event lookup 前に glyph identity check を置く。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepErrorKind` と `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError` を追加する。
- error kind は `ContourSpanFailed`、`CursorGlyphMismatch`、`PathSinkEventFailed` の 3 種にする。
- error payload は collection capacity、cursor、contour_index、edge_index、slot、span_error option、event_error option を保持する。
- `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step` を追加する。
- helper は source 上 `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span` を exactly once 呼ぶ。
- helper は source 上 `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at` を exactly once 呼ぶ。
- helper は source 上 `gui_sfnt_simple_glyph_path_sink_event_kind`、`gui_sfnt_simple_glyph_path_contour_next_from_cursor`、`gui_sfnt_simple_glyph_path_contour_step` をそれぞれ exactly once 呼ぶ。
- helper は F5ac/F5ab/F5aa 直接呼び出し、byte-backed F4 lookup、metadata parser、`*_with_tables`、F5z/F5y/F5x/F5w、drain/point-step、`Vec` / `push`、sink traversal、event consumer/action、render/raster/platform/host API を呼ばない。

完了条件:

- F5ae public helper が span lookup -> cursor glyph check -> F5ad event lookup -> event kind projection -> cursor next -> step constructor の順序を守る。
- F5ae public helper が `CursorGlyphMismatch` では F5ad event lookup へ進まない。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_contour_step.n.md` に first line step、second line step、end contour、span error、cursor glyph mismatch、event error、no fallback/no byte-backed traversal coverage label を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` で docs、typed error payload、helper body order、forbidden API、括弧なし prefix style を検査する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_contour_step.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_contour_step_f5ae.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_event_at.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_event_at_f5ae_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ae.json -j 1
git diff --check
```

## Phase F5af: sfnt simple glyph outline point stream item collection path sink step

目的:

- F5ae collection-backed contour step lookup を authority として、`GuiSfntSimpleGlyphPathSinkPolicy` を適用した `GuiSfntSimpleGlyphPathSinkStep` を返す。
- policy reject は `Result::Err` にせず、既存 pure helper と同じく success payload の primary action に保持する。
- F4 byte-backed helper、F5aa/F5ac/F5ad 直接呼び出し、sink traversal、action step、render/raster/platform/host API へ戻らない。

変更:

- Tesla plan review は `PLAN_APPROVED`。F5af は F5ae と pure `gui_sfnt_simple_glyph_path_sink_step_from_contour_step` の thin composition として妥当であり、新しい error type は不要と判断された。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_step` を追加する。
- helper は source 上 `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step` を exactly once 呼ぶ。
- F5ae error は wrap せず `Result::Err error` として返す。
- F5ae success では source 上 `gui_sfnt_simple_glyph_path_sink_step_from_contour_step` を exactly once 呼び、`Result::Ok sink_step` を返す。
- helper は F5ad/F5ac/F5aa 直接呼び出し、byte-backed F4 lookup、metadata parser、`*_with_tables`、lower collection helpers、`Vec` / `push`、sink traversal、action step、render/raster/platform/host API を呼ばない。

完了条件:

- F5af public helper が F5ae lookup -> error propagation -> pure sink-step projection -> `Result::Ok sink_step` の順序を守る。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_step.n.md` に primary line、tail close、error propagation、no fallback/no byte-backed traversal coverage label を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` で docs、helper body order、call count、forbidden API、括弧なし prefix style を検査する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_step.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_step_f5af.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_contour_step.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_contour_step_f5af_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5af.json -j 1
git diff --check
```

## Phase F5ag: sfnt simple glyph outline point stream item collection path sink action step

目的:

- F5af collection-backed sink step lookup を authority として、`GuiSfntSimpleGlyphPathSinkActionCursor` から `GuiSfntSimpleGlyphPathSinkActionStep` を返す。
- action cursor を contour cursor と action slot に分け、collection lookup と action selection の責務を混ぜない。
- F5af error は wrap せず伝播し、policy reject は `Result::Err` ではなく action payload として残す。
- F4 byte-backed helper、F5ae/F5ad/F5ac/F5aa 直接呼び出し、sink traversal、action advance/item/consumer、render/raster/platform/host API へ戻らない。

変更:

- Tesla plan review は `PLAN_APPROVED`。F5ag は F5af と pure `gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step` の thin composition として妥当であり、新しい error type は不要と判断された。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step` を追加する。
- helper は source 上 `gui_sfnt_simple_glyph_path_sink_action_cursor_contour_cursor` を exactly once 呼ぶ。
- helper は source 上 `gui_sfnt_simple_glyph_path_sink_action_cursor_action_slot` を exactly once 呼ぶ。
- helper は source 上 `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_step` を exactly once 呼ぶ。
- F5af error は wrap せず `Result::Err error` として返す。
- F5af success では source 上 `gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step` を exactly once 呼び、`Result::Ok action_step` を返す。
- helper は F5ae/F5ad/F5ac/F5aa 直接呼び出し、byte-backed F4 lookup、metadata parser、`*_with_tables`、lower collection helpers、`Vec` / `push`、sink traversal、action advance/item/consumer、render/raster/platform/host API を呼ばない。

完了条件:

- F5ag public helper が action cursor split -> F5af lookup -> error propagation -> pure action-step projection -> `Result::Ok action_step` の順序を守る。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_step.n.md` に primary action、tail action、error propagation、no fallback/no byte-backed traversal coverage label を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` で docs、helper body order、call count、forbidden API、括弧なし prefix style を検査する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_step.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_step_f5ag.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_step.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_step_f5ag_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ag.json -j 1
git diff --check
```

## Phase F5ah: sfnt simple glyph outline point stream item collection path sink action step advance and item

目的:

- F5ag collection-backed action step lookup を authority として、typed next state を checked advance へ変換する。
- 現在 action step と checked advance を `GuiSfntSimpleGlyphPathSinkActionStepItem` に束ねる。
- `EndContour` は `Result::Err` や `Option::None` にせず、successful terminal enum として保持する。
- action payload の解釈、consumer、contour-wide traversal、sink mutation、render/raster/platform/host API へ進まない。

変更:

- Tesla plan review は `PLAN_APPROVED`。F5ah は既存 byte-backed F4y/F4z の advance/item split を mirror しつつ、F5ag を唯一の collection-backed lookup authority とする計画として妥当である。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step_advance` を追加する。
- advance helper は source 上 `gui_sfnt_simple_glyph_path_sink_action_step_next` を exactly once 呼ぶ。
- `Continue cursor` の場合だけ source 上 `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step` を exactly once 呼ぶ。
- `EndContour` は `Result::Ok GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour` として返す。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step_item` を追加する。
- item helper は source 上 collection-backed advance helper を exactly once 呼ぶ。
- item helper は success path で `let stored_step %GuiSfntSimpleGlyphPathSinkActionStep *step` により現在 step を明示 copy し、`gui_sfnt_simple_glyph_path_sink_action_step_item stored_step advance` を返す。
- helper は F5af/F5ae/F5ad/F5ac/F5aa 直接呼び出し、byte-backed F4 lookup、metadata parser、`*_with_tables`、lower collection helpers、`Vec` / `push`、sink traversal、consumer、render/raster/platform/host API を呼ばない。

完了条件:

- F5ah advance helper が step next read -> Continue/F5ag lookup または EndContour success terminal の順序を守る。
- F5ah item helper が advance lookup -> error propagation -> current step copy -> item construction の順序を守る。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_step_item.n.md` に continue advance、end advance、item copy、error propagation、no fallback/no byte-backed traversal coverage label を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` で docs、helper body order、call count、forbidden API、括弧なし prefix style を検査する。
- `note.n.md` に plan review、実装、検証、残件を記録する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_step_item.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_step_item_f5ah.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_step.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_step_f5ah_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ah.json -j 1
git diff --check
```

## Phase F5ai: sfnt simple glyph outline point stream item collection path sink action item next and consumer item

目的:

- F5ah collection-backed action step item を authority として、checked advance を次 action item または `EndContour` へ 1 段だけ進める。
- current action payload と checked next state を `GuiSfntSimpleGlyphPathSinkActionConsumerItem` に束ねる。
- byte-backed F4ab/F4ac と同じ責務分割を保ちながら、font bytes / table metadata / byte-backed lookup helper へ戻らない。
- action payload の解釈、consumer apply / consume、contour-wide traversal、sink mutation、render/raster/platform/host API へ進まない。

変更:

- Tesla plan review は `PLAN_APPROVED`。F5ai は F5ah の checked advance/item を authority として F4ab/F4ac と同じ分割を collection-backed に写す計画として妥当である。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_item_next` を追加する。
- action item next helper は source 上 `gui_sfnt_simple_glyph_path_sink_action_step_item_advance` を exactly once 呼ぶ。
- `Continue next_step` の場合だけ source 上 `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step_item collection &next_step policy` を exactly once 呼ぶ。
- `EndContour` は `Result::Ok GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour` として返す。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item` を追加する。
- consumer item helper は source 上 `gui_sfnt_simple_glyph_path_sink_action_step_item_step` と `gui_sfnt_simple_glyph_path_sink_action_step_action` をそれぞれ exactly once 呼ぶ。
- consumer item helper は source 上 collection-backed action item next helper を exactly once 呼び、success path で `gui_sfnt_simple_glyph_path_sink_action_consumer_item action next` を返す。
- helper は F5ag/F5af/F5ae/F5ad/F5ac/F5aa 直接呼び出し、byte-backed F4 lookup、metadata parser、`*_with_tables`、lower collection helpers、`Vec` / `push`、consumer apply / consume、sink traversal、render/raster/platform/host API を呼ばない。

完了条件:

- F5ai action item next helper が checked advance read -> Continue/F5ah item lookup または EndContour success terminal の順序を守る。
- F5ai consumer item helper が stored step read -> action copy -> action item next -> consumer item construction の順序を守る。
- action payload を match せず、payload は consumer item の value としてだけ保持する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_consumer_item.n.md` に continue item next、end item next、consumer item action/next copy、error propagation、no fallback/no byte-backed traversal coverage label を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` で docs、helper body order、call count、forbidden API、括弧なし prefix style を検査する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_consumer_item.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_consumer_item_f5ai.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_step_item.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_step_item_f5ai_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ai.json -j 1
git diff --check
```

## Phase F5aj: sfnt simple glyph outline point stream item collection path sink action consumer next and consume once

目的:

- F5ai collection-backed action consumer item を authority として、consumer next、apply advance、consume-once の 3 境界を追加する。
- byte-backed F4ad/F4ah/F4ai と同じ typed value 分割を collection-backed item stream へ写す。
- consumer apply advance の Continue branch では apply step に保存済みの checked next を authority とし、original consumer item や action payload を再解釈しない。
- byte-backed F4 lookup、lower F5 collection helper、sink traversal、real sink mutation、render/raster/platform/host API へ戻らない。

変更:

- Tesla plan review 1 回目は `PLAN_BLOCKED`。`consumer_apply_advance` の Continue branch で saved next を読む順序と、元 item/action payload を再解釈しない禁止条件を明文化する必要があると指摘された。
- Tesla plan review 2 回目は `PLAN_APPROVED`。saved next authority、F5ai helper だけに戻る境界、helper ごとの source policy 分離が妥当と判断された。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item_next` を追加する。
- consumer item next helper は source 上 `gui_sfnt_simple_glyph_path_sink_action_consumer_item_next` を exactly once 呼ぶ。
- `Continue next_item` の場合だけ source 上 `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item collection &next_item policy` を exactly once 呼ぶ。
- `EndContour` は `Result::Ok GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::EndContour` として返す。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_apply_advance` を追加する。
- apply advance helper は source 上 `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step` を exactly once 呼ぶ。
- `Continue continue_step` の場合だけ source 上 `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next &continue_step` を exactly once 呼ぶ。
- saved next が `Continue next_item` の場合だけ source 上 F5ai consumer item helper を exactly once 呼ぶ。
- `Rejected reason` と `EndContour` は typed terminal advance として `Result::Ok` で返す。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item_consume_once` を追加する。
- consume-once helper は source 上 `gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply state item` を exactly once 呼び、collection apply advance helper を exactly once 呼ぶ。
- consume-once helper は success path で `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step apply_step advance` を exactly once 呼ぶ。
- helper は byte-backed F4 lookup、F5ah/F5ag/F5af/F5ae/F5ad/F5ac/F5aa 直接呼び出し、lower collection helpers、`Vec` / `push`、payload direct match、original item/action reinterpretation、sink traversal、render/raster/platform/host API を呼ばない。

完了条件:

- F5aj consumer item next helper が consumer next read -> Continue/F5ai consumer item lookup または EndContour success terminal の順序を守る。
- F5aj consumer apply advance helper が terminal read -> Continue saved next read -> Continue/F5ai consumer item lookup または typed terminal success の順序を守る。
- F5aj consume-once helper が apply -> collection apply advance -> consume step construction の順序を守る。
- original item/action payload を `consumer_apply_advance` が再解釈しない。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_consumer_next.n.md` に consumer item next、apply advance saved next、consume-once、error propagation、no fallback/no byte-backed traversal coverage label を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` で docs、helper body order、call count、forbidden API、括弧なし prefix style を検査する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_consumer_next.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_consumer_next_f5aj.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_consumer_item.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_consumer_item_f5aj_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5aj.json -j 1
git diff --check
```

## Phase F5ak: sfnt simple glyph outline point stream item collection path sink action start consumer

目的:

- collection-backed action stream の contour start boundary を追加し、first action item、first consumer item、first consume step、first consume summary を作る。
- caller supplied glyph を受け取らず、collection capacity の glyph を start cursor authority とする。
- collection-backed start boundary で forged cursor を作れないよう、`collection_capacity -> capacity.glyph -> start_cursor -> F5ag action step -> F5ah step item` の順序を固定する。
- F5ai / F5aj を高位 helper の authority とし、byte-backed F4 lookup、lower F5 直接呼び出し、sink traversal、real sink mutation、render/raster/platform/host API へ戻らない。

変更:

- Tesla plan review 1 回目は `PLAN_BLOCKED`。`start_item` が caller supplied glyph を受け取ると forged cursor を作れるため、collection capacity glyph を authority にする必要があると指摘された。
- Tesla revised plan review は `PLAN_APPROVED`。collection capacity glyph authority、F5ak helper 分割、summary advance/drain を次 slice に分ける責務境界が妥当と判断された。
- revised plan では `start_item` の public signature から `GuiGlyphId` を削除し、collection capacity から glyph を読み出す。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_item` を追加する。
- start item helper は source 上 `gui_sfnt_simple_glyph_outline_point_stream_item_collection_capacity collection` を exactly once 呼ぶ。
- start item helper は source 上 `gui_sfnt_simple_glyph_outline_storage_capacity_glyph &capacity` を exactly once 呼ぶ。
- start item helper は source 上 `gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index` を exactly once 呼ぶ。
- start item helper は source 上 F5ag `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step collection start_cursor policy` を exactly once 呼ぶ。
- start item helper は source 上 F5ah `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step_item collection &start_step policy` を exactly once 呼ぶ。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consumer_item` を追加する。
- start consumer item helper は source 上 start item helper を exactly once 呼び、成功時だけ F5ai consumer item helper を exactly once 呼ぶ。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_once` を追加する。
- start consume-once helper は source 上 start consumer item helper を exactly once 呼び、成功時だけ F5aj consume-once helper を exactly once 呼ぶ。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_summary` を追加する。
- start consume summary helper は source 上 start consume-once helper を exactly once 呼び、成功時だけ pure summary projection を exactly once 呼ぶ。
- helper は byte-backed F4 lookup、caller supplied glyph、consumer next、summary advance/drain、sink traversal、`Vec` / `push`、render/raster/platform/host API を呼ばない。

完了条件:

- F5ak start item helper が collection capacity -> capacity glyph -> start cursor -> F5ag action step -> F5ah step item の順序を守る。
- higher F5ak helper が直接 F5ag/F5ah や lower collection traversal に戻らず、直下の F5ak helper と F5ai/F5aj authority だけを使う。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_start_consumer.n.md` に start item authority、start consumer item、start consume-once、start consume summary、error propagation、no fallback/no byte-backed traversal coverage label を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` で docs、helper body order、call count、forbidden API、括弧なし prefix style を検査する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_start_consumer.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_start_consumer_f5ak.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_consumer_next.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_consumer_next_f5ak_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ak.json -j 1
git diff --check
```

## Phase F5al: sfnt simple glyph outline point stream item collection path sink action consume summary drain

目的:

- F5ak start consume summary と F5aj consume-once をつなぎ、collection-backed action consumer summary を explicit budget 内で terminal まで進める。
- F4aq の byte-backed drain と同じ `EndContour` / `Rejected` / `StepBudgetExhausted` terminal contract を使いながら、F4 byte-backed helper、lower F5 direct traversal、sink mutation、render/raster/platform/host API へ戻らない。
- `summary_state` / `summary_terminal` / F5aj consume-once / pure summary projection の exact one-call boundary を source policy で固定する。

変更:

- Tesla plan review は `PLAN_APPROVED`。F5ak start summary を入口にし、F5aj consume-once を advance-once の唯一の collection-backed continuation authority にする設計が妥当と判断された。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_consume_summary_advance_once` を追加する。
- advance-once helper は source 上 `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state summary` を exactly once 呼ぶ。
- advance-once helper は source 上 `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary` を exactly once 呼ぶ。
- `Continue item` の場合だけ source 上 F5aj `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item_consume_once collection state &item policy` を exactly once 呼ぶ。
- F5aj success の場合だけ source 上 `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step &consume_step` を exactly once 呼ぶ。
- advance-once helper は `Rejected` / `EndContour` を `Result::Ok` の typed terminal として返す。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_consume_summary_drain_budget` を追加する。
- drain helper は budget 判定より前に `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary` を exactly once 呼ぶ。
- `Rejected` と `EndContour` は budget を消費せず current summary と一緒に返す。
- `Continue` かつ `remaining_steps <= 0` は `StepBudgetExhausted current_summary` を返す。
- `Continue` かつ `remaining_steps > 0` の場合だけ F5al advance-once helper を exactly once 呼ぶ。
- advance-once が `Result::Err error` を返した場合は、その contour step error をそのまま返す。
- advance-once が `Continue next_summary` を返した場合は、`remaining_steps - 1` で drain helper を exactly one recursive step だけ進める。
- advance-once が保守上 `Rejected` / `EndContour` を返した場合は、advance-once に渡した current summary を drain result に入れる。
- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_summary_drain_budget` を追加する。
- start drain helper は F5ak `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_summary` を exactly once 呼び、成功時だけ F5al drain helper を exactly once 呼ぶ。
- start drain helper は F5al advance-once、F5aj consume-once、F5ak lower start helper を直接呼ばない。
- helper は F4 byte-backed lookup、lower collection path event / contour / step helper、payload direct match、`Vec` / `push`、sink traversal、render/raster/platform/host API、font fallback を呼ばない。

完了条件:

- F5al advance-once が summary state / terminal を exactly once 読み、`Continue` の場合だけ F5aj consume-once と summary projection を使う。
- F5al drain helper が terminal-before-budget を守り、budget exhaustion と domain terminal と contour step error を混同しない。
- F5al start drain helper が F5ak start summary -> F5al drain だけを合成し、下位 helper へ直接戻らない。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_consume_summary_drain.n.md` に advance-once、terminal handling、drain budget zero/negative、recursive drain、start drain、no fallback/no byte-backed traversal coverage label を追加する。
- `nodesrc/test_web_gui_font_rendering_contract.js` で docs、helper body order、call count、forbidden API、括弧なし prefix style を検査する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_consume_summary_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_consume_summary_drain_f5al.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_start_consumer.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_start_consumer_f5al_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5al.json -j 1
git diff --check
```

## Phase F5am: sfnt simple glyph outline point stream item collection path sink action drain outcome

目的:

- F5al start drain result を、同じ collection の capacity と一緒に後続 outline / path owner boundary へ渡す value-only outcome にする。
- `EndContour` / `Rejected` / `StepBudgetExhausted` の typed terminal を保ち、string fallback、silent no-op、owner allocation、path command push、sink mutation、render/raster/platform/host API へ進まない。
- arbitrary collection と arbitrary drain を外部 caller が組み合わせる public forged pairing API を作らない。

plan review:

- Tesla plan review 1 回目は `PLAN_BLOCKED`。任意の collection と任意の drain result を受け取る public projection helper は、capacity と drain result の forged pairing を許すため危険と指摘された。
- Tesla revised plan review は `PLAN_APPROVED`。public API を start drain outcome helper だけにし、同じ public call 内で F5al start drain を exactly once 呼び、その success result だけを private projection に exactly once 渡す設計が妥当と判断された。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary` を追加する。
- drain summary packet は `GuiSfntSimpleGlyphOutlineStorageCapacity` と `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary` を保持する。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainRejected` を追加する。
- drain rejected packet は `GuiSfntSimpleGlyphOutlineStorageCapacity` と `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryRejected` を保持する。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainOutcome` を追加する。
- outcome enum は `EndContour DrainSummary`、`Rejected DrainRejected`、`StepBudgetExhausted DrainSummary` だけを持つ。
- private `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consume_summary_drain_outcome` を追加する。
- private projection は `gui_sfnt_simple_glyph_outline_point_stream_item_collection_capacity collection` を exactly once 読み、`*drain` を `match` して packet constructor だけを呼ぶ。
- private projection は F5al start/drain/advance、F5ak lower start、F5aj consume-once、F4 byte-backed helper、lower collection path helper、Vec / push、render/raster/platform/host API、font fallback を呼ばない。
- public `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_summary_drain_outcome_budget` を追加する。
- public start outcome helper は F5al `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_summary_drain_budget` を exactly once 呼ぶ。
- public start outcome helper は success path だけ private projection を exactly once 呼ぶ。
- public start outcome helper は F5al advance/drain、F5ak lower start、F5aj consume-once、F4 byte-backed helper、lower collection path helper、Vec / push、render/raster/platform/host API、font fallback を呼ばない。

完了条件:

- public projection API が存在せず、caller が任意の collection / drain result を組み合わせられない。
- capacity は public start outcome helper と同じ collection から private projection 内で exactly once 読まれる。
- `Rejected` は既存 `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryRejected` を capacity と一緒に保持し、文字列化や fallback にしない。
- source policy が docs、types、public API、private projection、call count、forbidden API、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_drain_outcome.n.md` に outcome types、private projection、public forged pairing prevention、F5al start drain composition、terminal mapping、no owner/no fallback/no byte-backed traversal coverage label を追加する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_drain_outcome.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_drain_outcome_f5am.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_consume_summary_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_consume_summary_drain_f5am_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5am.json -j 1
git diff --check
```

## Phase F5an: sfnt simple glyph outline point stream item collection path sink action storage owner

目的:

- F5am の capacity 付き drain outcome を authority とし、`EndContour` だけを F5b outline storage allocation へ進める。
- `Rejected` / `StepBudgetExhausted` は storage allocation error ではなく typed terminal として caller へ戻し、owner を作らない。
- slot population、path command owner fill、sink traversal、render/raster/platform/host API へ進まない。
- caller が別 collection / 別 drain result / byte-backed traversal result を組み合わせて owner allocation へ進める API を作らない。

plan review:

- Tesla plan review は `PLAN_APPROVED`。
- `Result StorageTerminal StorageAllocError` は妥当である。allocation failure だけが新しい fallible operation であり、`Rejected` / `StepBudgetExhausted` は typed domain terminal として `Result::Ok` に残す。
- empty F5b storage owner の allocation だけに留め、slot population / path owner fill は後続 slice へ分ける。
- F5am outcome を authority とし、F5b allocation validation を使うため、F5an で別の collection/drain pairing check は追加しない。ただし separate collection / drain input を public API にしないことを document する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageOwner` を追加する。
- `StorageOwner` は `GuiSfntSimpleGlyphOutlineStorage` と `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary` を保持し、`Clone` / `Copy` を実装しない。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageAllocError` を追加する。
- `StorageAllocError` は `DrainSummary` と `GuiSfntSimpleGlyphOutlineStorageAllocError` を保持し、owner を含まないため `Clone` / `Copy` を実装する。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageTerminal` を追加する。
- terminal enum は `Allocated StorageOwner`、`Rejected DrainRejected`、`StepBudgetExhausted DrainSummary` だけを持ち、`Clone` / `Copy` を実装しない。
- public `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_drain_outcome_alloc_storage_owner` を追加する。
- `EndContour` branch では drain summary から capacity を exactly once 読み、F5b `gui_sfnt_simple_glyph_outline_storage_alloc &capacity limit` を exactly once 呼ぶ。
- allocation 成功時は `Ok Allocated StorageOwner` を返し、allocation 失敗時だけ `Err StorageAllocError` を返す。
- `Rejected` / `StepBudgetExhausted` branch は storage allocation を呼ばず、それぞれ typed terminal を `Ok` で返す。

完了条件:

- source policy が docs、types、owner no Clone/Copy、terminal no Clone/Copy、storage allocation call count、EndContour-only allocation、forbidden API、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_storage_owner.n.md` に EndContour allocation success、allocation failure、Rejected no allocation、StepBudgetExhausted no allocation、summary-preserving error、no fallback/no byte-backed traversal coverage label を追加する。
- implementation review で `Rejected` / `StepBudgetExhausted` branch が allocation を呼ばないこと、owner-bearing types が Clone/Copy でないことを確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_storage_owner.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_storage_owner_f5an.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_drain_outcome.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_drain_outcome_f5an_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5an.json -j 1
git diff --check
```

## Phase F5ao: sfnt simple glyph outline point stream item collection path sink action contour endpoint start

目的:

- F5an の storage terminal を authority とし、`Allocated StorageOwner` だけを F5d contour endpoint region cursor start へ進める。
- public constructor で forged された storage owner を fail-closed にするため、summary capacity と owner 内 storage capacity を非消費で照合する。
- capacity mismatch と cursor start failure では original storage owner を `Result::Err` payload として caller へ戻す。
- cursor start 成功時だけ storage owner を消費し、`previous_endpoint = none` の開始済み owner を返す。
- endpoint push、byte-backed traversal、path sink traversal、render/raster/platform/host API へ進まない。

plan review:

- Tesla plan review 1 回目は `PLAN_BLOCKED`。`StorageOwner` の consuming accessor しかない状態で capacity mismatch や cursor start failure を扱うと、error path が original owner を返せなくなることが blocker とされた。
- revised plan では、`storage_owner_storage_capacity &owner` を非消費 accessor として追加し、`field::get_ref owner "storage"` から既存 storage capacity reader を呼ぶことにした。
- revised plan では、private capacity-match helper が summary capacity と borrowed storage capacity の glyph、contour count、point count、edge count、path command pair count、path command count を照合する。
- Tesla revised plan review は `PLAN_APPROVED`。consuming storage accessor は capacity match と cursor start success の後にだけ現れること、`Rejected` / `StepBudgetExhausted` branch では capacity match / cursor start / storage consume を行わないことが実装条件である。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_storage_owner_storage_capacity` を追加する。
- non-consuming storage capacity accessor は `field::get_ref owner "storage"` と `gui_sfnt_simple_glyph_outline_storage_capacity storage` を使い、consuming `storage_owner_storage owner` を呼ばない。
- private `gui_sfnt_simple_glyph_outline_storage_capacity_matches` と `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_storage_owner_capacity_matches_summary` を追加する。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartOwner` を追加し、storage、summary、cursor、previous_endpoint を保持する。owner 型なので `Clone` / `Copy` を実装しない。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartErrorKind` を追加し、`StorageSummaryCapacityMismatch` と `CursorStartFailed` を持つ。owner を含まないため `Clone` / `Copy` を実装する。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartError` を追加し、original storage owner、kind、cursor_error を保持する。owner を含むため `Clone` / `Copy` を実装しない。
- `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartTerminal` を追加し、`Started ContourEndpointStartOwner`、`Rejected DrainRejected`、`StepBudgetExhausted DrainSummary` を持つ。owner を含むため `Clone` / `Copy` を実装しない。
- public `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_storage_terminal_start_contour_endpoint` を追加する。
- `Allocated` branch では capacity match を先に行い、mismatch は `Err StorageSummaryCapacityMismatch` で original owner を返す。
- capacity match 後に `gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint` を exactly once 呼ぶ。
- cursor start failure は `Err CursorStartFailed` として original owner と cursor error を返す。
- cursor start success 後だけ consuming `storage_owner_storage owner` を exactly once 呼び、`previous_endpoint = none` の `Started` terminal を返す。
- `Rejected` / `StepBudgetExhausted` branch は typed terminal を `Ok` で返し、capacity match、storage capacity read、cursor start、storage consume を行わない。

完了条件:

- source policy が docs、types、storage capacity accessor の `field::get_ref`、capacity match helper、owner no Clone/Copy、error no Clone/Copy、terminal no Clone/Copy、ErrorKind Clone/Copy、cursor start exact one-call、storage consume order、Rejected / StepBudgetExhausted pass-through、forbidden API、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_contour_endpoint_start.n.md` に types、borrowed storage capacity、capacity mismatch owner recovery、allocated cursor start、Rejected / StepBudget pass-through、no fallback / no byte-backed / no push coverage label を追加する。
- implementation review で original owner recovery、storage consume order、`Rejected` / `StepBudgetExhausted` no cursor/no storage consume、owner-bearing types no Clone/Copy を確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_contour_endpoint_start.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_contour_endpoint_start_f5ao.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_storage_owner.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_storage_owner_f5ao_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ao.json -j 1
git diff --check
```

## Phase F5ap: sfnt simple glyph outline point stream item collection path sink action contour endpoint push

目的:

- F5ao の contour endpoint start terminal を authority とし、`Started` branch だけを F5e typed contour endpoint push へ進める。
- endpoint slot を 1 件だけ push し、成功時は returned storage / cursor / previous endpoint を持つ owner を返す。
- push failure では lower error metadata を storage 回収前に読み、returned storage と保存済み summary / cursor / previous endpoint から start owner を復元する。
- `Rejected` / `StepBudgetExhausted` は endpoint push failure ではなく typed terminal として `Ok` で返し、endpoint read、F5e push、storage consume、owner/error construction を行わない。
- byte-backed endpoint read / read-push、F4 lookup、F5al/F5ak/F5aj traversal、path sink traversal、point / curve / path command population、raster/render/platform/host API、font fallback へ進まない。

plan review:

- Tesla plan review は `PLAN_APPROVED`。
- F5ap は F5e `gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint` を exactly once 呼ぶ typed owner-recovery boundary とし、F5e の lower error metadata を storage 回収より前に読むことが条件である。
- success branch は F5e returned cursor / returned storage / returned previous endpoint を使い、入力 endpoint から状態を再計算しない。
- `Rejected` / `StepBudgetExhausted` branch は endpoint を読まず、F5e push、storage consume、owner/error construction も行わない。
- owner-bearing `ContourEndpointPushOwner`、`ContourEndpointPushError`、`ContourEndpointPushTerminal` は `Clone` / `Copy` を実装しない。
- source policy は byte-backed read-push helper である `gui_sfnt_glyf_read_push_contour_endpoint` と `gui_sfnt_glyf_read_contour_endpoint` を明示的に禁止する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushOwner` を追加する。
- push owner は F5e returned storage、F5ao summary、F5e returned cursor、`some` に包んだ returned previous endpoint を保持し、`Clone` / `Copy` を実装しない。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushError` を追加する。
- push error は recovered start owner、rejected endpoint、F5e push error kind、optional region error kind、optional storage push error kind を保持し、`Clone` / `Copy` を実装しない。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushTerminal` を追加する。
- push terminal enum は `Pushed PushOwner`、`Rejected DrainRejected`、`StepBudgetExhausted DrainSummary` だけを持ち、`Clone` / `Copy` を実装しない。
- public `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_contour_endpoint_start_terminal_push_endpoint` を追加する。
- `Started` branch では start owner を消費する前に summary、cursor、previous endpoint を borrow-copy し、start owner storage を exactly once 消費する。
- `Started` branch は F5e `gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage cursor endpoint previous_endpoint` を exactly once 呼ぶ。
- F5e success branch は `gui_sfnt_simple_glyph_contour_endpoint_push_cursor &pushed`、`gui_sfnt_simple_glyph_contour_endpoint_push_previous_endpoint &pushed`、`gui_sfnt_simple_glyph_contour_endpoint_push_storage pushed` の returned state から push owner を構築する。
- F5e error branch は `gui_sfnt_simple_glyph_contour_endpoint_push_error_kind &push_error`、`gui_sfnt_simple_glyph_contour_endpoint_push_error_region_error_kind &push_error`、`gui_sfnt_simple_glyph_contour_endpoint_push_error_push_error_kind &push_error` を読んでから `gui_sfnt_simple_glyph_contour_endpoint_push_error_storage push_error` で storage を回収する。
- error branch は returned storage と保存済み summary / cursor / previous endpoint から `ContourEndpointStartOwner` を復元し、`Result::Err ContourEndpointPushError` を返す。
- `Rejected` / `StepBudgetExhausted` branch は typed terminal を `Ok` で返し、endpoint 引数を読まず、F5e push / storage consume / owner construction / error construction を行わない。

完了条件:

- source policy が docs、types、owner no Clone/Copy、error no Clone/Copy、terminal no Clone/Copy、Started branch の borrow-copy / storage consume / F5e call order、success returned-state use、error metadata-before-storage recovery、pass-through branch の no endpoint/no push/no consume、forbidden byte-backed read-push / traversal / render / platform API、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_contour_endpoint_push.n.md` に types、Started calls F5e once、success returned state、error start owner recovery、Rejected no endpoint/no push、StepBudget no endpoint/no push、no fallback/no byte-backed read-push coverage label を追加する。
- implementation review で lower error metadata read before storage recovery、success returned state usage、pass-through branches、owner-bearing no Clone/Copy、source policy の forbidden API 固定を確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_contour_endpoint_push.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_contour_endpoint_push_f5ap.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_contour_endpoint_start.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_contour_endpoint_start_f5ap_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ap.json -j 1
git diff --check
```

## Phase F5aq: sfnt simple glyph outline point stream item collection path sink action contour endpoint drain

目的:

- F5ap の `ContourEndpointPushOwner` を authority とし、collection-backed contour span から remaining contour endpoint slots を bounded drain する。
- contour endpoint region が完了した場合だけ PointX region cursor を開始し、PointX value push は次 phase に残す。
- PushOwner は public constructor を持つため、summary capacity、owner storage capacity、cursor、collection capacity を検査してから cursor interpretation / span lookup / storage consume へ進む。
- span source failure、F5e push failure、PointX cursor start failure を別 enum reason にし、それぞれ current PushOwner を保持または復元して返す。
- byte-backed endpoint read / read-push、F4 lookup、F5al/F5ak/F5aj traversal、path sink traversal、PointX value push、raster/render/platform/host API、font fallback へ進まない。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。collection contour span は collection 自体の topology だけを検査するため、PushOwner と collection capacity の照合が必要だと指摘された。
- Tesla plan review 2 は `PLAN_BLOCKED`。PushOwner も public constructor を持つため、summary / collection だけでなく owner 内 storage capacity と cursor validity も検査する必要があると指摘された。
- Tesla plan review 3 は `PLAN_APPROVED`。
- authority check は次の順に固定する。
  - summary capacity == owner storage capacity
  - cursor well formed
  - cursor region is `ContourEndpoint`
  - cursor matches summary capacity `ContourEndpoint` region
  - collection capacity == summary capacity
- 各 authority failure は owner-preserving typed error とする。
- `next_index` / `start` / `end` は authority check 後だけ読む。
- completion branch は PointX cursor start のみ行い、PointX value push はしない。
- span failure、PointX cursor failure、F5e push failure は別 typed error とし、current PushOwner を保持または復元する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXStartOwner` を追加する。
- PointXStartOwner は storage、summary、PointX cursor を保持し、`Clone` / `Copy` を実装しない。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointDrainErrorKind` を追加する。
- error kind は `StorageSummaryCapacityMismatch`、`CursorInvalid`、`CursorRegionMismatch`、`CursorCapacityMismatch`、`CollectionSummaryCapacityMismatch`、`EndpointSourceFailed`、`EndpointPushFailed`、`PointXCursorStartFailed` を持つ。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointDrainError` を追加する。
- drain error は current PushOwner、kind、contour_index、optional span source error、optional endpoint、optional F5e/F5d/F5c metadata、optional PointX cursor error を保持し、`Clone` / `Copy` を実装しない。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointDrainTerminal` を追加する。
- terminal は `PointXStarted PointXStartOwner` と `StepBudgetExhausted ContourEndpointPushOwner` のみを持ち、`Clone` / `Copy` を実装しない。
- PushOwner の non-consuming storage capacity accessor `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_contour_endpoint_push_owner_storage_capacity` を追加する。
- internal push helper `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_contour_endpoint_push_owner_push_endpoint` を追加し、F5e `gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage cursor endpoint previous_endpoint` を exactly once 呼ぶ。
- public `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_contour_endpoint_push_owner_drain_to_point_x_start_budget` を追加する。
- public boundary は authority check を終えた後だけ trusted drain helper へ委譲する。
- trusted drain helper は `next_index == end` なら PointX cursor start、`remaining_steps <= 0` なら owner-preserving StepBudget、budget がある場合だけ collection contour span を exactly once 呼ぶ。
- span success では `gui_sfnt_simple_glyph_contour_span_end_point_index &span` から endpoint slot を作る。
- push success は returned state から次 PushOwner を作り `remaining_steps - 1` で継続する。

完了条件:

- source policy が docs、types、PointXStartOwner / DrainError / DrainTerminal no Clone/Copy、PushOwner non-consuming storage capacity accessor、authority check order、span lookup before authority 禁止、span failure owner preservation、push failure owner recovery、PointX cursor failure owner preservation、completion-only PointX cursor start、StepBudget no span/no push、forbidden byte-backed / traversal / render / platform / font fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_contour_endpoint_drain.n.md` に types、authority checks、source span once、span failure recovery、push failure recovery、completion PointX start only、StepBudget no span/no push、no fallback/no byte-backed/no traversal coverage label を追加する。
- implementation review で authority check order、owner-preserving error、F5e lower metadata before storage recovery、PointX value push absence、forbidden API 固定を確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の PointX population boundary に進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_contour_endpoint_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_contour_endpoint_drain_f5aq.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_contour_endpoint_push.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_contour_endpoint_push_f5aq_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5aq.json -j 1
git diff --check
```

## Phase F5ar: sfnt simple glyph outline point stream item collection path sink action PointX drain

目的:

- F5aq の `PointXStartOwner` を authority とし、collection-backed point stream item source から PointX scalar slots を bounded drain する。
- PointX region が完了した場合だけ PointY region cursor を開始し、PointY value push は次 phase に残す。
- PointXStartOwner は public constructor を持つため、summary capacity、owner storage capacity、cursor、collection capacity を検査してから cursor interpretation / collection item read / storage consume へ進む。
- collection read failure、forged item failure、F5g PointX push failure、PointY cursor start failure を別 enum reason にし、それぞれ current PointXStartOwner を保持または復元して返す。
- byte-backed coordinate reader / read-push、F4 lookup、F5al/F5ak/F5aj traversal、path sink traversal、PointY value push、raster/render/platform/host API、font fallback へ進まない。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。internal PointX push helper で F5g lower error metadata を storage 回収前に読むことを docs/source policy に明示する必要があると指摘された。
- Tesla plan review 2 は `PLAN_APPROVED`。
- authority check は次の順に固定する。
  - summary capacity == owner storage capacity
  - cursor well formed
  - cursor region is `PointX`
  - cursor matches summary capacity `PointX` region
  - collection capacity == summary capacity
- `collection_read_item` は collection length / capacity / requested index の検査で十分であり、public-constructor item の glyph / point index / kind forge は F5ar caller 側で再検査する。
- PointX push failure branch は `gui_sfnt_simple_glyph_point_x_push_error_kind &push_error`、`gui_sfnt_simple_glyph_point_x_push_error_point &push_error`、`gui_sfnt_simple_glyph_point_x_push_error_region_error_kind &push_error`、`gui_sfnt_simple_glyph_point_x_push_error_push_error_kind &push_error` を読んでから `gui_sfnt_simple_glyph_point_x_push_error_storage push_error` で storage を回収する。
- completion branch は PointY cursor start のみ行い、PointY value push はしない。
- completion は budget check より前に行い、budget exhaustion は collection read / PointX push より前に行う。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYStartOwner` を追加する。
- PointYStartOwner は storage、summary、PointY cursor を保持し、`Clone` / `Copy` を実装しない。
- PointXStartOwner の non-consuming storage capacity accessor `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_point_x_start_owner_storage_capacity` を追加する。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXDrainErrorKind` を追加する。
- error kind は `StorageSummaryCapacityMismatch`、`CursorInvalid`、`CursorRegionMismatch`、`CursorCapacityMismatch`、`CollectionSummaryCapacityMismatch`、`PointSourceReadFailed`、`PointSourceGlyphMismatch`、`PointSourceIndexMismatch`、`PointSourceKindMismatch`、`PointXPushFailed`、`PointYCursorStartFailed` を持つ。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXDrainError` を追加する。
- drain error は current PointXStartOwner、kind、point_index、optional collection read error、optional item、optional PointX slot、optional F5g/F5d/F5c metadata、optional PointY cursor error を保持し、`Clone` / `Copy` を実装しない。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointXDrainTerminal` を追加する。
- terminal は `PointYStarted PointYStartOwner` と `StepBudgetExhausted PointXStartOwner` のみを持ち、`Clone` / `Copy` を実装しない。
- internal push helper `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_point_x_start_owner_push_point_x` を追加し、F5g `gui_sfnt_simple_glyph_outline_storage_push_point_x storage cursor point` を exactly once 呼ぶ。
- public `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_point_x_start_owner_drain_to_point_y_start_budget` を追加する。
- public boundary は authority check を終えた後だけ trusted drain helper へ委譲する。
- trusted drain helper は `next_index == end` なら PointY cursor start、`remaining_steps <= 0` なら owner-preserving StepBudget、budget がある場合だけ collection read item を exactly once 呼ぶ。
- read success では item point glyph、item point index、item kind を再検査し、成功後だけ `GuiSfntSimpleGlyphPointXSlot` を作って internal PointX push helper へ渡す。
- push success は returned state から次 PointXStartOwner を作り `remaining_steps - 1` で継続する。

完了条件:

- source policy が docs、types、PointYStartOwner / PointXDrainError / PointXDrainTerminal no Clone/Copy、PointXStartOwner non-consuming storage capacity accessor、authority check order、collection read before authority 禁止、forged item glyph/index/kind validation、PointX push failure owner recovery、lower metadata before storage recovery、PointY cursor failure owner preservation、completion-only PointY cursor start、StepBudget no read/no push、forbidden byte-backed / traversal / render / platform / font fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_point_x_drain.n.md` に types、authority checks、source read once、forged item checks、push failure recovery、completion PointY start only、StepBudget no read/no push、no fallback/no byte-backed/no traversal coverage label を追加する。
- implementation review で authority check order、owner-preserving error、F5g lower metadata before storage recovery、PointY value push absence、forged item validation、forbidden API 固定を確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の PointY population boundary に進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_point_x_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_point_x_drain_f5ar.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_contour_endpoint_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_contour_endpoint_drain_f5ar_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5ar.json -j 1
git diff --check
```

## Phase F5as: sfnt simple glyph outline point stream item collection path sink action PointY drain

目的:

- F5ar の `PointYStartOwner` を authority とし、collection-backed point stream item source から PointY scalar slots を bounded drain する。
- PointY region が完了した場合だけ Edge region cursor を開始し、edge value population は次 phase に残す。
- PointYStartOwner は public constructor を持つため、summary capacity、owner storage capacity、cursor、collection capacity を検査してから cursor interpretation / collection item read / storage consume へ進む。
- collection read failure、forged item failure、F5i PointY push failure、Edge cursor start failure を別 enum reason にし、それぞれ current PointYStartOwner を保持または復元して返す。
- byte-backed coordinate reader / read-push、F4 lookup、F5al/F5ak/F5aj traversal、path sink traversal、PointX value push、edge value population、path command population、raster/render/platform/host API、font fallback へ進まない。

plan review:

- Tesla plan review は `PLAN_APPROVED`。
- PointY push helper は F5i lower error metadata を `kind -> point -> region_error_kind -> push_error_kind -> storage` の順で読む。
- PointY push failure branch は `gui_sfnt_simple_glyph_point_y_push_error_kind &push_error`、`gui_sfnt_simple_glyph_point_y_push_error_point &push_error`、`gui_sfnt_simple_glyph_point_y_push_error_region_error_kind &push_error`、`gui_sfnt_simple_glyph_point_y_push_error_push_error_kind &push_error` を読んでから `gui_sfnt_simple_glyph_point_y_push_error_storage push_error` で storage を回収する。
- public authority check 前に `collection_read_item`、`storage_push_point_y`、`cursor_try_from_capacity Edge`、owner storage consume が出ないことを source policy に固定する。
- forbidden API regex は `EdgeStartOwner` / `ScalarRegion::Edge` を誤爆しない粒度にする。禁止対象は edge value population / path traversal / byte-backed lookup であり、Edge cursor start は許可する。
- `collection_read_item` の augmentation は不要で、F5as caller 側の glyph / index / kind 再検査で十分である。
- authority check は次の順に固定する。
  - summary capacity == owner storage capacity
  - cursor well formed
  - cursor region is `PointY`
  - cursor matches summary capacity `PointY` region
  - collection capacity == summary capacity
- completion branch は Edge cursor start のみ行い、edge value push / path command push はしない。
- completion は budget check より前に行い、budget exhaustion は collection read / PointY push より前に行う。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeStartOwner` を追加する。
- EdgeStartOwner は storage、summary、Edge cursor を保持し、`Clone` / `Copy` を実装しない。
- PointYStartOwner の non-consuming storage capacity accessor `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_point_y_start_owner_storage_capacity` を追加する。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYDrainErrorKind` を追加する。
- error kind は `StorageSummaryCapacityMismatch`、`CursorInvalid`、`CursorRegionMismatch`、`CursorCapacityMismatch`、`CollectionSummaryCapacityMismatch`、`PointSourceReadFailed`、`PointSourceGlyphMismatch`、`PointSourceIndexMismatch`、`PointSourceKindMismatch`、`PointYPushFailed`、`EdgeCursorStartFailed` を持つ。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYDrainError` を追加する。
- drain error は current PointYStartOwner、kind、point_index、optional collection read error、optional item、optional PointY slot、optional F5i/F5d/F5c metadata、optional Edge cursor error を保持し、`Clone` / `Copy` を実装しない。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPointYDrainTerminal` を追加する。
- terminal は `EdgeStarted EdgeStartOwner` と `StepBudgetExhausted PointYStartOwner` のみを持ち、`Clone` / `Copy` を実装しない。
- internal push helper `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_point_y_start_owner_push_point_y` を追加し、F5i `gui_sfnt_simple_glyph_outline_storage_push_point_y storage cursor point` を exactly once 呼ぶ。
- public `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_point_y_start_owner_drain_to_edge_start_budget` を追加する。
- public boundary は authority check を終えた後だけ trusted drain helper へ委譲する。
- trusted drain helper は `next_index == end` なら Edge cursor start、`remaining_steps <= 0` なら owner-preserving StepBudget、budget がある場合だけ collection read item を exactly once 呼ぶ。
- read success では item point glyph、item point index、item kind を再検査し、成功後だけ `GuiSfntSimpleGlyphPointYSlot` を作って internal PointY push helper へ渡す。
- push success は returned state から次 PointYStartOwner を作り `remaining_steps - 1` で継続する。

完了条件:

- source policy が docs、types、EdgeStartOwner / PointYDrainError / PointYDrainTerminal no Clone/Copy、PointYStartOwner non-consuming storage capacity accessor、authority check order、collection read before authority 禁止、forged item glyph/index/kind validation、PointY push failure owner recovery、lower metadata before storage recovery、Edge cursor failure owner preservation、completion-only Edge cursor start、StepBudget no read/no push、forbidden byte-backed / traversal / render / platform / font fallback、edge value population 禁止、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_point_y_drain.n.md` に types、authority checks、source read once、forged item checks、push failure recovery、completion Edge start only、StepBudget no read/no push、no fallback/no byte-backed/no traversal coverage label を追加する。
- implementation review で authority check order、owner-preserving error、F5i lower metadata before storage recovery、edge value push absence、forged item validation、forbidden API 固定を確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の Edge population boundary に進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_point_y_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_point_y_drain_f5as.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_point_x_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_point_x_drain_f5as_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5as.json -j 1
git diff --check
```

## Phase F5at: sfnt simple glyph outline point stream item collection path sink action Edge drain

目的:

- F5as の `EdgeStartOwner` を authority とし、owner storage endpoint marker と collection-backed contour span / contour edge source から Edge scalar slots を bounded drain する。
- Edge region が完了した場合だけ PathCommandTag region cursor を開始し、path command tag population と curve segment classification は次 phase に残す。
- EdgeStartOwner は public constructor を持つため、summary capacity、owner storage capacity、cursor、collection capacity を検査してから cursor interpretation / endpoint marker read / collection source / storage consume へ進む。
- endpoint marker failure、contour span failure、contour edge failure、forged source failure、F5d scalar push failure、PathCommandTag cursor start failure を別 enum reason にし、それぞれ current EdgeStartOwner を保持または復元して返す。
- byte-backed lookup、F4/F5al/F5ak/F5aj traversal、path sink traversal、curve segment source、path command tag population、raster/render/platform/host API、font fallback へ進まない。

plan review:

- Tesla plan review は `PLAN_BLOCKED` から修正済み。
- public authority check は `cursor.next_index == Edge region start` を要求しない。partial drain restart を許すため、cursor well formed、cursor region is `Edge`、cursor start/end match summary capacity Edge region のみにする。
- owner storage endpoint marker read は private helper で `field::get_ref owner "storage"` を使い、owner を消費しない。
- Edge region scalar contract は `slot global_edge_index == absolute start point index`、`stored scalar == owning contour_index`、`local edge index == global_edge_index - span.start_point_index` と文書化して検査する。
- F5at は collection curve segment source を呼ばない。curve segment/path command classification は次の PathCommandTag phase に残す。
- Edge push failure branch は `gui_sfnt_simple_glyph_outline_region_push_error_kind &push_error`、`gui_sfnt_simple_glyph_outline_region_push_error_scalar_value &push_error`、`gui_sfnt_simple_glyph_outline_region_push_error_push_error_kind &push_error` を読んでから `gui_sfnt_simple_glyph_outline_region_push_error_storage push_error` で storage を回収する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeSlot` を追加する。
- EdgeSlot は `edge_index`、`contour_index`、`contour_edge_index`、`next_contour_point_index` を保持し、scalar value accessor は `contour_index` を返す。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagStartOwner` を追加する。
- PathCommandTagStartOwner は storage、summary、PathCommandTag cursor を保持し、`Clone` / `Copy` を実装しない。
- EdgeStartOwner の non-consuming storage capacity accessor `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_edge_start_owner_storage_capacity` を追加する。
- private endpoint marker helper `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_edge_start_owner_read_endpoint_marker` を追加する。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeDrainErrorKind` を追加する。
- error kind は `StorageSummaryCapacityMismatch`、`CursorInvalid`、`CursorRegionMismatch`、`CursorCapacityMismatch`、`CollectionSummaryCapacityMismatch`、`EndpointMarkerReadFailed`、`EndpointMarkerGlyphMismatch`、`EndpointMarkerIndexMismatch`、`ContourSpanSourceFailed`、`ContourSpanInvariantMismatch`、`ContourEdgeSourceFailed`、`EdgeSourceContourMismatch`、`EdgeSourceIndexMismatch`、`EdgeSourceNextIndexMismatch`、`EdgePushFailed`、`PathCommandTagCursorStartFailed` を持つ。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeDrainError` を追加する。
- drain error は current EdgeStartOwner、kind、edge_index、optional endpoint marker error、optional contour span error、optional span、optional contour edge error、optional edge、optional EdgeSlot、optional scalar value、optional F5d/F5c metadata、optional PathCommandTag cursor error を保持し、`Clone` / `Copy` を実装しない。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionEdgeDrainTerminal` を追加する。
- terminal は `PathCommandTagStarted PathCommandTagStartOwner` と `StepBudgetExhausted EdgeStartOwner` のみを持ち、`Clone` / `Copy` を実装しない。
- internal push helper `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_edge_start_owner_push_edge` を追加し、F5d `gui_sfnt_simple_glyph_outline_storage_push_region_scalar storage cursor scalar_value` を exactly once 呼ぶ。
- public `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_edge_start_owner_drain_to_path_command_tag_start_budget` を追加する。
- public boundary は authority check を終えた後だけ trusted drain helper へ委譲する。
- trusted drain helper は `next_index == end` なら PathCommandTag cursor start、`remaining_steps <= 0` なら owner-preserving StepBudget、budget がある場合だけ endpoint marker read と collection contour source へ進む。
- endpoint marker success では marker glyph と marker point index を検査する。
- contour span success では glyph/index/range/count と `span.start_point_index <= global_edge_index <= span.end_point_index` を検査する。
- contour edge success では edge contour、local edge index、absolute start point index、wrap next local index を検査し、成功後だけ EdgeSlot を作る。
- push success は returned state から次 EdgeStartOwner を作り `remaining_steps - 1` で継続する。

完了条件:

- source policy が docs、types、PathCommandTagStartOwner / EdgeDrainError / EdgeDrainTerminal no Clone/Copy、EdgeSlot Clone/Copy、EdgeStartOwner non-consuming storage capacity accessor、non-consuming endpoint marker helper、authority check order、cursor start 固定禁止、source before authority 禁止、completion-only PathCommandTag cursor start、StepBudget no endpoint/source/push、endpoint marker forged checks、span invariant checks、contour edge invariant checks、Edge push failure owner recovery、lower metadata before storage recovery、forbidden byte-backed / traversal / curve segment / render / platform / font fallback、path command tag population 禁止、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_edge_drain.n.md` に types、authority checks、endpoint marker source、span/edge source validation、push failure recovery、completion PathCommandTag start only、StepBudget no source/no push、no fallback/no byte-backed/no traversal/no curve segment coverage label を追加する。
- implementation review で cursor partial restart、owner-preserving error、F5d lower metadata before storage recovery、PathCommandTag cursor start only、forbidden API 固定を確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の PathCommandTag population boundary に進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_edge_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_edge_drain_f5at.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_point_y_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_point_y_drain_f5at_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5at.json -j 1
git diff --check
```

## Phase F5au: sfnt simple glyph outline point stream item collection path sink action PathCommandTag drain

目的:

- F5at の `PathCommandTagStartOwner` を authority とし、owner storage Edge scalar と collection-backed path sink event kind source から PathCommandTag scalar slots を bounded drain する。
- PathCommandTag region が完了した場合だけ `PathCommandTagCompleteOwner` を返し、path command stream / raster preparation boundary は次 phase に残す。
- `PathCommandTagStartOwner` は public constructor を持つため、summary capacity、owner storage capacity、cursor、collection capacity を検査してから cursor interpretation / Edge owner scalar read / collection source / storage consume へ進む。
- Edge owner read failure、forged Edge owner mismatch、contour span failure、event kind source failure、F5d scalar push failure を別 enum reason にし、それぞれ current PathCommandTagStartOwner を保持または復元して返す。
- byte-backed lookup、F4/F5al/F5ak/F5aj traversal、old path sink action consumer、path command stream construction、raster/render/platform/host API、font fallback へ進まない。

plan review:

- Tesla plan review 1 は `PLAN_BLOCKED`。partial restart、logical path command index、non-consuming Edge owner scalar read、F5d push metadata order、forged Edge owner scalar checks の明文化が不足していた。
- Tesla plan review 2 は `PLAN_APPROVED`。以下を実装前 contract として固定する。
- public authority check は `cursor.next_index == PathCommandTag region start` を要求しない。partial drain restart を許すため、cursor well formed、cursor region is `PathCommandTag`、cursor start/end match summary capacity PathCommandTag region のみにする。
- logical path command index は `cursor.next_index - cursor.start` とし、absolute cursor `next_index` を command index として使わない。
- edge index は `div_s path_command_index 2`、event slot ordinal は `rem_s path_command_index 2` で導出し、ordinal は 0/1 だけを許す。
- Edge owner scalar read は storage-level helper `gui_sfnt_simple_glyph_outline_storage_read_edge_owner &storage edge_index` と private PathCommandTagStartOwner helper だけで行い、owner storage を消費しない。
- Edge owner marker success 後に marker glyph / edge index を再検査し、collection span で `edge_index` が span 内にあることを検査してから `contour_edge_index = edge_index - span.start_point_index` を導出する。
- F5d push failure branch は `gui_sfnt_simple_glyph_outline_region_push_error_kind &push_error`、`gui_sfnt_simple_glyph_outline_region_push_error_scalar_value &push_error`、`gui_sfnt_simple_glyph_outline_region_push_error_push_error_kind &push_error` を読んでから `gui_sfnt_simple_glyph_outline_region_push_error_storage push_error` で storage を回収する。
- `SkipNoSegment` reason は PathCommandTag scalar に保存せず、後続の path command value / stream boundary が collection-backed source から再導出する。

変更:

- `alloc/gui/font/sfnt/glyf.nepl` に value-only enum `GuiSfntSimpleGlyphPathCommandTag` を追加する。
- `GuiSfntSimpleGlyphPathCommandTag` は `MoveTo`、`LineTo`、`QuadraticTo`、`SkipNoSegment` を持ち、`Clone` / `Copy` を実装する。
- `gui_sfnt_simple_glyph_path_command_tag_from_sink_event_kind` と `gui_sfnt_simple_glyph_path_command_tag_scalar_value` を追加する。stable scalar value は `MoveTo = 1`、`LineTo = 2`、`QuadraticTo = 3`、`SkipNoSegment = 4` とする。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlineEdgeOwnerMarker`、`GuiSfntSimpleGlyphOutlineEdgeOwnerReadErrorKind`、`GuiSfntSimpleGlyphOutlineEdgeOwnerReadError`、`gui_sfnt_simple_glyph_outline_storage_read_edge_owner` を追加する。
- storage-level Edge owner read helper は capacity shape、scalar slot count、scalar storage capacity、edge index range、Edge slot readiness、slot presence、stored contour index range を検査し、storage owner を消費しない。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagSlot` を追加する。
- PathCommandTagSlot は `path_command_index`、`edge_index`、`contour_index`、`contour_edge_index`、`event_slot`、`tag` を保持し、scalar value accessor は tag scalar value を返す。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagCompleteOwner` を追加する。
- CompleteOwner は storage、summary を保持し、`Clone` / `Copy` を実装しない。
- PathCommandTagStartOwner の non-consuming storage capacity accessor `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_start_owner_storage_capacity` を追加する。
- private Edge owner helper `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_start_owner_read_edge_owner` を追加する。
- `alloc/gui/font/sfnt/glyf.nepl` に `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagDrainErrorKind` を追加する。
- error kind は `StorageSummaryCapacityMismatch`、`CursorInvalid`、`CursorRegionMismatch`、`CursorCapacityMismatch`、`CollectionSummaryCapacityMismatch`、`PathCommandIndexInvalid`、`EventSlotOrdinalInvalid`、`EdgeOwnerReadFailed`、`EdgeOwnerGlyphMismatch`、`EdgeOwnerIndexMismatch`、`ContourSpanSourceFailed`、`ContourSpanInvariantMismatch`、`EventKindSourceFailed`、`TagPushFailed` を持つ。
- `alloc/gui/font/sfnt/glyf.nepl` に owner-bearing `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagDrainError` と terminal `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagDrainTerminal` を追加する。
- terminal は `PathCommandTagCompleted PathCommandTagCompleteOwner` と `StepBudgetExhausted PathCommandTagStartOwner` のみを持ち、`Clone` / `Copy` を実装しない。
- internal push helper `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_start_owner_push_tag` を追加し、F5d `gui_sfnt_simple_glyph_outline_storage_push_region_scalar storage cursor scalar_value` を exactly once 呼ぶ。
- public `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_start_owner_drain_to_complete_budget` を追加する。
- public boundary は authority check を終えた後だけ trusted drain helper へ委譲する。
- trusted drain helper は `next_index == end` なら `PathCommandTagCompleteOwner`、`remaining_steps <= 0` なら owner-preserving StepBudget、budget がある場合だけ Edge owner scalar read と collection source へ進む。
- push success は returned state から次 PathCommandTagStartOwner を作り `remaining_steps - 1` で継続する。

完了条件:

- source policy が docs、types、PathCommandTagCompleteOwner / PathCommandTagDrainError / PathCommandTagDrainTerminal no Clone/Copy、PathCommandTagSlot / PathCommandTag / EdgeOwnerMarker / EdgeOwnerReadError Clone/Copy、PathCommandTagStartOwner non-consuming storage capacity accessor、non-consuming Edge owner helper、authority check order、cursor start 固定禁止、source before authority 禁止、logical path command index mapping、completion-only CompleteOwner、StepBudget no Edge owner read/no source/no push、Edge owner forged checks、span invariant checks、event kind source、tag push failure owner recovery、lower metadata before storage recovery、forbidden byte-backed / traversal / path command stream / render / platform / font fallback、括弧なし prefix style、focused doctest coverage label を検査する。
- `tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_path_command_tag_drain.n.md` に types、authority checks、partial restart、logical index mapping、Edge owner non-consuming read、span/event source checks、push failure recovery、completion CompleteOwner only、StepBudget no source/no push、no fallback/no byte-backed/no traversal/no raster の coverage label を追加する。
- implementation review で cursor partial restart、owner-preserving error、F5d lower metadata before storage recovery、completion-only CompleteOwner、forbidden API 固定を確認する。
- `note.n.md` に plan review、実装、検証、subagent 実装レビュー、残件を記録する。
- `todo.md` は次の path command value / stream preparation boundary に進める。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_path_command_tag_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_path_command_tag_drain_f5au.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i tests/stdlib/gui_font_sfnt_glyf_outline_point_stream_item_collection_path_sink_action_edge_drain.n.md --no-tree -o tmp_gui_font_outline_point_stream_item_collection_path_sink_action_edge_drain_f5au_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='180000'; node nodesrc/tests.js -i stdlib/alloc/gui/font/sfnt/glyf.nepl --no-tree -o tmp_gui_font_glyf_f5au.json -j 1
git diff --check
```

## Phase F5co: row tile RLE packet typed record reader and present run cursor

目的:

- F5cn の `GuiRgba8888RowTileRlePresentFrameOwner` を、host import へ飛ばす前に presenter-neutral な typed run cursor へ接続する。
- `row_tile_rle_packet` と `row_tile_rle_encoded` の no-reader contract は維持し、raw storage read は `row_tile_rle_packet_record` の quarantined typed record reader にだけ閉じる。
- Web / native / bare / headless presenter は後続 phase でこの cursor を消費し、packet storage へ直接到達しない。

変更:

- `alloc/gui/render2d/row_tile_rle_packet_record.nepl` を追加する。
- `GuiRgba8888RowTileRlePacketRecordReadErrorKind` を追加し、count、index、byte offset、projection/load、decoded i32、channel、run extent の失敗を enum で分ける。
- `gui_rgba8888_row_tile_rle_packet_record_at &packet record_index` を追加し、12 byte record を `GuiRgba8888RowTileRleRun` に戻す。
- `std/gui/tile_present_run_cursor.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentRunCursorOwner` は present owner、next record index、total run count を保持し、Clone / Copy を実装しない。
- `GuiRgba8888RowTileRlePresentRunCursorStepResult` は `RunReady run` と `Completed` を持つ Copy enum とする。
- step は `record_index == total_run_count` を explicit completion とし、`>` は owner-bearing error にする。
- lower record read failure は `PacketRecordReadFailed %GuiRgba8888RowTileRlePacketRecordReadErrorKind` として cursor owner を保持する。
- `alloc/gui/render2d.nepl` と `std/gui.nepl` の facade に追加する。
- `tests/stdlib/gui_render2d_row_tile_rle_packet_record.n.md` と `tests/stdlib/gui_std_tile_present_run_cursor.n.md` を追加する。
- source policy に F5co を追加し、typed record reader だけに raw projection / load を許し、std cursor に raw memory / host / platform が入らないことを固定する。
- `note.n.md` と `todo.md` を更新する。

完了条件:

- `row_tile_rle_packet_record` は public raw storage accessor、public byte reader、`Vec`、host/platform、video memory、Canvas / DOM / minifb、fallback を持たない。
- `tile_present_run_cursor` は host import、raw memory、surface command、video memory、fallback を持たない。
- existing F5cl / F5cm / F5cn source policy は壊さない。
- focused doctest、source policy、F5cn regression、`git diff --check` が通る。
- subagent implementation review で raw read quarantine、owner recovery、Completed と invalid index の分離が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_packet_record.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_packet_record_f5co.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_run_cursor.n.md --no-tree -o tmp_gui_std_tile_present_run_cursor_f5co.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present.n.md --no-tree -o tmp_gui_std_tile_present_f5co_regression.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_render2d_row_tile_rle_packet.n.md --no-tree -o tmp_gui_render2d_row_tile_rle_packet_f5co_regression.json -j 1
git diff --check
```

## Phase F5cp: std row tile RLE present command cursor

目的:

- F5co の `GuiRgba8888RowTileRlePresentRunCursorOwner` を、Web / native / bare / headless presenter が共通に消費できる std layer row tile RLE present command cursor へ昇格する。
- host import へ進む前に `BeginFrame`、`Run`、`EndFrame` の typed command stream を固定する。
- command cursor は F5co does not bypass F5co の境界であり、packet storage や raw record reader を再読しない。

変更:

- `std/gui/tile_present_command_cursor.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentCommand` に `GuiRgba8888RowTileRlePresentCommand::BeginFrame`、`Run`、`GuiRgba8888RowTileRlePresentCommand::EndFrame` を定義する。
- `GuiRgba8888RowTileRlePresentCommandCursorOwner` は lower run cursor owner、descriptor copy、phase を保持し、Clone / Copy を実装しない。
- `BeginPending`、`RunPending`、`Completed` の phase enum を定義する。
- public step は one typed output per public step を守り、lower `Completed` を同じ step の EndFrame command として返す。
- lower start / step failure は owner-bearing error に包み、present owner または command cursor owner を失わない。
- `std/gui.nepl` facade に再公開を追加する。
- `tests/stdlib/gui_std_tile_present_command_cursor.n.md` と source policy を追加する。
- `note.n.md` と `todo.md` を更新する。

完了条件:

- command cursor は F5co の present run cursor だけに依存する。
- command cursor は `gui_rgba8888_row_tile_rle_packet_record_at`、packet storage、`RegionToken`、`MemPtr`、byte load helper、host import、platform API、video memory、Canvas / DOM / minifb、fallback、silent no-op を使わない。
- start failure と step failure は lower owner を recover し、caller が recovery または free を選べる。
- focused doctest、source policy、F5co regression、`git diff --check` が通る。
- subagent implementation review で command stream 契約、owner recovery、raw boundary bypass 禁止が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_command_cursor.n.md --no-tree -o tmp_gui_std_tile_present_command_cursor_f5cp.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_run_cursor.n.md --no-tree -o tmp_gui_std_tile_present_run_cursor_f5cp_regression.json -j 1
git diff --check
```

## Phase F5cq: std row tile RLE present host-command record

目的:

- F5cp の command cursor step を、Web / native / bare / headless presenter の formal ABI に渡せる std layer row tile RLE present host-command record へ写す。
- actual host import には進まず、`BeginFrame descriptor`、`RunRecord run_record`、`EndFrame descriptor` の invalid-state-free enum shape だけを固定する。
- host-command record does not bypass F5cp。F5co run cursor、packet record reader、raw storage、host/platform/video memory API へ直接到達しない。

変更:

- `std/gui/tile_present_command_cursor.nepl` に public step descriptor accessor を追加する。
- `std/gui/tile_present_host_command.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostCommandRunRecord` を定義し、descriptor と `GuiRgba8888RowTileRleRun` の両方を持たせる。
- `GuiRgba8888RowTileRlePresentHostCommandRecord` を定義し、Run 用 variant は run record を 1 payload として持つ。
- `GuiRgba8888RowTileRlePresentHostCommandStepResult` を定義し、record と terminal Completed を分ける。
- `gui_rgba8888_row_tile_rle_present_host_command_step_result` は F5cp の public accessor だけから record を作る。
- `std/gui.nepl` facade に再公開を追加する。
- `tests/stdlib/gui_std_tile_present_host_command.n.md` と source policy を追加する。
- `note.n.md` と `todo.md` を更新する。

完了条件:

- host-command record は does not flatten to kind plus optional run。
- F5cq は `tile_present_run_cursor`、packet record reader、packet storage、`RegionToken`、`MemPtr`、byte load helper、host import、platform API、video memory、Canvas / DOM / minifb、fallback、silent no-op を使わない。
- F5cq は F5cp step 内部の owner field を直接読まず、F5cp の public accessor を使う。
- focused doctest、source policy、F5cp regression、`git diff --check` が通る。
- subagent implementation review で record shape、F5cp-only dependency、owner field bypass 禁止が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_command.n.md --no-tree -o tmp_gui_std_tile_present_host_command_f5cq.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_command_cursor.n.md --no-tree -o tmp_gui_std_tile_present_command_cursor_f5cq_regression.json -j 1
git diff --check
```

## Phase F5cr: std row tile RLE present host import request

目的:

- F5cq の host-command record を formal host import request へ写す std layer boundary を追加する。
- actual Web / native / bare presenter や host import call には進まず、request target と capability validation を固定する。
- F5cr は std layer row tile RLE present host import request の checkpoint であり、後続 presenter が受け取る request value の形だけを固定する。
- Headless is not a presentation target。headless / text grid は presentation request では `GuiError::Unsupported` とし、検査は host-command record drain で行う。

変更:

- `std/gui/tile_present_host_import.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostImportTarget` を定義し、target は `Window WindowId`、`Offscreen`、`Device` に限定する。
- `GuiRgba8888RowTileRlePresentHostImportRequest` を定義し、target と `GuiRgba8888RowTileRlePresentHostCommandRecord` を保持する。
- `gui_rgba8888_row_tile_rle_present_host_import_request` は `GuiHost` capability を検査し、`ColorFormat::FormatRgba8888` でない host を `GuiError::Unsupported` にする。
- Window target は `SurfaceKind::WindowPixel`、windowing capability、`default_window = Some` を同時に要求する。
- `std/gui.nepl` facade、focused doctest、source policy、note / todo を更新する。

完了条件:

- F5cr は `std/gui/tile_present_host_command` の F5cq record だけを消費し、F5cp / F5co cursor、packet record / storage、old `GuiSurfacePresentCommand`、platform API、video memory、Canvas / DOM / minifb、fallback、silent no-op に触れない。
- `GuiRgba8888RowTileRlePresentHostImportTarget` は headless target を持たない。
- `FormatRgba8888` の検査は target selection より前に行う。
- focused doctest、source policy、F5cq regression、`git diff --check` が通る。
- subagent implementation review で target model、headless rejection、RGBA8888 validation、F5cq-only dependency が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_import.n.md --no-tree -o tmp_gui_std_tile_present_host_import_f5cr.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_command.n.md --no-tree -o tmp_gui_std_tile_present_host_command_f5cr_regression.json -j 1
git diff --check
```

## Phase F5cs: std row tile RLE present virtual drain

目的:

- headless / test 用に、F5cq host-command record を検査する std layer row tile RLE present virtual drain を追加する。
- `GuiRgba8888RowTileRlePresentVirtualDrain` は presentation target ではなく、does not consume F5cr。
- Begin / Run / End の順序、descriptor の一致、expected run / pixel count、`run_pixel_offset == seen_pixel_count` を検査し、gap / overlap / reorder を拒否する。

変更:

- `std/gui/tile_present.nepl` に descriptor expected run / pixel count accessor を追加する。
- `std/gui/tile_present_virtual_drain.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentVirtualDrainPhase` を `WaitingBegin`、`InFrame`、`Ended` として定義する。
- `GuiRgba8888RowTileRlePresentVirtualDrain` は phase、`Option SurfaceId`、`Option FrameId`、expected / seen run count、expected / seen pixel count を保持する。
- `GuiRgba8888RowTileRlePresentVirtualDrainErrorKind` と error value を定義し、失敗時に直前 drain state を保持する。
- source policy と focused doctest を追加する。

完了条件:

- F5cs は F5cq host-command record だけを消費し、F5cr request、F5cp / F5co cursor、packet storage / record reader / owner、old `GuiSurfacePresentCommand`、platform API、video memory、Canvas / DOM / minifb、fallback、silent no-op に触れない。
- RunRecord では `run_pixel_offset == seen_pixel_count`、checked run end、expected pixel bound、expected run bound を検査する。
- EndFrame は expected run count と expected pixel count を満たした場合だけ success になる。
- focused doctest、source policy、F5cr / F5cq regression、`git diff --check` が通る。
- subagent implementation review で F5cq-only input、F5cr 非依存、run offset continuity、descriptor accessor boundary が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_virtual_drain.n.md --no-tree -o tmp_gui_std_tile_present_virtual_drain_f5cs.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_virtual_drain.nepl --no-tree -o tmp_gui_std_tile_present_virtual_drain_module_f5cs.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_import.n.md --no-tree -o tmp_gui_std_tile_present_host_import_f5cs_regression.json -j 1
git diff --check
```

## Phase F5ct: std row tile RLE present schedule boundary

目的:

- F5cq host-command record stream を、platform host へ渡す前の deterministic slice budget で区切る std layer row tile RLE present schedule boundary を追加する。
- `GuiRgba8888RowTileRlePresentScheduleState` は F5cs virtual drain state と slice-local counters だけを保持する。
- Begin / Run / End の順序、descriptor consistency、run offset continuity は F5cs virtual drain を single authority とし、scheduler は再実装しない。
- `Yield means exact slice budget` とし、over-budget is a typed error として扱う。

変更:

- `std/gui/tile_present_schedule.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentSchedulePolicy` を追加し、`max_commands_per_slice` と `max_pixels_per_slice` を positive `Result` constructor で検査する。
- `GuiRgba8888RowTileRlePresentScheduleState` を追加し、F5cs drain と `slice_command_count` / `slice_pixel_count` を保持する。
- `GuiRgba8888RowTileRlePresentSchedulePhase` を `Continue` / `Yield` / `Completed` として定義する。
- `GuiRgba8888RowTileRlePresentScheduleStepErrorKind` を追加し、policy invalid、slice counter invalid、checked add overflow、single run over pixel budget、command / pixel over-budget、lower F5cs failure を enum で分ける。
- step は record cost を読み、single run が pixel budget を超える場合は F5cs に渡す前に typed error にする。
- step は F5cs `gui_rgba8888_row_tile_rle_present_virtual_drain_step` を呼び、lower error は lower kind / category を保持した schedule error に包む。
- checked add 後、F5cs drain が ended なら `Completed`、budget 超過なら error、budget ちょうどなら `Yield`、それ以外は `Continue` を返す。
- `resume_slice` は slice counters だけを 0 に戻し、F5cs drain state を保持する。
- `std/gui.nepl` facade、focused doctest、source policy、note / todo を更新する。

完了条件:

- F5ct は F5cq host-command record だけを消費し、F5cs virtual drain を validation authority とする。
- F5ct は F5cr request、F5cp / F5co cursor、packet record / storage / owner、old `GuiSurfacePresentCommand`、timer、queue、platform API、video memory、Canvas / DOM / minifb、fallback、silent no-op に触れない。
- `Yield` は exact budget に限定し、budget 超過は `GuiRgba8888RowTileRlePresentScheduleStepErrorKind` と previous schedule state を返す。
- focused doctest、source policy、F5cs / F5cr / F5cq regression、`git diff --check` が通る。
- subagent implementation review で F5cs authority、over-budget error、resume slice semantics、禁止依存が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_schedule.n.md --no-tree -o tmp_gui_std_tile_present_schedule_f5ct.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_schedule.nepl --no-tree -o tmp_gui_std_tile_present_schedule_module_f5ct.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_virtual_drain.n.md --no-tree -o tmp_gui_std_tile_present_virtual_drain_f5ct_regression.json -j 1
git diff --check
```

## Phase F5cu: std row tile RLE present scheduled dispatch boundary

目的:

- F5ct schedule state と F5cr host import request construction を接続し、actual host import execution の手前で typed dispatch value を作る。
- F5cu は std layer row tile RLE present scheduled dispatch boundary の checkpoint であり、request construction と host import execution を分離する。
- F5ct before F5cr の順序を守り、stream validation / budget decision が成功した record だけを host import request value に包む。
- success path は `RequestReady request plus post phase` とし、exact-budget record の request と `Yield`、EndFrame request と `Completed` を同時に保持する。

変更:

- `std/gui/tile_present_dispatch.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentDispatchState` は `GuiRgba8888RowTileRlePresentScheduleState` だけを保持する。
- `GuiRgba8888RowTileRlePresentDispatchPostPhase` を `Continue` / `Yield` / `Completed` として定義する。
- `GuiRgba8888RowTileRlePresentDispatchReadyRequest` に `GuiRgba8888RowTileRlePresentHostImportRequest` と post phase を同時に保持する。
- `GuiRgba8888RowTileRlePresentDispatchOutput` は `RequestReady ready_request` を持つ。`Option request + phase` にはしない。
- step error は `ScheduleFailed lower_kind` と `HostImportRequestFailed host_error` を enum で分け、category と previous dispatch state を保持する。
- `resume_slice` は F5ct `gui_rgba8888_row_tile_rle_present_schedule_state_resume_slice` へ委譲する。
- `std/gui.nepl` facade、focused doctest、source policy、note / todo を更新する。

完了条件:

- F5cu は F5ct と F5cr だけを実装上の authority とし、F5cs を直接呼ばない。
- F5cu は F5cp / F5co cursor、raw packet storage、queue、timer、host import execution、platform API、Canvas / DOM / minifb、video memory、fallback、silent no-op に触れない。
- F5ct error と F5cr error は previous dispatch state を返す。F5cr error で updated schedule state を採用しない。
- focused doctest、source policy、F5ct / F5cr / F5cs regression、`git diff --check` が通る。
- subagent implementation review で request/post phase shape、F5ct-before-F5cr order、error state preservation、禁止依存が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_dispatch.n.md --no-tree -o tmp_gui_std_tile_present_dispatch_f5cu.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_dispatch.nepl --no-tree -o tmp_gui_std_tile_present_dispatch_module_f5cu.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_schedule.n.md --no-tree -o tmp_gui_std_tile_present_schedule_f5cu_regression.json -j 1
git diff --check
```

## Phase F5cv: std row tile RLE present dispatch loop outcome boundary

目的:

- F5cu の `RequestReady request plus post phase` を、future platform executor の host outcome と接続する std layer loop boundary に包む。
- F5cv は std layer row tile RLE present dispatch loop outcome boundary の checkpoint である。
- host import execution はまだ行わず、request submission 前後の state transition を one-shot pending value として固定する。
- `GuiRgba8888RowTileRlePresentDispatchLoopPendingRequest` が previous state、next state、request、post phase を保持し、host outcome が Err なら previous state、Ok なら next state へ進む contract にする。
- `complete_request consumes pending` ことにより、同じ host outcome の二重完了や next state の replay を避ける。

変更:

- `std/gui/tile_present_dispatch_loop.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentDispatchLoopState` は `GuiRgba8888RowTileRlePresentDispatchState` だけを保持する。
- `GuiRgba8888RowTileRlePresentDispatchLoopPendingRequest` は previous state、next state、`GuiRgba8888RowTileRlePresentHostImportRequest`、post phase を保持する。Clone / Copy は実装しない。
- `GuiRgba8888RowTileRlePresentDispatchLoopCompletion` を `Continue state` / `Yield state` / `Completed state` として定義する。
- error kind は `DispatchFailed lower_kind` と `HostImportExecutionFailed host_error` を enum で分け、category と rollback state を保持する。
- `dispatch_loop_step_record` は F5cu だけを呼び、success path で pending request を返す。
- `complete_request` は pending value と `Result unit GuiError` を受け、Err なら previous state を持つ error、Ok なら post phase に対応した completion を返す。
- `std/gui.nepl` facade、focused doctest、source policy、note / todo を更新する。

完了条件:

- F5cv は F5cu だけを実装上の authority とし、F5ct / F5cr / F5cs direct call を持たない。
- PendingRequest と Step は Clone / Copy を持たず、completion boundary は pending value を消費する。
- host outcome Err は previous state を返し、Ok は next state を Continue / Yield / Completed に包む。
- F5cv は lower cursors、raw packet storage、queue、timer、scheduler、host import execution、platform API、Canvas / DOM / minifb、video memory、fallback、silent no-op に触れない。
- focused doctest、source policy、F5cu / F5ct / F5cr regression、`git diff --check` が通る。
- subagent implementation review で one-shot pending、previous / next state、outcome mapping、禁止依存が承認される。

検証:

```powershell
node --check nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_dispatch_loop.n.md --no-tree -o tmp_gui_std_tile_present_dispatch_loop_f5cv.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i stdlib/std/gui/tile_present_dispatch_loop.nepl --no-tree -o tmp_gui_std_tile_present_dispatch_loop_module_f5cv.json -j 1
$env:NEPL_TEST_CASE_TIMEOUT_MS='60000'; node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_dispatch.n.md --no-tree -o tmp_gui_std_tile_present_dispatch_f5cv_regression.json -j 1
git diff --check
```
