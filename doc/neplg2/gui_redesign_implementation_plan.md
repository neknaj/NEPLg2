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

## Phase F5el: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler real loop driver boundary

2026-06-18 の F5el では、F5ek result を F5ef / F5eg へ接続する real loop driver boundary を追加する。`RealLoopDriverPolicy` は F5ef loop policy だけを保持し、F5ek step policy、scheduler policy、timer policy、backend executor、clock、queue を重複保持しない。`start` は F5ef `loop_step` と F5eg `loop_action_from_result` を 1 回ずつ呼び、`after_step` は F5ek result を `StateReady` / `YieldPending` / `Completed` として match する。`StateReady` は `loop_resume` へ戻し、`remaining_count == 0` は budget-yield semantics に従って yield action へ進め、error / completion / `CompleteAck` / fallback / silent no-op へ変換しない。

変更:

- F5ec drain に `drain_resume` を追加し、負の `remaining_count` だけを typed error にする。
- F5ee slice に `slice_resume` を追加し、継続 budget を F5ec へ渡す。
- F5ef loop に `loop_resume` を追加し、F5el が F5ek `StateReady` の budget を捨てずに戻せるようにする。
- F5el real loop driver module を追加し、`NeedInput` / `Completed` / typed lower error を返す。
- source policy と focused doctest で policy shape、start/after_step dispatch、zero-budget yield semantics、no backend / no fallback を固定する。

## Phase F5em: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler headless app-loop step boundary

2026-06-18 の F5em では、F5el `NeedInput` と caller supplied F5ek input を deterministic headless / offscreen test 用の 1 app-loop step として接続する。F5em は actual backend clock source、native / bare scheduler backend、queue、platform API、DOM / Canvas / minifb、video memory を実装しない。

変更:

- F5em headless app-loop step module を追加し、`NeedInput` / `Completed` / typed lower error を返す。
- `HeadlessAppLoopStepPolicy` は F5el `RealLoopDriverPolicy` と F5ek `RealLoopStepPolicy` だけを保持する。
- `start` は F5el start を 1 回だけ呼ぶ。
- `advance` は previous `NeedInput` と explicit input だけを受け、F5ek step を 1 回呼び、成功時だけ F5el after-step を 1 回呼ぶ。
- F5ek error では F5el after-step を呼ばない。
- `Completed` は terminal output だけであり、advance input にはしない。
- F5em は `CompleteAck`、executor outcome、clock delta、fallback success、silent no-op を合成しない。

## Phase F5en: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler bounded headless app-loop runner boundary

2026-06-18 の F5en では、F5em `NeedInput` / `Completed` result を fixed-slot script と explicit `max_advance_count` で bounded に進める。これは deterministic test / headless replay 用の boundary であり、not long-running real backend loop である。actual backend clock source、native / bare scheduler backend、executor backend、queue、platform API、DOM / Canvas / minifb、video memory は実装しない。

変更:

- F5en headless app-loop runner module を追加し、`BudgetExhausted` / `InputMissing` / `Completed` / typed lower error を返す。
- `HeadlessAppLoopRunnerPolicy` は F5em `HeadlessAppLoopStepPolicy` と `max_advance_count` だけを保持する。
- `HeadlessAppLoopRunnerScript` は 3 slot の `Option RealLoopStepInput`、`count`、`cursor` だけを保持する。
- script の `count` / `cursor` / slot hole は `ScriptInvalid` として typed error で返す。
- `run` は policy と script を検査してから F5em `start` を 1 回だけ呼ぶ。
- `Completed` は script を消費しない。
- `InputMissing` は `ClockDelta` / `ExecutorOutcome` / `CompleteAck` を合成しない。
- `BudgetExhausted` は F5em `advance` を呼ばない。
- source policy が no Vec / queue / push / backend / fallback / silent no-op を固定する。

## Phase F5eo: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler backend clock delta boundary

2026-06-18 の F5eo では、Web / native / bare / headless backend が取得した monotonic clock sample を F5ek `RealLoopStepInput::ClockDelta` へ変換する。これは caller supplied sample を検査する pure std boundary であり、actual clock source、sleep、timer backend、executor backend、queue、platform API、DOM / Canvas / minifb、video memory は実装しない。

変更:

- F5eo backend clock module を追加し、`BackendClockPolicy`、`BackendClockSample`、`BackendClockState`、`BackendClockAdvance` を定義する。
- `BackendClockPolicy` は `max_delta_ms` だけを保持し、negative max を typed error として拒否する。
- `BackendClockSample` は caller supplied `monotonic_ms` だけを保持し、negative sample を typed error として拒否する。
- `BackendClockState` は previous `last_monotonic_ms` だけを保持する。public value なので `advance` は forged negative state を typed error として拒否する。
- `start` は sample を entry で再検査して baseline state を返す。`ClockDelta` は発行しない。
- `advance` は policy / state / sample を entry で再検査し、backward time と too-large delta を typed error として返す。
- zero delta は no-op や error にせず `ClockDelta 0` として返す。
- delta が `max_delta_ms` を超える場合は clamp せず `DeltaTooLarge` を返す。
- error payload は policy / state / sample / previous / current / delta / max を回収可能な形で保持する。
- source policy が no platform / queue / fallback / silent no-op と `ExecutorOutcome` / `CompleteAck` 非生成を固定する。

## Phase F5ep: Web formal monotonic clock source backend boundary

2026-06-18 の F5ep では、Web runtime の actual monotonic clock source を `platforms/gui/web/clock` へ接続する。これは Web platform boundary だけの変更であり、native / bare / headless clock source、sleep、scheduler loop、executor backend、queue、DOM / Canvas rendering、video memory presentation は実装しない。

変更:

- `stdlib/platforms/gui/web/clock.nepl` を追加し、`nepl_gui_web.monotonic_clock_ms` を `Result BackendClockSample GuiError` に写す。
- `stdlib/platforms/gui/web.nepl` から clock boundary を export する。
- `web/src/runtime/worker.ts` に `monotonic_clock_ms` import を追加し、`performance.now` だけを source とする。
- Worker は `Number.isFinite`、0 以上、`i32::MAX` 以下、integer を検査してから Wasm に返し、範囲外は `BackendFailure` sentinel にする。
- `Date.now`、`setTimeout`、`setInterval`、stdout protocol、polling loop、queue、fallback、silent no-op は使わない。
- source policy と focused doctest で Web facade export、raw import、F5eo sample constructor bridge、i32 guard、forbidden fallback を固定する。

## Phase F5eq: Headless scripted monotonic clock source backend boundary

2026-06-18 の F5eq では、Headless scripted monotonic clock source backend boundary を追加する。これは headless / offscreen test 用の deterministic actual clock input source であり、wall clock、native / bare clock source、sleep、scheduler loop、executor backend、queue、DOM / Canvas rendering、video memory presentation は実装しない。

変更:

- `stdlib/platforms/gui/headless.nepl` facade と `stdlib/platforms/gui/headless/clock.nepl` を追加する。
- `GuiHeadlessBackendClockScript` は fixed-slot の `Option BackendClockSample` 3 件、`count`、`cursor` だけを持つ。
- constructor は raw i32 sample を F5eo `BackendClockSample` constructor で検査してから slot に保持する。
- poll は public script の count / cursor / slot shape / sample を再検査し、sample があれば cursor を 1 進める。
- `cursor == count` は `Option::None` を返し、zero sample、delta、fallback、silent no-op を合成しない。
- focused doctest と source policy で fixed-slot shape、constructor validation、poll validation、end None、forbidden timer / queue / fallback を固定する。

## Phase F5er: Native formal monotonic clock source backend boundary

2026-06-18 の F5er では、native runtime の actual monotonic clock source を `platforms/gui/native/clock` へ接続する。これは Native formal monotonic clock source backend boundary だけの変更であり、bare clock source、scheduler backend、executor backend、queue、DOM / Canvas rendering、minifb rendering、video memory presentation は実装しない。

変更:

- `stdlib/platforms/gui/native.nepl` facade と `stdlib/platforms/gui/native/clock.nepl` を追加する。
- `nepl_gui_native.monotonic_clock_ms` は単一 `i32` return ABI とし、0 以上を sample、-1 を unsupported、その他の負値を backend failure とする。
- NEPL wrapper は negative sentinel を `GuiError` へ写し、成功値だけを F5eo `BackendClockSample` constructor へ渡す。
- `nepl-gui-native` は `Instant` 由来 elapsed millisecond を `i32::MAX` 以下で検査し、範囲外は `BackendFailure` sentinel にする。
- wrap、clamp、saturating cast、wall clock、timer、sleep、queue、stdout protocol、fallback、silent no-op は使わない。
- focused doctest、source policy、native platform behavior regression で native facade export、raw import、F5eo sample constructor bridge、i32 guard、forbidden fallback を固定する。

## Phase F5es: Bare formal monotonic clock source backend boundary

2026-06-18 の F5es では、bare embedding host が明示提供する actual monotonic clock source を `platforms/gui/bare/clock` へ接続する。これは Bare formal monotonic clock source backend boundary だけの変更であり、stdlib が universal wall clock を生成する実装ではない。native / bare scheduler backend、timer backend、executor backend、queue、DOM / Canvas rendering、minifb rendering、video memory presentation は実装しない。

変更:

- `stdlib/platforms/gui/bare.nepl` facade と `stdlib/platforms/gui/bare/clock.nepl` を追加する。
- `nepl_gui_bare.monotonic_clock_ms` は単一 `i32` return ABI とし、0 以上を sample、-1 を `Unsupported`、その他の負値を `BackendFailure` とする。
- NEPL wrapper は negative sentinel を `GuiError` へ写し、成功値だけを F5eo `BackendClockSample` constructor へ渡す。
- `nodesrc/run_test.js` の `nepl_gui_bare` 既定 import は doctest-only unsupported source とし、`monotonic_clock_ms` は -1 を返す。
- doctest-only unsupported source は hidden fallback や hidden mock ではなく、host が clock を提供しない場合の明示 contract を検査するためだけに使う。
- Web `performance.now`、native `Instant`、wall clock、timer、sleep、queue、stdout protocol、fallback、silent no-op は使わない。
- focused doctest、source policy、bare platform behavior notes で bare facade export、raw import、F5eo sample constructor bridge、unsupported default、forbidden fallback を固定する。

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

## Phase 5.6: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler bounded drain boundary

目的:

- F5eb single-step boundary を `max_advance_count` で bounded に消費し、headless / offscreen test や後続 real scheduler loop が no-progress terminal を型で扱えるようにする。
- zero budget を F5eb step 呼び出しなしの `BudgetExhausted` として固定する。

実装:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainPolicy` は step policy と `max_advance_count` だけを持つ。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainResult` は `BudgetExhausted`、`BlockedWaitingTimer`、`BlockedExecute`、`Completed` を持つ。
- `Advanced` だけが budget を消費し、blocked / completed terminal は remaining count を保持して返る。
- `StepFailed` は lower F5eb error だけを保持する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` と `nodesrc/test_web_gui_font_rendering_contract.js` に Phase 5.6 / F5ec source policy を追加する。

完了条件:

- zero budget は F5eb step を呼ばず `BudgetExhausted` を返す。
- negative `max_advance_count` は construction と drain entry の両方で拒否される。
- F5ec は timer advance、executor completion、backend timer、queue drain、DOM / Canvas / minifb、video memory、fallback、silent no-op を含まない。

subagent review:

- Cicero に F5ec 実装計画を渡し、budget terminal、blocked remaining count、lower-only error、no backend / no queue / no fallback の観点で確認させる。
- 実装後に、source policy と focused doctest が Phase 5.6 の contract を検査していることを確認させる。

## Phase 5.7: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler transition boundary

目的:

- F5ec bounded drain result を、real scheduler loop / headless app-loop / host driver が次に処理すべき action enum へ写す。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransition` は `YieldSlice`、`AwaitTimer`、`ExecuteHostAction`、`Done` を持つ。
- drain payload struct を public transition payload として再公開せず、accessor で取り出した authority value と `remaining_count` を transition-owned payload へ詰め替える。

実装:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition.nepl` を追加する。
- `BudgetExhausted` は `YieldSlice`、`BlockedWaitingTimer` は `AwaitTimer`、`BlockedExecute` は `ExecuteHostAction`、`Completed` は `Done` へ変換する。
- `remaining_count` は正規化、減算、再計算を行わず、F5ec terminal の値をそのまま保持する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition.n.md` を追加し、facade、variant、drain terminal mapping、payload rewrap、remaining_count preservation、no wildcard、no timer advance / executor completion、no backend / queue / fallback label を固定する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` と `nodesrc/test_web_gui_font_rendering_contract.js` に Phase 5.7 / F5ed source policy を追加する。

非目標:

- timer advance、executor completion、actual scheduler loop、timeslice backend、queue drain、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op は含めない。
- F5ec drain を再実行しない。
- F5eb step を直接呼ばない。

完了条件:

- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransition` の 4 variant が F5ec の 4 terminal と 1 対 1 で対応する。
- transition payload は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainBudgetExhausted` などの F5ec payload struct を保持しない。
- `remaining_count` は各 transition payload で保持され、次の scheduler authority が budget 消費状況を判断できる。

subagent review:

- Curie に F5ed 実装計画を渡し、F5ec payload struct の再公開禁止、owner-bearing payload の non-Copy / non-Clone、4 terminal の explicit match、no timer advance / executor completion / backend / queue / fallback の観点で確認させる。
- 実装後に、source policy と focused doctest が Phase 5.7 の contract を検査していることを確認させる。

## Phase 5.8: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler slice boundary

目的:

- F5ec bounded drain と F5ed transition を、real scheduler loop / headless app-loop が 1 work slice として消費できる public boundary に接続する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSliceResult` は `YieldSlice`、`AwaitTimer`、`ExecuteHostAction`、`Done` を持つ。
- `YieldSlice` は F5ed transition payload をそのまま公開せず、state、`remaining_count`、`yield_delay_ms` を slice-owned payload として保持する。

実装:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice.nepl` を追加する。
- policy は F5ec drain policy と `yield_delay_ms` だけを保持する。
- `yield_delay_ms` は policy construction と slice entry の両方で 0 以上に検査する。
- public slice entry は F5ec drain を 1 回だけ呼び、成功時だけ F5ed transition mapping を 1 回だけ呼ぶ。
- F5ec / F5ed payload struct を slice payload として再公開せず、state / pending / execute / completed と `remaining_count` を slice-owned payload へ詰め替える。
- drain failure は lower F5ec error だけを `DrainFailed` に保持する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice.n.md` を追加し、facade、policy validation、result variant、one drain / one transition、yield payload、payload rewrap、lower-only drain failure、no wildcard、no timer advance / executor completion、no backend / queue / fallback label を固定する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` と `nodesrc/test_web_gui_font_rendering_contract.js` に Phase 5.8 / F5ee source policy を追加する。

非目標:

- timer advance、executor completion、actual scheduler loop、native / bare / headless real backend、queue drain、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op は含めない。
- F5eb step を直接呼ばない。
- F5ec drain を複数回呼ばない。
- F5ed transition mapping を複数回呼ばない。

完了条件:

- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSliceResult` の 4 variant が F5ed の 4 transition と 1 対 1 で対応する。
- policy construction と entry revalidation の両方で negative `yield_delay_ms` が拒否される。
- `remaining_count` は各 slice payload で保持され、`YieldSlice` は `yield_delay_ms` も保持する。
- drain failure は F5ec lower error だけを持ち、duplicate scheduler state を保持しない。

subagent review:

- Hegel に F5ee 実装計画を渡し、policy revalidation、one drain / one transition、F5ec / F5ed payload struct 再公開禁止、owner-bearing payload の non-Copy / non-Clone、lower-only drain failure、no backend / no queue / no fallback の観点で確認させる。
- 実装後に、source policy と focused doctest が Phase 5.8 の contract を検査していることを確認させる。

## Phase 5.9: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop boundary

目的:

- F5ee slice result を、real scheduler loop / headless app-loop が match する loop-owned public result へ詰め替える。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopResult` は `Yield`、`AwaitTimer`、`ExecuteHostAction`、`Done` を持つ。
- F5ef は actual while loop ではなく、外側 loop authority が次に実行する request を 1 slice ぶんだけ返す境界である。

実装:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop.nepl` を追加し、`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopResult` を public loop result として公開する。
- policy は F5ee slice policy だけを保持する。
- public step は F5ee `virtual_scheduler_slice` を 1 回だけ呼ぶ。
- F5ee payload struct を loop payload として再公開せず、state / pending / execute / completed と `remaining_count` を loop-owned payload へ詰め替える。
- `Yield` payload は state、`remaining_count`、`yield_delay_ms` を保持する。
- failure は lower-only slice error として F5ee slice error だけを保持する。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop.n.md` を追加し、facade、policy owns F5ee only、result variants、one slice call、payload rewrap、lower-only slice error、no wildcard、no timer / executor / backend / queue / fallback label を固定する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` と `nodesrc/test_web_gui_font_rendering_contract.js` に Phase 5.9 / F5ef source policy を追加する。

非目標:

- timer advance、executor completion、actual scheduler while loop、native / bare / headless real backend、queue drain、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op は含めない。
- F5ec drain、F5ed transition、F5eb step、F5ea helper を直接呼ばない。
- F5ee `virtual_scheduler_slice` を複数回呼ばない。

完了条件:

- F5ee の `YieldSlice` / `AwaitTimer` / `ExecuteHostAction` / `Done` が F5ef の `Yield` / `AwaitTimer` / `ExecuteHostAction` / `Done` へ explicit match で写る。
- F5ef payload は F5ee payload struct を保持せず、loop-owned payload だけを公開する。
- F5ef error は lower F5ee slice error だけを保持する。
- F5ef source policy が F5ec / F5ed / F5eb / F5ea direct call、backend、queue、fallback を禁止する。

subagent review:

- Aquinas に F5ef 実装計画を渡し、implementation may start を確認した。実装後に、one slice call、F5ee payload 再公開禁止、direct lower call 禁止、owner-bearing payload の non-Copy / non-Clone、no backend / no queue / no fallback の観点で再確認させる。

## Phase 5.10: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop action boundary

目的:

- F5ef loop result を、real scheduler loop / headless app-loop の outer authority が処理する action value へ詰め替える。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopAction` は `YieldToClock`、`AwaitTimerAdvance`、`ExecuteHostAction`、`Complete` を持つ。
- F5eg は actual loop ではなく、caller supplied F5ef loop result から次 action を total mapping で返す boundary である。

実装:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action.nepl` を追加し、`loop_action_from_result` を public entry として公開する。
- F5ef `Yield` / `AwaitTimer` / `ExecuteHostAction` / `Done` を F5eg `YieldToClock` / `AwaitTimerAdvance` / `ExecuteHostAction` / `Complete` へ explicit match で写す。
- F5ef payload struct を action payload として再公開せず、state / pending / execute / completed と `remaining_count`、`yield_delay_ms` を action-owned payload へ詰め替える。
- Mapping は total なので F5eg 自体は error `Result` を作らない。実 timer / executor authority は後続 slice で typed `Result` を返す。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action.n.md` を追加し、facade、action variants、F5ef-only import、explicit match、payload rewrap、total mapping、no wildcard、no timer / executor / backend / queue / fallback label を固定する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` と `nodesrc/test_web_gui_font_rendering_contract.js` に Phase 5.10 / F5eg source policy を追加する。

非目標:

- timer advance、executor completion、actual scheduler while loop、native / bare / headless real backend、queue drain、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op は含めない。
- F5ee / F5ec / F5ed / F5eb / F5ea、virtual timer、host、platform module を import しない。
- F5ef `virtual_scheduler_loop_step` を呼ばない。

完了条件:

- F5eg source policy が F5ef-only input、total mapping、F5ef payload 再公開禁止、direct lower call、loop step call、backend、queue、fallback を検査する。
- focused doctest が source policy label を持つ。
- 次の再開 target は F5eg action を消費する timer advance / executor completion authority であり、これ以上の pure rename layer を増やさない。

subagent review:

- Aquinas に F5eg 実装計画を渡し、implementation may start を確認した。実装後に、F5ef-only input、total mapping、F5ef payload 再公開禁止、direct lower call 禁止、owner-bearing payload の non-Copy / non-Clone、no backend / no queue / no fallback の観点で再確認させる。

## Phase 5.11: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop timer advance boundary

目的:

- F5eg `AwaitTimerAdvance` action payload を consumed authority として扱い、F5ea `virtual_scheduler_advance_timer` を 1 回だけ呼ぶ。
- Timer advance の結果を real scheduler loop / headless app-loop が次の loop step へ戻せる typed result にする。
- pure rename layer を増やさず、F5eg action から timer authority へ進める。

実装:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance.nepl` を追加し、`loop_timer_advance` を public entry として公開する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopTimerAdvanceCompleted` は next scheduler state と original `remaining_count` を保持する。
- `AdvanceFailed` は lower F5ea `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerAdvanceError` と original `remaining_count` を保持する。
- `loop_timer_advance` は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionAwaitTimerAdvance`、`TurnTimerPolicy`、`delta_ms` だけを受ける。
- `remaining_count` は pending owner を消費する前に読み、F5ea `virtual_scheduler_advance_timer` を 1 回だけ呼ぶ。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance.n.md` を追加し、facade、result shape、F5eg / F5ea import、AwaitTimerAdvance consumed authority、one F5ea call、remaining_count preservation、lower error、no wildcard / backend / queue / fallback label を固定する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` と `nodesrc/test_web_gui_font_rendering_contract.js` に Phase 5.11 / F5eh source policy を追加する。

非目標:

- general `LoopAction` を受けない。
- F5eg `loop_action_from_result`、F5ef `loop_step`、F5ee / F5ec / F5ed / F5eb direct call、direct `virtual_timer_advance` は呼ばない。
- executor completion、yield-to-clock handling、actual scheduler loop、native / bare / headless real backend、queue drain、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op は含めない。

完了条件:

- F5eh source policy が AwaitTimerAdvance-only input、F5ea advance exactly once、remaining_count before owner consumption、lower error wrapping、backend / queue / fallback 禁止を検査する。
- focused doctest が source policy label を持つ。
- 次の再開 target は executor completion authority または YieldToClock / Complete を含む real scheduler loop integration である。

subagent review:

- Aquinas に F5eh 実装計画を渡し、implementation may start を確認した。実装後に、F5eg AwaitTimerAdvance only input、F5ea one advance call、remaining_count preservation、lower F5ea error wrapping、non-Copy / non-Clone、no backend / no queue / no fallback の観点で再確認させる。

## Phase 5.12: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop executor complete boundary

目的:

- F5eg `ExecuteHostAction` action payload を consumed authority として扱い、caller supplied `Result unit GuiError` を F5du `turn_driver_complete` へ 1 回だけ戻す。
- Driver completion 後の step を F5dv `scheduler_decide`、F5ea `virtual_scheduler_decide` へ順に渡し、real scheduler loop / headless app-loop が次の loop step へ戻せる typed result にする。
- executor completion authority を actual backend executor から分離し、backend は outcome だけを返す構造にする。

実装:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete.nepl` を追加し、`loop_executor_complete` を public entry として公開する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorCompletePolicy` は scheduler policy と timer policy だけを保持する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorCompleteCompleted` は next scheduler state と original `remaining_count` を保持する。
- `DriverCompleteFailed`、`SchedulerDecisionFailed`、`TimerDecisionFailed` は lower error と original `remaining_count` を保持し、F5du / F5dv 由来の失敗では `category` と `timer_state` も保持する。
- `loop_executor_complete` は policy、`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionExecuteHostAction`、caller supplied outcome だけを受ける。
- `remaining_count` と `timer_state` は pending owner を消費する前に読み、F5du `turn_driver_complete`、F5dv `scheduler_decide`、F5ea `virtual_scheduler_decide` をそれぞれ 1 回だけ呼ぶ。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete.n.md` を追加し、facade、policy shape、result shape、F5eg / F5du / F5dv / F5ea import、ExecuteHostAction consumed authority、caller supplied outcome、driver / scheduler / timer order、remaining_count preservation、lower error、no wildcard / backend / queue / fallback label を固定する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` と `nodesrc/test_web_gui_font_rendering_contract.js` に Phase 5.12 / F5ei source policy を追加する。

非目標:

- general `LoopAction` を受けない。
- F5eg `loop_action_from_result`、F5ef `loop_step`、F5ee / F5ec / F5ed / F5eb direct call、F5dt direct call、direct `virtual_timer_advance` / `virtual_scheduler_advance_timer` は呼ばない。
- executor outcome を合成しない。
- yield-to-clock handling、complete handling、actual scheduler loop、native / bare / headless real backend、queue drain、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op は含めない。

完了条件:

- F5ei source policy が ExecuteHostAction-only input、caller supplied outcome only、F5du / F5dv / F5ea exactly once、remaining_count / timer_state before owner consumption、lower error wrapping、backend / queue / fallback 禁止を検査する。
- focused doctest が source policy label を持つ。
- 次の再開 target は YieldToClock / Complete を扱う F5ej deterministic clock-delta authority と complete ack boundary であり、actual real scheduler loop、native / bare scheduler backend、headless app-loop integration はその後に進める。

subagent review:

- Aquinas に F5ei 実装計画を渡し、implementation may start を確認した。実装後に、ExecuteHostAction only input、caller supplied outcome only、F5du / F5dv / F5ea one call each、remaining_count / timer_state preservation、lower error wrapping、non-Copy / non-Clone、no backend / no queue / no fallback の観点で再確認させる。

## Phase 5.13 / F5ej: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop yield complete boundary

目的:

- F5eg `YieldToClock` action を caller supplied clock delta で進める deterministic clock-delta authority を固定する。
- F5eg `Complete` action を terminal completed payload へ明示 ack する。
- actual real scheduler loop / headless app-loop integration の前段として、typed action payload ごとの再開境界を揃える。

実装:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYieldCompleteYieldAdvanceResult` は `YieldReady` と `YieldPending` を持つ。
- `DeltaInvalid` と `YieldDelayInvalid` を分け、どちらも `Option::Some GuiError::InvalidCommand` category を保持する owner-bearing error にする。
- `loop_yield_complete_yield_advance` は `remaining_count` と `yield_delay_ms` を state owner consumption 前に読み、`delta_ms >= 0` と `yield_delay_ms >= 0` を検査してから pending branch のみで `sub yield_delay_ms delta_ms` を行う。
- `loop_yield_complete_complete_ack` は `Complete` payload の `remaining_count` を completed owner consumption 前に読み、terminal completed payload を返す。
- `stdlib/std/gui.nepl` facade から export する。
- `tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete.n.md` を追加し、facade、result shape、error shape、F5eg / F5ea import、read-before-consume、validation、pending / ready、complete ack、no wildcard / backend / queue / fallback label を固定する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` と `nodesrc/test_web_gui_font_rendering_contract.js` に Phase 5.13 / F5ej source policy を追加する。

非目標:

- general `LoopAction` を受けない。
- F5eg `loop_action_from_result`、F5ef `loop_step`、F5eh `loop_timer_advance`、F5ei `loop_executor_complete`、F5du / F5dv scheduler decision path は呼ばない。
- actual real scheduler loop、native / bare / headless real backend、queue drain、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op は含めない。

完了条件:

- F5ej source policy が negative delta / negative yield delay separation、read-before-consume、sub-after-validation、pending / ready branch、complete ack、backend / queue / fallback 禁止を検査する。
- focused doctest が source policy label を持つ。
- 次の再開 target は actual real scheduler loop、headless app-loop integration、native / bare scheduler backend である。

subagent review:

- Aquinas に F5ej 実装計画を渡した。
- Review change として `yield_delay_ms < 0` の検査、`YieldDelayInvalid` error kind、明示 `YieldAdvanceResult` enum、read-before-consume / validate-before-sub source policy を要求されたため、実装計画に反映した。
- 実装後に、F5eg / F5ea only import、negative delta / negative delay separation、non-Copy / non-Clone、no timer advance / executor complete / actual real scheduler loop / queue / fallback の観点で再確認させる。

## Phase 5.14 / F5ek: std layer row tile RLE present host span operation presenter executor session turn virtual scheduler real loop step boundary

目的:

- F5eg `LoopAction` と caller supplied explicit input を照合し、F5ej / F5eh / F5ei の typed authority へ 1 段だけ進める。
- actual real scheduler loop / headless app-loop が使う dispatch 境界を、backend、queue、timer sleep、platform API から分離して固定する。
- 入力種別不一致を silent no-op にせず、action owner と input owner を保持する mismatch error として返す。

実装:

- `stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step.nepl` を追加する。
- `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerRealLoopStepPolicy` は `scheduler_policy` と `timer_policy` だけを保持する。`LoopExecutorCompletePolicy` は保持しない。
- F5ei に borrowed policy entry を追加し、F5ek Execute branch は同じ `scheduler_policy` と `timer_policy` を借用して `loop_executor_complete_with_policy_refs` を呼ぶ。
- `RealLoopStepInput` は `ClockDelta`、`ExecutorOutcome`、`CompleteAck` を持つ。
- `RealLoopStepResult` は `StateReady`、`YieldPending`、`Completed` を持つ。
- `RealLoopStepError` は action ごとの input mismatch と、F5ej / F5eh / F5ei lower failure を分ける。
- `stdlib/std/gui.nepl` facade から export する。
- focused doctest は import smoke と source policy label を固定する。
- `nodesrc/test_web_gui_offscreen_headless_contract.js` と `nodesrc/test_web_gui_font_rendering_contract.js` に Phase 5.14 / F5ek source policy を追加する。

非目標:

- actual while loop、queue drain、scheduler sleep、setTimeout / setInterval、host backend、platform API、DOM / Canvas / minifb、video memory は含めない。
- F5ef loop step、F5ee slice、F5ec drain、F5ed transition、F5eb step、direct virtual timer を呼ばない。
- executor outcome を合成しない。
- fallback と silent no-op は含めない。

完了条件:

- F5ek source policy が single timer policy authority、explicit input shape、action/input dispatch pair、mismatch owner recovery、F5ej / F5eh / F5ei single call、backend / queue / fallback 禁止を検査する。
- focused doctest が source policy label を持つ。
- 次の再開 target は F5ek result を使う actual real scheduler loop driver、headless app-loop integration、native / bare scheduler backend である。

subagent review:

- Dirac plan review は `PLAN_CHANGES`。`executor_policy` と `timer_policy` を同時に保持すると timer policy authority が二重化するため、F5ek policy は `scheduler_policy` と `timer_policy` だけを保持し、Execute branch では F5ei borrowed policy entry を使うように変更した。

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

Phase F5fw の Bare display hardware flush accepted boundary を現在の checkpoint とする。直前の F5fv は row-tile RLE packet readiness を full-height surface へ集約しただけで、hardware flush や scheduler completion は主張していなかった。そのため、次の実装では F5fv の sealed owner-bearing completed value だけを authority とし、bare host import が flush request を accepted として返したことを evidence 化する。

- `stdlib/platforms/gui/bare/display_surface_readiness.nepl` の `GuiBareDisplayWholeSurfacePacketReadinessCompleted` に module-private completed seal を追加し、owner + copy evidence だけから completed authority を偽造できないようにする。
- `stdlib/platforms/gui/bare/display_flush_completion.nepl` を追加する。
- public authority は `GuiBareDisplayWholeSurfacePacketReadinessCompleted` value だけとし、copyable whole-surface evidence、flush evidence、driver step / outcome、raw storage を input authority にしない。
- host import 名は `nepl_gui_bare.display_hardware_flush` とし、target kind、window raw、surface、frame、frame id、batch index、width、height、stride bytes、tile rows、tile count、expected pixel count、ready pixel count、surface byte count を渡す。
- preflight は host import の前に width / height / tile rows / tile count、checked `width * height`、`ready_pixel_count == expected_pixel_count`、checked `width * 4`、checked `height * stride_bytes` を検査する。
- preflight error は `status == Option::None`、host status error は `status == Option::Some status` として区別する。
- status `0` だけを accepted とし、`-1` Unsupported、`-2` / `-6` InvalidCommand、`-3` / `-4` ResourceExhausted、その他は BackendFailure として fail-closed にする。
- success は module-private accepted seal を持つ `GuiBareDisplayHardwareFlushAccepted` とし、physical scanout completion、scheduler completion、long-running backend completion を主張しない。
- owner-bearing accepted / error は `Clone` / `Copy` にしない。
- docs、focused doctest、source-policy、note、todo、default bare test import を同じ slice で更新する。

Phase 5.13 / F5ej の deterministic virtual scheduler loop yield complete boundary までは既存 checkpoint として完了済みである。2026-06-19 の F5gb では、Native / Bare scheduler bounded real-loop runner として F5el start から F5fz / F5ga platform step を `max_step_count` で bounded に進める checkpoint を追加した。これは long-running real backend loop へ向かう platform-neutral runner であり、queue、sleep、timer wait、fallback、silent no-op は持たない。F5gb の次の再開 target は、formal `std/gui` present host import 接続、OS window loop / minifb event pump、FHD 60fps measurement、2D compositor drain へ進めることである。

## Phase F5gc: std layer row tile RLE present host import scheduler start boundary

Phase F5gc では、`stdlib/std/gui/tile_present_host_import_scheduler_start.nepl` を追加し、F5cr request から F5cw action、F5du turn start、F5ea `virtual_scheduler_turn` へつながる formal `std/gui` present host import 接続の最小境界を固定する。

- public input authority は support、span policy、dynamic timer state、F5cr request だけに限定する。
- action は F5cw `gui_rgba8888_row_tile_rle_present_host_execution_action &request` で作り、F5du `turn_start` に 1 回だけ渡す。
- F5du が成功した場合だけ F5ea `virtual_scheduler_turn` に timer state と turn state を渡し、initial virtual scheduler state を ready value に入れる。
- F5du が失敗した場合は original request、derived action、lower F5du error、category を error value に保持する。support / span policy は Copy policy input なので recovery authority として保持しない。
- `start_with_empty_timer` は active timer を持たない明示 initial `GuiVirtualTimerState` を作る helper であり、fallback や silent no-op ではない。
- scheduler step、virtual scheduler drain / slice / loop、real loop driver、loop action mapping、turn driver complete、host import execution、timer backend、queue、platform API、DOM、Canvas、minifb、video memory、RenderTarget / DrawTarget fallback へは進まない。
- `nodesrc/test_web_gui_font_rendering_contract.js`、`nodesrc/test_web_gui_offscreen_headless_contract.js`、`nodesrc/test_stdlib_gui_layering_policy.js`、focused doctest、GUI spec、note、todo を同じ slice で更新する。
- plan review では、explicit empty timer semantics、layering policy の追加、後続 authority の禁止、support / span policy を recovery authority として保持しないことを確認する。

## Phase F5gd: Native window event pump boundary

Phase F5gd では、native smoke runner の OS window observation を型付き event pump snapshot として切り出す。これは F5ff の exact-size resize redraw を、main.rs の生 minifb polling ではなく future native backend / test harness へ接続可能な境界にする作業である。

実装:

- `nepl-gui-native/src/lib.rs` に `NativeWindowSize`、`NativeWindowEventPumpInput`、`NativeWindowEventPumpSnapshot`、`NativeWindowEventPumpCloseState`、`NativeWindowPointerButtonTransition`、`NativeWindowPointerSample`、`NativeWindowEventPumpError` を追加する。
- pure builder は current size、previous size、previous mouse state、current mouse state、pointer raw sample、OS close request、exit shortcut request だけを受け、minifb に依存しない。
- `poll_minifb_window_event_pump` は `cfg(all(feature = "window", not(target_arch = "wasm32")))` の薄い adapter とし、minifb から読んだ値を pure builder へ渡すだけにする。
- `NativeWindowSize` は observed size として zero を許す。zero dimension は `NativeWindowPresenterSurfaceState::Unavailable` に写し、Drawable として扱わない。
- close state は OS close と Escape shortcut を別 variant にし、現 smoke runner ではどちらも terminal side process 終了へ写すが、contract 上は close request / lifecycle / virtual event test が区別できるようにする。
- main.rs は `Key` / `MouseButton` / `MouseMode` / `is_open` / `is_key_down` / `get_mouse_down` / `get_unscaled_mouse_pos` を直接使わず、snapshot を `match` する。
- zero-size path では `window.update` で event pump だけを進め、positive drawable path では surface size と same width / height の RGB0 buffer を再生成して `update_with_buffer` する。
- `nodesrc/test_native_gui_platform_behavior.js`、`doc/neplg2/gui_standard_library_spec.md`、`doc/neplg2/gui_native_platform_behavior.md`、note、todo を同じ slice で更新する。

非目標:

- formal `std/gui` host import execution、scheduler loop、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は含めない。
- event pump helper は `window.update` / `update_with_buffer` を呼ばない。
- pointer sample がない状態を error にしない。非有限 coordinate だけを typed error にする。
- fallback frame、blank frame、silent no-op、synthetic click は作らない。

完了条件:

- pure builder tests が unchanged size、positive resize、zero resize、zero-to-positive restore、Pressed / Held / Released / Idle、pointer unavailable、non-finite pointer error、OS close / shortcut close 分離を検査する。
- source-policy が minifb input API を `poll_minifb_window_event_pump` に閉じ、main.rs から直接読まないことを検査する。
- `cargo test -p nepl-gui-native --lib` と `cargo check -p nepl-gui-native --features window` を通す。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5ge: Native backend loop step boundary

Phase F5ge では、F5gd の event pump snapshot を受けた後の native smoke backend loop state transition を `NativeWindowBackendLoop` へ切り出す。F5gd で minifb input polling は分離済みだが、`main.rs` には resize redraw、frame id update、counter hit test、presenter state commit が残っていた。F5ge はこれを OS 非依存の typed loop step とし、future native backend / test harness / scheduler host integration が同じ state transition を再利用できるようにする。

実装:

- `nepl-gui-native/src/lib.rs` に `NativeWindowBackendLoop`、`NativeWindowBackendLoopState`、`NativeWindowBackendLoopPresentation`、`NativeWindowBackendLoopPointerAction`、`NativeWindowBackendLoopStepOutcome`、`NativeWindowBackendLoopError` を追加する。
- `NativeWindowBackendLoop::new_for_scale` は demo、counter value、scale から initial frame、checked initial size、presenter state、initial present を作る。初期化失敗は `NativeWindowBackendLoopError` の variant で返し、`String` に潰さない。
- `NativeWindowBackendLoop::event_pump_input` は previous observed size と previous mouse state から F5gd `NativeWindowEventPumpInput` を返す。
- `NativeWindowBackendLoop::step` は `NativeWindowEventPumpSnapshot` を 1 件だけ処理し、close no-progress、unavailable observation update、positive resize redraw、counter pointer action、final frame evidence を enum / struct で返す。
- positive resize は new-size frame の rasterize / RGB0 buffer validation / present が成功した後だけ surface state、previous size、frame id を commit する。
- counter hit は pointer unavailable、letterbox/outside、actual hit を分け、hit の場合だけ counter checked add と frame id checked add を mutation 前に検査する。
- `main.rs` は `NativeWindowBackendLoop` に state transition を委譲し、minifb window creation、event pump adapter、title update、`window.update`、`window.update_with_buffer` だけを持つ。
- `nodesrc/test_native_gui_platform_behavior.js` は、`main.rs` から `counter_hit`、`map_native_window_point_to_image`、`checked_add`、`rasterize_frame_to_surface`、`present_buffer`、`resize_surface` が戻らないことと、loop helper が minifb / DOM / Canvas / video memory / stdout / fallback / silent no-op を持たないことを検査する。
- `doc/neplg2/gui_standard_library_spec.md`、`doc/neplg2/gui_native_platform_behavior.md`、`doc/neplg2/gui_tui_implementation_plan.md`、note、todo を同じ slice で更新する。

非目標:

- formal `std/gui` host import execution、scheduler loop、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は含めない。
- loop outcome は pixel borrow を持たない。final committed frame の pixel borrow は `current_present_frame_for_window` からだけ取得する。
- blank frame、fallback frame、silent no-op、synthetic click、best-effort counter action は作らない。

完了条件:

- tests が close no-progress、unavailable no blank、positive resize commit-after-present、zero-to-positive restore、resize+counter two presentation evidences、pointer unavailable/outside/hit、frame id overflow、counter overflow、rasterize failure preservation を検査する。
- `cargo test -p nepl-gui-native --lib`、`cargo check -p nepl-gui-native --features window`、`node nodesrc/test_native_gui_platform_behavior.js` を通す。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gf: Native host action boundary

Phase F5gf では、F5ge の `NativeWindowBackendLoopStepOutcome` を `main.rs` が直接解釈しないようにし、native host がこの iteration で実行する操作を `NativeWindowHostAction` として受け取る境界を追加する。これは単なる rename ではなく、backend state transition evidence と host execution instruction を分け、future formal native OS scheduler / window backend loop が同じ action contract を消費できるようにする作業である。

実装:

- `nepl-gui-native/src/lib.rs` に `NativeWindowHostTerminalReason`、`NativeWindowHostAction`、`NativeWindowHostActionError` を追加する。
- `NativeWindowHostAction` は `Terminate`、`PumpEventsOnly`、`PresentFrame` を持つ。`PumpEventsOnly` は `NativeWindowSize` と `size_changed` を保持し、`PresentFrame` は `NativeWindowBackendLoopPresentation`、`NativeWindowSize`、`size_changed` を保持する。pixel borrow は持たない。
- `NativeWindowBackendLoop::step_host_action` は `step` を呼び、`CloseRequested` を `Terminate`、`Unavailable` を `PumpEventsOnly`、`Drawable` を `PresentFrame` へ写す。
- `CloseRequested Open` のような contradictory outcome は `UnsupportedCloseState` として typed error にする。backend step の失敗は `StepFailed NativeWindowBackendLoopError` として original error を保持する。
- `main.rs` は `NativeWindowHostAction` を match し、`NativeWindowBackendLoopStepOutcome` を import / match しない。`PresentFrame` の actual pixel borrow は `current_present_frame_for_window` からだけ取得する。
- `nodesrc/test_native_gui_platform_behavior.js` は `step_host_action`、`NativeWindowHostAction`、`NativeWindowHostTerminalReason`、`NativeWindowHostActionError`、main.rs の outcome 直接 match 禁止を検査する。
- `doc/neplg2/gui_standard_library_spec.md`、`doc/neplg2/gui_native_platform_behavior.md`、`doc/neplg2/gui_tui_implementation_plan.md`、note、todo を同じ slice で更新する。

非目標:

- formal scheduler loop、OS wait strategy、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は含めない。
- minifb / DOM / Canvas / video memory transport を lib helper に入れない。
- blank frame、fallback frame、silent no-op は作らない。

完了条件:

- tests が terminal reason preservation、unavailable pump-only action、drawable present action、impossible open close error を検査する。
- `cargo test -p nepl-gui-native --lib`、`cargo check -p nepl-gui-native --features window`、`node nodesrc/test_native_gui_platform_behavior.js` を通す。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gg: Native minifb window run-loop adapter boundary

Phase F5gg では、F5gf の `NativeWindowHostAction` を実際の minifb smoke window に適用する cfg-gated run-loop adapter として `run_minifb_window_loop` を追加する。F5ge / F5gf で `main.rs` が minifb window lifecycle と host action execution を持つと説明していた部分はこの phase で supersede し、future formal native OS scheduler / window backend loop が同じ runner-facing action contract を消費できる形へ近づける。

実装:

- `nepl-gui-native/src/lib.rs` に `NativeWindowRunLoopConfig`、`NativeWindowRunLoopExit`、`NativeWindowRunLoopError`、`native_window_title`、cfg-gated `run_minifb_window_loop` を追加する。
- `run_minifb_window_loop` は `WindowOptions`、`ScaleMode::UpperLeft`、window title update、`window.update`、`update_with_buffer` を所有する。
- `main.rs` は CLI option を `NativeWindowRunLoopConfig` へ変換し、`run_minifb_window_loop` を呼ぶだけにする。`main.rs` では minifb、`WindowOptions`、`ScaleMode`、`window.update`、`update_with_buffer` を使わない。
- run-loop adapter は `poll_minifb_window_event_pump` と `step_host_action` を呼ぶだけで、`Key`、`MouseButton`、`MouseMode`、`is_open`、`is_key_down`、`get_mouse_down`、`get_unscaled_mouse_pos` を直接読まない。
- `NativeWindowRunLoopError` は backend initialization、window creation、event pump、host action selection、presenter frame availability、window present failure を enum で分ける。`window.update` は error を返さないため、present failure variant は `WindowPresentFailed` として `update_with_buffer` failure だけを表す。
- source policy は `run_minifb_window_loop` slice にだけ minifb window lifecycle API を許可し、その slice でも queue、timer、`std::thread::sleep`、`Duration`、`setTimeout`、`setInterval` を禁止する。`set_target_fps(60)` は smoke runner の busy spin 抑制としてだけ許可する。
- `doc/neplg2/gui_standard_library_spec.md`、`doc/neplg2/gui_native_platform_behavior.md`、`doc/neplg2/gui_tui_implementation_plan.md`、note、todo を同じ slice で更新する。

非目標:

- formal native OS scheduler loop、OS wait strategy、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は含めない。
- DOM / Canvas / video memory transport、fallback、silent no-op は作らない。

完了条件:

- tests が run-loop config preservation、title helper、main.rs の minifb ownership 排除、run-loop slice の direct input API 禁止、queue / timer 禁止を検査する。
- `cargo test -p nepl-gui-native --lib`、`cargo check -p nepl-gui-native --features window`、`node nodesrc/test_native_gui_platform_behavior.js` を通す。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gh: Native window host-loop core boundary

Phase F5gh では、F5gg の `run_minifb_window_loop` から minifb 非依存の host-loop core を切り出す。`NativeWindowRunLoopHost` は event snapshot polling、title update、pump-only、present を表す trait であり、`run_native_window_host_loop` は `&mut NativeWindowBackendLoop` と `&mut Host` を受けて host action を実行する。

実装:

- `NativeWindowRunLoopHost` trait と `NativeWindowHostLoopError EventError PresentError` を追加する。
- `run_native_window_host_loop` は backend loop を value で消費せず、`&mut NativeWindowBackendLoop` を受ける。error path でも caller が backend state を回収できるようにする。
- core loop は initial title 設定、`poll_event_snapshot`、`step_host_action`、`Terminate` / `PumpEventsOnly` / `PresentFrame` の execution だけを行う。
- `PresentFrame` では `current_present_frame_for_window` から借用した exact-size frame だけを host present へ渡す。
- minifb 依存 API は private `MinifbNativeWindowRunLoopHost` に移し、`run_minifb_window_loop` は backend loop / minifb window / host adapter を初期化して `run_native_window_host_loop` を呼ぶだけにする。
- tests は terminal reason、unavailable pump-only、exact frame present、host event error、host present error、host action error、presenter frame unavailable error を検査する。
- source policy は core loop slice と minifb host adapter slice を分ける。core loop は minifb、direct input API、`window.update`、`update_with_buffer`、DOM / Canvas / video memory、queue / timer / sleep、fallback、silent no-op を禁止する。minifb host adapter は direct input API と queue / timer / fallback を禁止する。
- note、todo、GUI spec、native behavior doc、GUI/TUI implementation plan を同じ slice で更新する。

非目標:

- formal native OS scheduler loop、OS wait strategy、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は含めない。
- DOM / Canvas / video memory transport、fallback、silent no-op は作らない。

完了条件:

- `cargo test -p nepl-gui-native --lib`、`cargo check -p nepl-gui-native --features window`、`node nodesrc/test_native_gui_platform_behavior.js` を通す。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gi: Native window host-loop turn boundary

Phase F5gi では、F5gh の `run_native_window_host_loop` に残る long loop body を typed one-turn boundary へ分ける。これは formal native OS scheduler / window backend loop の準備であり、scheduler queue や timer wait そのものは実装しない。

実装:

- `NativeWindowHostLoopTurn` enum を追加し、`Continue` と `Exit NativeWindowRunLoopExit` だけを持たせる。
- `step_native_window_host_loop` を追加し、host event snapshot、`step_host_action`、`Terminate` / `PumpEventsOnly` / `PresentFrame` の 1 turn だけを実行する。
- `step_native_window_host_loop` は initial title を設定しない。initial title は `run_native_window_host_loop` 側にだけ残す。
- `run_native_window_host_loop` は initial title を設定した後、`step_native_window_host_loop` の `Continue` / `Exit` だけを match する。`poll_event_snapshot`、`step_host_action`、`NativeWindowHostAction` の直接 match、present frame borrow、host pump / present は run loop body に戻さない。
- tests は close turn が title / pump / present を増やさないこと、pump-only resize が title update と `Continue` を返すこと、drawable resize が exact frame present と `Continue` を返すこと、event / host action / presenter frame / present error が typed variant のまま返ることを検査する。
- source policy は one-turn core slice と long loop runner slice を分ける。long loop runner は `step_native_window_host_loop` だけを呼び、one-turn core は minifb、direct input API、queue、timer、sleep、DOM / Canvas / video memory、fallback、silent no-op を禁止する。
- note、todo、GUI spec、native behavior doc、GUI/TUI implementation plan を同じ slice で更新する。

非目標:

- formal native OS scheduler loop、OS wait strategy、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は含めない。
- DOM / Canvas / video memory transport、fallback、silent no-op は作らない。

完了条件:

- `cargo test -p nepl-gui-native --lib`、`cargo check -p nepl-gui-native --features window`、`node nodesrc/test_native_gui_platform_behavior.js` を通す。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gj: Native window host-loop bounded runner boundary

Phase F5gj では、F5gi の `step_native_window_host_loop` を bounded に反復する Rust smoke/native layer の runner を追加する。これは infinite loop に入らず `Exited` と `BudgetExhausted` を区別する cooperative turn boundary であり、formal OS wait strategy、queue、timer wait は実装しない。

実装:

- `NativeWindowHostLoopRunnerState` を追加し、initial title 設定済みかどうかだけを保持する。
- `NativeWindowHostLoopInitialization` を追加し、`initialize_native_window_host_loop` が `Initialized` / `AlreadyInitialized` を返す。idempotent path は silent no-op にしない。
- `NativeWindowHostLoopBoundedRunResult` を追加し、`Exited exit completed_turns` と `BudgetExhausted completed_turns` を分ける。
- `run_native_window_host_loop_bounded` を追加し、初期化を確認した後、`max_turn_count` の範囲で `step_native_window_host_loop` だけを呼ぶ。
- `max_turn_count == 0` は event poll を行わず `BudgetExhausted completed_turns 0` を返すが、initial title の初期化確認は行う。
- `Continue` は completed turn count を増やし、`Exit` は exit turn を含めた count で `Exited` を返す。
- `run_native_window_host_loop` は `usize::MAX` を使って bounded runner を呼ばず、infinite loop の意味を明示したまま initializer と one-turn function だけを共有する。
- tests は initializer first / already initialized、zero budget no poll、exit turn count、continue budget exhaustion、複数 bounded call での initial title 重複なし、event / host action / presenter frame / present error preservation を検査する。
- source policy は initializer、bounded runner、long loop runner、one-turn core の slice を分け、bounded runner が direct action body を持たず、queue、timer、sleep、DOM / Canvas / video memory、fallback、silent no-op を含まないことを検査する。
- note、todo、GUI spec、native behavior doc、GUI/TUI implementation plan を同じ slice で更新する。

非目標:

- formal native OS scheduler loop、OS wait strategy、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は含めない。
- DOM / Canvas / video memory transport、fallback、silent no-op は作らない。

完了条件:

- `cargo test -p nepl-gui-native --lib`、`cargo check -p nepl-gui-native --features window`、`node nodesrc/test_native_gui_platform_behavior.js` を通す。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gk: Native window frame pacing config boundary

Phase F5gk では、native smoke window loop の frame pacing を hidden constant から typed config へ移す。F5gj で bounded runner boundary は入ったが、`run_minifb_window_loop` に `set_target_fps 60` が直接置かれているため、frame pacing contract が static data として検査できない。F5gk はこの root boundary を直す。

実装:

- `NativeWindowTargetFps` newtype、`NativeWindowTargetFpsInvalidReason`、`NativeWindowTargetFpsError` を追加する。
- `NATIVE_WINDOW_RUN_LOOP_MIN_TARGET_FPS = 1`、`NATIVE_WINDOW_RUN_LOOP_MAX_TARGET_FPS = 240`、`NATIVE_WINDOW_RUN_LOOP_DEFAULT_TARGET_FPS = 60` を追加する。
- `NativeWindowRunLoopConfig` に `target_fps NativeWindowTargetFps` を追加し、既存 `new` は default 60 を使う。
- `new_with_target_fps` と raw value 用の fail-closed helper を追加し、invalid raw value は `NativeWindowRunLoopError::TargetFpsInvalid value reason` とする。
- `run_minifb_window_loop` は validation 済み `target_fps.as_usize` だけを `window.set_target_fps` に渡す。`set_target_fps 60` や `set_target_fps config.target_fps` は禁止する。
- CLI に `--fps N` を追加する。headless mode では frame pacing は使われないが、window mode 用 option として usage に明記する。
- tests は default / custom / zero / too-high / run-loop error mapping を検査する。
- source policy は typed FPS config、range constants、validated value pass、hard-coded FPS 禁止を固定する。
- note、todo、GUI spec、native behavior doc、GUI/TUI implementation plan を同じ slice で更新する。

非目標:

- formal OS wait strategy、queue / timer wait backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- fallback、silent no-op、invalid FPS clamp、raw `usize` FPS の minifb 直渡しは作らない。

subagent review:

- Feynman the 2nd に F5gk 計画を渡し、typed newtype、upper bound、invalid reason/value、CLI boundary、source-policy の観点で確認させる。指摘があれば実装前に反映する。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gl: Native window host-loop run policy boundary

Phase F5gl では、F5gj の bounded runner を native smoke window の long loop path に接続し、実行予算を `NativeWindowHostLoopRunPolicy` として明示する。F5gj では finite turn count の runner が入ったが、F5gk 後の `run_minifb_window_loop` はまだ direct long runner を呼んでいる。F5gl はこの root boundary を直し、future formal native OS scheduler / window backend loop が同じ bounded runner contract を使えるようにする。

実装:

- `NativeWindowHostLoopTurnSlice` newtype、`NativeWindowHostLoopTurnSliceInvalidReason`、`NativeWindowHostLoopTurnSliceError` を追加する。
- `NATIVE_WINDOW_HOST_LOOP_MIN_TURN_SLICE = 1`、`NATIVE_WINDOW_HOST_LOOP_MAX_TURN_SLICE = 4096`、`NATIVE_WINDOW_HOST_LOOP_DEFAULT_TURN_SLICE = 1` を追加する。`4096` は OS wait / timer / performance guarantee ではなく bounded turn budget の上限である。
- `NativeWindowHostLoopRunPolicy` を追加し、`NativeWindowRunLoopConfig.host_loop_policy` に保持する。既存 constructor は default policy を使う。
- `run_native_window_host_loop_with_policy` を追加し、`run_native_window_host_loop_bounded` だけを反復する。`Exited` は return、`BudgetExhausted` は同じ runner state で次 slice に進める。
- `run_native_window_host_loop` は default policy runner に委譲し、`run_minifb_window_loop` は `config.host_loop_policy` を渡して policy runner を呼ぶ。
- tests は default / custom / zero / too-high / config preservation / single-turn slices across exit / error preservation を検査する。
- source policy は policy type、range constants、default policy delegation、minifb policy path、policy runner -> bounded runner、`usize::MAX` 禁止を固定する。
- note、todo、GUI spec、native behavior doc、GUI/TUI implementation plan を同じ slice で更新する。

非目標:

- formal OS wait strategy、queue / timer wait backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- CLI option は追加しない。これは user-facing tuning ではなく native host-loop policy boundary である。
- sleep、queue、timer、DOM / Canvas / video memory transport、fallback、silent no-op、`usize::MAX` による unbounded slice は作らない。

subagent review:

- Feynman the 2nd に F5gl 計画を渡し、default turn slice、upper bound、CLI 非公開、policy runner delegation、source-policy の観点で確認させる。指摘があれば実装前に反映する。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gm: Native window host-loop turn evidence boundary

Phase F5gm では、F5gi / F5gj / F5gl の host loop turn boundary に、future wait decision 用の value evidence を追加する。現状の `NativeWindowHostLoopTurn::Continue` は pump-only と present-frame を同じ signal に潰すため、後続の formal native OS scheduler / window backend loop が surface-unavailable pump と successfully-presented frame を型で区別できない。

実装:

- `NativeWindowHostLoopContinueEvidence` enum を追加し、`PumpedEventsOnly window_size size_changed` と `PresentedFrame presentation window_size size_changed` を持たせる。
- `NativeWindowHostLoopTurn::Continue` は `NativeWindowHostLoopContinueEvidence` を保持する。
- `step_native_window_host_loop` は `PumpEventsOnly` branch では `host.pump_events_only` 後に `PumpedEventsOnly` evidence を返す。
- `step_native_window_host_loop` は `PresentFrame` branch では `current_present_frame_for_window` と `host.present_frame` が成功した後だけ `PresentedFrame` evidence を返す。pixel borrow は evidence に含めない。
- `run_native_window_host_loop_bounded` と policy runner は `Continue _` を turn count として扱い、evidence を scheduler policy に先取り利用しない。
- tests は pump-only resize evidence、drawable resize evidence、drawable no-resize evidence、bounded runner count preservation、present error で evidence が返らないことを検査する。
- source policy は plain `Continue` への逆戻り、queue / timer / sleep / fallback / silent no-op、pixel borrow 混入を禁止する。
- note、todo、GUI spec、native behavior doc、GUI/TUI implementation plan を同じ slice で更新する。

非目標:

- formal OS wait strategy、queue / timer wait backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- CLI option、sleep、queue、timer、DOM / Canvas / video memory transport、fallback、silent no-op は作らない。

subagent review:

- Feynman the 2nd に F5gm 計画を渡し、F5gi の plain turn 方針との整合、evidence name、present 成功後だけ evidence を返すこと、source-policy の観点で確認させる。指摘があれば実装前に反映する。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gn: Native window host-loop wait decision boundary

Phase F5gn では、F5gm の turn evidence を future native OS scheduler が消費できる wait class へ分類する。`Continue` evidence のまま bounded runner の外へ出すだけでは、後続の wait strategy が pump-only と presented-frame の意味を毎回再解釈することになるため、分類結果を `NativeWindowHostLoopWaitDecision` として固定する。

実装:

- `NativeWindowHostLoopWaitDecision` enum を追加し、`WaitForHostEvent window_size size_changed` と `WaitForFrameInterval presentation window_size size_changed` を持たせる。
- `native_window_host_loop_wait_decision` は `NativeWindowHostLoopContinueEvidence` を受け、`PumpedEventsOnly` を `WaitForHostEvent`、`PresentedFrame` を `WaitForFrameInterval` へ写す pure helper とする。
- `NativeWindowHostLoopBoundedRunResult::BudgetExhausted` は `completed_turns` と `last_wait_decision Option NativeWindowHostLoopWaitDecision` を持つ。
- `run_native_window_host_loop_bounded` は zero budget では `None`、各 `Continue evidence` で `last_wait_decision` を更新し、最後の decision だけを返す。
- policy runner での実 wait dispatch は F5go に分け、F5gn は分類結果を bounded runner output として保持するところまでにする。
- tests は helper の全域写像、zero budget の `None`、pump-only budget exhaustion の `WaitForHostEvent`、pump-only 後 present の `WaitForFrameInterval` 上書きを検査する。
- source policy は helper / bounded runner に pixel borrow、host handle、scheduler state、queue / timer、sleep / Duration、minifb / DOM / Canvas / video memory / stdout、fallback、silent no-op が混入しないことを固定する。

非目標:

- formal OS wait strategy、queue / timer wait backend、実時間 sleep、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- `WaitForFrameInterval` は frame-paced wait class evidence であり、実際の FPS 保証や timer registration ではない。

subagent review:

- Feynman the 2nd に F5gn 計画を渡し、`last_continue_evidence` で止めず wait decision に写像する妥当性、pure helper、`BudgetExhausted` の `last_wait_decision`、source-policy の観点で確認させる。指摘があれば実装前に反映する。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5go: Native window host-loop wait dispatch boundary

Phase F5go では、F5gn の wait decision classification を `NativeWindowRunLoopHost` の wait hook に接続する。F5gn のままでは policy runner が bounded slice を繰り返すだけで、host 側の wait authority が型として表に出ない。F5go は `NativeWindowHostLoopWaitDecision` を `wait_after_budget_exhausted` に渡し、host wait failure と missing wait evidence を enum error として分離する。

実装:

- `NativeWindowRunLoopHost` に `type WaitError` と `wait_after_budget_exhausted decision` を追加する。
- `NativeWindowHostLoopWaitOutcome` enum を追加し、`HostEventPumpAlreadyPaced window_size size_changed`、`FramePresentAlreadyPaced presentation window_size size_changed`、後続 F5gx の `FrameIntervalTimerRegistered presentation window_size size_changed wait_nanos timer_registration_id` を区別できる outcome evidence とする。
- `NativeWindowHostLoopError` を `EventError`、`PresentError`、`WaitError` の 3 generic にし、`HostWaitFailed WaitError` と `WaitDecisionMissing` を追加する。
- `run_native_window_host_loop_with_policy` は `BudgetExhausted last_wait_decision = Some decision` で host wait hook を呼び、`None` は `WaitDecisionMissing` として返す。
- minifb adapter の `WaitError` は `std::convert::Infallible` とし、wait hook は `Window::set_target_fps` によって `window.update` / `update_with_buffer` 内部ですでに pace されたことを outcome に写すだけにする。
- tests は wait dispatch が次の event poll より前に呼ばれること、`WaitForFrameInterval` が outcome へ渡ること、wait error が次 poll を発生させず preserved されることを検査する。
- source policy は wait method slice に追加の `window.update`、`update_with_buffer`、`Duration`、`std::thread::sleep`、queue、timer、fallback、silent no-op が入らないことを固定する。

非目標:

- formal OS wait strategy、queue / timer wait backend、real timer registration、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- minifb adapter は F5go で追加の event pump や frame present を行わない。minifb の rate limit authority は `Window::set_target_fps` と、その後の `Window::update` / `update_with_buffer` call path に残す。

subagent review:

- Feynman the 2nd に F5go 計画を渡し、wait hook の配置、minifb pacing の二重実行回避、`HostWaitFailed` / `WaitDecisionMissing` の fail-closed contract、source-policy の観点で確認させる。指摘があれば実装前に反映する。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gp: Native window host-loop scheduler slice boundary

Phase F5gp では、F5go の long runner 内部に閉じている bounded slice execution と wait dispatch を、external scheduler が呼び出せる typed boundary に切り出す。これは pure rename layer ではなく、future formal native OS scheduler / window backend loop が hidden infinite loop を再実装せずに同じ state と result enum を消費するための root boundary である。

実装:

- `NativeWindowHostLoopSchedulerState` を追加し、`NativeWindowHostLoopRunnerState` を所有させる。
- `NativeWindowHostLoopSchedulerSliceResult` を追加し、`Exited exit completed_turns` と `Waited completed_turns decision outcome` を分ける。
- `run_native_window_host_loop_scheduler_slice_with_policy` を追加し、policy の `turn_slice` で `run_native_window_host_loop_bounded` を 1 回だけ呼ぶ。
- bounded result が `BudgetExhausted last_wait_decision = Some decision` の場合だけ `wait_after_budget_exhausted` を 1 回呼び、wait outcome を `Waited` result に保持する。
- bounded result が `last_wait_decision = None` の場合は `WaitDecisionMissing` として fail closed にする。
- `run_native_window_host_loop_with_policy` は scheduler slice API を反復する wrapper にする。
- tests は slice が wait hook を 1 回だけ呼ぶこと、slice 間で initial title を二重設定しないこと、wait failure が次 poll を消費しないこと、close event では wait hook を呼ばないことを検査する。
- source policy は scheduler slice と long runner に追加の `window.update`、`update_with_buffer`、`Duration`、`std::thread::sleep`、queue、timer、fallback、silent no-op が入らないことを固定する。

非目標:

- stdlib NEPL 側 scheduler runner と minifb window loop の一気接続は行わない。
- formal OS wait strategy、queue / timer wait backend、real timer registration、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- wait outcome を unit success に潰さない。

subagent review:

- Ptolemy the 2nd に F5gp 計画を渡し、hidden long loop authority を typed slice に切る妥当性、pure rename でないこと、wait outcome preservation、source-policy の観点で確認させる。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gq: Native window host-loop wait request plan boundary

Phase F5gq では、F5gp の scheduler slice が保持している `NativeWindowHostLoopWaitDecision` を、actual backend が消費できる `NativeWindowHostLoopWaitRequest` へ変換する。これは actual OS wait strategy そのものではなく、host event wait と frame interval wait を typed request plan として固定する境界である。

実装:

- `NativeWindowFrameIntervalRequest` を追加し、validated `NativeWindowTargetFps` から `nanos_per_frame` と `remainder_nanos_per_second` を計算する。
- `NativeWindowHostLoopWaitRequest` を追加し、host event wait と frame interval wait を区別する。
- `native_window_host_loop_wait_request` を追加し、`NativeWindowHostLoopWaitDecision` と validated target fps から backend wait request plan を作る。
- F5gq は decision から request plan を作る。F5gr 以降の `NativeWindowRunLoopHost::wait_after_budget_exhausted` は request から生成される instruction を受け取る。
- scheduler slice は request と、F5gr 以降で wait hook へ渡した instruction を `NativeWindowHostLoopSchedulerSliceResult::Waited` に保持する。
- `run_native_window_host_loop_with_policy_and_target_fps` と scheduler slice の explicit target fps 版を追加し、native minifb runner は `config.target_fps` を request plan へ渡す。
- tests は default fps と explicit fps の frame interval request、host event request、wait failure が次 poll を消費しないことを検査する。
- source policy は request plan が `Duration`、`std::thread::sleep`、queue、timer registration、minifb update、fallback、silent no-op を持たないことを固定する。

非目標:

- actual OS wait strategy、queue / timer wait backend、real timer registration、FHD 60fps measurement harness は含めない。
- `std::time::Duration` や `std::thread::sleep` にはまだ接続しない。
- host event request は event payload や queue owner を持たない。
- 2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5gq 計画を渡し、typed backend wait request plan が pure rename ではないこと、target fps 由来の checked interval request、no fallback / no sleep / no queue の観点で確認させる。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gr: Native window host-loop wait strategy instruction boundary

Phase F5gr では、F5gq の `NativeWindowHostLoopWaitRequest` を、host wait backend が消費できる `NativeWindowHostLoopWaitInstruction` へ変換する。これは actual OS wait strategy や sleep 実装ではなく、scheduler slice 間で持つ frame pacing remainder を typed instruction に反映する境界である。

実装:

- `NativeWindowHostLoopWaitInstruction` を追加し、host event wait と frame interval wait を区別する。
- frame interval instruction は `NativeWindowFrameIntervalRequest` と `wait_nanos` を持つ。
- `NativeWindowHostLoopWaitStrategyState` を追加し、scheduler state の中で frame pacing target FPS と remainder accumulator を保持する。
- target FPS が変わった場合は accumulator を reset し、同じ target FPS の場合だけ previous remainder を使う。
- accumulator invariant は `0 <= remainder < fps` とし、saturating、clamp、sentinel、zero-fill fallback は使わない。
- `native_window_host_loop_wait_instruction_plan` を追加し、strategy state と wait request から next strategy state と instruction を作る。
- `NativeWindowRunLoopHost::wait_after_budget_exhausted` は request ではなく instruction を受け取る。
- scheduler slice は wait hook 成功後だけ `NativeWindowHostLoopSchedulerState.wait_strategy_state` を次状態へ進める。
- tests は remainder distribution、target FPS 変更時の reset、scheduler slice 間の accumulator 継続、wait failure 時に state を消費しないことを検査する。
- source policy は instruction plan が `Duration`、`std::thread::sleep`、queue、timer registration、minifb update、fallback、silent no-op を持たないことを固定する。

非目標:

- actual OS wait strategy、queue / timer wait backend、real timer registration、FHD 60fps measurement harness は含めない。
- `std::time::Duration` や `std::thread::sleep` には接続しない。
- host event instruction は event payload、queue owner、poll result を持たない。
- 2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5gr 計画を渡し、scheduler remainder accumulator が pure rename ではないこと、target FPS 変更時の reset、no fallback / no sleep / no queue の観点で確認させる。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gs: Native window host-loop thread wait backend boundary

Phase F5gs では、F5gr の `NativeWindowHostLoopWaitInstruction` を native thread wait backend へ渡す execution boundary を追加する。minifb smoke backend は `Window::set_target_fps` により `update` / `update_with_buffer` 内部で pace されるため、この thread wait backend を minifb wait hook へ接続しない。

実装:

- `NativeWindowHostLoopThreadSleeper` trait を追加し、`sleep_for_nanos wait_nanos` を injected boundary にする。
- `NativeWindowHostLoopThreadWaitError` を追加し、`HostEventWaitUnsupported`、`FrameIntervalWaitNanosMismatch`、`SleeperFailed` を分ける。
- `NativeWindowHostLoopThreadWaitOutcome` を追加し、frame interval sleep 成功を typed outcome として返す。
- `execute_native_window_host_loop_thread_wait_with_sleeper` を追加し、frame interval instruction の場合だけ sleeper を 1 回呼ぶ。
- `wait_nanos` は実行前に `nanos_per_frame` または `nanos_per_frame + 1` であることを再検査し、不一致なら sleeper を呼ばず fail closed にする。
- host event wait は OS event queue backend がないため `HostEventWaitUnsupported` を返す。
- `StdNativeWindowHostLoopThreadSleeper` と `execute_native_window_host_loop_thread_wait` を `cfg(not(target_arch = "wasm32"))` で追加し、actual std backend だけが `std::thread::sleep(std::time::Duration::from_nanos(...))` を呼ぶ。
- source policy は `Duration` / `std::thread::sleep` が F5gs backend slice だけにあること、scheduler / planner / minifb hook へ漏れていないことを検査する。

非目標:

- minifb wait hook へ thread wait を接続しない。
- host event queue、selector、message pump、timer registration は含めない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- fallback、silent no-op、busy loop は含めない。

subagent review:

- Darwin the 2nd に F5gs 計画を渡し、minifb double pacing を避けること、host event wait が unsupported として fail closed になること、sleep が F5gs backend helper に閉じることを確認させる。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gt: Native window host-loop timer registration backend boundary

Phase F5gt では、F5gr の `NativeWindowHostLoopWaitInstruction` を native timer registration backend へ渡す execution boundary を追加する。F5gs は thread sleep backend であり、F5gt は later native scheduler が frame interval wait を timer registration として扱うための separate backend contract である。

実装:

- `NativeWindowHostLoopTimerRegistrationId` を追加し、positive raw timer id だけを typed id として扱う。
- `NativeWindowHostLoopTimerRegistrar` trait を追加し、`register_timer_nanos wait_nanos` は host boundary として raw `u32` id を返す。
- `NativeWindowHostLoopTimerRegistrationError` を追加し、`HostEventTimerRegistrationUnsupported`、`FrameIntervalWaitNanosMismatch`、`InvalidTimerRegistrationId`、`RegistrarFailed` を分ける。
- `NativeWindowHostLoopTimerRegistrationOutcome` を追加し、frame interval timer registration 成功を typed outcome として返す。
- `execute_native_window_host_loop_timer_registration_with_registrar` を追加し、frame interval instruction の場合だけ registrar を 1 回呼ぶ。
- `wait_nanos` は実行前に `nanos_per_frame` または `nanos_per_frame + 1` であることを再検査し、不一致なら registrar を呼ばず fail closed にする。
- registrar が返した raw id が `0` の場合は `InvalidTimerRegistrationId` として拒否し、typed id を作らない。
- host event wait は OS event queue backend がないため `HostEventTimerRegistrationUnsupported` を返す。
- source policy は timer registration backend が minifb、window update、thread sleep、`Duration`、queue/event payload、DOM、Canvas、video memory、fallback、silent no-op を持たないことを検査する。

非目標:

- minifb wait hook へ timer registration を接続しない。
- host event queue、selector、message pump、real OS timer backend は含めない。
- `std::thread::sleep` / `Duration` は使わない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- fallback、silent no-op、busy loop は含めない。

subagent review:

- Darwin the 2nd に F5gt 計画を渡し、raw timer id validation、host event wait unsupported、timer registration と thread wait / minifb pacing の分離、source-policy の観点で確認させる。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gu: Native window host-loop event queue wait backend boundary

Phase F5gu では、F5gr の `NativeWindowHostLoopWaitInstruction` を host event queue wait backend へ渡す execution boundary を追加する。F5gs は frame interval の thread sleep、F5gt は frame interval の timer registration を扱い、F5gu は host event wait だけを扱う。

実装:

- `NativeWindowHostLoopEventQueueWaiter` trait を追加し、`wait_for_host_event window_size size_changed` は host event wait boundary として `Result unit Error` を返す。
- `NativeWindowHostLoopEventQueueWaitError` を追加し、`FrameIntervalEventQueueWaitUnsupported` と `WaiterFailed` を分ける。
- `NativeWindowHostLoopEventQueueWaitOutcome` を追加し、host event wait success を `HostEventReady` として返す。
- `execute_native_window_host_loop_event_queue_wait_with_waiter` を追加し、host event instruction の場合だけ waiter を 1 回呼ぶ。
- frame interval instruction は `FrameIntervalEventQueueWaitUnsupported` として拒否し、timer registration、thread sleep、busy loop、silent no-op へ変換しない。
- source policy は event queue wait backend が minifb、window update、timer registration、thread sleep、`Duration`、DOM、Canvas、video memory、fallback、silent no-op を持たないことを検査する。

非目標:

- real OS event queue / selector / message pump adapter は含めない。
- minifb wait hook へ event queue wait を接続しない。
- real OS timer backend は含めない。
- `std::thread::sleep` / `Duration` は使わない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- fallback、silent no-op、busy loop は含めない。

subagent review:

- Darwin the 2nd に F5gu 計画を渡し、`Result unit Error` waiter がこの slice では十分であること、raw OS status validation は later adapter slice に残すこと、event queue wait と timer / thread wait / minifb pacing の分離、source-policy の観点で確認させる。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gv: Native window host-loop event queue normalized status adapter boundary

Phase F5gv では、F5gu で残した raw event queue status validation を adapter boundary として追加する。actual OS event queue / selector / message pump へはまだ接続せず、platform adapter が内部正規化して返す raw status だけを検証する。

実装:

- `NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY` を追加し、これは OS API 固有の値ではなく `nepl-gui-native` adapter boundary の internal normalized status であると document する。
- `NativeWindowHostLoopEventQueueStatusAdapter` trait を追加し、`wait_for_host_event_raw_status` は `NativeWindowSize` と `size_changed` evidence を受け取って `Result u32 Error` を返す。
- `NativeWindowHostLoopEventQueueStatusAdapterError` を追加し、unknown / zero status は `InvalidRawStatus`、adapter failure は `AdapterFailed` として分離する。
- `wait_native_window_host_loop_event_queue_raw_status_with_adapter` は adapter を 1 回だけ呼び、ready status 以外を fail closed にする。
- `NativeWindowHostLoopEventQueueStatusWaiter` を追加し、F5gu の `NativeWindowHostLoopEventQueueWaiter` へ status adapter を接続する。
- F5gu executor 経由で frame interval instruction が渡された場合、F5gu 側の `FrameIntervalEventQueueWaitUnsupported` で停止し、status adapter を呼ばないことを test する。
- source policy は status adapter slice が minifb、window update、timer registration、thread sleep、`Duration`、DOM、Canvas、video memory、fallback、silent no-op を持たないことを検査する。

非目標:

- real OS event queue / selector / message pump adapter は含めない。
- minifb wait hook へ接続しない。
- real OS timer backend は含めない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- fallback、silent no-op、busy loop は含めない。

subagent review:

- Darwin the 2nd に F5gv 計画を渡し、raw status validation が pure rename ではないこと、ready status だけを accepted にすること、actual OS API 接続を later slice へ残すこと、source-policy の観点で確認させる。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gw: Native window host-loop message pump adapter boundary

Phase F5gw では、F5gv の normalized status adapter へ actual message pump adapter を接続する。ここでいう actual は platform window backend が持つ message pump を実行するという意味であり、標準 API に OS handle、DOM、Canvas、minifb 型を出すという意味ではない。

実装:

- `NativeWindowHostLoopMessagePumpAdapter` trait を追加し、`pump_host_messages` は `NativeWindowSize` と `size_changed` evidence を受け取って `Result unit Error` を返す。
- `NativeWindowHostLoopMessagePumpStatusAdapter` を追加し、pump 成功時だけ F5gv の `NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY` を返す。
- pump failure は `NativeWindowHostLoopMessagePumpStatusAdapterError::PumpFailed` として保持する。
- minifb smoke backend では `MinifbNativeWindowHostLoopMessagePumpAdapter` に `window.update` を閉じ込め、`wait_after_budget_exhausted` は direct update ではなく `wait_minifb_window_host_event_message_pump` 経由で F5gu / F5gv の event queue waiter 境界を通す。
- source policy は message pump adapter slice だけに `window.update` を許可し、event pump helper、host-loop core、message pump status adapter には minifb / timer / sleep / DOM / Canvas / video memory / fallback / silent no-op が混入しないことを検査する。

非目標:

- real OS timer backend connection は含めない。
- frame interval wait を message pump adapter で扱わない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- fallback、silent no-op、busy loop は含めない。

subagent review:

- Darwin the 2nd に F5gw 計画を渡し、OS 固有結果を normalized status / typed error へ写す adapter boundary とすること、platform detail 分離、timer backend を別 slice に残すことを確認させた。結果は `PLAN_APPROVED` である。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gx: Native window host-loop frame interval timer registration outcome boundary

Phase F5gx では、F5gt の timer registration backend を wait outcome evidence へ接続する。ただし timer registration は future fire / wakeup を予約するだけであり、frame wait の完了ではない。そのため `FramePresentAlreadyPaced` へ写すことは禁止し、専用の `FrameIntervalTimerRegistered` outcome を使う。

実装:

- `NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered` を追加し、`presentation`、`window_size`、`size_changed`、`wait_nanos`、`timer_registration_id` を保持する。
- `execute_native_window_host_loop_timer_registration_wait_with_registrar` を追加し、F5gt executor の成功 outcome だけを `FrameIntervalTimerRegistered` へ写す。
- `WaitForHostEvent` は F5gt executor の `HostEventTimerRegistrationUnsupported` のまま返し、message pump / event queue path へ残す。
- minifb smoke backend の wait hook はこの helper に接続しない。minifb は引き続き `set_target_fps` と `update_with_buffer` の pacing authority を使う。
- source policy は timer registration backend が `FramePresentAlreadyPaced`、minifb、queue、sleep、DOM、Canvas、video memory、fallback、silent no-op を含まないことを固定する。

非目標:

- actual timer fire / wakeup、real OS timer backend connection は含めない。
- thread sleep、busy loop、message pump、event queue wait を timer registration backend で代替しない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- fallback、silent no-op、wait completion の偽装は含めない。

subagent review:

- Darwin the 2nd の初回 plan review は、timer registration 成功を `FramePresentAlreadyPaced` へ写す計画を `PLAN_BLOCKED` とした。
- 修正版では `FrameIntervalTimerRegistered` outcome で timer registration evidence と pacing completion を型で分離する方針へ変更し、再 review は `PLAN_APPROVED` である。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gy: Native window host-loop timer fire/wakeup backend boundary

Phase F5gy では、F5gx の `FrameIntervalTimerRegistered` を、実際に backend が観測した timer fire / wakeup evidence へ接続する。ここで実装するのは OS 固有 timer API ではなく、registered timer id と fired timer id の照合境界である。

実装:

- `NativeWindowHostLoopTimerFireWaiter` trait を追加し、`wait_for_timer_fire` は `NativeWindowHostLoopTimerRegistrationId` を受け取って backend-observed raw fired id を返す。
- `NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired` を追加し、`presentation`、`window_size`、`size_changed`、`wait_nanos`、`timer_registration_id` を保持する。
- `NativeWindowHostLoopTimerFireError` は `HostEventPumpOutcomeUnsupported`、`FramePresentOutcomeUnsupported`、`InvalidFiredTimerRegistrationId`、`FiredTimerRegistrationMismatch`、`WaiterFailed` を持つ。
- `execute_native_window_host_loop_timer_fire_wait_with_waiter` は `FrameIntervalTimerRegistered` の場合だけ waiter を 1 回呼ぶ。already-paced outcome は waiter を呼ばず unsupported とする。
- fired raw id `0` と registered id に一致しない fired raw id は success にしない。
- source policy は registration backend と fire backend を分離し、registration backend が `FramePresentAlreadyPaced` を含まないこと、fire backend が already-paced outcome を success にしないことを検査する。

非目標:

- OS 固有 timer API、selector wakeup ownership、minifb wait hook 接続は含めない。
- thread sleep、busy loop、message pump、event queue wait を timer fire backend で代替しない。
- scheduler resume policy、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- fallback、silent no-op、mismatched id の成功扱いは含めない。

subagent review:

- Darwin the 2nd に F5gy 計画を渡し、registration success と fire success を分けること、timer id mismatch を fail closed にすること、source policy の観点を確認させた。結果は `PLAN_APPROVED` である。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gz: Native window host-loop timer wakeup executor boundary

Phase F5gz では、F5gx の registration wait helper と F5gy の fire wait helper を順に呼ぶ backend wakeup executor を追加する。これは real OS timer API ではなく、scheduler resume policy が後で消費できる registration-to-fire boundary である。

実装:

- `NativeWindowHostLoopTimerWakeError<RegistrarError, FireWaiterError>` を追加し、`RegistrationFailed NativeWindowHostLoopTimerRegistrationError` と `FireFailed NativeWindowHostLoopTimerFireError` を分ける。
- `execute_native_window_host_loop_timer_wakeup_with_backend` を追加し、`NativeWindowHostLoopWaitInstruction`、`NativeWindowHostLoopTimerRegistrar`、`NativeWindowHostLoopTimerFireWaiter` を受け取る。
- executor は先に `execute_native_window_host_loop_timer_registration_wait_with_registrar` を呼ぶ。registration error は `RegistrationFailed` として返し、waiter を呼ばない。
- registration outcome が得られた場合だけ `execute_native_window_host_loop_timer_fire_wait_with_waiter` を呼ぶ。fire error は `FireFailed` として返す。
- success は registered id と fired id が完全一致した `NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired` のみである。
- source policy は wakeup backend slice が direct `register_timer_nanos` / `wait_for_timer_fire`、minifb、queue、thread sleep、`Duration`、DOM / Canvas、fallback、silent no-op を含まないことを検査する。

非目標:

- OS 固有 timer API、selector wakeup ownership、minifb wait hook 接続は含めない。
- scheduler resume policy、real scheduler driver、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。
- message pump、event queue wait、thread sleep、busy loop を timer wakeup executor 内で代替しない。

subagent review:

- Darwin the 2nd に F5gz 計画を渡し、registration/fire の error 分離と helper 合成に限定すること、source policy の観点を確認させた。結果は `PLAN_APPROVED` である。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5ha: Native window host-loop scheduler timer resume gate boundary

Phase F5ha では、scheduler long runner が `Waited` result を常に再開可能として扱う問題を修正する。`FrameIntervalTimerRegistered` は timer fire を待つための registration evidence であり、scheduler resume evidence ではない。

実装:

- `NativeWindowHostLoopSchedulerResumeReady` を追加し、`HostEventPumped`、`FramePresentPaced`、`FrameIntervalTimerFired` を分ける。
- `NativeWindowHostLoopSchedulerResumeState` を追加し、`Ready` と `WaitingForFrameIntervalTimer` を分ける。
- `native_window_host_loop_scheduler_resume_state_from_wait_outcome` を追加し、already-paced outcome だけを ready にし、`FrameIntervalTimerRegistered` は waiting にする。
- `native_window_host_loop_scheduler_resume_ready_from_timer_fire` を追加し、F5gy/F5gz の `FrameIntervalTimerFired` evidence を ready へ写す。
- `NativeWindowHostLoopError::TimerFireResumeRequired` を追加する。
- `run_native_window_host_loop_with_policy_and_target_fps` は `Waited` outcome を resume gate に通し、`WaitingForFrameIntervalTimer` の場合は `TimerFireResumeRequired` を返して次の poll へ進まない。
- source policy は `Waited { .. }` の無条件継続を禁止し、resume gate 呼び出しと `TimerFireResumeRequired` path を検査する。

非目標:

- OS 固有 timer API、selector wakeup ownership、minifb wait hook 接続は含めない。
- timer fire を long runner 内で待たない。
- thread sleep、busy loop、message pump、event queue wait、fallback、silent no-op で timer fire を代替しない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5ha 計画を渡し、`Waited` 無条件継続を止めること、timer fire wait 実装へ踏み込まないこと、source policy の観点を確認させた。結果は `PLAN_APPROVED` である。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5hb: Native window host-loop std deadline timer adapter boundary

Phase F5hb では、native host-loop 用の host-owned deadline timer adapter を追加する。これは selector / message loop timer integration ではなく、F5gt の registration trait と F5gy の fire waiter trait を同じ adapter state で実装できることを固定する checkpoint である。

実装:

- `NativeWindowHostLoopDeadlineTimerRecord` を追加し、timer registration id と deadline nanos を保持する。
- `NativeWindowHostLoopDeadlineTimerAdapterError` を追加し、active timer overlap、missing active timer、timer id overflow、deadline overflow、clock failure、sleeper failure、mismatched fired id を enum で分ける。
- `NativeWindowHostLoopDeadlineTimerClock` と `NativeWindowHostLoopDeadlineTimerSleeper` を追加し、unit test は scripted clock / sleeper で deterministic に動かす。
- `NativeWindowHostLoopDeadlineTimerAdapter` を追加し、registration 成功時だけ active timer を作り、fire wait 成功時だけ active timer を消費する。
- `execute_native_window_host_loop_deadline_timer_wakeup_with_adapter` を追加し、F5gt/F5gy helper を同じ adapter state で順に呼ぶ。
- cfg-gated std implementation として `StdNativeWindowHostLoopDeadlineTimerClock` / `StdNativeWindowHostLoopDeadlineTimerSleeper` / `native_window_host_loop_std_deadline_timer_adapter` を追加する。
- source policy は std sleep / Duration を deadline timer adapter slice 内だけ許可し、minifb / presenter / scheduler / event pump への混入を禁止する。

非目標:

- minifb wait hook へ接続しない。`Window::set_target_fps` が smoke backend の pacing authority である間、frame interval wait は `FramePresentAlreadyPaced` のまま扱う。
- selector wakeup ownership、OS message loop timer、queue integration は含めない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- 初回 plan review は `CHANGES_REQUESTED`。minifb の double pacing を避けること、slice 名を std deadline timer adapter に絞ること、overlap / missing / mismatch / id overflow / deadline overflow を typed error にすること、unit test を実 sleep に依存させないことが要求された。
- 実装はこの指摘に合わせ、minifb connection と selector ownership を後続 residual に残す。

## Phase F5hc: Native window host-loop timer fired wait outcome boundary

Phase F5hc では、F5gz/F5hb の timer fire evidence を host-loop wait outcome として運ぶ boundary を追加する。`FrameIntervalTimerRegistered` は timer reservation / pending evidence であり、scheduler resume completion ではない。`FrameIntervalTimerFired` だけを wait outcome の ready evidence として扱う。

実装:

- `NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired` を追加し、`presentation`、`window_size`、`size_changed`、`wait_nanos`、`timer_registration_id` を保持する。
- `native_window_host_loop_wait_outcome_from_timer_fire` を追加し、`NativeWindowHostLoopTimerFireOutcome` を wait outcome へ写す。
- `execute_native_window_host_loop_timer_wakeup_wait_with_backend` を追加し、registration / fire error の段階を保ったまま、成功時だけ fired wait outcome を返す。
- `execute_native_window_host_loop_deadline_timer_wakeup_wait_with_adapter` を追加し、F5hb adapter の成功 path を fired wait outcome へ写す。
- `native_window_host_loop_scheduler_resume_state_from_wait_outcome` は `FrameIntervalTimerRegistered` を `WaitingForFrameIntervalTimer` のまま保ち、`FrameIntervalTimerFired` だけを `Ready(FrameIntervalTimerFired)` へ写す。
- source policy は registration-only outcome が `TimerFireResumeRequired` に進むこと、fired outcome が resume ready になること、minifb pacing path が変更されないことを検査する。

非目標:

- selector / message loop timer ownership、OS 固有 timer API、minifb wait hook への接続、`Window::set_target_fps` の置換は含めない。
- registration-only outcome を ready として扱わない。
- fallback、silent no-op、thread sleep、busy loop、message pump、event queue wait で timer fire を代替しない。

subagent review:

- Darwin the 2nd に F5hc 計画を渡し、F5hb 後の root-cause slice として承認された。実装条件は、registered は waiting のままにすること、fired だけを ready にすること、F5gz/F5hb の error separation を潰さないこと、minifb を変更しないこと、source policy で両 path を検査することである。

## Phase F5hd: Native window host-loop wait owner boundary

Phase F5hd では、host-loop wait instruction を host event queue wait backend と frame interval deadline timer backend へ分配する owner boundary を追加する。これは OS selector / message-loop timer 実装ではなく、F5gu/F5gv/F5gw の host event path と F5hb/F5hc の timer path を 1 つの composition point で選ぶための root-cause slice である。

実装:

- `NativeWindowHostLoopWaitOwner` を追加し、event queue waiter と `NativeWindowHostLoopDeadlineTimerAdapter` を所有させる。
- `NativeWindowHostLoopWaitOwnerError` を追加し、`EventQueueWaitFailed NativeWindowHostLoopEventQueueWaitError` と `FrameIntervalTimerWakeFailed NativeWindowHostLoopDeadlineTimerWakeError` を分ける。
- `execute_native_window_host_loop_wait_with_owner` を追加し、`WaitForHostEvent` は event queue wait helper だけへ、`WaitForFrameInterval` は deadline timer wakeup wait helper だけへ渡す。
- event queue success は `HostEventPumpAlreadyPaced` に正規化し、timer success は `FrameIntervalTimerFired` として返す。
- unit test は host event path が timer clock / sleeper を呼ばないこと、frame interval path が event queue waiter を呼ばないこと、event queue error と timer wake error が別 variant で保持されることを検査する。
- source policy は wait owner slice に minifb、window update、presenter、scheduler action、DOM / Canvas / video memory、fallback、silent no-op が混入しないことを固定する。

非目標:

- selector ownership、OS message loop timer、minifb wait hook の pacing 置換、`Window::set_target_fps` の置換は含めない。
- event queue backend が timer を代替すること、timer backend が queue wait を代替すること、thread sleep / busy loop / fallback / silent no-op は禁止する。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5hd 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、lower error 全体を wrapper に保持すること、host event と frame interval の branch exclusivity を test すること、minifb を変更しないことである。

## Phase F5he: Native minifb frame pacing authority boundary

Phase F5he では、minifb smoke backend の existing `Window::set_target_fps` pacing を typed authority として切り出す。F5hd wait owner と std deadline timer adapter は formal native wait backend であり、この phase では minifb wait hook へ接続しない。

実装:

- `NativeWindowMinifbFramePacingAuthority` を追加し、validated `NativeWindowTargetFps` を保持する。
- `NativeWindowMinifbFramePacingAuthorityError` を追加し、target fps mismatch と wait nanos mismatch を enum で分ける。
- authority helper は `WaitForFrameInterval` instruction の `frame_interval.target_fps` と `wait_nanos` を検査し、一致する場合だけ `FramePresentAlreadyPaced` を返す。
- `MinifbNativeWindowRunLoopHost` は authority を保持し、frame interval wait を authority helper へ委譲する。
- `run_minifb_window_loop` は authority から得た `target_fps_usize` だけを `window.set_target_fps` へ渡す。
- unit test は matching frame interval、remainder carry wait nanos、target fps mismatch、wait nanos mismatch を検査する。
- source policy は `FramePresentAlreadyPaced` が minifb wait method の inline return ではなく authority helper 経由であること、minifb path が F5hd wait owner / std deadline timer adapter をまだ使わないこと、`set_target_fps 0` を使わないことを固定する。

非目標:

- `Window::set_target_fps` を置換しない。`set_target_fps 0` は host event wait path を tight loop にするため禁止する。
- selector ownership、OS message-loop timer、formal deadline timer owner の minifb hook 接続は含めない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- 初回 plan review は `CHANGES_REQUESTED`。minifb internal wait を無効化する前に host-event blocking / selector boundary が必要であり、`set_target_fps 0` はこの slice では不適切と指摘された。
- 改訂 plan は `PLAN_APPROVED`。required constraints は、authority が raw fps ではなく `NativeWindowTargetFps` を保持すること、`FramePresentAlreadyPaced` は authority helper だけから返すこと、docs/source policy が already-paced の意味を minifb internal pacing evidence として明記すること、deadline timer owner を minifb path へ接続しないことである。

## Phase F5hf: Native frame interval wait authority mode boundary

Phase F5hf では、frame interval wait の authority を `NativeWindowFrameIntervalWaitAuthorityMode` として typed data にする。これは selector / message-loop timer ownership の実装ではなく、minifb internal target-fps pacing と host-owned deadline timer の二重 authority を先に拒否する safety boundary である。

実装:

- `NativeWindowFrameIntervalWaitAuthorityMode` を追加し、`MinifbInternalTargetFps target_fps` と `HostOwnedDeadlineTimer` を分ける。
- `NativeWindowFrameIntervalWaitAuthorityModeError` を追加し、authority conflict と target FPS mismatch を enum で分ける。
- `combine_native_window_frame_interval_wait_authority_mode` は同じ minifb target fps 同士と host-owned deadline timer 同士だけを受け入れる。
- minifb mode と host-owned mode の組み合わせは順序に関係なく `ConflictingFrameIntervalAuthorities` として拒否する。target fps が異なる minifb mode 同士も同じ conflict として扱う。
- `validate_native_window_frame_interval_wait_authority_mode` は minifb mode の場合だけ instruction の `NativeWindowFrameIntervalRequest.target_fps` と authority target を照合する。
- host-owned deadline timer mode の validation は、すでに計画された frame interval instruction と authority mode が矛盾しないことだけを表し、`FramePresentAlreadyPaced`、`FrameIntervalTimerRegistered`、`FrameIntervalTimerFired` を生成しない。
- `NativeWindowMinifbFramePacingAuthority` は自分の authority mode を返し、`FramePresentAlreadyPaced` を返す前に新しい validation helper を通る。
- unit test は same minifb target の合成、minifb と host-owned の双方向 conflict、minifb target mismatch、minifb validation mismatch、host-owned validation no-evidence を検査する。
- source policy は mode helper が wait outcome / timer evidence / `set_target_fps` / deadline timer adapter / OS wait を持たないことと、minifb authority path が helper を実際に使うことを固定する。

非目標:

- selector ownership、OS message-loop timer、F5hd wait owner の minifb hook 接続、`Window::set_target_fps` の置換は含めない。
- host-owned deadline timer mode は compatibility marker であり、この phase では timer owner の実行 path ではない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5hf 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、mode type を pure / non-platform-specific にすること、host-owned mode validation が wait evidence を生成しないこと、minifb authority path が新 helper を実際に使うこと、minifb と host-owned deadline timer の conflict を双方向で拒否すること、docs が selector/message-loop 実装ではないと明記することである。

## Phase F5hg: Native wait owner frame interval authority connection boundary

Phase F5hg では、F5hf の `NativeWindowFrameIntervalWaitAuthorityMode` を F5hd の `NativeWindowHostLoopWaitOwner` に接続する。これは実 OS selector / message-loop timer backend ではなく、formal owner path が frame interval wait を `HostOwnedDeadlineTimer` authority として扱うことを明示する ownership safety slice である。

実装:

- `NativeWindowHostLoopWaitOwner::frame_interval_wait_authority_mode` を追加し、`HostOwnedDeadlineTimer` を返す。
- `NativeWindowHostLoopWaitOwnerError::FrameIntervalAuthorityFailed` を追加し、F5hf の authority conflict / validation failure を lower error と分けて保持する。
- `execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode` を追加し、明示的な requested authority mode を受け取れるようにする。
- frame interval branch は `combine_native_window_frame_interval_wait_authority_mode` と `validate_native_window_frame_interval_wait_authority_mode` を通ってから、deadline timer wakeup helper を呼ぶ。
- existing `execute_native_window_host_loop_wait_with_owner` は owner の `HostOwnedDeadlineTimer` authority を渡す wrapper とする。
- unit test は owner の authority mode、host event wait が frame authority を参照しないこと、host-owned frame interval path が deadline timer へ進むこと、minifb authority が渡された frame interval path は timer registration / clock read / sleeper call / active timer mutation 前に失敗することを検査する。
- source policy は owner helper が F5hf helper を timer wakeup より前に呼ぶこと、minifb path は wait owner / deadline timer adapter を呼ばないことを固定する。

非目標:

- macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd は実装しない。
- minifb wait hook を F5hd wait owner や std deadline timer adapter へ接続しない。
- `Window::set_target_fps` の置換、`set_target_fps 0`、fallback、silent no-op、busy loop は導入しない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5hg 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、authority validation が deadline timer registration / clock read / sleeper call / active timer mutation より前であること、minifb authority rejection の no-mutation test、host event wait の authority 非依存、既存 owner helper の wrapper 化、minifb path 非変更である。

## Phase F5hh: Native run-loop frame interval wait backend selection boundary

Phase F5hh では、native run-loop config に frame interval wait backend selection を追加する。これは実 OS selector / message-loop timer backend ではなく、future backend selection が minifb smoke backend へ暗黙 fallback しないための fail-closed boundary である。

実装:

- `NativeWindowRunLoopFrameIntervalWaitBackend` を追加し、`MinifbInternalTargetFps` と `HostOwnedDeadlineTimer` を持たせる。
- `NativeWindowRunLoopConfig` に `frame_interval_wait_backend` を追加し、existing constructors は default `MinifbInternalTargetFps` を設定する。
- explicit constructor は target fps、host-loop policy、frame interval backend をまとめて指定できる。
- backend は `NativeWindowFrameIntervalWaitAuthorityMode` へ変換できる。validation は F5hf の `combine_native_window_frame_interval_wait_authority_mode` を使い、authority conflict を typed reason として保持する。
- `run_minifb_window_loop` は `validate_minifb_window_run_loop_frame_interval_wait_backend` を window 作成前に呼ぶ。`HostOwnedDeadlineTimer` が指定された場合は `NativeWindowRunLoopError::FrameIntervalWaitBackendUnsupported` として返す。
- unit test は default config、explicit backend selection、backend-to-authority mapping、minifb acceptance、minifb rejection を検査する。
- source policy は validation が backend-loop initialization / minifb window creation / `set_target_fps` より前にあること、minifb path が wait owner / deadline timer adapter / std deadline timer helper を呼ばないことを固定する。

非目標:

- macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd は実装しない。
- minifb wait hook を F5hd wait owner / std deadline timer adapter へ接続しない。
- `Window::set_target_fps` の置換、`set_target_fps 0`、fallback、silent no-op、busy loop は導入しない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5hh 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、config backend を typed enum にすること、validation が F5hf authority type を再利用すること、default が minifb internal pacing であること、minifb runner は `HostOwnedDeadlineTimer` を side effect 前に拒否すること、fallback を禁止すること、source policy で ordering と deadline-timer 禁止を固定することである。

## Phase F5hi: Native host-owned deadline wait run-loop host wrapper boundary

Phase F5hi では、formal wait owner を native run-loop host contract に接続できる wrapper を追加する。F5hg までで host event wait と frame interval timer wait の owner は独立していたため、この phase では future native OS backend / deterministic test backend が同じ `NativeWindowRunLoopHost` interface から owner wait を使えるようにする。

実装:

- `NativeWindowHostOwnedDeadlineWaitRunLoopHost` を追加し、inner `NativeWindowRunLoopHost` と `NativeWindowHostLoopWaitOwner` を所有する。
- `poll_event_snapshot`、`set_window_title`、`pump_events_only`、`present_frame` は inner host へ委譲する。
- `wait_after_budget_exhausted` は inner host の wait hook を呼ばず、`execute_native_window_host_loop_wait_with_owner` だけを呼ぶ。
- associated type は `EventError = Host::EventError`、`PresentError = Host::PresentError`、`WaitError = NativeWindowHostLoopWaitOwnerError<...>` とし、error を文字列化しない。
- unit test は non-wait operation の委譲、host-event wait が event queue waiter だけを使うこと、frame-interval wait が deadline timer だけを使うこと、owner wait error が typed enum のまま返ることを検査する。
- source policy は wrapper と minifb smoke path を分け、minifb runner / host adapter / wait method が wrapper、wait owner、deadline timer adapter、std deadline helper を参照しないことを固定する。

非目標:

- minifb runner を host-owned wait owner へ接続しない。
- macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd は実装しない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5hi 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、wait hook が owner helper だけを呼ぶこと、inner host wait hook を呼ばない test を入れること、non-wait operation は inner host に正確に委譲すること、owner wait error を typed error のまま保持すること、host-event wait と frame-interval wait の dispatch 先を分けて検査すること、minifb path へ wrapper を混ぜないことである。

## Phase F5hj: Native interruptible deadline wait boundary

Phase F5hj では、real selector / message-loop timer backend の前段として、host event readiness と frame deadline のどちらでも wake できる interruptible wait boundary を追加する。これは actual OS selector / message-loop/timerfd/waitable-timer implementation ではなく、timer-only deadline wait の semantic gap を先に閉じる checkpoint である。

実装:

- `NativeWindowHostLoopInterruptibleDeadlineWake` を追加し、`HostEventReady` と `DeadlineReached` を持たせる。
- `NativeWindowHostLoopInterruptibleDeadlineWaiter` を追加し、host event wait と deadline-or-host-event wait を別 method にする。
- `NativeWindowHostLoopInterruptibleDeadlineWaitAdapter` を追加し、clock、waiter、positive timer id cursor を所有させる。
- `execute_native_window_host_loop_interruptible_deadline_wait_with_adapter` は `NativeWindowHostLoopWaitInstruction` を受け、`NativeWindowHostLoopWaitOutcome` を返す。
- `WaitForHostEvent` は host event waiter だけを呼び、`HostEventPumpAlreadyPaced` に写す。
- `WaitForFrameInterval` は wait nanos を clock/id/waiter side effect より前に検査し、checked timer id、checked deadline arithmetic、interruptible waiter call を順に実行する。
- deadline 到達時だけ `FrameIntervalTimerFired` を返す。host event readiness で wake した場合は `HostEventPumpAlreadyPaced` を返し、timer fired evidence を生成しない。
- candidate timer id は wait 開始前に advance するため、host event wake や frame wait failure の場合も id reuse はしない。
- unit test は host-event-only wait、deadline reached、host event interrupt、wait-nanos mismatch no-side-effect、host-event error、clock error、deadline overflow、id overflow、frame wait error を検査する。
- source policy は new boundary が minifb / DOM / Canvas / video memory / fallback / silent no-op を含まないこと、minifb runner / adapter / wait method が interruptible adapter を参照しないことを固定する。

非目標:

- minifb runner を interruptible deadline wait adapter へ接続しない。
- timer-only deadline wait、thread sleep、busy loop、minifb internal pacing、synthetic fired evidence へ fallback しない。
- macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd は実装しない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5hj 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、explicit executor/helper が `NativeWindowHostLoopWaitOutcome` を返すこと、wait-nanos validation を side effect より前に行うこと、host event wake を timer fired と偽装しないこと、distinct error stages を保持すること、fallback / thread sleep / busy loop / minifb internal pacing / synthetic fired evidence を禁止すること、docs が OS-specific implementation ではなく semantic interruptible wake boundary だと明記することである。

## Phase F5hk: Native interruptible deadline wait run-loop host wrapper boundary

Phase F5hk では、F5hj の interruptible deadline wait adapter を native run-loop host contract に接続できる wrapper を追加する。F5hj では semantic wait adapter だけを固定したが、future native OS backend / deterministic test backend が `NativeWindowRunLoopHost` interface から同じ wait semantics を使う接続点はまだ無い。F5hk はこの root boundary を直す。

実装:

- `NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost` を追加し、inner `NativeWindowRunLoopHost` と `NativeWindowHostLoopInterruptibleDeadlineWaitAdapter` を所有する。
- `poll_event_snapshot`、`set_window_title`、`pump_events_only`、`present_frame` は inner host へ委譲する。
- `wait_after_budget_exhausted` は inner host の wait hook を呼ばず、`execute_native_window_host_loop_interruptible_deadline_wait_with_adapter` だけを呼ぶ。
- associated type は `EventError = Host::EventError`、`PresentError = Host::PresentError`、`WaitError = NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError<...>` とし、error を文字列化しない。
- unit test は non-wait operation の委譲、inner wait hook が呼ばれないこと、deadline reached が timer fired evidence になること、host event ready が timer fired evidence にならないこと、adapter wait error が typed enum のまま返ることを検査する。
- source policy は wrapper と minifb smoke path を分け、minifb runner / host adapter / wait method が wrapper、interruptible adapter、owner wait、deadline timer adapter、std deadline helper を参照しないことを固定する。

非目標:

- minifb runner を interruptible deadline wait wrapper へ接続しない。
- macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd は実装しない。
- thread sleep、busy loop、minifb internal pacing、timer-only wait、synthetic fired evidence への fallback は導入しない。
- FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5hk 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、F5hi を変更せず別 wrapper として追加すること、wait hook が interruptible helper だけを呼ぶこと、inner wait hook を呼ばないことを test すること、deadline reached / host event ready の evidence を区別すること、typed wait error を保持すること、minifb path に新 wrapper を混ぜないことである。

## Phase F5hl: Native platform wait backend selection boundary

Phase F5hl では、F5hk の interruptible wait wrapper を real native backend へ接続する前段として、現在 platform と platform-specific wait backend の対応を typed enum と `Result` で固定する。macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd はそれぞれ別 backend kind として表し、runtime string、environment probe、fallback、silent no-op で backend を選ばない。

実装:

- `NativeWindowHostLoopPlatformKind` を追加し、`Macos`、`Windows`、`Linux`、`Unsupported` を持たせる。
- `NativeWindowHostLoopPlatformWaitBackendKind` を追加し、`MacosRunLoopTimer`、`WindowsWaitableTimerMessageWait`、`LinuxSelectorTimerFd`、`HeadlessScripted` を持たせる。
- `NativeWindowHostLoopPlatformWaitBackendSupportError` は default unsupported platform、requested unsupported platform、platform/backend mismatch を分け、current platform と requested backend を typed data として保持する。
- `native_window_host_loop_current_platform_kind` は `cfg(target_os = ...)` だけで current platform を決める。
- `validate_native_window_host_loop_platform_wait_backend_kind_for_platform` は current platform と requested backend の一致だけを success とし、`HeadlessScripted` を native platform fallback として成功させない。
- `native_window_host_loop_default_platform_wait_backend_kind_for_platform` は macOS、Windows、Linux の default backend を返し、unsupported platform では error を返す。default は `HeadlessScripted` を返さない。
- unit test は matching backend、全 real platform mismatch、unsupported platform rejection、default mapping、default headless fallback 禁止、cfg current platform sanity を検査する。
- source policy は typed enum / `cfg` / `Result` 境界を要求し、`std::env`、runtime OS string、format/stringify、fallback、silent no-op、minifb 接続を拒否する。

非目標:

- macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd の actual OS API は実装しない。
- `HeadlessScripted` は headless/test backend 用の future kind であり、native platform default や native platform validation の fallback にしない。
- minifb runner、minifb host adapter、frame pacing authority には接続しない。
- thread sleep、busy loop、timer-only wait、synthetic fired evidence、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5hl 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、`HeadlessScripted` を default / native fallback にしないこと、standard spec も同期すること、source policy が typed enum と `cfg` selection を固定し runtime string / env probing / fallback / silent no-op を拒否すること、全 mismatch pair と unsupported platform を test することである。

## Phase F5jb: Native Linux X11 setup and event observation boundary

Phase F5jb では、F5ja で保持した acquired fd owner を actual X11 decoder へ進める最小 boundary として、X11 setup request write、setup response prefix / body drain、32 byte event packet read、normalized observation decode を追加する。Wayland は registry / surface / xdg-shell まで広がるため、この phase では対象外にする。

実装:

- `NativeWindowLinuxX11EventSourceRawApi` を追加し、raw byte write / read / last error code / would-block 判定を trait-injected にする。
- cfg Linux `NativeWindowLinuxX11EventSourceSysApi` は `send` / `recv MSG_DONTWAIT` と errno 分類の薄い wrapper にする。
- `NativeWindowLinuxX11EventSourceObservationReader` は setup request write progress、setup prefix progress、setup body remaining、event packet progress を保持する。
- `WouldBlock` は retryable typed error として返すが、partial bytes は reader state に保持し、次回 poll で続きを読む。
- setup rejected / auth required / invalid status / EOF / raw failure / unsupported event type は enum error にする。
- `ConfigureNotify` は current size observation、`MotionNotify` / `ButtonPress` / `ButtonRelease` は pointer / mouse observation へ写す。
- `NativeWindowLinuxX11WindowEventSourceObservationProvider` は descriptor provider owner と X11 reader を同じ owner に保持し、F5ix / F5iy の observation provider contract へ接続できる形にする。ただし provider mutable escape、reader mutable escape、consuming `into_parts` は公開しない。
- Rust unit tests、source policy、GUI spec、native platform behavior、`todo.md`、`note.n.md` を F5jb contract へ更新する。

非目標:

- Linux support gate の `Ok` 化、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop` は行わない。
- Wayland decoding、X11 authorization file lookup、window creation、event mask selection、WM_DELETE_WINDOW / ClientMessage、keyboard / IME は扱わない。
- minifb fallback、ObservedInputOnly promotion、synthetic readiness、timer fired evidence、fallback snapshot、silent no-op は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_observation_provider_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で owner保持、typed error、partial byte retry、no fallback / no runner dispatch が承認される。

## Phase F5jc: Native Linux X11 authorization setup request boundary

Phase F5jc では、F5jb の X11 setup request を no-auth 固定から、authorization credential を持てる typed setup request builder へ進める。X11 protocol encoding は setup request に authorization protocol name / data の length、STRING8 payload、4 byte padding を持つため、この phase ではその byte encoding と validation を先に固定する。

実装:

- `NativeWindowLinuxX11AuthorizationCredential` は borrowed name / data slice を validation input としてだけ扱い、reader には保持しない。
- `NativeWindowLinuxX11SetupRequest` は encoded request bytes を private owner として保持し、read-only accessor だけを公開する。
- `native_window_linux_x11_setup_request_from_authorization` は name / data が `u16` length に収まること、4 byte padding、total length を checked arithmetic で検査してから owned request を返す。
- build failure は `NativeWindowLinuxX11SetupRequestBuildError` の enum とし、raw API owner を消費する前に返す。
- `NativeWindowLinuxX11EventSourceObservationReader` は validated setup request owner を受ける infallible constructor を持ち、partial setup write retry は reader-owned bytes に対して継続する。
- 既存の `native_window_linux_x11_setup_request_bytes` は exact 12 byte no-auth helper として残し、checked no-auth request と同じ bytes であることを test する。
- Rust unit tests、source policy、GUI spec、native platform behavior、`todo.md`、`note.n.md` を F5jc contract へ更新する。

非目標:

- `.Xauthority` file lookup、`XAUTHORITY` / `HOME` / env / fs / vfs access は扱わない。
- Linux support gate の `Ok` 化、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop` は行わない。
- X11 window creation、event mask selection、WM_DELETE_WINDOW / ClientMessage、keyboard / IME、Wayland decoding は扱わない。
- minifb fallback、ObservedInputOnly promotion、synthetic readiness、timer fired evidence、fallback snapshot、silent no-op は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で owner消費前 validation、typed error、request byte ownership、partial write retry、no fs/env / no fallback / no runner dispatch が承認される。

## Phase F5jd: Native Linux Xauthority record parser boundary

Phase F5jd では、F5jc の authorization setup request owner に渡す credential を、caller supplied bytes から zero-copy で選ぶ parser / selector boundary を追加する。ここでは file path、environment variable、home directory、VFS / filesystem は扱わず、すでに caller が持っている authority file bytes だけを入力にする。

実装:

- Xauthority record の family、address、display number、protocol name、protocol data を borrowed slice として表す。
- record parser は `u16` の MSB-first length field を順に読み、offset advance は checked arithmetic にする。
- parse failure は length field truncation、payload truncation、offset overflow を enum error として返す。
- selector は caller supplied `family + address + display_number` と exact match する。`FamilyLocal` の hostname 推測や `FamilyWild` の暗黙 fallback は行わない。
- `preferred_protocol_name` が `Some` の場合だけ protocol name を追加条件にし、`None` では最初の exact family/address/display match を選ぶ。
- selection result は `Selected credential` / `NoMatchingRecord` の typed enum とし、no-auth fallback policy は持たせない。
- selected credential は record bytes から borrowed name / data を返し、F5jc setup request builder に copy なしで渡せる。
- Rust unit tests、source policy、GUI spec、native platform behavior、`todo.md`、`note.n.md` を F5jd contract へ更新する。

非目標:

- `.Xauthority` file lookup、`XAUTHORITY` / `HOME` / env / fs / vfs access、file lock handling、path resolution は扱わない。
- Hostname、Unix socket peer identity、TCP/IP address、SSH forwarding display policy は扱わない。これらは exact selector criteria を作る後続 boundary とする。
- Linux support gate の `Ok` 化、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop` は行わない。
- X11 window creation、event mask selection、WM_DELETE_WINDOW / ClientMessage、keyboard / IME、Wayland decoding は扱わない。
- minifb fallback、ObservedInputOnly promotion、synthetic readiness、timer fired evidence、fallback snapshot、silent no-op は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で zero-copy parse、exact selector、typed selection、no fs/env / no fallback / no runner dispatch が承認される。

## Phase F5je: Native Linux Xauthority selector criteria boundary

Phase F5je では、F5jd の `NativeWindowLinuxX11XauthoritySelector` に渡す exact selector criteria を、local X11 display name と caller supplied local authority address から作る純粋境界を追加する。ここでは `.Xauthority` lookup、`XAUTHORITY` / `HOME` / env / fs / vfs access、hostname / gethostname / TCP identity lookup は扱わない。

実装:

- X11 display name parse error を `NativeWindowLinuxX11DisplayNameError` として fd acquisition error から分離する。
- fd acquisition path は shared display parser の error を既存の `NativeWindowLinuxWindowEventSourceFdAcquisitionError` へ写像し、public behavior を変えない。
- accepted display form は `:N`、`:N.screen`、`unix/:N`、`unix/:N.screen` に限定する。
- display number parse は allocation なしの checked decimal accumulation とし、overflow や malformed screen suffix は typed error にする。
- selector criteria owner は criteria-owned display number bytes を fixed `[u8; 10]` に保持し、record selector はその owner を借りる。
- criteria は caller supplied local authority address bytes と optional preferred protocol name をそのまま使い、address synthesis や hostname inference を行わない。
- Rust unit tests、source policy、GUI spec、native platform behavior、`todo.md`、`note.n.md` を F5je contract へ更新する。

非目標:

- `.Xauthority` file lookup、`XAUTHORITY` / `HOME` / env / fs / vfs access、path resolution、file lock handling は扱わない。
- Hostname / `gethostname`、Unix socket peer identity、TCP/IP address、SSH forwarding display policy は扱わない。
- `FamilyLocal` の broad match、`FamilyWild` fallback、no-auth fallback、silent no-op、synthetic readiness は作らない。
- Linux support gate の `Ok` 化、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop` は行わない。
- X11 window creation、event mask selection、WM_DELETE_WINDOW / ClientMessage、keyboard / IME、Wayland decoding は扱わない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で pure display parser、criteria-owned display bytes、caller supplied address only、no fs/env / no fallback / no runner dispatch が承認される。

## Phase F5jf: Native Linux Xauthority lookup path request boundary

Phase F5jf では、Xauthority bytes を読む前段として、caller supplied authority file path と caller supplied home directory path から「どの path を要求すべきか」を決める pure path request boundary を追加する。これは path selection contract であり、environment variable acquisition、filesystem / VFS read、file locking、credential selection integration は扱わない。

実装:

- `NativeWindowLinuxX11XauthorityLookupInput` は `authority_file_path: Option str` と `home_directory_path: Option str` を受け取る。ここでは `std::env` や raw environment API を呼ばない。
- `NativeWindowLinuxX11XauthorityPathSource` は `ExplicitAuthorityFile` と `HomeDirectoryDefault` を分ける。
- `NativeWindowLinuxX11XauthorityPathPlan` は source と owned path string を保持する。
- `authority_file_path = Some nonempty` は byte-for-byte に preserving し、home directory より優先する。
- `authority_file_path = Some empty` は `EmptyAuthorityFilePath` として fail closed にし、home directory へ落とさない。
- `authority_file_path = None` かつ `home_directory_path = Some nonempty` の場合だけ、HOME default source として default file name を結合する。
- HOME が `/` で終わる場合は `.Xauthority`、終わらない場合は `/.Xauthority` を append し、normalize / canonicalize / tilde expansion はしない。
- NUL を含む path は Unix path conversion の前段で typed error にする。
- suffix append は checked length にし、overflow は `PathLengthOverflow` とする。
- Rust unit tests、source policy、GUI spec、native platform behavior、`todo.md`、`note.n.md` を F5jf contract へ更新する。

非目標:

- `XAUTHORITY` / `HOME` の env acquisition、`std::env`、filesystem / VFS open / read、metadata / exists / canonicalize、file locking は扱わない。
- Xauthority bytes parse / credential selection / setup request integration は扱わない。
- Hostname / `gethostname`、Unix socket peer identity、TCP/IP address、SSH forwarding display policy は扱わない。
- Linux support gate の `Ok` 化、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop` は行わない。
- X11 window creation、event mask selection、WM_DELETE_WINDOW / ClientMessage、keyboard / IME、Wayland decoding は扱わない。
- fallback、silent no-op、synthetic readiness は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で pure path plan、explicit source enum、no env/fs/VFS / no fallback / no runner dispatch が承認される。

## Phase F5jg: Native Linux Xauthority file bytes acquisition boundary

Phase F5jg では、F5jf の `NativeWindowLinuxX11XauthorityPathPlan` から Xauthority file bytes を取得する境界を追加する。ここでは実 filesystem / VFS adapter を実装せず、trait-injected reader へ exact plan path を渡し、返された bytes を size / nonempty validation 済み owner に閉じ込める。

実装:

- `NativeWindowLinuxX11XauthorityFileBytesReader` は `read_xauthority_file_bytes path` だけを持つ injected reader trait とする。
- `NativeWindowLinuxX11XauthorityFileBytes` は private `Vec u8` owner とし、read-only `as_bytes` / `len` だけを公開する。success authority として raw `Vec u8` は返さない。
- `NativeWindowLinuxX11XauthorityFileBytesReadError` は `EmptyFile`、`FileTooLarge`、`ReadFailed` を分け、source と path を保持する。
- public helper は `plan.path` をそのまま reader に渡し、source reinterpretation、alternate path synthesis、home fallback、no-auth fallback を行わない。
- nonempty / max byte length を検査してから owner を返す。max byte length は bounded resource policy として定数を持ち、test 用には explicit limit helper も用意する。
- Rust unit tests、source policy、GUI spec、native platform behavior、`todo.md`、`note.n.md` を F5jg contract へ更新する。

非目標:

- `XAUTHORITY` / `HOME` の env acquisition、`std::env`、`std::fs`、`File`、`OpenOptions`、`read_to*`、metadata / exists / canonicalize、file locking、VFS adapter 実装は扱わない。
- Xauthority record parse / credential selection / setup request integration は helper 内に入れない。test で owner bytes が F5jd parser 入力として使えることを確認するに留める。
- Hostname / `gethostname`、Unix socket peer identity、TCP/IP address、SSH forwarding display policy は扱わない。
- Linux support gate の `Ok` 化、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop` は行わない。
- X11 window creation、event mask selection、WM_DELETE_WINDOW / ClientMessage、keyboard / IME、Wayland decoding は扱わない。
- fallback、silent no-op、synthetic readiness は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で exact plan path use、typed nonempty / size-checked byte owner、no env/fs/VFS / no credential/setup coupling / no fallback / no runner dispatch が承認される。

## Phase F5jh: Native Linux Xauthority environment acquisition boundary

Phase F5jh では、F5jf の caller supplied path request の前段として、platform environment 由来の authority-file path variable と home-directory path variable を取得する trait-injected boundary を追加する。これは environment acquisition contract であり、actual `std::env` adapter、filesystem / VFS read、credential selection、setup request integration は扱わない。

実装:

- `NativeWindowLinuxX11XauthorityEnvironmentValueKind` は authority-file path variable と home-directory path variable を型付き enum として表す。raw variable name string は public helper の引数にしない。
- `NativeWindowLinuxX11XauthorityEnvironmentReader` は variable kind を受け取り、`Result Option String Error` 相当を返す injected reader trait とする。
- public helper は authority-file path variable を先に読む。`Some value` が返った場合は home-directory path variable を読まず、F5jf path plan に `Some value, None` を渡す。
- authority-file path variable が missing の場合だけ home-directory path variable を読み、F5jf path plan に `None, home` を渡す。
- authority-file path variable が present empty の場合は `EmptyAuthorityFilePath` として fail closed にし、home-directory path variable へ fallback しない。
- read failure は `EnvironmentReadFailed variable error` として保持し、F5jf path plan failure は `PathPlanFailed error` として保持する。
- success は既存 `NativeWindowLinuxX11XauthorityPathPlan` を返し、別の success evidence owner は作らない。
- Rust unit tests、source policy、GUI spec、native platform behavior、`todo.md`、`note.n.md` を F5jh contract へ更新する。

非目標:

- direct `std::env`、raw environment API、actual filesystem / VFS adapter、`std::fs`、`File`、`OpenOptions`、`read_to*`、metadata / exists / canonicalize、file locking は扱わない。
- Xauthority file bytes read、record parse、credential selection、setup request integration は扱わない。
- Hostname / `gethostname`、Unix socket peer identity、TCP/IP address、SSH forwarding display policy は扱わない。
- Linux support gate の `Ok` 化、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop` は行わない。
- fallback、silent no-op、synthetic readiness は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で environment acquisition が path plan boundary にだけ接続され、no direct env/fs/VFS / no file read / no credential selection / no fallback / no runner dispatch が承認される。

## Phase F5ji: Native Linux Xauthority process environment adapter boundary

Phase F5ji では、F5jh の injected environment reader に対する cfg Linux actual process environment adapter を追加する。これは process environment adapter contract であり、filesystem / VFS read、credential selection、setup request integration、runner / CLI dispatch は扱わない。

実装:

- `NativeWindowLinuxX11XauthorityProcessEnvironmentReader` は `NativeWindowLinuxX11XauthorityEnvironmentReader` を実装する cfg Linux reader とする。
- `AuthorityFilePath` は `XAUTHORITY`、`HomeDirectoryPath` は `HOME` へ固定 mapping する。mapping は runtime string input や fallback table にしない。
- per-variable read は `std::env::var` だけを呼び、`VarError::NotPresent` は `Ok None`、`VarError::NotUnicode` は `NotUnicode variable OsString` として typed error にする。
- `std::env::var(...).ok()` で NotUnicode を missing と同一視しない。
- priority、empty explicit path rejection、home default path creation は F5jh / F5jf helper に委譲し、adapter は path planning を実装しない。
- convenience helper は process environment reader を作り、F5jh `native_window_linux_x11_xauthority_path_plan_from_environment` を呼ぶだけにする。
- source policy は F5jh injected surface と actual process environment adapter surface を分け、actual adapter surface だけで `std::env::var` と `XAUTHORITY` / `HOME` を許可する。

非目標:

- actual filesystem / VFS adapter、`std::fs`、`File`、`OpenOptions`、`read_to*`、metadata / exists / canonicalize、file locking は扱わない。
- Xauthority file bytes read、record parse、credential selection、setup request integration は扱わない。
- Hostname / `gethostname`、Unix socket peer identity、TCP/IP address、SSH forwarding display policy は扱わない。
- Linux support gate の `Ok` 化、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop` は行わない。
- fallback、silent no-op、synthetic readiness は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `cargo check -p nepl-gui-native --target x86_64-unknown-linux-gnu`
- `git diff --check`
- subagent implementation review で actual env adapter が F5jh にだけ接続され、no fs/VFS / no file read / no credential selection / no fallback / no runner dispatch が承認される。

## Phase F5jj: Native Linux Xauthority filesystem file bytes adapter boundary

Phase F5jj では、F5jg の injected file bytes reader に対する cfg Linux actual filesystem adapter を追加する。これは exact path file bytes adapter contract であり、VFS、path normalization、credential selection、setup request integration、runner / CLI dispatch は扱わない。

実装:

- `NativeWindowLinuxX11XauthorityFilesystemFileBytesReader` は `NativeWindowLinuxX11XauthorityFileBytesReader` を実装する cfg Linux reader とする。
- adapter は F5jg から渡された exact `path` に対して `std::fs::read(path)` だけを行う。
- read failure は exact requested path と original `std::io::Error` を保持する `NativeWindowLinuxX11XauthorityFilesystemFileBytesReadError` にする。
- empty file / file too large validation は F5jg `native_window_linux_x11_xauthority_read_file_bytes` に委譲し、adapter 内で重複実装しない。
- convenience helper は filesystem reader を作り、F5jg `native_window_linux_x11_xauthority_read_file_bytes` を呼ぶだけにする。
- source policy は F5jg injected surface と actual filesystem adapter surface を分け、actual adapter surface だけで `std::fs::read(path)` を許可する。

非目標:

- VFS adapter、`File`、`OpenOptions`、`read_to*`、metadata / exists / canonicalize、file locking は扱わない。
- path normalization、home fallback、alternate path synthesis、no-auth fallback は扱わない。
- record parse、credential selection、setup request integration は扱わない。
- Linux support gate の `Ok` 化、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop` は行わない。
- fallback、silent no-op、synthetic readiness は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `cargo test -p nepl-gui-native --lib`
- `cargo check -p nepl-gui-native --target x86_64-unknown-linux-gnu`
- `cargo check -p nepl-gui-native --lib --tests --target x86_64-unknown-linux-gnu`
- `git diff --check`
- subagent implementation review で actual filesystem adapter が F5jg にだけ接続され、no VFS / no credential selection / no fallback / no runner dispatch が承認される。

## Phase F5jk: Native Linux Xauthority VFS file bytes adapter boundary

Phase F5jk では、F5jg の injected file bytes reader に対する VFS source adapter を追加する。これは host / test / future Web resource root が持つ virtual file bytes source を Xauthority file bytes reader contract へ接続する境界であり、actual filesystem、path normalization、credential selection、setup request integration、runner / CLI dispatch は扱わない。

実装:

- `NativeWindowLinuxX11XauthorityVfsFileBytesSource` は `read_xauthority_vfs_file_bytes path` だけを持つ injected virtual source trait とする。
- `NativeWindowLinuxX11XauthorityVfsFileBytesReader` は mutable VFS source を借用し、`NativeWindowLinuxX11XauthorityFileBytesReader` を実装する adapter とする。
- adapter は F5jg から渡された exact `path` を source にそのまま渡し、path normalization、alias lookup、alternate path synthesis は行わない。
- source failure は exact requested path と source error を保持する `NativeWindowLinuxX11XauthorityVfsFileBytesReadError` として返す。
- empty file / file too large validation は F5jg `native_window_linux_x11_xauthority_read_file_bytes` に委譲し、adapter 内で重複実装しない。
- convenience helper は caller supplied VFS source を借用して adapter を作り、F5jg helper を呼ぶだけにする。
- source policy は F5jg injected surface、F5jj filesystem adapter surface、F5jk VFS adapter surface を分け、F5jk surface では `std::fs` / `File` / `OpenOptions` / `read_to*` を禁止する。

非目標:

- actual Web VFS、native resource root、filesystem fallback は扱わない。
- `std::fs`、`File`、`OpenOptions`、`read_to*`、metadata / exists / canonicalize、file locking は扱わない。
- path normalization、home fallback、alternate path synthesis、no-auth fallback は扱わない。
- record parse、credential selection、setup request integration は扱わない。
- Linux support gate の `Ok` 化、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop` は行わない。
- fallback、silent no-op、synthetic readiness は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `cargo test -p nepl-gui-native --lib`
- `cargo check -p nepl-gui-native --target x86_64-unknown-linux-gnu`
- `cargo check -p nepl-gui-native --lib --tests --target x86_64-unknown-linux-gnu`
- `git diff --check`
- subagent implementation review で VFS adapter が F5jg にだけ接続され、exact path forwarding、typed source failure、no fs / no credential selection / no fallback / no runner dispatch が承認される。

## Phase F5jl: Native Linux Xauthority credential setup request boundary

Phase F5jl では、F5jg / F5jk までで得た `NativeWindowLinuxX11XauthorityFileBytes` と F5je の `NativeWindowLinuxX11XauthoritySelectorCriteria` から、Xauthority credential を exact selector で選び、F5jc の authorization setup request owner を作る。これは credential selection と setup request encoding の接続境界であり、environment / filesystem / VFS acquisition、raw fd / raw API owner、window creation、runner / CLI dispatch は扱わない。

実装:

- `NativeWindowLinuxX11XauthorityCredentialSetupRequestError` は `ParseFailed`、`NoMatchingCredential`、`SetupRequestBuildFailed` を持つ。
- `native_window_linux_x11_setup_request_from_xauthority` は `NativeWindowLinuxX11XauthorityFileBytes` と `NativeWindowLinuxX11XauthoritySelectorCriteria` だけを借用する。
- helper は `native_window_linux_x11_xauthority_select_credential file_bytes.as_bytes criteria.selector` を 1 回だけ呼ぶ。
- `Selected credential` の場合だけ `native_window_linux_x11_setup_request_from_authorization credential` を 1 回だけ呼び、encoded setup request owner を返す。
- `NoMatchingRecord` は no-auth fallback にせず、`NoMatchingCredential` として fail closed にする。
- parse failure と setup request build failure は lower error を string 化せず enum branch に保持する。
- source policy は F5jl surface に env / fs / VFS read、raw API、runner、window setup、`AuthorizationCredential::none`、`no_authorization`、fallback、silent no-op、synthetic readiness が混入しないことを固定する。

非目標:

- hostname / display identity acquisition は扱わない。
- `XAUTHORITY` / `HOME` env acquisition、filesystem / VFS file bytes read、path planning は扱わない。
- X11 raw fd / raw API owner、window creation、event mask、WM_DELETE_WINDOW、keyboard / IME、Wayland concrete decoding は扱わない。
- Linux support gate の `Ok` 化、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop` は行わない。
- no-auth fallback、fallback snapshot、silent no-op、synthetic readiness は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `cargo test -p nepl-gui-native --lib`
- `cargo check -p nepl-gui-native --target x86_64-unknown-linux-gnu`
- `cargo check -p nepl-gui-native --lib --tests --target x86_64-unknown-linux-gnu`
- `git diff --check`
- subagent implementation review で F5jl が one-call selection / one-call setup builder の接続に留まり、no-auth fallback、raw API、env / fs / VFS、runner dispatch が混入していないことが承認される。

## Phase F5jm: Native Linux Xauthority local authority address owner boundary

Phase F5jm では、Xauthority `FamilyLocal` record の address と照合する local authority address を typed owner として扱う境界を追加する。これは injected reader から得た caller / host supplied hostname-equivalent bytes を検査して owner 化し、既存の selector criteria helper へ渡すための境界である。actual hostname / process identity acquisition は扱わない。

実装:

- `NativeWindowLinuxX11LocalAuthorityAddress` は private `Vec<u8>` owner とし、`as_bytes` / `len` だけを公開する。
- `NativeWindowLinuxX11LocalAuthorityAddressReader` は `read_x11_local_authority_address` だけを持つ injected reader trait とする。
- `NativeWindowLinuxX11LocalAuthorityAddressReadError` は `ReadFailed`、`EmptyAddress`、`AddressTooLong`、`AddressContainsNul` を持つ。
- `native_window_linux_x11_local_authority_address_with_limit` は reader を 1 回だけ呼び、空 address、max byte length 超過、NUL byte を typed error として拒否する。
- `native_window_linux_x11_local_authority_address` は standard max byte length で helper を呼ぶ。
- `native_window_linux_x11_xauthority_local_selector_criteria_from_authority_address` は `NativeWindowLinuxX11LocalAuthorityAddress` を借用し、既存 `native_window_linux_x11_xauthority_local_selector_criteria_from_display` へ接続する。
- source policy は F5jm surface に `Family::Wild` fallback、empty fallback、hostname / gethostname / env / fs / VFS、raw API、window setup、runner、support gate `Ok` 化、fallback、silent no-op、synthetic readiness が混入しないことを固定する。

非目標:

- actual `gethostname` / process hostname / platform identity acquisition は扱わない。
- `DISPLAY` env acquisition、`XAUTHORITY` / `HOME` env acquisition、filesystem / VFS file bytes read、path planning は扱わない。
- credential selection、setup request integration、X11 raw fd / raw API owner、window creation、event mask、WM_DELETE_WINDOW、keyboard / IME、Wayland concrete decoding は扱わない。
- Linux support gate の `Ok` 化、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop` は行わない。
- no-auth fallback、wild fallback、fallback snapshot、silent no-op、synthetic readiness は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `cargo test -p nepl-gui-native --lib`
- `cargo check -p nepl-gui-native --target x86_64-unknown-linux-gnu`
- `cargo check -p nepl-gui-native --lib --tests --target x86_64-unknown-linux-gnu`
- `git diff --check`
- subagent implementation review で F5jm が injected local-authority-address owner boundary に留まり、actual hostname acquisition、env / fs / VFS、raw API、window setup、runner dispatch が混入していないことが承認される。

## Phase F5jn: Native Linux Xauthority process hostname address adapter boundary

Phase F5jn では、F5jm の `NativeWindowLinuxX11LocalAuthorityAddressReader` に接続する cfg Linux process hostname adapter を追加する。これは Linux process identity acquisition を raw API trait に隔離し、`gethostname` 相当の結果を F5jm の address validation へ渡すための境界である。selector criteria construction、Xauthority credential selection、setup request integration、raw X11 fd、window setup、runner / CLI dispatch は扱わない。

実装:

- `NativeWindowLinuxX11LocalAuthorityAddressRawApi` は `get_hostname_raw buffer` だけを持つ injected raw API trait とする。
- `NativeWindowLinuxX11LocalAuthorityAddressProcessReadError` は raw API failure と non-terminated hostname buffer を分ける。
- `NativeWindowLinuxX11ProcessLocalAuthorityAddressReader` は raw API owner を保持し、buffer を `NATIVE_WINDOW_LINUX_X11_LOCAL_AUTHORITY_ADDRESS_MAX_BYTE_LEN + 1` で確保して raw API を 1 回だけ呼ぶ。
- raw API が書いた buffer は最初の NUL byte までを hostname bytes とし、NUL が見つからない場合は truncation / too-long suspect として typed error にする。
- empty hostname bytes は F5jm helper の `EmptyAddress` に委譲し、reader 側で empty fallback を作らない。
- cfg Linux sys adapter は `libc::gethostname` だけを呼び、`std::env`、filesystem / VFS、DISPLAY parsing、Xauthority file read には触れない。
- convenience helper は process hostname reader を構築し、F5jm `native_window_linux_x11_local_authority_address` へ接続するだけにする。
- source policy は F5jn surface に selector criteria construction、credential selection、setup request integration、raw X11 fd read/write、window setup、runner、support gate `Ok` 化、fallback、silent no-op、synthetic readiness が混入しないことを固定する。

非目標:

- `DISPLAY` env acquisition、Xauthority path planning、Xauthority file bytes read、VFS、credential selection、setup request integration は扱わない。
- X11 raw fd / raw API owner、window creation、event mask、WM_DELETE_WINDOW、keyboard / IME、Wayland concrete decoding は扱わない。
- Linux support gate の `Ok` 化、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop` は行わない。
- no-auth fallback、wild fallback、empty fallback、fallback snapshot、silent no-op、synthetic readiness は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `cargo test -p nepl-gui-native --lib`
- `cargo check -p nepl-gui-native --target x86_64-unknown-linux-gnu`
- `cargo check -p nepl-gui-native --lib --tests --target x86_64-unknown-linux-gnu`
- `git diff --check`
- subagent implementation review で F5jn が process hostname adapter boundary に留まり、selector / credential / setup request / raw X11 fd / window setup / runner dispatch が混入していないことが承認される。

## Phase F5jo: Native Linux X11 top-level window create/map request owner boundary

Phase F5jo では、X11 local Unix connection 上で使う top-level window の CreateWindow / MapWindow request bytes を typed owner として作る。これは request byte owner boundary であり、actual `write_x11_bytes_raw` integration、server error handling、resource id allocator、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、support gate `Ok` 化はまだ行わない。

実装:

- `NativeWindowLinuxX11TopLevelWindowCreateInput`、`NativeWindowLinuxX11TopLevelWindowCreateRequest`、`NativeWindowLinuxX11TopLevelWindowCreateRequestBuildError` を追加する。
- window id と parent window id は zero と top 3 bits set を typed error として拒否する。
- width / height は zero を typed error として拒否する。
- default event mask は F5jo request 自体が MapWindow を直後に送ることを考慮し、current F5jb decoder が non-fatal に扱える pointer / button event だけに合わせ、`ButtonPress | ButtonRelease | PointerMotion` とする。StructureNotify は ConfigureNotify だけでなく MapNotify なども購読するため、追加 event decode phase まで含めない。Expose もまだ decode しないため含めない。
- CreateWindow は opcode `1`、depth `CopyFromParent`、class `InputOutput`、visual `CopyFromParent`、value mask `background-pixel | event-mask`、value-list `background-pixel` then `event-mask` として encode する。
- background-pixel と event-mask の 2 value だけを持つため、CreateWindow request length は `10` units とする。
- MapWindow は opcode `8`、request length `2`、同じ window id として encode する。
- request owner は private `Vec u8` を保持し、public surface は `window_id` / `as_bytes` / `len` に限定する。
- Rust focused tests は exact byte layout、zero ids、high-bit ids、zero width / height、unused event-mask bits、default event mask に StructureNotify / Expose が無いことを検査する。
- `nodesrc/test_native_gui_platform_behavior.js` に F5jo 専用 source-policy slice を追加し、F5jb raw API surface と混ぜない。
- `doc/neplg2/gui_standard_library_spec.md`、`doc/neplg2/gui_native_platform_behavior.md`、note、todo を同じ slice で更新する。

非目標:

- actual raw fd write / read、setup observation reader への接続、Xauthority credential lookup、resource id allocation、server sequence / error handling、WM_DELETE_WINDOW、keyboard / IME、Linux runner / CLI dispatch は含めない。
- StructureNotify / Expose subscription は F5jo では行わない。MapNotify / ConfigureNotify / Expose decode を追加する phase まで、それらを silent no-op にしない。
- fallback、silent no-op、synthetic readiness、Linux support gate `Ok` 化は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で pure request-owner boundary、default mask scope、exact request length、no raw IO / no runner / no fallback が承認される。

## Phase F5jp: Native Linux X11 top-level window request partial-write boundary

Phase F5jp では、F5jo の CreateWindow / MapWindow request owner を、既存の `NativeWindowLinuxX11EventSourceObservationReader` が setup completion 後に partial write できる境界へ接続する。これは reader-owned request write boundary であり、resource id allocation、root window discovery、server error / reply handling、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、StructureNotify / Expose subscription、MapNotify / ConfigureNotify / Expose decode、Linux runner / CLI dispatch、support gate `Ok` 化はまだ行わない。

実装:

- `NativeWindowLinuxX11TopLevelWindowRequestWriteState` を追加し、`NotConfigured`、`RequestPending`、`Ready`、`Failed` を enum で表す。
- `NativeWindowLinuxX11EventSourceObservationReader` は optional `NativeWindowLinuxX11TopLevelWindowCreateRequest`、request write state、written byte count を private field として保持する。
- 既存 constructor は `NotConfigured` のままにし、既存 setup/event observation behavior を維持する。
- 新 constructor は setup request と top-level request owner を同時に受け取り、setup ready 後、event read 前に CreateWindow / MapWindow request を `write_x11_bytes_raw` へ渡す。
- partial write は written byte count を保持し、would-block は retryable error として返す。
- hard failure、zero write、overflow は typed error にし、state を `Failed` にする。以後の poll は `TopLevelWindowRequestPreviouslyFailed` として fail-closed にする。
- provider wrapper に同じ明示 constructor / helper を追加する。
- Rust focused tests は setup write -> top-level request write -> event read の順序、partial write resume、hard failure 後の failed state、既存 no-request constructor の挙動を検査する。
- source-policy は F5jp が Xauthority lookup、resource allocation、runner dispatch、WM_DELETE_WINDOW、keyboard / IME、fallback、silent no-op、synthetic readiness を含まないことを検査する。

非目標:

- resource id allocator、setup response からの root window discovery、X11 server error / reply decode、window manager protocol、keyboard / IME、Linux runner / CLI dispatch は含めない。
- StructureNotify / Expose subscription は行わない。MapNotify / ConfigureNotify / Expose decode を追加する phase まで、それらを silent no-op にしない。
- support gate `Ok` 化、fallback、synthetic readiness は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_observation_provider -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で setup-ready 後 request write boundary、partial write recovery、no runner / no fallback が承認される。

## Phase F5jq: Native Linux X11 setup resource info and resource id allocation boundary

Phase F5jq では、X11 setup success body から resource-id-base、resource-id-mask、first screen root window id を取り出し、client resource id allocation helper へ渡せる typed owner を追加する。これは setup body parse / resource id allocation boundary であり、top-level request generation、server error / reply handling、WM_DELETE_WINDOW、keyboard / IME、Linux runner / CLI dispatch、support gate `Ok` 化はまだ行わない。

実装:

- X11 setup success body の 32 byte fixed body、vendor string + 4 byte padding、pixmap format list、first screen header を checked arithmetic で読む parser を追加する。
- `NativeWindowLinuxX11SetupResourceInfo` は `resource_id_base`、`resource_id_mask`、`first_root_window_id` を private field として保持し、read-only accessor だけを公開する。
- parse error は `NativeWindowLinuxX11SetupResourceInfoParseError` とし、short body、section truncation、offset overflow、mask zero、base/mask overlap、client id high bits、root count zero、root id zero を分ける。
- first root window id は server-owned id なので client resource id mask/base validation を適用せず、zero だけ拒否する。
- `native_window_linux_x11_resource_id_from_serial` は sparse mask に対応し、serial bit を mask set bit へ low-to-high に詰める。
- allocation error は `NativeWindowLinuxX11ResourceIdAllocationError` とし、serial zero、serial exhausted、generated id zero、base/mask invariant violation を分ける。
- `NativeWindowLinuxX11EventSourceObservationReader` は setup body bytes を保持し、body read 完了時に parser を呼ぶ。parse failure は `SetupResourceInfoParseFailed` で fail-closed にする。
- empty setup body は historical scripted observation compatibility の `Ok(None)` として扱い、native GUI readiness や synthetic root window discovery には使わない。
- Rust focused tests は success body parse、invalid client id space、missing/truncated screen、sparse mask allocation、partial setup body resume、parse failure fail-closed を検査する。
- source-policy は F5jq が top-level request generation、runner dispatch、support gate `Ok` 化、WM_DELETE_WINDOW、keyboard / IME、fallback、silent no-op、synthetic readiness を含まないことを検査する。

非目標:

- setup resource info から actual CreateWindow request を生成する mutable window id owner は含めない。
- server error / reply decode、sequence tracking、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、StructureNotify / Expose subscription は含めない。
- Linux runner / CLI dispatch、support gate `Ok` 化、fallback、synthetic readiness は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_setup_resource -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_resource_id -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_observation_provider -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で setup resource parser、sparse resource allocator、reader fail-closed integration、no runner / no fallback が承認される。

## Phase F5jr: Native Linux X11 setup-backed top-level window request owner boundary

Phase F5jr では、F5jq の setup resource info を使い、caller supplied serial から generated client window id を作り、setup success body 由来の first root window id を parent にした CreateWindow / MapWindow request owner を生成する。これは setup-backed request owner boundary であり、reader 自動接続、server error / reply handling、WM_DELETE_WINDOW、keyboard / IME、Linux runner / CLI dispatch、support gate `Ok` 化はまだ行わない。

実装:

- `NativeWindowLinuxX11SetupBackedTopLevelWindowCreateInput` を追加し、setup resource info、window resource serial、geometry、background pixel、event mask を private field として保持する。
- `NativeWindowLinuxX11SetupBackedTopLevelWindowCreateRequestError` を追加し、resource id allocation failure と request build failure を分ける。
- 既存 `native_window_linux_x11_top_level_window_create_request` は caller supplied window id / parent id の high-bit validation を維持する。
- request byte encoding は private helper に分離し、ID 検証済みの caller-supplied path と setup-backed path で共有する。
- setup-backed builder は `native_window_linux_x11_resource_id_from_serial` で generated client window id を作り、window id は client resource id validation、root parent id は zero-only validation を通す。
- source-policy は setup-backed path が Xauthority lookup、reader mutation、raw fd write、runner dispatch、support gate `Ok` 化、WM_DELETE_WINDOW、keyboard / IME、fallback、silent no-op、synthetic readiness を含まないことを検査する。

非目標:

- `NativeWindowLinuxX11EventSourceObservationReader` が setup ready 後に request を自動生成する接続は含めない。
- server sequence / error / reply decode、window manager protocol、keyboard / IME、Linux runner / CLI dispatch は含めない。
- StructureNotify / Expose subscription は行わない。MapNotify / ConfigureNotify / Expose decode を追加する phase まで、それらを silent no-op にしない。
- support gate `Ok` 化、fallback、synthetic readiness は作らない。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_setup_backed -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_top_level_window_create_request -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で setup-backed resource id allocation、root id zero-only validation、no reader mutation / no runner / no fallback が承認される。

## Phase F5js: Native Linux X11 reader setup-backed top-level request generation boundary

Phase F5js では、F5jr の setup-backed request owner builder を X11 observation reader の setup state へ接続する。ただし reader が geometry や event mask を暗黙 default として発明してはいけない。caller supplied typed plan として `NativeWindowLinuxX11SetupBackedTopLevelWindowCreatePlan` が window resource serial、geometry、border width、background pixel、event mask を保持し、reader は setup が `Ready` になって `setup_resource_info` を保持した後だけ、その plan と setup resource info から `NativeWindowLinuxX11TopLevelWindowCreateRequest` を作る。

実装:

- `NativeWindowLinuxX11SetupBackedTopLevelWindowCreatePlan` を追加し、setup resource info を含まない typed generation config として serial、geometry、background pixel、event mask を保持する。
- `NativeWindowLinuxX11TopLevelWindowRequestWriteState` に `SetupBackedBuildPending` を追加し、setup ready 後の request owner build と既存 F5jp partial-write を同じ state machine で扱う。
- request build は必ず `native_window_linux_x11_top_level_window_create_request_from_setup_resource_info` に委譲し、resource id allocation と root parent validation の authority を F5jr に残す。
- `SetupBackedBuildPending` なのに caller supplied typed plan が無い、setup resource info が無い、allocation に失敗した、request build に失敗した場合は `TopLevelWindowRequestSetupBackedPlanMissing`、`TopLevelWindowRequestSetupResourceInfoMissing`、または `TopLevelWindowRequestBuildFailed` で fail-closed にし、以後の poll は `TopLevelWindowRequestPreviouslyFailed` を返す。
- prebuilt request owner constructor と no-request constructor は互換 path として残す。no-request constructor は readiness evidence ではなく `NotConfigured` compatibility path である。
- source-policy は typed plan が serial / geometry / background / event mask を保持すること、reader が hidden default / fallback / silent no-op を作らないこと、build 後は F5jp partial-write path を再利用することを検査する。

非目標:

- server sequence / error / reply decode、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、StructureNotify / Expose subscription、MapNotify / ConfigureNotify / Expose decode は含めない。
- Linux runner / CLI dispatch、support gate `Ok` 化、Wayland concrete decoding、fallback、synthetic readiness は作らない。
- reader は geometry、event mask、serial を生成しない。暗黙の default を使いたい場合も caller が explicit default constructor を通して plan を渡す。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_observation_provider -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で typed plan、setup-backed plan missing fail-closed、setup resource info missing fail-closed、F5jp partial-write reuse、no runner / no fallback が承認される。

## Phase F5jt: Native Linux X11 server error / reply header decode boundary

Phase F5jt では、X11 observation reader が event packet と同じ 32 byte 境界で届く server error packet と reply header を typed data として識別する。`packet[0]` の raw response type が `0` の場合は X11 server error、`1` の場合は server reply header として扱い、通常 event の send-event high bit mask はその後の event decode だけに適用する。

実装:

- `NativeWindowLinuxX11ServerErrorPacket` を追加し、error code、sequence、bad value、minor opcode、major opcode を fixed offset から little-endian で取得する。
- `NativeWindowLinuxX11ServerReplyHeader` を追加し、reply data、sequence、length units を fixed offset から little-endian で取得する。
- `NativeWindowLinuxX11EventSourceObservationError` に `ServerErrorReceived` と `ServerReplyReceived` を追加し、server error / reply を `EventTypeUnsupported` に落とさない。
- normal event は raw response type が `0` / `1` ではない場合だけ、既存の `response_type & 0x7f` による Configure / Motion / Button decode へ進む。
- source-policy は raw response type の先行分岐、fixed offset parser、typed error variant、send-event high bit 付き通常 event decode を検査する。

非目標:

- server sequence tracking、request / reply correlation、reply body drain、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、StructureNotify / Expose subscription、MapNotify / ConfigureNotify / Expose decode は含めない。
- Linux runner / CLI dispatch、support gate `Ok` 化、Wayland concrete decoding、fallback、synthetic readiness は作らない。
- server error を top-level request write failure として推測しない。request authority は sequence correlation phase まで raw packet decode から分離する。

完了条件:

- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_observation_provider_reports_server -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_observation_provider_masks_send_event_bit -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_observation_provider -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で raw response type 先行分岐、typed server error / reply data、normal event mask 維持、no sequence correlation / no runner / no fallback が承認される。

## Phase F5ju: Native Linux X11 request sequence correlation boundary

Phase F5ju では、X11 observation reader が top-level CreateWindow / MapWindow request owner の accepted write progress から normal request sequence を追跡し、F5jt の server error packet を sequence によって request に結び付ける。X11 setup handshake は normal request sequence に含めない。sequence authority は reader が所有し、writer が受理した byte range が request boundary を越えた時だけ進める。

実装:

- `NativeWindowLinuxX11TopLevelWindowRequestSequencePlan` を追加し、window id、CreateWindow sequence、MapWindow sequence を保持する。
- `NativeWindowLinuxX11ServerErrorCorrelation` を追加し、`Unmatched`、`TopLevelWindowCreate`、`TopLevelWindowMap` を enum として返す。
- reader は first normal request sequence を `1` として保持し、accepted write range が CreateWindow byte length を越えた時だけ CreateWindow sequence を記録し、combined top-level byte length を越えた時だけ MapWindow sequence を記録する。
- `ServerErrorReceived` は decoded packet と correlation を同時に返す。correlation は packet `sequence` だけで決め、major opcode は authority にしない。
- source-policy は sequence plan、correlation enum、accepted range recording、`ServerErrorReceived` の correlation payload を検査する。

非目標:

- server reply body drain / reply correlation、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、StructureNotify / Expose subscription、MapNotify / ConfigureNotify / Expose decode は含めない。
- Linux runner / CLI dispatch、support gate `Ok` 化、Wayland concrete decoding、fallback、synthetic readiness は作らない。
- opcode だけで request を推測しない。bad value や major opcode は decoded evidence として残すだけで、correlation authority にしない。

完了条件:

- setup handshake だけでは `next_x11_request_sequence` が進まないことを focused test で検査する。
- CreateWindow 境界より手前の partial write では top-level sequence が未記録であることを検査する。
- CreateWindow 境界だけを越えた partial write では CreateWindow sequence だけが記録されることを検査する。
- CreateWindow / MapWindow の両方を越えた accepted write では sequence が順に記録されることを検査する。
- write failure / would-block before acceptance では sequence が進まないことを検査する。
- major opcode と異なる request sequence でも packet sequence に従って correlation することを検査する。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_observation_provider -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で accepted range sequence tracking、sequence-only correlation、no runner / no fallback が承認される。

## Phase F5jv: Native Linux X11 server reply body drain boundary

Phase F5jv では、X11 server reply header の `length_units` が示す reply body を reader が drain してから `ServerReplyReceived` を返す。F5jt は header decode boundary だけを担当していたため、`length_units > 0` の body byte が socket に残ると、次回 poll がその body を event packet として誤 decode しうる。F5jv は request-specific reply parser ではなく、generic observation stream の同期を壊さないための drain boundary である。

実装:

- `NativeWindowLinuxX11EventSourceObservationReader` は pending reply header と remaining body byte count を保持する。
- `length_units * 4` は checked arithmetic と `usize` conversion で検査し、失敗時は typed `ServerReplyBodyLengthOverflow` を返す。
- `ServerReplyReceived` は body drain 完了後だけ返す。body read が would-block した場合は pending header と remaining byte count を保持し、次回 poll は新しい event packet を読まずに drain を再開する。
- body read failure / EOF / overflow は header と remaining byte count を含む typed error として返し、partial state を silent clear しない。
- 現段階では body payload を parse / retain せず discard する。将来 InternAtom / GetProperty などの request-specific reply を扱う phase では、同じ stream sync contract を維持したまま request-specific reply body owner / parser へ接続する。
- source-policy は pending reply state、checked body byte count、would-block resume、`ServerReplyReceived` が drain 完了後だけ返ること、no fallback / no silent no-op を検査する。

非目標:

- request / reply correlation、InternAtom / WM_DELETE_WINDOW / ChangeProperty、keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch は含めない。
- reply body を application payload として公開しない。generic unexpected reply は stream sync のために drain し、typed header evidence だけを返す。
- fallback、silent no-op、synthetic readiness、support gate `Ok` 化は作らない。

完了条件:

- zero-length reply は pending state を残さず `ServerReplyReceived` を返す。
- nonzero reply は body を drain してから `ServerReplyReceived` を返し、次の event packet が body byte ではなく実 event として decode される。
- partial body read の would-block 後、pending header と remaining byte count を保持し、次回 poll で drain を再開する。
- EOF / read failure / overflow は typed error として返し、partial reply state を silent clear しない。
- F5ju の setup handshake exclusion、accepted write range sequence tracking、sequence-only server error correlation を変更しない。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_observation_provider -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で reply body drain state、would-block resume、no request-specific parser / no fallback が承認される。

## Phase F5jw: Native Linux X11 InternAtom request owner boundary

Phase F5jw では、WM_DELETE_WINDOW protocol registration の前段として、X11 `InternAtom` request bytes を typed owner として構築する。F5jw は pure encoding / validation boundary であり、raw fd write/read、reply parsing、reply correlation は行わない。

実装:

- `NativeWindowLinuxX11InternAtomRequest` は atom name byte length、`only_if_exists` bool、encoded request bytes を所有する。
- `native_window_linux_x11_intern_atom_request` は opcode `16`、`only_if_exists` 0/1、request length units、name length、unused zero、name bytes、4 byte zero padding を little-endian で encode する。
- request length units は `2 + padded_name_len / 4` であり、name length は `u16`、request length units も `u16` として検査する。
- generic `InternAtom` owner は X11 counted bytes を扱うため、NUL byte を C string terminator として拒否しない。
- 空 name、name length 超過、total length overflow、request length units 超過は enum error で返す。
- source-policy は request owner surface が raw fd write/read、reply parse/correlation、Atom ID owner、WM protocol registration、ChangeProperty、ClientMessage、fallback、silent no-op を含まないことを検査する。

非目標:

- `InternAtom` request の actual write、accepted write progress、sequence assignment、reply body retention/parser、request / reply correlation は含めない。
- `WM_PROTOCOLS` / `WM_DELETE_WINDOW` の well-known name helper、Atom ID owner、ChangeProperty、ClientMessage decode は含めない。
- keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、support gate `Ok` 化、fallback、silent no-op、synthetic readiness は含めない。

完了条件:

- `WM_PROTOCOLS` と `WM_DELETE_WINDOW` の request bytes が protocol 通り encode される。
- 1/2/3/4 byte mod の atom name padding が zero-filled になり、request length units と name length が little-endian で一致する。
- empty name、too-long name、overflow helper は typed error で fail closed になる。
- generic counted name として NUL byte を保持でき、C string terminator として扱わない。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_intern_atom_request -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で InternAtom request owner、counted bytes contract、no raw write/read / no reply correlation / no fallback が承認される。

## Phase F5jx: Native Linux X11 WM protocol atom InternAtom request batch boundary

Phase F5jx では、F5jw の generic `InternAtom` request owner を使い、ICCCM の `WM_PROTOCOLS` と `WM_DELETE_WINDOW` に対する request batch bytes owner を追加する。F5jx は WM protocol registration の前段であり、actual property registration ではない。raw fd write/read、accepted write progress、sequence assignment、reply parsing / correlation、Atom ID owner、`ChangeProperty`、`ClientMessage` decode は行わない。

実装:

- owner は `NativeWindowLinuxX11WmProtocolAtomInternRequestBatch` とし、concatenated request bytes と request boundary offset を所有する。
- atom kind は `WmProtocols` と `WmDeleteWindow` の enum で表し、future sequence assignment は byte boundary を越えた accepted write progress によってだけ行える形にする。
- well-known names は `WM_PROTOCOLS`、`WM_DELETE_WINDOW` の fixed ASCII bytes とし、caller supplied arbitrary name helper は public API にしない。
- 両 request は `only_if_exists = false` で F5jw `native_window_linux_x11_intern_atom_request` に委譲して作る。
- lower `InternAtom` build failure は atom kind と lower error を保持し、batch length overflow は typed error で fail closed にする。
- source-policy は batch surface が raw fd write/read、reply parse/correlation、Atom ID owner、`ChangeProperty`、`ClientMessage`、support gate `Ok` 化、fallback、silent no-op、synthetic readiness、registration naming を含まないことを検査する。

非目標:

- request batch の actual write、accepted write progress、sequence assignment は含めない。
- reply body retention/parser、request / reply correlation、Atom ID owner は含めない。
- `WM_PROTOCOLS` property の `ChangeProperty`、`WM_DELETE_WINDOW` `ClientMessage` decode は含めない。
- keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、support gate `Ok` 化、fallback、silent no-op、synthetic readiness は含めない。

完了条件:

- request order は `WM_PROTOCOLS`、`WM_DELETE_WINDOW` の順に固定される。
- 両 request は opcode `16` かつ `only_if_exists = false` として encode される。
- combined bytes は F5jw owner で作った 2 request の単純連結であり、byte boundary offset が exact である。
- lower error と concat overflow は enum error として保持される。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_wm_protocol_atom_intern_request_batch -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で WM protocol atom request batch owner、exact offset、no raw write/read / no registration / no fallback が承認される。

## Phase F5jy: Native Linux X11 top-level CreateWindow/MapWindow split request owner boundary

Phase F5jy では、F5jo 以来 combined owner に閉じていた top-level `CreateWindow` と `MapWindow` の request bytes を standalone owner として分離する。これは WM protocol registration のための ordering prerequisite であり、registration phase そのものではない。後続は `CreateWindow -> InternAtom batch -> InternAtom replies / Atom IDs -> ChangeProperty WM_PROTOCOLS -> MapWindow` の順序へ進めるが、F5jy は reader state、raw fd write/read、sequence assignment、reply correlation、Atom ID、`ChangeProperty`、`ClientMessage` decode を扱わない。

実装:

- `NativeWindowLinuxX11CreateWindowRequest` と `NativeWindowLinuxX11MapWindowRequest` を追加し、それぞれ window id と encoded bytes だけを所有する。
- 既存の `NativeWindowLinuxX11TopLevelWindowCreateRequest` は互換 owner として残し、standalone `CreateWindow` owner と `MapWindow` owner の bytes をこの順で連結する。
- standalone `CreateWindow` は既存 F5jo の resource id、width / height、event mask、request length overflow validation を再利用する。
- standalone `MapWindow` は window id validation を同じ public error enum で返す。
- combined owner は create byte length、map byte length、total bytes、window id を metadata として公開し、sequence number は公開しない。
- source-policy は split owner surface が raw fd write/read、reader integration、sequence assignment、InternAtom、reply / Atom ID、`ChangeProperty`、`ClientMessage`、support gate、fallback、silent no-op、synthetic readiness を含まないことを検査する。

非目標:

- current reader partial-write path は combined owner のまま維持する。
- InternAtom batch write、reply parser / reply correlation、Atom ID owner、WM property registration、MapWindow scheduling relocation は含めない。
- keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、support gate `Ok` 化、fallback、silent no-op、synthetic readiness は含めない。

完了条件:

- standalone `CreateWindow` bytes は従来 combined owner の prefix と byte-for-byte に一致する。
- standalone `MapWindow` bytes は従来 combined owner の suffix と byte-for-byte に一致する。
- combined owner bytes は `create.as_bytes() + map.as_bytes()` と完全一致し、既存 focused tests が維持される。
- setup-backed generated window id / root parent path の validation と bytes が維持される。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_create_window_request -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_map_window_request -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_top_level_window_create_request -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で CreateWindow/MapWindow split owner、byte exactness、no writer/reply/registration integration、no fallback が承認される。

## Phase F5jz: Native Linux X11 InternAtom reply packet AtomId owner boundary

Phase F5jz では、X11 `InternAtom` の fixed 32 byte reply packet から nonzero Atom ID を取り出す pure parser と owner を追加する。これは F5jw / F5jx の request bytes と、後続の WM protocol registration を接続するための request-specific reply payload boundary である。ただし F5jz は current reader の generic reply handling を変更せず、request / reply correlation も行わない。

実装:

- `NativeWindowLinuxX11AtomId` は nonzero raw atom id を保持し、`raw` accessor だけを公開する。
- Atom 0 は `only_if_exists = true` では仕様上あり得るため、F5jz の AtomId owner は nonzero Atom ID を要求する境界だと明記する。
- `NativeWindowLinuxX11InternAtomReply` は sequence と `NativeWindowLinuxX11AtomId` を保持する。
- `native_window_linux_x11_intern_atom_reply_from_packet` は fixed 32 byte packet を受け、response type、sequence、reply length units、atom id を little-endian で読む。
- parser は response type が reply であること、InternAtom reply length units が `0` であること、atom id が nonzero であることを検査する。
- source-policy は parser surface が reader mutation、raw fd write/read、accepted write progress、request / reply correlation、WM batch sequence assignment、`ChangeProperty`、`ClientMessage`、MapWindow scheduling、support gate、fallback、silent no-op、synthetic readiness を含まないことを検査する。

非目標:

- current reader の `ServerReplyReceived` / generic reply body drain behavior は変更しない。
- WM protocol atom request batch の write、accepted write progress、sequence correlation、Atom kind mapping は含めない。
- `ChangeProperty` による actual `WM_PROTOCOLS` registration、`WM_DELETE_WINDOW` `ClientMessage` decode、MapWindow scheduling relocation は含めない。
- keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、support gate `Ok` 化、fallback、silent no-op、synthetic readiness は含めない。

完了条件:

- valid packet で sequence と nonzero Atom ID が保持される。
- response type が reply でない packet は typed error になる。
- nonzero reply length units は typed error になる。
- Atom ID 0 は nonzero AtomId owner 境界として typed error になる。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_intern_atom_reply -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で nonzero AtomId owner、fixed 32 byte packet parser、no reader/correlation/registration integration、no fallback が承認される。

## Phase F5ka: Native Linux X11 InternAtom batch partial-write scheduling boundary

Phase F5ka では、F5jx の `WM_PROTOCOLS` / `WM_DELETE_WINDOW` `InternAtom` request batch owner を reader-owned partial-write state に接続する。F5jy の split request owner を前提に、明示的に WM protocol batch を渡した reader だけが `CreateWindow -> InternAtom batch -> MapWindow` の順序で書く。これは MapWindow を batch 前に送らないための最小 scheduling split を含む boundary であり、compatibility path の combined top-level request は hidden fallback ではなく batch 未構成時の明示状態として残す。

実装:

- `NativeWindowLinuxX11WmProtocolAtomInternBatchWriteState` を追加し、`NotConfigured`、`BatchPending`、`Ready`、`Failed` を持たせる。
- reader は optional `NativeWindowLinuxX11WmProtocolAtomInternRequestBatch` と written length を private field に保持する。
- batch configured path では top-level request writer が CreateWindow byte boundary までを先に書き、次に InternAtom batch writer を完了させ、最後に MapWindow byte boundary までを書き進める。
- CreateWindow と MapWindow の sequence tracking は既存 `NativeWindowLinuxX11TopLevelWindowRequestSequencePlan` を authority とし、InternAtom request sequence は batch offset boundary で進めるが、Atom meaning assignment や reply correlation は行わない。
- would-block は retryable typed error として state を保持し、hard failure / zero write / overflow は batch write state を `Failed` にする。

非目標:

- InternAtom reply packet の reader dispatch、request / reply correlation、`WM_PROTOCOLS` / `WM_DELETE_WINDOW` への meaning assignment は含めない。
- `ChangeProperty` による actual WM property mutation、`ClientMessage` decode、keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、support gate `Ok` 化は含めない。
- batch 未構成 path を WM protocol registration 完了と偽装しない。

完了条件:

- partial batch write が would-block 後に同じ byte offset から再開する。
- batch configured path では write order が setup、CreateWindow、InternAtom batch、MapWindow になる。
- batch が完了するまで MapWindow write と MapWindow sequence record は発生しない。
- batch failure 後の再 poll は previous failure として fail closed する。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_wm_protocol_atom_intern_batch_write -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で split writer ordering、batch partial-write state、no reply/correlation/registration integration、no fallback が承認される。

## Phase F5kb: Native Linux X11 InternAtom reply sequence correlation boundary

Phase F5kb では、F5ka で request boundary ごとに進めていた `InternAtom` request sequence を、reader-owned sequence plan として保持する。X11 reply packet が届いたとき、generic `ServerReplyReceived` に縮約する前に sequence を照合し、`WM_PROTOCOLS` / `WM_DELETE_WINDOW` request に一致する reply だけを F5jz の `InternAtom` reply parser に渡す。

実装:

- `NativeWindowLinuxX11WmProtocolAtomInternRequestSequencePlan` を追加し、`WM_PROTOCOLS` request sequence と `WM_DELETE_WINDOW` request sequence を `Option u16` として保持する。
- `record_wm_protocol_atom_intern_batch_accepted_range` は request boundary crossing 時に `take_next_x11_request_sequence` の戻り値を sequence plan へ記録する。
- reply dispatch は pending body drain と stream sync contract を保ったまま、full 32 byte reply packet を保持し、body drain 完了後に sequence correlation を行う。
- matched `InternAtom` reply は `NativeWindowLinuxX11WmProtocolAtomInternReplyCorrelation` と `NativeWindowLinuxX11InternAtomReply` を持つ typed observation error として返す。
- matched reply の parse failure は lower `NativeWindowLinuxX11InternAtomReplyParseError` と correlation を保持する typed error として返す。
- unmatched reply は既存の generic `ServerReplyReceived` のままにする。

非目標:

- `WM_PROTOCOLS` / `WM_DELETE_WINDOW` Atom ID の meaning assignment / persistent registration state は含めない。
- `ChangeProperty` による actual WM property mutation、`WM_DELETE_WINDOW` `ClientMessage` decode は含めない。
- keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、support gate `Ok` 化は含めない。
- matched reply の malformed body length を silent success や generic reply へ fallback しない。

完了条件:

- batch write が `WM_PROTOCOLS` / `WM_DELETE_WINDOW` の request sequence を保持する。
- sequence が matched した zero-body `InternAtom` reply は typed correlated reply として返る。
- matched reply parse failure は correlation と lower parser error を保持する。
- unmatched reply は既存 generic reply behavior を維持する。
- reply body drain が必要な packet でも pending state は full packet を保持し、drain 完了後に dispatch する。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_wm_protocol_atom_intern_reply -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で sequence retention、reply packet retention、matched reply parse/correlation、no meaning assignment / no ChangeProperty / no ClientMessage / no fallback が承認される。

## Phase F5kc: Native Linux X11 WM protocol Atom meaning assignment boundary

Phase F5kc では、F5kb で相関済みになった `InternAtom` reply を、`WM_PROTOCOLS` property Atom と `WM_DELETE_WINDOW` protocol Atom の semantic slot に割り当てる。これは actual WM registration ではなく、後続の `ChangeProperty` phase が「どの Atom ID が何を意味するか」を typed owner から受け取れるようにする境界である。

実装:

- `NativeWindowLinuxX11WmProtocolAtomMeaning` を追加し、相関種別と semantic slot を分けて表す。
- `NativeWindowLinuxX11WmProtocolAtomAssignmentState` は `WM_PROTOCOLS` Atom ID と `WM_DELETE_WINDOW` Atom ID を `Option` として保持する。
- `NativeWindowLinuxX11WmProtocolAtoms` は両方の Atom ID が揃った時だけ作れる completed owner とする。
- correlated `InternAtom` reply の parse が成功した場合だけ、reader は assignment state を更新する。
- duplicate assignment は typed error とし、既存 slot を上書きしない。
- `WmProtocolAtomInternReplyReceived` は parsed reply と completed owner の `Option` を持ち、1 つ目の reply と 2 つ目の reply を明示的に区別する。

非目標:

- `ChangeProperty` による actual `WM_PROTOCOLS` property mutation は含めない。
- `WM_DELETE_WINDOW` `ClientMessage` decode は含めない。
- Linux runner / CLI dispatch、support gate `Ok` 化、keyboard / IME、Wayland concrete decoding は含めない。
- Atom ID の fallback、default Atom ID、synthetic readiness、silent no-op は行わない。

完了条件:

- assignment state は correlated reply だけを semantic slot へ割り当てる。
- parse failure と unmatched reply は assignment state を変えない。
- duplicate assignment は typed error として fail closed し、既存 Atom ID を保持する。
- 2 つの Atom ID が揃った時だけ completed owner が返る。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_wm_protocol_atom_assignment -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_wm_protocol_atom_intern_reply -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で typed assignment state、completed owner、duplicate fail-closed、no ChangeProperty / no ClientMessage / no fallback が承認される。

- scheduler loop は F5eg の `YieldToClock` / `AwaitTimerAdvance` / `ExecuteHostAction` / `Complete` action を明示的に進める必要がある。
- `YieldToClock` は F5ej の deterministic clock-delta authority によってだけ pending / ready を判断する必要がある。
- `WaitingTimer` は F5eh の `loop_timer_advance` または later real timer backend authority によってだけ再開する必要がある。
- `ExecuteHostAction` は F5ei の `loop_executor_complete` または later real backend executor authority が返す caller supplied outcome によってだけ再開する必要がある。
- `Complete` は F5ej の `loop_yield_complete_complete_ack` によって terminal payload へ明示的に変換する必要がある。
- slice policy は `YieldSlice` と timer schedule の契約を乱さず、FHD 60fps 目標に向けて bounded turn progress を表す必要がある。
- headless app-loop は presentation fallback ではなく、virtual event / virtual timer / offscreen snapshot を組み合わせた test target として扱う必要がある。
- 実装開始前に subagent review を通し、Required がある場合は doc を修正して再 review する。

## Phase F5kd: Native Linux X11 WM protocol ChangeProperty registration boundary

Phase F5kd では、F5kc の completed Atom owner を使い、X11 top-level window に対する `WM_PROTOCOLS` property mutation を `ChangeProperty` request として送る。これは actual registration request write boundary であり、`WM_DELETE_WINDOW` `ClientMessage` decode や Linux runner dispatch ではない。

実装:

- `NativeWindowLinuxX11WmProtocolRegistrationRequest` を追加し、window id、completed Atom owner、encoded request bytes を保持する。
- request bytes は X11 `ChangeProperty` の fixed header 24 byte と 32-bit Atom data item 1 個を合わせた 28 byte / 7 units とする。
- request encoding は opcode `18`、mode Replace、property `WM_PROTOCOLS`、type predefined `ATOM` id `4`、format `32`、data length `1`、data item `WM_DELETE_WINDOW` とする。
- `NativeWindowLinuxX11WmProtocolRegistrationWriteState` と partial write offset を reader に追加し、WaitingForAtoms / RequestPending / Ready / Failed を明示する。
- correlated `InternAtom` replies から completed Atom owner が返った時だけ registration request を構築する。
- setup-ready write path は `CreateWindow -> InternAtom batch -> InternAtom replies / Atom IDs -> ChangeProperty WM_PROTOCOLS -> MapWindow` の順序にする。
- registration request が accepted write boundary を越えるまで MapWindow を送らない。
- would-block は retryable として state を保持し、write failure / zero write / overflow / request build failure は Failed にして MapWindow を block する。
- registration request の accepted sequence を `NativeWindowLinuxX11WmProtocolRegistrationSequencePlan` に保持し、server error correlation に渡す。
- source-policy は registration surface が `ClientMessage` decode、runner dispatch、support gate `Ok` 化、fallback、silent no-op、synthetic readiness を含まないことを検査する。

非目標:

- `WM_DELETE_WINDOW` `ClientMessage` decode は含めない。
- StructureNotify / Expose decode、keyboard / IME、Wayland concrete decoding は含めない。
- Linux runner / CLI dispatch、support gate `Ok` 化は含めない。
- Atom ID fallback、default Atom、synthetic readiness、silent no-op は行わない。

完了条件:

- request owner は 28 byte / length 7 units の `ChangeProperty` request を protocol 通り encode する。
- completed Atom owner が無い状態では registration request を作らない。
- reader は registration accepted write 前に MapWindow を送らない。
- registration write failure は failed state と typed error を残し、MapWindow を block する。
- server error は top-level request correlation を優先し、registration sequence に一致する場合だけ registration error として扱う。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_wm_protocol -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `cargo test -p nepl-gui-native --lib -- --nocapture`
- `node --check nodesrc/test_native_gui_platform_behavior.js`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で 28 byte / 7 units encoding、no MapWindow before registration、fail-closed write state、no ClientMessage / no fallback が承認される。

## Phase F5ke: Native Linux X11 WM_DELETE_WINDOW ClientMessage decode boundary

Phase F5ke では、F5kd の `ChangeProperty` accepted boundary を越えた reader-owned registration context を使い、X11 `ClientMessage` event を `WM_DELETE_WINDOW` close request observation へ decode する。これは event packet decode boundary であり、runner dispatch や support gate `Ok` 化ではない。

実装:

- `NativeWindowLinuxX11RegisteredWmProtocolContext` を追加し、accepted registration に対応する window id と `WM_PROTOCOLS` / `WM_DELETE_WINDOW` Atom owner を保持する。
- reader は `wm_protocol_registration_write_state == Ready` の場合だけ `registered_wm_protocol_context()` から context を返す。request owner が存在しても accepted write boundary 前なら context は返さない。
- `ClientMessage` 判定は raw response byte ではなく `native_window_linux_x11_event_response_type(packet)` を使う。send-event bit 付き raw `33 | 0x80` も event type `33` として扱う。
- decode helper は format `32`、window id、message type Atom `WM_PROTOCOLS`、data32[0] `WM_DELETE_WINDOW` を registration context と照合する。
- mismatch は format / window / message type Atom / protocol Atom の typed error とする。
- registration context が無い `ClientMessage` は `ClientMessageWmProtocolNotRegistered` として fail closed にし、generic unsupported event や silent ignore へ落とさない。
- matching close は os close requested observation だけを返し、size と mouse state は previous input を保持する。
- source-policy は ClientMessage decode が accepted registration context に依存し、fallback、silent no-op、synthetic readiness、runner dispatch を含まないことを検査する。

非目標:

- StructureNotify / Expose decode、keyboard / IME、Wayland concrete decoding は含めない。
- Linux runner / CLI dispatch、support gate `Ok` 化は含めない。
- Atom fallback、default Atom、synthetic readiness、silent no-op は行わない。

完了条件:

- raw `33` と send-event bit 付き raw `161` の `ClientMessage` が同じ close decode path を通る。
- assigned atoms や registration request owner があっても、registration accepted context が無ければ close observation を返さない。
- format、window id、message type Atom、protocol Atom の mismatch が typed error になる。
- provider integration test で accepted registration 後の `WM_DELETE_WINDOW` `ClientMessage` が os close requested observation になる。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_client_message -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `cargo test -p nepl-gui-native --lib -- --nocapture`
- `node --check nodesrc/test_native_gui_platform_behavior.js`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で send-event bit masking、accepted registration context requirement、typed mismatch errors、no fallback が承認される。

## Phase F5kf: Native Linux X11 StructureNotify and Expose event evidence boundary

Phase F5kf では、X11 `StructureNotifyMask` と `ExposureMask` を top-level window default event mask に追加し、購読した event packet を platform-neutral な `NativeWindowEventPumpEventKind` へ写す。これは event evidence boundary であり、keyboard / IME、Wayland concrete decoding、Linux runner dispatch、support gate `Ok` 化ではない。

実装:

- `NativeWindowEventPumpEventKind` を追加し、`Poll`、`CloseRequested`、`WindowResized`、`PointerMotion`、`PointerButton`、`WindowMapped`、`RedrawRequested` を区別する。
- `NativeWindowEventPumpSnapshot`、`NativeWindowBackendLoopStepOutcome::Unavailable`、`NativeWindowBackendLoopDrawableStep`、`NativeWindowHostAction` に event kind を保持し、observation から backend loop / host action まで消さずに伝える。
- 既存の compatibility builder は close、resize、pointer button だけを推論し、pointer sample が存在するだけでは `PointerMotion` を合成しない。具体 backend は explicit builder で event kind を渡す。
- X11 event decode は `ConfigureNotify` を `WindowResized`、`MotionNotify` を `PointerMotion`、button press / release を `PointerButton`、`MapNotify` を `WindowMapped`、`Expose` を `RedrawRequested`、accepted `WM_DELETE_WINDOW` `ClientMessage` を `CloseRequested` として返す。
- `MapNotify` と `Expose` は previous size / mouse state を保持し、dummy resize、dummy pointer、blank redraw、silent no-op へ落とさない。
- source-policy は default event mask に StructureNotify / Exposure が含まれること、MapNotify / Expose decode が明示 event kind を持つこと、pointer sample だけで motion を合成しないことを検査する。

非目標:

- keyboard / IME decode は含めない。
- Wayland concrete event decoding は含めない。
- Linux runner / CLI dispatch、support gate `Ok` 化は含めない。
- fallback redraw、silent no-op、synthetic readiness は行わない。

完了条件:

- default event mask が `ButtonPress | ButtonRelease | PointerMotion | Exposure | StructureNotify` を含む。
- `ConfigureNotify`、`MapNotify`、`Expose` がそれぞれ `WindowResized`、`WindowMapped`、`RedrawRequested` として観測できる。
- `Expose` は send-event bit 付き packet でも event type を正規化して扱う。
- pointer sample availability だけでは `PointerMotion` を推論せず、X11 `MotionNotify` のような concrete source だけが explicit motion evidence を渡す。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_event_pump -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `cargo test -p nepl-gui-native --lib -- --nocapture`
- `node --check nodesrc/test_native_gui_platform_behavior.js`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で explicit event evidence、no inferred pointer motion、no silent no-op、no runner/support-gate scope creep が承認される。

## Phase F5kg: Native Linux X11 raw keyboard event evidence boundary

Phase F5kg では、X11 `KeyPressMask` と `KeyReleaseMask` を top-level window default event mask に追加し、購読した `KeyPress` / `KeyRelease` packet を raw keyboard event evidence として保持する。これは keyboard event evidence boundary であり、IME、text input、keysym / layout mapping、shortcut policy、Wayland keyboard decoding、Linux runner dispatch、support gate `Ok` 化ではない。

実装:

- `NativeWindowEventPumpEventKind` に `Keyboard` を追加し、keyboard event を pointer / resize / redraw と混同しない。
- `NativeWindowKeyboardEventKind` と `NativeWindowKeyboardEvent` を追加し、`Pressed` / `Released` と raw X11 keycode を保持する。
- `NativeWindowEventPumpSnapshot`、`NativeWindowBackendLoopStepOutcome::Unavailable`、`NativeWindowBackendLoopDrawableStep`、`NativeWindowHostAction` に optional keyboard event evidence を保持する。
- 既存 compatibility builder は keyboard event を推論せず `None` とする。X11 concrete decode だけが explicit builder で `Some NativeWindowKeyboardEvent` を渡す。
- X11 event decode は `KeyPress` を `Pressed`、`KeyRelease` を `Released` として返し、send-event bit は既存の `response_type & 0x7f` 正規化で扱う。
- raw keycode `0` は typed error として拒否し、unknown / empty keyboard event として silent success にしない。
- source-policy は KeyPress / KeyRelease mask、event type、raw key evidence propagation、zero keycode rejection、IME / keysym / shortcut / runner / fallback 非導入を検査する。

非目標:

- IME composition、text input、multi-scalar text は含めない。
- keysym / layout / modifier mapping、shortcut policy は含めない。
- Wayland concrete keyboard decoding は含めない。
- Linux runner / CLI dispatch、support gate `Ok` 化は含めない。
- fallback text、silent no-op、synthetic readiness は行わない。

完了条件:

- default event mask が `KeyPress | KeyRelease | ButtonPress | ButtonRelease | PointerMotion | Exposure | StructureNotify` を含む。
- `KeyPress` / `KeyRelease` が raw keycode 付き `Keyboard` event evidence として observation / snapshot / backend outcome / host action に残る。
- raw keycode `0` は typed observation error になる。
- host-loop wait decision は keyboard event を scheduler readiness や timer fired evidence に変換しない。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_keyboard -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_backend_loop_host_action_preserves_keyboard_evidence -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_ -- --nocapture`
- `cargo test -p nepl-gui-native --lib -- --nocapture`
- `node --check nodesrc/test_native_gui_platform_behavior.js`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で raw keyboard evidence、zero keycode typed error、no IME / keysym / runner / fallback scope creep が承認される。

## Phase F5kh: Native Linux X11 raw keyboard modifier evidence boundary

Phase F5kh では、F5kg の raw keyboard event evidence に X11 core event の `state` field を追加する。`state` は X11 の key/button mask bitset であり、NEPLg2 portable modifier、layout 済み key、text input、shortcut command へは変換しない。

実装:

- `NativeWindowKeyboardModifierState` を追加し、raw X11 `state` を `u16` の typed evidence として保持する。
- `NativeWindowKeyboardEvent` は `Pressed` / `Released`、raw X11 keycode、raw modifier state を持つ。
- `NativeWindowKeyboardEvent::new` は互換用に empty modifier state を使い、X11 concrete decode は modifier-aware constructor を使う。
- X11 `KeyPress` / `KeyRelease` decode は packet offset 28 の little-endian `u16` を raw state として読む。
- raw keycode `0` だけを typed error にし、raw state は全 `u16` 値を valid evidence として保持する。
- source-policy は raw state offset、modifier state wrapper、X11 decode、host action propagation、IME / keysym / portable modifier mapping 非導入を検査する。

非目標:

- portable modifier mapping、keysym / layout mapping、shortcut policy は含めない。
- IME composition、text input、multi-scalar text は含めない。
- Wayland concrete keyboard decoding は含めない。
- Linux runner / CLI dispatch、support gate `Ok` 化は含めない。
- fallback text、silent no-op、synthetic readiness は行わない。

完了条件:

- `KeyPress` / `KeyRelease` が raw keycode と raw modifier state を observation / snapshot / backend outcome / host action に残す。
- raw modifier state は X11 `state` の key/button mask raw evidence として document される。
- raw keycode `0` rejection は維持され、raw state の値で error にならない。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_keyboard -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_backend_loop_host_action_preserves_keyboard_evidence -- --nocapture`
- `cargo test -p nepl-gui-native --lib -- --nocapture`
- `cargo test -p nepl-gui-native --features window --lib -- --nocapture`
- `cargo check -p nepl-gui-native --target x86_64-unknown-linux-gnu`
- `node --check nodesrc/test_native_gui_platform_behavior.js`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で raw modifier evidence、no mapping、no fallback / no silent no-op scope creep が承認される。

## Phase F5ki: Native Linux X11 portable keyboard modifier evidence boundary

Phase F5ki では、F5kh の raw X11 `state` evidence から Shift / Control / Alt / Meta の portable modifier evidence だけを導出する。これは X11 core state bitset の固定 projection であり、keysym、layout 済み key、text input、IME composition、shortcut command へは変換しない。

実装:

- `NativeWindowPortableKeyboardModifiers` を追加し、Shift / Control / Alt / Meta を bool evidence として保持する。
- projection は `NativeWindowKeyboardModifierState` からだけ作る。raw state と portable modifiers を別々に受け取る public constructor は作らない。
- `NativeWindowKeyboardEvent::new_with_modifier_state` は raw state を保持したうえで portable modifier evidence を内部導出する。
- X11 core state の `ShiftMask = 0x0001`、`ControlMask = 0x0004`、`Mod1Mask = 0x0008`、`Mod4Mask = 0x0040` だけを portable projection に使う。
- Lock、Mod2、Mod3、Mod5、button mask、unknown high bit は portable modifier へ写さず、raw state にだけ保持する。
- source-policy は raw state preservation、portable projection bit、constructor consistency、KeySym / IME / shortcut / runner / fallback 非導入を検査する。

非目標:

- keysym / layout mapping、shortcut policy は含めない。
- IME composition、text input、multi-scalar text は含めない。
- Wayland concrete keyboard decoding は含めない。
- Linux runner / CLI dispatch、support gate `Ok` 化は含めない。
- fallback text、silent no-op、synthetic readiness は行わない。

完了条件:

- `KeyPress` / `KeyRelease` が raw modifier state と portable Shift / Control / Alt / Meta evidence の両方を host action まで残す。
- raw modifier state は全 `u16` 値を valid evidence として引き続き保持される。
- ignored X11 state bit が portable modifier evidence を誤って立てないことを test で固定する。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_keyboard -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_backend_loop_host_action_preserves_keyboard_evidence -- --nocapture`
- `cargo test -p nepl-gui-native --lib -- --nocapture`
- `cargo test -p nepl-gui-native --features window --lib -- --nocapture`
- `cargo check -p nepl-gui-native --target x86_64-unknown-linux-gnu`
- `node --check nodesrc/test_native_gui_platform_behavior.js`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で portable modifier evidence、raw state preservation、no KeySym / IME / shortcut / fallback scope creep が承認される。

## Phase F5kj: Native Linux Wayland raw message header evidence boundary

Phase F5kj では、caller supplied packet bytes から Wayland raw message header だけを typed evidence として読む。Wayland wire value は connection host byte order であり、portable file format の固定 endian ではないため、parser は `NativeWindowLinuxWaylandByteOrder` を明示入力に取る。header は 8 byte で、word 1 が object id、word 2 の上位 16 bit が message size、下位 16 bit が opcode である。

実装:

- `NativeWindowLinuxWaylandByteOrder`、`NativeWindowLinuxWaylandMessageHeader`、`NativeWindowLinuxWaylandMessageHeaderError` を追加する。
- parser は 8 byte 未満の packet、object id 0、size 8 未満、4 byte alignment 違反、declared size が supplied packet byte len を超えて packet 外を指す場合をそれぞれ typed error として返す。
- parser は payload の signature、object interface、xdg-shell semantic、keyboard、IME、text input へ進まない。
- parser は fd read / drain / close、Linux runner / CLI dispatch、support gate `Ok` 化、event queue、fallback、silent no-op、synthetic readiness を追加しない。
- source-policy は explicit byte order、header split、shape validation、no fd / no runner / no semantic decode を固定する。

非目標:

- Wayland event loop、xdg-shell semantic decode、keyboard / IME / text input decode は含めない。
- fd read / drain / close、selector registration、Linux runner / CLI dispatch、support gate `Ok` 化は含めない。
- fallback event、silent no-op、synthetic readiness は行わない。

完了条件:

- little endian / big endian の caller supplied packet が object id、opcode、message size、payload size evidence として読める。
- invalid shape が enum error として区別される。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_wayland_message_header -- --nocapture`
- `cargo test -p nepl-gui-native --lib -- --nocapture`
- `cargo test -p nepl-gui-native --features window --lib -- --nocapture`
- `cargo check -p nepl-gui-native --target x86_64-unknown-linux-gnu`
- `node --check nodesrc/test_native_gui_platform_behavior.js`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で Wayland header evidence、explicit byte order、no event loop / no semantic decode / no fallback scope creep が承認される。

## Phase F5kk: Native Linux X11 caller-supplied keysym value projection boundary

Phase F5kk では、caller supplied X11 keysym value を backend-local typed evidence として保持し、ごく狭い portable key evidence へ射影する。keysym は raw X11 keycode から layout / keymap を使って取得済みであるとはみなさない。この phase は「すでに caller が持っている raw keysym integer を分類する」だけであり、X11 event packet decode や keyboard layout query へは接続しない。

実装:

- `NativeWindowLinuxX11KeysymValue` を追加し、caller supplied raw keysym value を `u32` evidence として保持する。
- `NativeWindowPortableKey` を追加し、`NoSymbol`、`Unknown { raw_keysym }`、ASCII `0x20..0x7e`、Return、Escape、Tab、Backspace、Delete、arrow、Home / End、PageUp / PageDown だけを表す。
- `NativeWindowLinuxX11KeysymProjection` を追加し、raw keysym owner と portable projection を同時に保持する。
- projection は `NativeWindowLinuxX11KeysymValue` からだけ作る。raw value と portable key を別々に受け取る public constructor は作らない。
- X11 `NoSymbol = 0x0000` は `Unknown` ではなく `NoSymbol` として明示する。
- X11 named Delete は `0xffff` であり、ASCII DEL `0x007f` は portable `Delete` にしない。
- source-policy は projection helper が event packet decode から分離されていること、NoSymbol / Unknown が明示されていること、XLookupString / Xutf8LookupString / XmbLookupString / XKB / xkbcommon / keymap / runner / queue / fallback 非導入を検査する。

非目標:

- X11 keycode から keysym を取得する layout / keymap query は含めない。
- `XLookupString`、`Xutf8LookupString`、`XmbLookupString`、XKB、xkbcommon は呼ばない。
- `native_window_linux_x11_event_packet_to_observation` へは接続しない。
- IME composition、text input、shortcut policy、multi-scalar text は含めない。
- Wayland concrete keyboard decoding は含めない。
- Linux runner / CLI dispatch、support gate `Ok` 化は含めない。
- fallback key、silent no-op、synthetic readiness は行わない。

完了条件:

- caller supplied raw keysym value が projection 後も失われず保持される。
- `NoSymbol`、unknown raw value、ASCII printable range、navigation key の分類が test で固定される。
- ASCII DEL `0x007f` を X11 named Delete と誤分類しないことを test で固定する。
- event packet decode 関数に KeySym / keysym projection / text / shortcut / fallback policy が混入しないことを source-policy で固定する。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_keysym -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_keyboard -- --nocapture`
- `cargo test -p nepl-gui-native --lib -- --nocapture`
- `cargo test -p nepl-gui-native --features window --lib -- --nocapture`
- `cargo check -p nepl-gui-native --target x86_64-unknown-linux-gnu`
- `node --check nodesrc/test_native_gui_platform_behavior.js`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で keysym value projection、raw preservation、no event decode / no keymap / no fallback scope creep が承認される。

## Phase F5kl: Native Linux X11 GetKeyboardMapping request/reply owner boundary

Phase F5kl では、X11 core protocol の `GetKeyboardMapping` request bytes と reply raw keysym table を typed owner として扱う。これは F5kk の caller supplied raw keysym value projection より 1 段低い protocol shape boundary であり、keycode からどの keysym を選ぶか、modifier state をどう解釈するか、text input をどう生成するかは決めない。

実装:

- `NativeWindowLinuxX11GetKeyboardMappingRequest` を追加し、caller supplied `first_keycode`、`keycode_count`、encoded request bytes を保持する。
- opcode は `101`、request length `2` words / 8 byte とし、bytes は `opcode, unused, length, first-keycode, count, unused` の X11 wire order に合わせる。
- `first_keycode == 0` と `keycode_count == 0` は typed error として拒否する。setup reply の min-keycode / max-keycode を使う範囲検査は、後続の setup-owned keymap request phase に残す。
- `NativeWindowLinuxX11KeyboardMappingReplyHeader` を追加し、fixed 32 byte reply packet から `keysyms_per_keycode`、sequence、length units を読む。
- `NativeWindowLinuxX11KeyboardMappingRawKeysyms` を追加し、reply body の raw `KEYSYM` list を `NativeWindowLinuxX11KeysymValue` の列として保持する。
- reply body byte length は `length_units * 4` を checked arithmetic で求め、`keycode_count * keysyms_per_keycode * 4` と一致する場合だけ受理する。
- raw keysyms are not projected to `NativeWindowPortableKey` in F5kl; projection is an explicit later caller phase.
- source-policy は request byte owner、reply header、raw keysym table、checked length/count arithmetic、event decode / reader / fd IO / runner / queue / IME / text input / shortcut / fallback 非接続を検査する。

非目標:

- `native_window_linux_x11_event_packet_to_observation` へは接続しない。
- `NativeWindowLinuxX11EventSourceObservationReader` の state へは接続しない。
- raw fd write / read、reply correlation、pending keymap state、setup min-keycode / max-keycode range validation は含めない。
- `XLookupString`、`Xutf8LookupString`、`XmbLookupString`、XKB、xkbcommon は呼ばない。
- raw keysyms から `NativeWindowPortableKey` へ projection しない。
- IME composition、text input、shortcut policy、Wayland concrete keyboard decoding、Linux runner / CLI dispatch、support gate `Ok` 化は含めない。
- fallback key、silent no-op、synthetic readiness は行わない。

完了条件:

- `GetKeyboardMapping` request bytes が opcode `101`、request length `2` words / 8 byte、caller supplied first-keycode / count を保持する。
- malformed reply response type、zero `keysyms_per_keycode`、zero `keycode_count`、reply body length mismatch、expected keysym body length mismatch が typed error で区別される。
- reply body の raw keysym が `NativeWindowLinuxX11KeysymValue` として保持され、portable key projection へ進まないことを test / source-policy で固定する。
- event packet decode、reader state、fd IO、runner、queue、IME / text input、shortcut policy、fallback、support gate `Ok` 化には接続しないことを source-policy で固定する。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_get_keyboard_mapping -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_keyboard_mapping -- --nocapture`
- `cargo test -p nepl-gui-native --lib -- --nocapture`
- `cargo test -p nepl-gui-native --features window --lib -- --nocapture`
- `cargo check -p nepl-gui-native --target x86_64-unknown-linux-gnu`
- `node --check nodesrc/test_native_gui_platform_behavior.js`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で GetKeyboardMapping owner、raw keysym preservation、no event decode / no keymap / no fallback scope creep が承認される。

## Phase F5km: Native Linux X11 setup-owned keyboard mapping request boundary

Phase F5km では、X11 setup success response の `min-keycode` / `max-keycode` を setup resource info として保持し、その範囲から F5kl の `GetKeyboardMapping` request owner を導出する。これは protocol setup owner から request owner への純粋な変換であり、reader state、fd IO、reply correlation、keymap selection へは進まない。

実装:

- `NativeWindowLinuxX11SetupResourceInfo` に `min_keycode` / `max_keycode` を追加し、read-only accessor を公開する。
- setup success body parser は offset 26 / 27 を読み、`min_keycode < 8` と `max_keycode < min_keycode` を typed parse error として拒否する。
- `NativeWindowLinuxX11SetupKeyboardMappingRequest` を追加し、setup-owned range と F5kl `NativeWindowLinuxX11GetKeyboardMappingRequest` を同時に保持する。
- `keycode_count = max - min + 1` は `u16` へ拡張して `checked_sub` / `checked_add` / `u8::try_from` で計算し、`wrapping_*`、`saturating_*`、unchecked `as u8` は使わない。
- request bytes の最終 encoding は F5kl の `native_window_linux_x11_get_keyboard_mapping_request` に委譲し、F5kl に setup range validation を戻さない。
- source-policy は setup parser の offset / validation、F5km helper の checked arithmetic と F5kl builder delegation、event decode / reader / fd IO / reply / pending keymap / keymap selection / IME / text input / runner / queue / fallback 非接続を検査する。

非目標:

- `NativeWindowLinuxX11EventSourceObservationReader` の state へは接続しない。
- raw fd write / read、request sequence plan、reply correlation、pending keymap state は含めない。
- raw keysyms から `NativeWindowPortableKey` へ projection しない。
- keycode からどの keysym を選ぶか、modifier state と group / level をどう扱うかは決めない。
- IME composition、text input、shortcut policy、Wayland concrete keyboard decoding、Linux runner / CLI dispatch、support gate `Ok` 化は含めない。
- fallback key、silent no-op、synthetic readiness は行わない。

完了条件:

- setup success body の min/max keycode が `NativeWindowLinuxX11SetupResourceInfo` に保存される。
- `min_keycode < 8` と `max_keycode < min_keycode` が typed error として test で固定される。
- setup-owned `GetKeyboardMapping` request が `first-keycode = min-keycode`、`count = max - min + 1` を持つ。
- event packet decode、reader state、fd IO、runner、queue、IME / text input、shortcut policy、fallback、support gate `Ok` 化には接続しないことを source-policy で固定する。
- `cargo fmt -p nepl-gui-native -- --check`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_setup_resource_info -- --nocapture`
- `cargo test -p nepl-gui-native --lib native_window_linux_x11_setup_keyboard_mapping -- --nocapture`
- `cargo test -p nepl-gui-native --lib -- --nocapture`
- `cargo test -p nepl-gui-native --features window --lib -- --nocapture`
- `cargo check -p nepl-gui-native --target x86_64-unknown-linux-gnu`
- `node --check nodesrc/test_native_gui_platform_behavior.js`
- `node nodesrc/test_native_gui_platform_behavior.js`
- `git diff --check`
- subagent implementation review で setup-owned range validation、checked count derivation、no reader / no event decode / no fallback scope creep が承認される。
