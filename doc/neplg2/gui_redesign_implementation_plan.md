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

- scheduler loop は F5eg の `YieldToClock` / `AwaitTimerAdvance` / `ExecuteHostAction` / `Complete` action を明示的に進める必要がある。
- `YieldToClock` は F5ej の deterministic clock-delta authority によってだけ pending / ready を判断する必要がある。
- `WaitingTimer` は F5eh の `loop_timer_advance` または later real timer backend authority によってだけ再開する必要がある。
- `ExecuteHostAction` は F5ei の `loop_executor_complete` または later real backend executor authority が返す caller supplied outcome によってだけ再開する必要がある。
- `Complete` は F5ej の `loop_yield_complete_complete_ack` によって terminal payload へ明示的に変換する必要がある。
- slice policy は `YieldSlice` と timer schedule の契約を乱さず、FHD 60fps 目標に向けて bounded turn progress を表す必要がある。
- headless app-loop は presentation fallback ではなく、virtual event / virtual timer / offscreen snapshot を組み合わせた test target として扱う必要がある。
- 実装開始前に subagent review を通し、Required がある場合は doc を修正して再 review する。
