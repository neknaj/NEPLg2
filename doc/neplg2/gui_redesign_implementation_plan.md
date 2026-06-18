# NEPLg2 GUI bitmap surface implementation plan

作成日: 2026-06-13

## 目的

この文書は `gui_redesign_spec.md` と `gui_redesign_detailed_design.md` に基づく実装計画である。実装は doc review gate を通過してから開始する。

## 実装開始 gate

実装開始前に次を満たす。

1. `gui_redesign_spec.md`、`gui_redesign_detailed_design.md`、`gui_redesign_implementation_plan.md` が存在する。
2. 既存の `gui_standard_library_spec.md` と `gui_tui_implementation_plan.md` が、新設 3 文書と矛盾しない。特に `SurfaceKind`、stdout transport、Canvas2D adapter、fallback 表現、same app code contract を揃える。
3. subagent が 5 文書を読み、Zenn 方針、no fallback、platform abstraction、testability を確認する。
4. subagent が `implementation may start` 相当の結論を返す。
5. Blocker / Required 指摘がある場合は doc を修正し、再 review する。

この gate を満たすまで stdlib / Web / examples の実装変更は行わない。

## Phase 1: documentation and policy

変更:

- GUI bitmap surface redesign の 3 文書を追加する。
- 現行 `gui_standard_library_spec.md` と `gui_tui_implementation_plan.md` は、正式 path が bitmap video memory であり、stdout protocol は legacy smoke transport であることへ更新する。
- `SurfaceKind` は `WindowPixel` / `OffscreenPixel` / `DevicePixel` / `TextGrid` / `Headless` に揃え、旧 `Pixel` / `Command` の意味を移行注記に閉じる。
- Web presentation は 2 slot 以上の video memory ownership protocol と `putImageData` に限定し、単一 buffer の共有読み書きは禁止する。
- 同じ NEPL app code が Web / native / bare / headless へ接続される正式 path は host surface ABI とし、`platforms/gui/web/stdout_protocol.nepl` は app-facing formal path にしない。
- source policy test の追加方針を決める。

Review:

- subagent に doc review を依頼する。
- 指摘を修正し、再 review する。

検証:

```powershell
git diff --check
```

## Phase 2: Web bitmap renderer slice

目的:

- visible canvas direct drawing を廃止する。
- 現行 frame DTO を維持したまま、`DrawCommand -> PixelBuffer -> putImageData` へ経路を変える。

変更:

- `web/src/gui-preview/bitmap-buffer.ts` を追加する。
- `web/src/gui-preview/bitmap-rasterizer.ts` を追加する。
- `web/src/gui-preview/bitmap-presenter.ts` を追加する。
- `web/src/gui-preview/canvas-renderer.ts` を bitmap renderer facade に変更する。
- `nodesrc/test_web_gui_preview_renderer.js` を更新し、visible renderer の Canvas2D primitive 使用禁止を検査する。

初期 command 対応:

- `fill-rect`
- `rgba-row`
- `text-run` with deterministic ASCII bitmap text

禁止する visible canvas API:

- `ctx.fillRect`
- `ctx.strokeRect`
- `ctx.fillText`
- `ctx.strokeText`
- `ctx.stroke`
- `ctx.drawImage`
- app content の canvas background clear

Visible canvas context は `ImageData` 作成と `putImageData` presentation 以外に使わない。Background clear は pixel buffer 側で行う。

検証:

```powershell
node nodesrc/test_web_gui_preview_renderer.js
node nodesrc/test_web_gui_host_bridge.js
node nodesrc/test_web_gui_stdout_protocol.js
node nodesrc/test_web_gui_mandelbrot_transport_contract.js
npm --prefix web run build:ts
git diff --check
```

Subagent review:

- Web renderer slice 完了後、subagent に direct Canvas primitive が消えているか、Zenn 方針に反しないか確認させる。

## Phase 3: video memory surface slice

目的:

- `SharedArrayBuffer` video memory surface を正式 Web path として追加する。
- 現行 event queue SAB とは別の framebuffer SAB を定義する。

変更:

- `web/src/gui-preview/video-memory-surface.ts` を追加する。
- header layout、2 slot 以上の pixel plane、epoch、dirty region、surface state、slot ownership state を実装する。
- writer は `Free -> Writing -> Published`、presenter は `Published -> Reading -> Free` を `Atomics.compareExchange` / `Atomics.store` / `Atomics.notify` で進める。
- presenter が `putImageData` を完了するまで slot を `Reading` として保持し、writer が同じ plane を上書きできないようにする。
- Web presenter は `ImageData` を `SharedArrayBuffer` と slot index ごとに cache し、same-size frame の hot path で `ImageData` を再生成しない。
- Dirty region は integer / non-negative / surface 内に収まることを検査し、範囲外なら typed error にする。Clamp しない。
- Zero-size dirty region は valid no-op present とし、`putImageData` は呼ばず release して presented epoch を進める。
- Canvas `putImageData` failure や invalid dirty region は slot を discard して writer を詰まらせない。ただし表示済みではないため presented epoch は進めない。
- SAB unavailable は typed error にする。
- invalid header、unsupported version、stale resize generation、presenter unavailable、writer closed、unsupported command も typed error にする。
- `GuiWebRuntimeBridge` に `presentVideoMemory` を追加し、`windowId`、`title`、`SharedArrayBuffer` だけを受ける typed runtime boundary とする。
- `ArrayBuffer`、typed array、numeric id、string handle、transfer object は `invalid-video-memory-frame` で拒否し、stdout protocol や command frame path へ fallback しない。
- `GuiPreviewPanel` は `none` / `command-frame` / `video-memory` の state を分け、video memory state では resize 時に command renderer や background fallback へ戻らない。
- Panel は同じ `SharedArrayBuffer` identity の opened video memory surface を再利用し、buffer identity が変わった時だけ open し直す。
- Surface size と drawable surface size が異なる場合は 1:1 top-left presentation とし、CSS scale や `drawImage` で引き伸ばさない。resize event が新 surface を促す。
- `nodesrc/test_web_gui_video_memory_surface.js` を追加する。

検証:

```powershell
node nodesrc/test_web_gui_video_memory_surface.js
npm --prefix web run build:ts
git diff --check
```

Subagent review:

- Synchronization、tearing、resize generation、unsupported handling を review させる。

## Phase 3.5: same app code host surface gate

目的:

- Web-only stdout helper を正式 application contract から外し、同じ NEPL app code が host surface ABI へ接続される経路を固定する。

変更:

- formal Web host import は video memory surface / pixel frame present を持つ。
- Web video memory host import は `nepl_gui_web` の Web-only scalar ABI として実装する。最初の import set は `video_memory_create_surface`、`video_memory_acquire_write_slot`、`video_memory_write_slot_bytes`、`video_memory_write_rgba8888_row`、`video_memory_discard_write_slot`、`video_memory_publish_slot`、`video_memory_present_surface`、`video_memory_close_surface`、`request_timer` である。`video_memory_fill_rect_rgba8888` は early smoke 用の pixel slot writer であり、Canvas `fillRect` ではない。
- `write_rgba8888_row` は formal row payload 用の checked writer である。app は byte offset を計算せず、origin、pixel width、source pointer を渡す。zero width row、範囲外 row、`width * 4` と一致しない source length は typed error で拒否し、dirty / epoch は publish まで更新しない。
- `examples/gui_video_memory_rows.nepl` は formal row host import の focused source contract を固定する。row bytes は `ByteBuilder` / `ByteBuf` owner で構築し、borrowed `MemPtr u8` だけを `gui_web_video_memory_write_rgba8888_row` へ渡す。CI の通常 doctest は `nepl_gui_web` を unsupported stub として持つ。happy path 実行は fake positive `nepl_gui_web` host import harness で opt-in 検査し、通常 path を `--contract` なしの NEPL/Wasm として実行する。
- `discard_write_slot` は publish しない write frame を `Writing -> Free` に戻す ownership recovery path である。成功時だけ Worker の frame record を削除し、dirty metadata を消し、published / presented epoch は進めない。
- `publish_slot` と `present_surface` は分離する。`publish_slot` は Worker 所有の slot を `Published` にするだけで、visible window へ表示しない。`present_surface` は Worker から main thread へ typed `gui_video_memory_present` message を送り、ack `SharedArrayBuffer` に書かれた actual presenter status を待ってから戻る。
- `request_timer` は Worker から main thread へ typed `gui_timer_request` message を送り、ack `SharedArrayBuffer` に書かれた actual scheduler status を待ってから戻る。未提示 window は invalid、`interval_ms == 0` は clear、`repeating == 1` は repeating timer、`repeating == 0` は one-shot timer とする。Web host は repeating timer を `setInterval`、one-shot timer を `setTimeout` へ接続し、one-shot timer は timer event enqueue の前に active timer entry を clear する。
- `surface_id` と `frame_id` は Worker-local opaque positive integer とし、NEPL/Wasm は `SharedArrayBuffer`、DOM handle、Canvas handle、ArrayBuffer transfer object、JS object handle、string handle を受け取らない。
- `title_ptr` と `title_len` は Wasm linear memory の UTF-8 byte slice として検査する。pointer / length / UTF-8 の不正は typed negative status から `GuiError::InvalidCommand` へ写す。
- `platforms/gui/web/surface.nepl` は raw negative status を module private helper で `Result` / `GuiError` へ写し、public wrapper へ sentinel を漏らさない。
- Web stdout protocol は legacy smoke transport として隔離し、正式 ABI の代替として参照しない。
- native / bare / headless は同じ app-facing effect / present command を受け、capability 不足時だけ `GuiError::Unsupported` を返す。

検証:

```powershell
node nodesrc/test_web_gui_video_memory_host_import.js
node nodesrc/test_web_gui_same_app_code_contract.js
node nodesrc/test_stdlib_gui_layering_policy.js
git diff --check
```

Subagent review:

- Web-specific import が app-facing stdlib contract へ漏れていないか確認させる。

## Phase 4: stdlib contract slice

目的:

- Web 実装の具体型を stdlib public API に漏らさず、pixel buffer / surface capability の contract を追加する。

変更:

- `stdlib/core/gui/capability.nepl` の surface kind / memory kind を拡張する。
- `stdlib/core/gui` に pixel buffer descriptor / surface descriptor の no_alloc value を追加する。
- `stdlib/std/gui` に host surface operation の typed command を追加する。
- doc comment は日本語で、目的、契約、注意、計算量を記述する。

検証:

```powershell
node nodesrc/tests.js -i tests/stdlib/gui_core.n.md --no-tree -o tmp/gui-core-bitmap-surface.json -j 1 --dist web/dist --assert-io
node nodesrc/test_stdlib_gui_layering_policy.js
node nodesrc/run_source_policy_regressions.js --warn-only
git diff --check
```

Subagent review:

- core / alloc / std に platform detail が漏れていないか確認させる。

## Phase 4.1: surface present effect/runtime bridge

目的:

- `PresentSurfaceEffect` を application-facing effect とし、runtime が checked `GuiSurfacePresentCommand` を生成する。
- app code が Web stdout helper を import せず、`GuiEffect` だけで host surface presentation を要求できるようにする。
- `std/gui/runtime` が capability gate を持ち、pixel surface を持たない backend では `GuiError::Unsupported` を返す。

変更:

- `alloc/gui/app/types.nepl` に `PresentSurfaceEffect` と `GuiEffect::PresentSurface` を追加する。
- `PresentSurfaceEffect` は `surface`、`frame`、`width`、`height`、`stride_bytes`、`format`、`dirty` を持つ request data とし、`std/gui/surface` の型を持たない。
- `present_surface` helper は app-facing request data を作るだけで、Web stdout helper、platform host、`GuiSurfacePresentCommand` を直接要求しない。
- `alloc/gui/app/types.nepl` は `std/gui` や `platforms/gui` を import しないことを source policy で固定する。
- `stdlib/std/gui/runtime.nepl` に `GuiRuntimeCommand::PresentSurface` を追加する。
- `gui_runtime_interpret_effect` は `surface_id_result`、`frame_id_result`、`gui_pixel_buffer_descriptor` で request data を検査し、checked `GuiSurfacePresentCommand` を作る。
- `gui_runtime_interpret_effect` は `SurfaceKind::WindowPixel`、`OffscreenPixel`、`DevicePixel` だけで present を許可する。
- `SurfaceKind::TextGrid` と `SurfaceKind::Headless` は `GuiError::Unsupported` を返す。pixel frame を text grid や no-surface backend に暗黙変換しない。
- `GuiRuntimeCommandBatch` の bounded capacity 2 はこの slice では維持する。capacity を超える場合は既存通り `GuiError::ResourceExhausted` とする。
- `tests/stdlib/gui_app.n.md` と `tests/stdlib/gui_std.n.md` に present effect / runtime command / unsupported gate の doctest を追加する。
- `nodesrc/test_web_gui_same_app_code_contract.js` または新規 source policy で、stdout helper ではなく `PresentSurfaceEffect -> GuiEffect::PresentSurface -> GuiRuntimeCommand::PresentSurface` へ繋がることを固定する。

検証:

```powershell
node nodesrc/test_web_gui_same_app_code_contract.js
node nodesrc/test_stdlib_gui_layering_policy.js
node nodesrc/tests.js -i tests/stdlib/gui_app.n.md --no-tree -o tmp/gui-app-present-surface.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_std.n.md --no-tree -o tmp/gui-std-present-surface.json -j 1 --dist web/dist --assert-io
node nodesrc/test_stdlib_documentation_contract.js
git diff --check
```

Subagent review:

- 実装開始前に、3 文書と Zenn 方針を読ませ、`PresentSurface` effect / runtime command の方針に `implementation may start` が出るまで実装しない。
- 実装後に、platform detail が `alloc/gui` / `std/gui` へ漏れていないこと、headless / text grid が hidden fallback になっていないこと、unsupported が typed error で返ることを確認させる。

## Phase 5: offscreen and headless slice

目的:

- Screenshot / offscreen rendering / virtual event replay を正式 backend として実装する。

変更:

- offscreen pixel buffer host を追加する。
- headless host は present / screenshot を unsupported にする。
- virtual event source と virtual clock の test helper を追加する。
- screenshot capture は deterministic pixel hash を返す。

検証:

```powershell
node nodesrc/test_web_gui_offscreen_headless_contract.js
node nodesrc/test_web_gui_shared_event_queue.js
npm --prefix web run build:ts
git diff --check
```

Subagent review:

- headless が fallback になっていないこと、event virtualization が platform event と同じ `GuiEvent` を使うことを確認させる。

## Phase 5.1: stdlib offscreen snapshot and virtual event contract

目的:

- std layer に offscreen snapshot data boundary を追加する。
- headless が screenshot / present fallback ではないことを typed error で固定する。
- test helper として virtual event script と virtual clock を追加し、Web / native / bare と同じ `GuiEvent` を application に渡せるようにする。

変更:

- `stdlib/std/gui/offscreen.nepl` を追加する。
- `GuiOffscreenSnapshot` を追加し、`SurfaceId`、`FrameId`、width、height、stride、format、dirty region、backend-supplied pixel hash を保持する。
- `gui_offscreen_snapshot_from_runtime_command` は `GuiHost`、`GuiRuntimeCommand`、pixel hash から snapshot を作る。
- `SurfaceKind::OffscreenPixel` だけが snapshot 生成を許可される。
- `WindowPixel`、`DevicePixel`、`TextGrid`、`Headless` は screenshot source として `GuiError::Unsupported` を返す。
- `GuiRuntimeCommand::PresentSurface` 以外の command から snapshot を作らない。
- `stdlib/std/gui/virtual_event.nepl` を追加する。
- `GuiOffscreenSnapshot.pixel_hash` は signed opaque `i32` とし、0 や -1 を sentinel にしない。
- `GuiVirtualClock` は deterministic clock value として `now_ms` と `tick` を持つ。
- `gui_virtual_clock_result` は negative initial time を `GuiError::InvalidCommand` として拒否する。
- `gui_virtual_clock_advance` は negative delta と i32 overflow を `GuiError::InvalidCommand` として拒否する。
- `GuiVirtualEventScript` は初期 implementation では capacity 2 の bounded script とし、slot は `Option GuiEvent` とする。empty script は dummy event ではなく `Option::None` を保持する。
- `gui_virtual_event_script_push` は empty slot に `Option::Some event` を入れ、overflow は `GuiError::ResourceExhausted` を返す。
- `gui_virtual_event_script_poll` は script と `Option GuiEvent` を返す。queue empty は `Option::None` であり、sentinel event を作らない。
- `std/gui.nepl` facade に offscreen / virtual event を公開する。
- `tests/stdlib/gui_std.n.md` に offscreen snapshot、headless rejection、virtual event polling、virtual clock negative delta の doctest を追加する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` を追加し、doc / source policy として hidden fallback、DOM / Canvas / OS handle 混入、raw event string 混入を禁止する。

検証:

```powershell
node nodesrc/test_web_gui_offscreen_headless_contract.js
node nodesrc/test_stdlib_gui_layering_policy.js
node nodesrc/tests.js -i tests/stdlib/gui_std.n.md --no-tree -o tmp/gui-std-offscreen-headless.json -j 1 --dist web/dist --assert-io
node nodesrc/test_stdlib_documentation_contract.js
git diff --check
```

Subagent review:

- 実装開始前に、Zenn 方針と GUI redesign 3 文書を読ませ、offscreen snapshot / virtual event の方針に `implementation may start` が出るまで実装しない。
- 実装後に、offscreen と headless の混同がないこと、screenshot が hidden fallback になっていないこと、virtual event が `GuiEvent` を使っていることを確認させる。

## Phase 5.2: stdlib virtual timer scheduler contract

目的:

- headless / offscreen test 用に、実 OS / browser timer へ接続しない deterministic timer scheduler を std layer に追加する。
- `TimerRequest` から `GuiEvent::Timer` を生成し、Web / native / bare と同じ app-facing event shape を使えるようにする。
- event queue overflow を避けるため、timer catch-up は queue ではなく timer state の remainder と zero-delta drain で表す。

変更:

- `stdlib/std/gui/virtual_timer.nepl` を追加する。
- `GuiVirtualTimerState` は `Option TimerRequest`、elapsed、tick を保持する。
- `GuiVirtualTimerAdvance` は next state と `Option GuiEvent` を保持する。
- `request == None` の state は elapsed と tick が 0 であることを schedule / advance で再検査する。
- `request == Some` の state は positive window id、positive timer id、positive interval、non-negative elapsed、non-negative tick を要求する。
- `gui_virtual_timer_schedule` は incoming state と request を検査し、`interval_ms == 0` を clear request として扱う。
- `gui_virtual_timer_advance` は negative delta、elapsed overflow、tick overflow、malformed state を `GuiError::InvalidCommand` として拒否する。
- repeating timer は 1 advance あたり最大 1 event を返し、extra elapsed を remainder として保持する。remainder が interval 以上なら `advance state 0` で 1 event ずつ drain する。
- one-shot timer は event を返すときに state を empty へ戻す。
- `std/gui.nepl` facade に virtual timer を公開する。
- `tests/stdlib/gui_std_virtual_timer.n.md` を追加し、one-shot、repeating catch-up、clear、malformed state、overflow、source policy label を検査する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` に virtual timer の source policy を追加する。

完了条件:

- `virtual_timer.nepl` は DOM、Canvas、minifb、OS timer、browser timer、stdout、event queue、video memory、platform API、fallback、silent no-op を含まない。
- state invariant は public constructor で壊されても schedule / advance で拒否される。
- repeating catch-up は modulo や discard ではなく remainder と `advance state 0` で表現される。
- focused doctest、offscreen/headless source policy、`git diff --check` が通る。
- subagent implementation review で std layer placement、state invariant、zero-delta drain、no fallback が承認される。

検証:

```powershell
node nodesrc/test_web_gui_offscreen_headless_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_std_virtual_timer.n.md --no-tree -o tmp/gui-std-virtual-timer.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i stdlib/std/gui/virtual_timer.nepl --no-tree -o tmp/gui-std-virtual-timer-module.json -j 1 --dist web/dist --assert-io
git diff --check
```

Subagent review:

- 実装前 review では `PLAN_CHANGES` として plain `GuiError`、state invariant 再検査、repeating remainder 保持、`delta_ms == 0` drain を指摘された。
- 実装後に、指摘がすべて満たされていること、headless が presentation fallback になっていないことを確認させる。

## Phase 5.3: stdlib virtual timer turn bridge contract

目的:

- F5dw の target-neutral timer pending request と F5dy の deterministic virtual timer scheduler を std layer で接続する。
- headless / offscreen test は actual Web / native / bare timer backend、queue、real scheduler loop を使わず、`GuiEvent::Timer` によって scheduled turn を再開できる。
- bridge は `gui_virtual_timer_schedule`、`gui_virtual_timer_advance`、F5dw `turn_timer_complete` の接続順序と owner recovery だけを担当する。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerPending` は F5dw pending と `GuiVirtualTimerState` を保持する。
- schedule は F5dw pending から borrowed `TimerRequest` を読み、`gui_virtual_timer_schedule` を 1 回だけ呼ぶ。
- advance は `gui_virtual_timer_advance` を 1 回だけ呼び、`Option::None` は next pending、`GuiEvent::Timer` は F5dw `turn_timer_complete`、timer 以外の event は owner-bearing error に写す。
- schedule failure は original pending と original virtual timer state と lower `GuiError` を保持する。
- advance failure は original combined pending と lower `GuiError` を保持する。
- unexpected event は F5dw pending、advance-after virtual timer state、event を保持する。
- timer complete failure は F5dw complete error と advance-after virtual timer state を保持する。
- owner-bearing pending / advance / error payload には Clone / Copy を実装しない。
- `std/gui.nepl` facade に virtual timer turn bridge を公開する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.n.md` を追加し、owner recovery、exact authority calls、no loop / backend / queue / fallback の source policy label を固定する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` と `nodesrc/test_web_gui_font_rendering_contract.js` に source policy を追加する。

完了条件:

- F5dz は DOM、Canvas、minifb、OS timer、browser timer、stdout、event queue、video memory、platform API、fallback、silent no-op、loop drain を含まない。
- schedule / advance / complete の各 lower authority を重複して呼ばない。
- failure path は pending と virtual timer state を失わず、caller が recovery accessor で回収できる。
- focused doctest、offscreen/headless source policy、font rendering source policy、`git diff --check` が通る。
- subagent implementation review で owner recovery、exact authority calls、no backend / queue / fallback が承認される。

検証:

```powershell
node nodesrc/test_web_gui_offscreen_headless_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/tests.js -i tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.n.md --no-tree -o tmp/gui-std-turn-virtual-timer.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.nepl --no-tree -o tmp/gui-std-turn-virtual-timer-module.json -j 1 --dist web/dist --assert-io
git diff --check
```

Subagent review:

- 実装前 review では `PLAN_BLOCKED` として timer complete failure / unexpected event / advance failure の owner recovery 不足を指摘された。
- revised plan では advance-after virtual timer state と lower error を保持する設計に直し、Cicero revised plan review は `PLAN_APPROVED`。
- 実装後に、指摘がすべて満たされていること、bridge が real scheduler や presentation fallback に進んでいないことを確認させる。

## Phase 5.4: stdlib virtual scheduler state boundary

目的:

- F5dv scheduler decision、F5dw timer request、F5dz virtual timer bridge を deterministic scheduler state として接続する。
- actual scheduler loop、timeslice policy、event queue、platform timer backend を実装する前に、headless / offscreen test 用の phase-owned state を固定する。
- `GuiVirtualTimerState` を static policy に入れず、各 phase payload の dynamic state として保持する。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState` は `Turn`、`WaitingTimer`、`Execute`、`Completed` を持つ。
- `Turn`、`Execute`、`Completed` の payload は `GuiVirtualTimerState` を保持し、`WaitingTimer` は F5dz pending が timer state を保持する。
- decision boundary は F5dw `turn_timer_interpret_decision` を 1 回だけ呼び、`ContinueNow` を reusable decision ではなく `Turn` phase へ写す。
- `ScheduleTimer` だけが F5dz schedule を呼ぶ。
- timer advance boundary は F5dz `virtual_timer_advance` を 1 回だけ呼び、`Ready` decision では one-shot complete 済みとして `gui_virtual_timer_empty` を渡して decision boundary へ戻す。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler.n.md` を追加し、phase-owned state、ContinueNow -> Turn、schedule owner recovery、ready empty timer、exact authority calls、no loop / backend / queue / fallback の source policy label を固定する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` と `nodesrc/test_web_gui_font_rendering_contract.js` に source policy を追加する。

完了条件:

- F5ea は DOM、Canvas、minifb、OS timer、browser timer、stdout、event queue、video memory、platform API、fallback、silent no-op、loop drain、timeslice budget を含まない。
- `GuiVirtualTimerState` が policy ではなく phase payload にある。
- `ContinueNow` が `Turn` phase へ写り、`Ready` decision 後に `gui_virtual_timer_empty` が明示的に使われる。
- focused doctest、offscreen/headless source policy、font rendering source policy、`git diff --check` が通る。
- subagent implementation review で owner recovery、exact authority calls、no backend / queue / fallback が承認される。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler.n.md
node nodesrc/test_web_gui_offscreen_headless_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_stdlib_gui_layering_policy.js
```

Subagent review:

- 初回 review では `PLAN_BLOCKED`。`GuiVirtualTimerState` を policy に入れる設計、`ContinueNow` を reusable decision に戻す no-progress state、`Ready` 後の timer state 消失が指摘された。
- revised plan では dynamic state を phase payload に移し、`ContinueNow` を `Turn` phase、`Ready` 後を明示 `gui_virtual_timer_empty` として固定し、Cicero revised plan review は `PLAN_APPROVED`。
- 実装後に、指摘がすべて満たされていること、real scheduler loop や presentation fallback に進んでいないことを確認させる。

## Phase 5.5: stdlib virtual scheduler single step boundary

目的:

- F5ea state を使う real scheduler loop の前段として、1 回だけ state を前進させる std layer row tile RLE present host span operation presenter executor session turn virtual scheduler single step boundary を固定する。
- Turn path の authority order を F5du driver poll、F5dv scheduler decide、F5ea timer decide に限定する。
- blocked phase を no-progress success にせず、外側 loop authority が処理できる typed result として返す。

変更:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerStepResult` は `Advanced`、`BlockedWaitingTimer`、`BlockedExecute`、`Completed` を持つ。
- step policy は scheduler policy と timer policy だけを持ち、dynamic `GuiVirtualTimerState` を保持しない。
- poll failure と scheduler decision failure は current `GuiVirtualTimerState` と lower error を保持する。
- timer decision failure は F5ea lower owner-bearing error を保持する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step.n.md` を追加する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` と `nodesrc/test_web_gui_font_rendering_contract.js` に Phase 5.5 / F5eb source policy を追加する。

完了条件:

- Turn path は driver poll、scheduler decide、F5ea timer decide をそれぞれ 1 回だけ呼ぶ。
- `WaitingTimer` は `BlockedWaitingTimer`、`Execute` は `BlockedExecute`、`Completed` は `Completed` として返る。
- F5eb は DOM、Canvas、minifb、OS timer、browser timer、stdout、event queue、video memory、platform API、fallback、silent no-op、loop drain、timeslice budget を含まない。
- focused doctest、offscreen/headless source policy、font rendering source policy、`git diff --check` が通る。
- subagent implementation review で owner recovery、exact authority order、blocked branch no backend / queue / fallback が承認される。

検証:

```powershell
rg -n "[()]" stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step.nepl tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step.n.md
node nodesrc/test_web_gui_offscreen_headless_contract.js
node nodesrc/test_web_gui_font_rendering_contract.js
node nodesrc/test_stdlib_gui_layering_policy.js
```

Subagent review:

- 初回 review では `PLAN_CHANGES`。blocked phase を `Ok same state` にしないこと、`BlockedWaitingTimer` / `BlockedExecute` / `Completed` を result に持たせること、poll / scheduler failure に current timer state を保持すること、step policy に dynamic timer state を入れないことが指摘された。
- revised plan では single-step boundary と blocked result を明示し、Cicero revised plan review は `PLAN_APPROVED`。
- 実装後に、Turn path の正確な順序、blocked branch で backend / queue を呼ばないこと、owner-bearing error が保たれることを確認させる。

## Phase 6: migration and cleanup

目的:

- stdout GUI presentation と Canvas2D direct drawing を正式経路から除去する。

変更:

- examples の GUI output を video memory path へ移行する。
- stdout GUI protocol を削除、または正式 path から参照されない legacy quarantine に隔離する。
- docs から fallback 表現を削除し、unsupported / missing capability に置き換える。
- native smoke backend を framebuffer presenter contract に寄せる。

検証:

```powershell
node nodesrc/test_web_gui_preview_renderer.js
node nodesrc/test_web_gui_shared_event_queue.js
node nodesrc/test_native_gui_platform_behavior.js
node nodesrc/test_stdlib_gui_layering_policy.js
node nodesrc/run_source_policy_regressions.js --warn-only
node nodesrc/issues.js check --dir issues
git diff --check
```

Subagent review:

- cleanup 後に no fallback、platform boundary、same app code contract が崩れていないか確認させる。

## Phase 7: font and 2D renderer contract slice

目的:

- Bitmap surface / offscreen / headless の formal contract の上に、本格 font rendering と 2D renderer の typed boundary を追加する。
- `MockTextMeasurer` と fixed bitmap font は test utility として残し、formal font renderer の代替にしない。

関連文書:

- `doc/neplg2/gui_font_rendering_spec.md`
- `doc/neplg2/gui_font_rendering_detailed_design.md`
- `doc/neplg2/gui_font_rendering_implementation_plan.md`
- `doc/neplg2/gui_font_rendering_design.md`
- `doc/neplg2/gui_2d_rendering_design.md`

実装順:

1. `core/gui/font` と `core/gui/render_style` に no_alloc contract を追加する。multi-shadow は `GuiShadowRef` で表し、core は `Vec` を持たない。
2. `std/gui/font_resource` に typed resource request を追加する。resource hash と path は専用 value で表し、display name や path suffix を authority にしない。
3. Web VFS / native resource root / bare embedded blob の resource provider contract を接続する。
4. alloc layer に sfnt metadata parser、metrics、glyph outline、shaping、ruby、vertical、math bridge を段階実装する。

Gate:

- Phase F1/F2 は subagent が font/2D 文書と Zenn 方針を確認し、implementation may start を返してから実装する。
- 実装中も core/alloc/std/platform の dependency direction、fallback 禁止、typed error、doctest と source policy を review させる。

## Checkpoint commit policy

- Phase ごとに focused verification を通して commit する。
- commit 前に `git diff --check` を通す。
- `plan.md` は変更しない。
- `note.n.md` には現在の実装状況、plan.md との差異、verification を記録する。

## Resumed implementation target

Phase 2 と Phase 3 の最小縦 slice は完了済みである。

- Web visible canvas direct drawing を廃止する。
- pixel buffer renderer を通す。
- video memory surface module と tests を追加する。

再開 target は Phase 3.5 の same app code host surface gate である。

- Web-only stdout helper を正式 application contract から外す。
- `std/gui` に platform 非依存の host surface / pixel frame present value を置く。
- `platforms/gui/web` は Web backend の formal surface descriptor を持つが、application model へ Web-specific import を要求しない。
- stdout protocol は legacy smoke/debug transport として隔離し、正式 ABI の代替として扱わない。
- `nodesrc/test_web_gui_same_app_code_contract.js` と `nodesrc/test_stdlib_gui_layering_policy.js` で regression を固定する。

理由:

- Phase 2 / 3 により Web visible canvas direct drawing と single-buffer presentation risk は解消済みである。
- 次の根本課題は、同じ NEPL app code が Web / native / bare / headless の host surface ABI へ接続できる境界を stdlib 側に固定することである。
- stdout transport を正式経路として残すと、Web 専用 transport が application contract に混入し、platform boundary と no fallback 方針が崩れる。

## Current implementation target

Phase 5.5 の deterministic virtual scheduler single step boundary までを現在の checkpoint とする。次の再開 target は、F5eb step result を消費する real scheduler loop / timeslice contract / headless app-loop integration である。

- scheduler loop は F5eb の `Advanced` / `BlockedWaitingTimer` / `BlockedExecute` / `Completed` result を明示的に進める必要がある。
- `WaitingTimer` は event queue drain ではなく timer backend または virtual timer advance によってだけ再開する必要がある。
- timeslice policy は `Yield` と timer schedule の契約を乱さず、FHD 60fps 目標に向けて bounded turn progress を表す必要がある。
- headless app-loop は presentation fallback ではなく、virtual event / virtual timer / offscreen snapshot を組み合わせた test target として扱う必要がある。
- 実装開始前に subagent review を通し、Required がある場合は doc を修正して再 review する。
