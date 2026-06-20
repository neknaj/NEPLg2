# NEPLg2 GUI/TUI 標準ライブラリ仕様

作成日: 2026-06-01

## 目的

NEPLg2 の GUI 標準ライブラリは、単一の GUI framework ではなく、Web Playground、native desktop、mobile、embedded、terminal UI を同じ application model へ接続する UI substrate として定義する。

この仕様では、GUI と TUI を別系統の library として育てない。TUI は text-cell surface を持つ backend であり、GUI と同じ event、capability、layout、application update、host effect の抽象に載せる。既存の `features/tui` / `platforms/wasix/tui` は、最終的にこの共通 substrate の terminal backend として再設計・再実装する。

## 設計原則

- 最下層は embedded を最低制約として扱い、heap allocation、OS、window system、clipboard、DOM、GPU、font shaping に依存しない。
- Web Playground は最初の実動 backend として扱う。
- native desktop と mobile は Web や embedded の派生ではなく、host interface を実装する別 backend として扱う。
- public standard API は `Canvas`、`DOM`、`UIKit`、`Android View`、`Win32`、`Wayland`、`Skia`、`SSD1306` などへ直接依存しない。
- TUI 固有 API も raw ANSI helper 集ではなく、text-cell render target、terminal host、keyboard/text input backend として GUI substrate に接続する。
- error と error display は分離し、失敗は `Option` / `Result` と enum で表す。
- callback-heavy widget を避け、widget は `ActionId` を持ち、application `update` が `GuiEvent` を `match` する。
- GUI/TUI の executable NEPLg2 code は括弧付き call に戻さない。stdlib implementation、`//:` doctest、`tests/stdlib/gui_*.n.md`、`examples/gui_*.nepl` では、nested call を中間 `let`、block、pipeline で分け、prefix expression の式境界を明示する。通常文の `O(1)`、WIT sketch、非 NEPL pseudo code の括弧はこの制約の対象外である。

## F5el real loop driver checkpoint

2026-06-18 の F5el では、std layer row tile RLE present host span operation presenter executor session turn virtual scheduler real loop driver boundary を追加する。`RealLoopDriverPolicy` は F5ef loop policy だけを保持し、F5ek step policy、scheduler policy、timer policy、backend executor、clock、queue を重複保持しない。`start` は F5ef `loop_step` と F5eg `loop_action_from_result` を 1 回ずつ呼び、`after_step` は F5ek result を `StateReady` / `YieldPending` / `Completed` として match する。`StateReady` は `loop_resume` へ戻し、`remaining_count == 0` は budget-yield semantics に従って yield action へ進め、error / completion / `CompleteAck` / fallback / silent no-op へ変換しない。

## F5em headless app-loop step checkpoint

2026-06-18 の F5em では、std layer row tile RLE present host span operation presenter executor session turn virtual scheduler headless app-loop step boundary を追加する。`HeadlessAppLoopStepPolicy` は F5el `RealLoopDriverPolicy` と F5ek `RealLoopStepPolicy` だけを保持し、F5ef loop policy、scheduler policy、timer policy、backend clock、executor backend、queue、platform API を直接保持しない。`start` は F5el `real_loop_driver_start` を 1 回だけ呼び、`advance` は previous `NeedInput` action と caller supplied F5ek input を受け、F5ek `real_loop_step` を 1 回、成功時だけ F5el `real_loop_driver_after_step` を 1 回呼ぶ。`Completed` は terminal output だけであり advance input ではない。`Complete` action は caller が `CompleteAck` を渡すまで `NeedInput` のまま保持し、F5em は ack を合成しない。`remaining_count == 0` は F5em で解釈せず、F5el / F5ec の budget-yield semantics に任せる。fallback と silent no-op は行わない。

## F5en bounded headless app-loop runner checkpoint

2026-06-18 の F5en では、std layer row tile RLE present host span operation presenter executor session turn virtual scheduler bounded headless app-loop runner boundary を追加する。これは fixed-slot script を使う deterministic test boundary であり、not long-running real backend loop である。`HeadlessAppLoopRunnerPolicy` は F5em `HeadlessAppLoopStepPolicy` と `max_advance_count` だけを保持し、F5ek / F5el の内部 policy、backend clock、executor backend、queue、platform API を保持しない。`HeadlessAppLoopRunnerScript` は 3 slot の `Option RealLoopStepInput`、`count`、`cursor` だけを保持し、slot hole、負 cursor、capacity 超過は `ScriptInvalid` として typed error にする。`InputMissing` は `NeedInput` に対する次 input が本当に存在しない場合だけ返し、`ClockDelta`、`ExecutorOutcome`、`CompleteAck` を合成しない。`BudgetExhausted` は `max_advance_count == 0` または bounded drain の budget を使い切った場合の terminal result であり、F5em `advance` を呼ばない。`Completed` は script を消費しない。fallback と silent no-op は行わない。

## F5eo backend clock delta checkpoint

2026-06-18 の F5eo では、std layer row tile RLE present host span operation presenter executor session turn virtual scheduler backend clock delta boundary を追加する。これは Web / native / bare / headless backend が取得した monotonic clock sample を、F5ek `RealLoopStepInput::ClockDelta` へ変換する pure std boundary である。`BackendClockPolicy` は `max_delta_ms` だけを保持し、`BackendClockSample` は caller supplied `monotonic_ms`、`BackendClockState` は previous `last_monotonic_ms` だけを保持する。sample / state は public value なので、`start` と `advance` は constructor を信用せず entry で再検査する。`start` は baseline state を返し delta を発行しない。`advance` は negative policy、negative sample、forged negative state、backward time、too-large delta を typed error として返し、error payload は policy / state / sample / previous / current / delta / max を回収可能な形で保持する。zero delta は no-op や error にせず `ClockDelta 0` として返す。delta が `max_delta_ms` を超えた場合は clamp せず `DeltaTooLarge` を返す。F5eo は actual clock source、sleep、timer backend、executor outcome、complete ack、queue、platform API、DOM、Canvas、minifb、video memory、fallback、silent no-op を実装しない。

## F5ep Web monotonic clock source checkpoint

2026-06-18 の F5ep では、Web formal monotonic clock source backend boundary を追加する。`platforms/gui/web/clock` は `nepl_gui_web.monotonic_clock_ms` の単一 `i32` return ABI を受け、0 以上を `performance.now` 由来の floored millisecond sample、-1 を unsupported、その他の負値を `BackendFailure` として扱う。Web worker は `performance.now` を呼んだ後、`Number.isFinite`、0 以上、`i32::MAX` 以下、integer 化後の妥当性を検査してから Wasm 境界へ返す。`i32::MAX` ms を超えた sample は wrap や clamp ではなく `BackendFailure` である。NEPL wrapper は negative sentinel を `GuiError` へ写した後だけ F5eo `BackendClockSample` constructor を呼ぶ。`Date.now`、`setTimeout`、`setInterval`、stdout protocol、polling loop、queue、DOM、Canvas、fallback、silent no-op は clock source として使わない。native / bare / headless の actual clock source は後続 slice で実装する。

## F5eq Headless scripted monotonic clock source checkpoint

2026-06-18 の F5eq では、Headless scripted monotonic clock source backend boundary を追加する。`platforms/gui/headless/clock` は deterministic headless / offscreen test 用の actual clock input source であり、wall clock ではなく fixed-slot script から F5eo `BackendClockSample` を 1 件ずつ返す。script は `Option BackendClockSample` の 3 slot、`count`、`cursor` だけを保持し、`count` は 0 から 3、`cursor` は 0 から `count`、slot は count に一致する `Some` / `None` shape でなければならない。constructor は raw i32 sample を F5eo constructor で検査してから保持し、poll も public script を信用せず count / cursor / slot shape / sample を再検査する。`cursor == count` は `Option::None` を返し、zero sample や delta を合成しない。timer、queue、host import、platform API、wall clock、fallback、silent no-op は使わない。native / bare actual clock source と long-running backend loop は後続 slice で実装する。

## F5er Native formal monotonic clock source checkpoint

2026-06-18 の F5er では、Native formal monotonic clock source backend boundary を追加する。`platforms/gui/native/clock` は `nepl_gui_native.monotonic_clock_ms` の単一 `i32` return ABI を受け、0 以上を native `Instant` 由来の monotonic millisecond sample、-1 を unsupported、その他の負値を `BackendFailure` として扱う。Rust `nepl-gui-native` 側は elapsed millisecond を `i32::MAX` 以下で検査し、超過は wrap や clamp ではなく backend failure sentinel にする。NEPL wrapper は negative sentinel を `GuiError` へ写した後だけ F5eo `BackendClockSample` constructor を呼ぶ。timer、sleep、queue、window loop、present、scheduler backend、minifb rendering、stdout protocol、fallback、silent no-op は clock source として使わない。bare actual clock source、native / bare scheduler backend、long-running real backend loop は後続 slice で実装する。

## F5es Bare formal monotonic clock source checkpoint

2026-06-18 の F5es では、Bare formal monotonic clock source backend boundary を追加する。`platforms/gui/bare/clock` は `nepl_gui_bare.monotonic_clock_ms` の単一 `i32` return ABI を受け、0 以上を embedding host が明示提供する monotonic millisecond sample、-1 を `Unsupported`、その他の負値を `BackendFailure` として扱う。Bare stdlib は universal wall clock を仮定せず、Web `performance.now`、native `Instant`、wall clock、timer、sleep、queue、window loop、present、scheduler backend、minifb rendering、stdout protocol、fallback、silent no-op を clock source として使わない。`nodesrc/run_test.js` の `nepl_gui_bare` 既定 import は doctest-only unsupported source であり hidden fallback や hidden mock ではない。native / bare scheduler backend、long-running real backend loop は後続 slice で実装する。

## F5et Native and Bare scheduler clock one-tick helper boundary

2026-06-18 の F5et では、Native and Bare scheduler clock one-tick helper boundary を追加する。これは not long-running scheduler backend であり、platform clock source が返す sample を F5eo `BackendClockPolicy` / `BackendClockState` と組み合わせて 1 tick 分だけ処理する境界である。`start` は F5eo `backend_clock_start`、`tick` は F5eo `backend_clock_advance` を authority とし、`ClockDelta` を直接合成しない。成功時の tick は F5eo `BackendClockAdvance` を返す。sample failure は policy、tick では state も保持する typed error とし、unsupported や backend failure を fallback や silent no-op に変換しない。timer、sleep、queue、while loop、present、minifb、Canvas、video memory は扱わない。

## F5eu Native and Bare scheduler clock action input helper boundary

2026-06-18 の F5eu では、Native and Bare scheduler clock action input helper boundary を追加する。これは action input helper only であり、not long-running scheduler backend である。Native / bare backend は F5eg `YieldToClock` / `AwaitTimerAdvance` typed payload だけを受け、F5et の one-tick helper を 1 回呼び、その F5eo `BackendClockAdvance` から `BackendClockState` と F5ek `RealLoopStepInput` を success payload に保存する。error payload は original action、input clock state、lower `GuiNativeSchedulerClockError` / `GuiBareSchedulerClockError` を保持する。`ExecuteHostAction` / `Complete` / `ExecutorOutcome` / `CompleteAck` / real loop driver / headless app-loop step には進まない。fallback と silent no-op は禁止であり、unsupported clock source は lower error を持つ `Result` として返す。

## F5ev Native and Bare scheduler executor outcome input helper boundary

2026-06-18 の F5ev では、Native and Bare scheduler executor outcome input helper boundary を追加する。これは backend-facing input boundary であり、not long-running scheduler backend である。Native / bare backend は F5eg `ExecuteHostAction` typed payload と caller supplied `Result unit GuiError` outcome だけを受け、F5ek `RealLoopStepInput::ExecutorOutcome` へ total packaging する。ready payload は original action と `RealLoopStepInput` を保持し、helper は does not return Result である。unsupported path は型で除外されるため silent no-op や fallback branch を持たない。`YieldToClock` / `AwaitTimerAdvance` / `Complete` / `ClockDelta` / `CompleteAck` / F5ei executor complete / F5ek real loop step / action sink / driver / support validation / queue / timer / present / platform renderer には進まない。

## F5fg Native presenter operation identity input boundary

2026-06-18 の F5fg では、Native presenter operation identity input boundary を追加する。これは presenter-facing input boundary であり、not long-running scheduler backend である。F5ev is the scheduler step input boundary; F5fg is the native presenter-facing identity input boundary. F5fg は typed `ExecuteHostAction` だけを受け、action を F5ev ready payload へ移す前に borrowed accessor で pending span operation identity を取り出す。

operation identity は F5cx `GuiRgba8888RowTileRlePresentHostSpanOperation` として保持し、`WindowBegin`、`WindowRunSpan`、`WindowEnd`、`OffscreenBegin`、`OffscreenRunSpan`、`OffscreenEnd`、`DeviceBegin`、`DeviceRunSpan`、`DeviceEnd` を型付き value のまま失わない。scheduler completion input は `gui_native_scheduler_executor_input` を 1 回だけ呼んで作り、F5fg 自身は `RealLoopStepInput::ExecutorOutcome` を再実装しない。backend execution、raw status mapping、scheduler step、window loop、queue、timer、minifb、Canvas、DOM、video memory、fallback、silent no-op は扱わない。

## F5fh Native formal presenter session boundary

2026-06-18 の F5fh では、Native formal presenter session boundary を `nepl-gui-native` に追加する。これは Rust lib-only boundary であり、`NativeWindowPresenterSession` が `NativeRgb0PresenterSink` と `NativeWindowPresenterState` を所有する。session public entry は typed `NativeSpanOperation` だけを受け、`NativeWindowPresenterSessionOutcome` / `NativeWindowPresenterSessionError` を返す。

Begin / RunSpan 成功は `NativeWindowPresenterSessionOutcome::NotPresented` であり、window presenter state を更新しない。End 成功だけが completed RGB0 frame と frame id を使って `NativeWindowPresenterState::present_sink_frame` を呼び、`NativeWindowPresenterSessionOutcome::Presented` を返す。sink failure は `NativeWindowPresenterSessionError::SinkFailed`、presenter failure は `NativeWindowPresenterSessionError::PresenterFailed` として分ける。failure は previous presenter frame id / dimensions / pixels を保持し、blank frame 合成、stretch、crop、fallback、silent no-op を行わない。

F5fh は formal native window presenter integration への lib boundary であり、minifb、OS window loop、actual scheduler backend、timer、queue、stdout protocol、Canvas、DOM、video memory host import は持たない。F5fg の presenter operation identity input と、F5ex/F5ey 由来の span operation execution path を後続でこの session boundary へ接続する。

## F5fi Native presenter session host helper boundary

2026-06-18 の F5fi では、F5ey の scalar span operation host ABI validator と F5fh の `NativeWindowPresenterSession` を接続する Rust lib-only helper boundary を追加する。`execute_native_window_presenter_session_begin`、`execute_native_window_presenter_session_run`、`execute_native_window_presenter_session_end` は existing scalar validation を通した後だけ typed `NativeSpanOperation` を session へ渡す。

validation failure は `NativeWindowPresenterSessionHostError::ValidationFailed NativeSpanOperationStatus` として返し、session / sink / presenter state を変更しない。session execution failure は `NativeWindowPresenterSessionHostError::SessionFailed` として返し、lower `NativeWindowPresenterSessionError::SinkFailed` / `PresenterFailed` を保つ。Begin / RunSpan の success は `NotPresented`、End success だけが `Presented { frame_id, width, height }` になる。

F5fi は long-running scheduler backend、queue、timer wait、minifb loop、bare runtime host import、formal NEPL `#extern` import 名の差し替え、Canvas、DOM、video memory host import を持たない。raw status projection は outer ABI のための `status` helper に閉じ、内部 contract は enum / `Result` で保持する。

## F5fj Native presenter session host import boundary

2026-06-18 の F5fj では、native `platforms/gui/native/scheduler_host_executor` の formal NEPL host import ABI を F5fi の Rust `NativeWindowPresenterSession` helper contract へ接続する。native host import 名は `window_presenter_session_begin`、`window_presenter_session_run`、`window_presenter_session_end` とし、generic `execute_span_operation_*` は native public import contract として出さない。

F5fj は existing scalar ABI shape を保ち、status `0` を `Result::Ok unit`、`-1` を `GuiError::Unsupported`、その他の negative sentinel を typed `GuiError` へ写す。host が session import を提供しない doctest / CLI-only runtime では default stub が `-1` を返すため、fallback、silent no-op、blank frame 合成、old stdout transport へ落ちない。

F5fj は native NEPL `#extern` boundary だけを扱う。bare runtime host import、long-running scheduler backend、queue、timer wait、minifb loop、formal `std/gui` host implementation、Canvas、DOM、video memory host import、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は後続 slice に分ける。

## F5fk Bare display presenter session host import boundary

2026-06-18 の F5fk では、bare `platforms/gui/bare/scheduler_host_executor` の formal NEPL host import ABI を bare display presenter session contract へ接続する。bare host import 名は `display_presenter_session_begin`、`display_presenter_session_run`、`display_presenter_session_end` とし、generic `execute_span_operation_*` は bare public import contract として出さない。

Bare は window manager を持つとは限らないため、native の `window_presenter_session_*` とは異なり、device / offscreen / display surface へ接続される presenter session として扱う。scalar ABI shape と status mapping は F5ex と同じであり、status `0` は `Result::Ok unit`、`-1` は `GuiError::Unsupported`、その他の negative sentinel は typed `GuiError` へ写す。host が display presenter session import を提供しない doctest / CLI-only runtime では default stub が `-1` を返し、fallback、silent no-op、blank frame 合成、old stdout transport へ落ちない。

F5fk は bare NEPL `#extern` boundary だけを扱う。bare actual display driver、framebuffer adapter、polling input、long-running scheduler backend、queue、timer wait、present loop、formal `std/gui` host implementation、Canvas、DOM、video memory host import、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は後続 slice に分ける。

## F5fl Bare display framebuffer adapter boundary

2026-06-18 の F5fl では、bare `platforms/gui/bare/framebuffer` を追加する。これは actual display driver ではなく、F5fk の existing bare scheduler host executor へ operation を渡す前の pure validation state machine である。`GuiBareFramebufferConfig`、`GuiBareFramebufferState`、`GuiBareFramebufferPhase`、`GuiBareFramebufferErrorKind` を持ち、Begin / RunSpan / End の順序、target、surface、frame、shape、row-major progress、incomplete end を `Result` と enum で明示する。Begin descriptor と RunSpan / End 前の active descriptor は `std/gui/tile_present` の descriptor contract と同じく、frame id 一致、positive geometry / counts、plan row extent、tile row extent、stride、tile count / index、pixel count、encoded byte count を host execution 前に再検査する。active state の `seen_run_count` / `seen_pixel_count` は non-negative かつ descriptor count 以下でなければならず、public state が偽造されても host executor には進まない。

`gui_bare_framebuffer_validate_operation` は host import を呼ばないため、headless / CLI-only doctest で検査できる。`gui_bare_framebuffer_execute_operation` は validation 成功後だけ existing bare scheduler host executor を 1 回呼び、host failure は `HostExecutionFailed` として original state と operation を保持する。validation failure は host call へ進まず、wrapper は advanced state を採用しない。F5fl は not long-running scheduler backend であり、actual display storage、timer queue、present loop、formal `std/gui` host implementation、Canvas、DOM、video memory host import、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization、fallback、silent no-op は後続 slice に分ける。

## F5fm Bare display storage adapter boundary

2026-06-19 の F5fm では、bare `platforms/gui/bare/display_storage` を追加する。これは actual display driver ではなく、F5fl の validation result を display storage が消費する typed effect ledger へ変換する境界である。`GuiBareDisplayStorageState` は canonical framebuffer state、storage phase、last presented frame を保持し、`GuiBareDisplayStorageEffect` は `FrameBegin`、`SpanWrite`、`FramePresent` を表す。

`GuiBareFramebufferStepApplied` は public value なので、storage adapter は supplied next state をそのまま信用しない。`gui_bare_display_storage_apply` はまず storage state invariant を検査し、storage が保持する canonical framebuffer state から operation を `gui_bare_framebuffer_validate_operation` で再検証する。再検証で得た expected next framebuffer state と supplied next framebuffer state が一致しない場合は `AppliedStateMismatch` を返す。replay、stale applied、別 surface / frame / target、RunSpan before Begin、Incomplete End、frame mismatch は F5fl の enum error を `FramebufferValidationFailed` として保持する。

storage phase は framebuffer phase と target、descriptor、accepted run count、accepted pixel count が一致しなければならない。偽造された storage state は `StoragePhaseMismatch`、`TargetMismatch`、`DescriptorMismatch`、`AcceptedRunCountMismatch`、`AcceptedPixelCountMismatch` として fail-closed に拒否する。F5fm は raw memory、actual display driver、host import、long-running scheduler backend、timer queue、present loop、Canvas、DOM、minifb、video memory host import、fallback、silent no-op を実装しない。actual display driver は F5fm の typed effect ledger を消費する後続 slice とする。

## F5fn Bare display memory write plan boundary

2026-06-19 の F5fn では、bare `platforms/gui/bare/display_memory` を追加する。これは actual display driver そのものではなく、F5fm の typed effect ledger を actual bare display driver が消費できる checked byte write plan へ変換する境界である。`GuiBareDisplayMemoryState` は canonical storage state として `GuiBareDisplayStorageState` を保持し、public value である `GuiBareDisplayStorageStepApplied` や `GuiBareDisplayStorageEffect` をそのまま信用しない。

`gui_bare_display_memory_apply` は、memory state が保持する canonical storage state から supplied storage step の framebuffer step を `gui_bare_display_storage_apply` で再適用する。expected storage state と supplied storage state が一致しない場合は `StorageStepStateMismatch`、expected effect と supplied effect が一致しない場合は `StorageStepEffectMismatch` を返す。これにより stale / forged storage step は actual driver 直前の byte plan に進めない。

span write では、config と span から `height * stride_bytes`、`y * stride_bytes`、`x * 4`、`width * 4`、`byte_start`、`byte_end` を全て checked arithmetic で計算する。negative geometry、overflow、`byte_end > surface_byte_count` は `SpanXInvalid`、`SpanYInvalid`、`SpanWidthInvalid`、`RowByteOffsetOverflow`、`XByteOffsetOverflow`、`SpanByteLengthOverflow`、`ByteStartOverflow`、`ByteEndOverflow`、`ByteOutOfBounds` などの enum error と `Result` で fail-closed に拒否する。present は storage present effect と descriptor の expected run / pixel count が一致する complete-frame evidence に基づく場合だけ `FramePresent` action へ変換する。

F5fn は raw byte buffer ownership、actual display driver write、host import、long-running scheduler backend、timer queue、present loop、Canvas、DOM、minifb、video memory host import、fallback、silent no-op を実装しない。後続 slice では F5fn の checked byte write plan を消費する actual driver adapter と、native / bare long-running backend loop へ進む。

## F5fo Bare display driver outcome ledger boundary

2026-06-19 の F5fo では、bare `platforms/gui/bare/display_driver` を追加する。これは actual hardware display driver そのものではなく、F5fn の checked byte write plan と caller supplied driver outcome を照合する pure ledger boundary である。`GuiBareDisplayDriverState` は canonical memory state として `GuiBareDisplayMemoryState` を保持し、public value である `GuiBareDisplayMemoryStepApplied` をそのまま信用しない。

`gui_bare_display_driver_apply` は driver state が保持する canonical memory state から supplied memory step の storage step を `gui_bare_display_memory_apply` で再適用する。expected memory state と supplied memory state が一致しない場合は `MemoryStepStateMismatch`、expected action と supplied action が一致しない場合は `MemoryStepActionMismatch` を返す。これにより stale / forged memory step は driver outcome matching へ進めない。

driver outcome は `BeginAccepted`、`SpanWriteAccepted`、`FramePresentAccepted`、`DriverRejected` の enum で表す。`BeginAccepted` は target / descriptor / surface byte count、`SpanWriteAccepted` は target / span / run index / pixel range / row byte start / x byte offset / byte_start / byte_len / byte_end / surface byte count / color、`FramePresentAccepted` は target / descriptor / frame / run count / pixel count / surface byte count を持つ。selected memory action と outcome variant が合わない場合は `OutcomeActionMismatch`、payload が一致しない場合は field ごとの mismatch error を返す。`DriverRejected` は lower `GuiError` を保持し、fallback や silent no-op へ変換しない。

F5fo は raw byte buffer ownership、actual hardware write、host import、long-running scheduler backend、timer queue、present loop、Canvas、DOM、minifb、video memory host import、fallback、silent no-op を実装しない。後続 slice では F5fo の driver outcome ledger を実際の bare display driver adapter または formal host import と接続する。

## F5fp Bare display driver host import boundary

2026-06-19 の F5fp では、bare `platforms/gui/bare/display_driver_host_import` を追加する。これは F5fo の pure driver outcome ledger を、embedding host が提供する actual bare display driver import へ接続する backend boundary である。host import 名は `display_driver_begin`、`display_driver_span_write`、`display_driver_frame_present` とし、F5fk の `display_presenter_session_*` とは別の raw display driver contract として扱う。

`gui_bare_display_driver_host_import_step` は、caller supplied `GuiBareDisplayMemoryStepApplied` を host import へ渡す前に F5fo ledger で preflight する。preflight は driver state の canonical memory state から supplied memory step を再適用し、expected state / action と supplied state / action を照合する。preflight が失敗した場合は host import を呼ばない。preflight が成功した場合だけ、preflight で得た action variant に対応する import を 1 回だけ呼び、raw status を `GuiBareDisplayDriverOutcome` へ写してから F5fo ledger へ再適用する。

status `0` だけを accepted outcome として扱う。`-1` は `GuiError::Unsupported`、既知の負 status は `InvalidCommand` または `ResourceExhausted`、未知の負 status と positive non-zero status は `GuiError::BackendFailure` として `DriverRejected` に変換する。success outcome は host import が accepted status を返し、F5fo ledger が同じ action/outcome を受け入れたことを示すだけである。raw byte buffer を読み戻して actual bytes を検証した証明ではないため、byte-readable framebuffer ownership や echo evidence は後続 slice とする。

`display_driver_span_write` には x / y / width / height だけでなく、F5fn が checked arithmetic で計算した run index、pixel start / end、row byte start、x byte offset、byte_start、byte_len、byte_end、surface byte count、RGBA8888 color evidence を渡す。`display_driver_begin` と `display_driver_frame_present` は descriptor の surface / frame / packet metadata と surface byte count を渡す。これにより host 側は same app code の checked action evidence を使って actual device / offscreen display に反映できる。

F5fp は long-running scheduler backend、timer queue、present loop、polling input、raw byte buffer ownership、DOM、Canvas、minifb、video memory host import、fallback、silent no-op を実装しない。host が import を提供しない doctest / CLI-only runtime では `nodesrc/run_test.js` の default stub が `-1` を返し、`Unsupported` として検出される。

## F5fq Bare display driver byte echo verification boundary

2026-06-19 の F5fq では、bare `platforms/gui/bare/display_driver_byte_echo` を追加する。これは F5fp の host import accepted status の後に host が返す単一 byte echo を、F5fn/F5fo の checked span write evidence と照合する verification boundary である。

`GuiBareDisplayDriverStepApplied` は public Copy value として偽造できるため、F5fq の public entry は supplied driver step を受け取らない。`gui_bare_display_driver_byte_echo_verify` は `GuiBareDisplayDriverState`、`GuiBareDisplayMemoryStepApplied`、`GuiBareDisplayDriverOutcome`、`GuiBareDisplayDriverByteEcho` を受け取り、内部で必ず `gui_bare_display_driver_apply` を呼ぶ。F5fo ledger が stale / forged memory step、state mismatch、action mismatch、driver outcome mismatch を検出した場合は `DriverStepInvalid %GuiBareDisplayDriverErrorKind` として `Result` error を返し、byte extraction へ進まない。

ledger が成功した後、F5fq は canonical step の action と outcome が `SpanWrite` / `SpanWriteAccepted` であることを確認する。Begin / Present は `NonSpanWriteAction` または `NonSpanWriteOutcome` として fail-closed に拒否する。`GuiBareDisplayDriverByteEcho` は `byte_index` と `value` を持ち、`value` は 0..255、`byte_index` は accepted span の `byte_start <= index < byte_end` を満たさなければならない。relative byte offset は `rem_s relative 4` で計算し、0 / 1 / 2 / 3 を `Red` / `Green` / `Blue` / `Alpha` の typed channel enum に写す。期待値は accepted `Rgba8888` の corresponding channel から取り出す。echo value が一致しない場合は `EchoValueMismatch` を返し、silent no-op にはしない。

F5fq は raw display memory ownership、full byte buffer readback、actual hardware write、host import、long-running scheduler backend、timer queue、present loop、polling input、DOM、Canvas、minifb、video memory host import、fallback、silent no-op を実装しない。後続 slice では F5fq の byte echo verifier を raw display memory ownership / actual display driver adapter と接続し、単一 byte echo から bulk verification へ拡張する。

## F5fr Bare raw display memory ownership boundary

2026-06-19 の F5fr では、bare `platforms/gui/bare/display_memory_owner` を追加する。これは F5fq の byte echo verifier を、bare display backend が所有する raw RGBA8888 byte memory に接続する ownership boundary である。`GuiBareDisplayMemoryOwner` は canonical `GuiBareDisplayDriverState`、`surface_byte_count`、private `RegionToken u8`、`verified_byte_count`、直近の `GuiBareDisplayDriverByteEchoVerified` を保持する。ただし public `GuiBareDisplayDriverByteEchoVerified` は Copy value として偽造可能なので、F5fr の public write API はこれを権威として受け取らない。

`gui_bare_display_memory_owner_write_echo` は `GuiBareDisplayMemoryOwner`、`GuiBareDisplayMemoryStepApplied`、`GuiBareDisplayDriverOutcome`、`GuiBareDisplayDriverByteEcho` を受ける。まず owner 内の canonical driver state から `gui_bare_display_driver_byte_echo_verify` を呼び、F5fo/F5fq の再検証を通過した場合だけ raw memory write に進む。write は exact echoed byte だけを `RegionToken u8` に store し、同じ byte index を load して expected value と一致することを確認した後にだけ owner の driver state と verified byte count を進める。store / readback 前に state を進めることは禁止する。

F5fr の owner と owner-bearing write error は `Clone` / `Copy` を実装しない。write failure は input owner を error payload に保持し、caller が recovery / free / retry policy を選べるようにする。read API は checked single-byte readback だけを提供し、raw pointer、storage accessor、bulk slice、full span readiness、full frame readiness は公開しない。1 byte echo は exact echoed byte 1 個だけの evidence であり、span 全体や frame 全体の検証完了とはみなさない。

F5fr は actual hardware driver、host import、long-running scheduler backend、timer queue、present loop、polling input、DOM、Canvas、minifb、video memory transport、zero-fill fallback、silent no-op、bulk readback を実装しない。後続 slice では owner を actual display driver adapter と接続し、bulk byte readback と frame readiness を別の typed evidence として追加する。

## F5fs Bare display memory span write/readback boundary

2026-06-19 の F5fs では、bare `platforms/gui/bare/display_memory_span_readback` を追加する。これは F5fr の `GuiBareDisplayMemoryOwner` が保持する raw RGBA8888 memory に対して、canonical `SpanWrite` step の全 byte を store し、同じ byte range を readback して expected RGBA8888 channel value と一致することを確認する owner-side boundary である。F5fs は actual hardware driver adapter ではなく、frame readiness evidence でもない。

F5fs の public entry は `gui_bare_display_memory_owner_write_span_readback owner memory_step outcome` とする。`GuiBareDisplayDriverSpanWriteAccepted` や `GuiBareDisplayDriverByteEchoVerified` は public Copy value として偽造できるため、public entry の authority にはしない。entry は owner 内の canonical `GuiBareDisplayDriverState` から `gui_bare_display_driver_apply` を再実行し、returned step の action / outcome が `GuiBareDisplayMemoryAction::SpanWrite` / `GuiBareDisplayDriverOutcome::SpanWriteAccepted` である場合だけ span memory write へ進む。Begin / Present / DriverRejected は enum error として fail-closed に返す。

F5fs は canonical accepted span から `byte_start`、`byte_len`、`byte_end`、`surface_byte_count`、`Rgba8888` color を取り出す。`byte_start >= 0`、`byte_len > 0`、`byte_start + byte_len == byte_end`、`byte_end <= owner.surface_byte_count`、accepted surface byte count と owner surface byte count の一致を検査する。span write loop は byte index ごとに `relative % 4` を `Red` / `Green` / `Blue` / `Alpha` へ写し、expected channel value を owner 内 `RegionToken u8` へ store する。store loop が成功した後に readback loop を実行し、全 byte が expected value と一致した場合だけ owner の driver state と verified byte count を進める。state advance は full store と full readback の後でなければならない。

成功 payload `GuiBareDisplayMemoryOwnerSpanReadbackCompleted` は owner と pure `GuiBareDisplayMemoryOwnerSpanReadbackEvidence` を保持する。owner-bearing success / error は `Clone` / `Copy` を実装しない。pure evidence は write / read authority を持たない metadata であり、単独では authority にしない。F5fs 成功時は single byte echo evidence を `Option::None` へ clear し、stale echo evidence が span readback を意味するように見えないようにする。

F5fs は public `RegionToken` / `MemPtr` / storage accessor、raw pointer、raw byte slice、actual hardware driver、host import、long-running scheduler backend、timer queue、present loop、DOM、Canvas、minifb、video memory transport、fallback、silent no-op、frame ready / present ready evidence を実装しない。frame readiness は future slice で、all spans / present command / scheduler completion の別 evidence として扱う。

## F5ft Bare actual display driver adapter boundary

2026-06-19 の F5ft では、bare `platforms/gui/bare/display_driver_adapter` を追加する。これは F5fp の actual bare display driver host import と、F5fs の owner-side span write/readback を接続する adapter boundary である。F5ft の public entry は `gui_bare_display_driver_adapter_step owner memory_step` であり、caller supplied `GuiBareDisplayDriverStepApplied`、`GuiBareDisplayDriverOutcome`、`GuiBareDisplayDriverSpanWriteAccepted`、`GuiBareDisplayDriverByteEchoVerified` は authority として受け取らない。

adapter step はまず owner 内の canonical `GuiBareDisplayDriverState` を取り出し、`gui_bare_display_driver_host_import_step driver_state memory_step` を呼ぶ。これにより F5fo ledger preflight、actual host import 1 回、status outcome 変換、F5fo ledger reapply が通った returned `GuiBareDisplayDriverStepApplied` だけを host accepted ledger authority とする。host import failure は owner-bearing `GuiBareDisplayDriverAdapterError` として返し、owner state は進めない。

returned step の action / outcome は enum `match` で明示的に分ける。`SpanWrite` / `SpanWriteAccepted` の場合だけ、returned outcome を `gui_bare_display_memory_owner_write_span_readback owner memory_step outcome` へ渡す。F5fs が full store と full readback を完了した場合だけ、adapter success は next owner と `SpanHostAcceptedAndReadback` evidence を返す。span readback failure after host accepted は owner を unadvanced state として error payload に回収する。ただし external host side effect は rollback された証明ではなく、verified owner memory としても扱わない。

`FrameBegin` / `BeginAccepted` と `FramePresent` / `FramePresentAccepted` は raw memory store を行わず、returned step の next driver state を owner に反映し、storage と verified byte count を保持し、stale single-byte evidence を `Option::None` へ clear する。これらの success kind は `BeginHostAccepted` / `FramePresentHostAccepted` であり、frame ready、present ready、byte readback evidence ではない。returned step に `DriverRejected` や action/outcome mismatch が現れた場合は adapter error として fail-closed に返す。

F5ft の owner-bearing success / error は `Clone` / `Copy` を実装しない。pure evidence は copyable metadata であり、それ単独では write / read authority にならない。F5ft は public raw storage accessor、scheduler loop、queue、timer backend、DOM、Canvas、minifb、video memory transport、fallback、silent no-op、frame readiness evidence を実装しない。

## F5fu Bare presented packet readiness evidence boundary

2026-06-19 の F5fu では、bare `platforms/gui/bare/display_present_readiness` を追加する。これは F5ft の `GuiBareDisplayDriverAdapterCompleted` を value として消費し、`FramePresentHostAccepted` に限って row-tile RLE packet 単位の presented packet readiness evidence へ昇格する境界である。copyable `GuiBareDisplayDriverStepApplied`、`GuiBareDisplayDriverOutcome`、`GuiBareDisplayDriverFramePresentAccepted`、adapter evidence 単体は public authority として受け取らない。

F5fu のために `GuiBareDisplayMemoryOwner` は lifetime 累積の `verified_byte_count` とは別に `packet_verified_byte_count` を持つ。F5ft adapter は `BeginHostAccepted` の時だけ `packet_verified_byte_count` を 0 に reset し、`SpanWrite` / `SpanWriteAccepted` は F5fs の full span store / readback success 後に `byte_len` を checked add する。`FramePresentHostAccepted` は packet-local count を保持し、readiness check がその count を検査する。これにより過去 packet の累積 count を current packet readiness と誤認しない。

`gui_bare_display_present_readiness_from_adapter_completed completed` は、adapter evidence kind が `FramePresentHostAccepted`、adapter step outcome が `FramePresentAccepted`、owner 内 canonical driver state の phase が `Idle`、`last_present` が `Some` で step outcome の present evidence と target / descriptor / frame / run count / pixel count / surface byte count すべてで一致することを検査する。さらに present の `pixel_count * 4` を checked arithmetic で `packet_rgba_byte_count` にし、`packet_verified_byte_count == packet_rgba_byte_count` を要求する。ここでの byte count は RGBA memory byte count であり、RLE encoded byte count ではない。

成功 payload `GuiBareDisplayPresentedPacketReady` は owner と pure `GuiBareDisplayPresentedPacketReadyEvidence` を保持する。failure payload `GuiBareDisplayPresentReadinessError` も owner と adapter evidence を保持し、caller が owner recovery を選べる。owner-bearing success / error は `Clone` / `Copy` を実装しない。F5fu は packet readiness だけを主張し、whole surface ready、hardware flush completion、scheduler loop completion、tile aggregation、actual display driver flush、DOM、Canvas、minifb、video memory transport、fallback、silent no-op は実装しない。

## F5fv Bare whole-surface packet-readiness aggregation boundary

2026-06-19 の F5fv では、bare `platforms/gui/bare/display_surface_readiness` を追加する。これは F5fu の `GuiBareDisplayPresentedPacketReady` を value として消費し、row-tile RLE packet readiness を full-height surface の ordered packet readiness evidence へ集約する境界である。`GuiBareDisplayPresentedPacketReadyEvidence` 単体、driver step / outcome / present accepted value、raw storage、`RegionToken`、`MemPtr` は input authority にしない。

F5fv の full-height plan は `plan_row_start == 0` かつ `plan_row_count == height` の場合だけである。`start ready` は tile 0 だけを受け、`tile_count == ceil height tile_rows`、`stride_bytes == width * 4`、`expected_pixel_count == width * height`、`row_start == 0`、`row_count` が first/final tile の期待値、`pixel_count == row_count * width` を checked arithmetic で検査する。single-tile full-height の場合だけ `Completed` を返し、それ以外は non-Copy cursor と owner を持つ `Continue` を返す。

`advance cursor ready` は non-Copy cursor と次の owner-bearing `GuiBareDisplayPresentedPacketReady` を両方消費する。fixed metadata は surface、frame、packet frame id、batch index、plan range、width、height、stride、tile rows、tile count、surface byte count に限定して比較し、tile-local metadata である `tile_index`、`row_start`、`row_count`、`pixel_count` は cursor の `next_tile_index` から再計算して照合する。`tile_index < next_tile_index` は `DuplicateOrReorderedTile`、`tile_index > next_tile_index` は `TileGap` として fail-closed にする。

`Cursor` は module-private seal を持ち、public metadata だけから過去 tile の aggregation authority を偽造できない。`Continue` は `continue_take` で cursor と owner を同じ `Handoff` value に移し、future bridge や caller が同じ境界で両方を move できるようにする。`AdvanceError` は `advance_error_take` で cursor と incoming ready を同じ recovery value へ移し、failure path でも両方を回収できる。`GuiBareDisplayWholeSurfacePacketReadinessCompleted` も owner と pure evidence と module-private completed seal を保持し、owner recovery を失わない。`Cursor`、`Continue`、`Handoff`、`Completed`、start / advance error は `Clone` / `Copy` を実装しない。F5fv は hardware flush completion、scheduler loop completion、actual backend completion、DOM、Canvas、minifb、video memory transport、fallback、silent no-op へは進まない。

## F5fw Bare display hardware flush accepted boundary

2026-06-19 の F5fw では、bare `platforms/gui/bare/display_flush_completion` を追加する。これは F5fv の sealed `GuiBareDisplayWholeSurfacePacketReadinessCompleted` を value として消費し、`nepl_gui_bare.display_hardware_flush` host import へ whole-surface metadata を 1 回だけ渡す境界である。copyable `GuiBareDisplayWholeSurfacePacketReadinessEvidence`、flush evidence 単体、driver step / outcome、raw storage は input authority にしない。

`gui_bare_display_hardware_flush_accept_from_whole_surface completed` は、completed value から owner と evidence を取り出した後、host import の前に geometry preflight を行う。preflight は width / height / tile rows / tile count が正であること、`expected_pixel_count == width * height`、`ready_pixel_count == expected_pixel_count`、`stride_bytes == width * 4`、`surface_byte_count == height * stride_bytes` を checked arithmetic で検査する。preflight failure では host import を呼ばず、error payload の `status` は `Option::None` である。

host status は `0` だけを accepted とし、`-1` は `GuiError::Unsupported`、`-2` と `-6` は `GuiError::InvalidCommand`、`-3` と `-4` は `GuiError::ResourceExhausted`、その他の負値と正の non-zero status は `GuiError::BackendFailure` とする。host status failure では `status` を `Option::Some` で保持し、owner と evidence を `GuiBareDisplayHardwareFlushError` に回収する。

成功 payload `GuiBareDisplayHardwareFlushAccepted` は owner、pure `GuiBareDisplayHardwareFlushEvidence`、module-private accepted seal を保持する。これは host import が completed whole-surface evidence に対して accepted status を返したことだけを示し、physical scanout completion、long-running scheduler backend completion、timer queue、present loop、DOM、Canvas、minifb、video memory transport、fallback、silent no-op を意味しない。physical scanout や real backend loop completion は、将来の scheduler / backend checkpoint で別の authority として扱う。

## F5fx Bare display operation-to-driver-adapter bridge

2026-06-19 の F5fx では、bare `platforms/gui/bare/display_operation_driver_bridge` を追加する。これは `GuiBareDisplayMemoryOwner` と `GuiRgba8888RowTileRlePresentHostSpanOperation` だけを public input authority とし、Begin / RunSpan / End を含む existing host span operation enum を actual display driver adapter へ 1 step 渡す境界である。owner + `GuiRgba8888RowTileRlePresentHostSpanOperation` 以外の `GuiBareFramebufferState`、`GuiBareDisplayStorageState`、`GuiBareDisplayMemoryState`、`GuiBareFramebufferStepApplied`、`GuiBareDisplayStorageStepApplied`、`GuiBareDisplayMemoryStepApplied`、driver outcome、raw storage、`RegionToken`、`MemPtr` は public input authority にしない。

`gui_bare_display_operation_driver_bridge_step owner operation` は、owner から `GuiBareDisplayDriverState`、`GuiBareDisplayMemoryState`、`GuiBareDisplayStorageState`、`GuiBareFramebufferState` を accessor 経由で復元する。その後、`gui_bare_framebuffer_validate_operation`、`gui_bare_display_storage_apply`、`gui_bare_display_memory_apply`、`gui_bare_display_driver_adapter_step` の順に進める。framebuffer / storage / memory validation failure では host import へ進まず、original owner、operation、lower error、`Option GuiError` category を owner-bearing error に回収する。

adapter failure では `GuiBareDisplayDriverAdapterError` を lower error として保持し、owner recovery は adapter error が保持する owner に従う。adapter / host が external side effect を accepted した後の failure は、NEPL 側 owner recovery を返すが host side effect rollback は主張しない。成功 payload `GuiBareDisplayOperationDriverBridgeCompleted` は operation、adapter completed value、copyable adapter evidence、module-private completed seal を保持し、`Clone` / `Copy` を実装しない。F5fx は long-running scheduler backend、timer queue、present loop、DOM、Canvas、minifb、video memory transport、hardware flush、fallback、silent no-op へは進まない。

## F5fy Bare display presenter input boundary

2026-06-19 の F5fy では、bare `platforms/gui/bare/display_presenter_input` を追加する。これは F5fx の operation-to-driver-adapter bridge を real scheduler executor outcome path へ戻す platform boundary であり、public input authority は `GuiBareDisplayMemoryOwner` と typed `ExecuteHostAction` だけである。general `LoopAction`、scheduler completed payload、raw operation、framebuffer / storage / memory state、driver step、host status、raw storage は受け取らない。

`gui_bare_display_presenter_input owner action` は、action を `gui_bare_scheduler_executor_input` へ渡す前に borrowed accessor で pending operation identity を読む。その後、`gui_bare_display_operation_driver_bridge_step owner operation` を 1 回だけ呼ぶ。success では `GuiBareDisplayPresenterInputReady` が operation identity、`GuiBareDisplayOperationDriverBridgeCompleted`、`GuiBareSchedulerExecutorInputReady` を保持し、scheduler outcome は `Result::Ok unit` である。

bridge failure では lower bridge error を失わない。category がある failure は `Result::Err GuiError` として `gui_bare_scheduler_executor_input action outcome` に渡し、`GuiBareDisplayPresenterInputBridgeFailedReady` が lower bridge error と scheduler ready を同時に保持する。category が無い bridge failure では別の `GuiError` へ丸めず、scheduler ready を作らず、original `ExecuteHostAction` と lower bridge error を `GuiBareDisplayPresenterInputBridgeFailedMissingCategory` に保持する。F5fy は direct host import、long-running scheduler backend、timer queue、present loop、DOM、Canvas、minifb、video memory transport、hardware flush、fallback、silent no-op、host side effect rollback claim へは進まない。

## F5fz Native scheduler real-loop action step boundary

2026-06-19 の F5fz では、native-only `platforms/gui/native/scheduler_real_loop_step` を追加する。これは F5el `NeedInput` を native clock helper と native presenter host executor へ 1 action だけ接続し、success した F5ek result を F5el `real_loop_driver_after_step` まで戻す boundary である。public input authority は native policy、backend clock state、F5el `NeedInput` だけであり、general queue、raw host event、bare owner、renderer、surface、video memory は受け取らない。

`GuiNativeSchedulerRealLoopStepPolicy` は F5ek real-loop step policy、F5el real-loop driver policy、F5eo backend clock policy だけを保持する。`YieldToClock` と `AwaitTimerAdvance` は F5eu native clock helper を 1 回だけ呼び、その success payload に含まれる next clock state、action、F5ek input をそのまま使う。clock branch は `ClockDelta` を再合成せず、helper が返した action / input だけを F5ek へ渡す。

`ExecuteHostAction` は `gui_native_scheduler_host_executor_step` を 1 回だけ呼ぶ。host import failure は F5fz の direct error ではなく、F5fj / F5ex path で `Result unit GuiError` outcome として包まれ、F5ek / F5el path へ戻る。したがって F5fz execute branch は追加で F5ek を呼ばない。`Complete` branch だけが `RealLoopStepInput::CompleteAck` を作り、clock branch や execute branch は `CompleteAck` を合成しない。

F5fz は native action step bridge であり、not long-running scheduler backend である。timer wait、sleep、queue drain、present loop、minifb window loop、DOM、Canvas、video memory transport、bare display owner path、fallback、silent no-op は実装しない。bare の同等接続は F5fy owner path から別 slice で設計する。

## F5ga Bare scheduler real-loop action step boundary

2026-06-19 の F5ga では、bare-only `platforms/gui/bare/scheduler_real_loop_step` を追加する。これは F5el `NeedInput` を bare clock helper と F5fy display presenter input owner path へ 1 action だけ接続し、success した F5ek result を F5el `real_loop_driver_after_step` まで戻す boundary である。public input authority は bare policy、backend clock state、`GuiBareDisplayMemoryOwner`、F5el `NeedInput` だけであり、general queue、raw host event、raw driver state、raw storage、renderer、surface、video memory は受け取らない。

`YieldToClock` と `AwaitTimerAdvance` は F5eu の bare clock input helper を 1 回だけ呼ぶ。clock helper success payload から next clock state、action、F5ek input を取り出し、その action / input を F5ek へ渡す。F5ga は clock delta を再計算しない。clock helper failure は original owner と lower clock input error を保持する `Result` error として返し、fallback clock source や silent no-op にはしない。

`ExecuteHostAction` は F5fy `gui_bare_display_presenter_input owner execute_action` を 1 回だけ呼ぶ。success では `bridge_completed` から recovered owner を先に取り出し、その後 scheduler ready payload を F5ew `gui_bare_scheduler_executor_step` へ渡す。F5fy の `BridgeFailedReady` は category 付き bridge failure を `Result unit GuiError` outcome として包んだ scheduler ready なので、F5ga はこれも F5ew へ 1 回だけ渡す。`BridgeFailedMissingCategory` では `GuiError` を捏造せず direct error とし、lower bridge error から owner を回収できる形で保持する。

F5ek / F5el failure では、branch、relevant clock state、recovered owner、lower error を error payload に保持する。clock branch の failure は successful clock input 後なら next clock state、execute / complete branch の failure は unchanged clock state を保持する。これにより bare display owner を消費した後のどの失敗経路でも owner recovery が失われない。

`Complete` branch だけが `RealLoopStepInput::CompleteAck` を作る。clock branch や execute branch は `CompleteAck` を合成しない。F5ga は bare action step bridge であり、not long-running scheduler backend である。timer wait、sleep、queue drain、present loop、direct host import、DOM、Canvas、minifb、video memory transport、hardware flush、fallback、silent no-op は実装しない。

## F5gb Native / Bare scheduler bounded real-loop runner boundary

2026-06-19 の F5gb では、`platforms/gui/native/scheduler_real_loop_runner` と `platforms/gui/bare/scheduler_real_loop_runner` を追加する。これは F5el `real_loop_driver_start` から始め、F5fz / F5ga の platform action step を `max_step_count` の範囲で繰り返し、`Completed` または `BudgetExhausted` を返す bounded real backend loop checkpoint である。

runner policy は platform real-loop step policy と `max_step_count` だけを保持する。F5el driver policy は F5fz / F5ga policy の accessor から借用し、runner policy に二重保持しない。`max_step_count < 0` は `PolicyInvalid` として start / step を呼ばずに fail-closed にする。`max_step_count == 0` でも F5el start は 1 回だけ呼び、start result が `NeedInput` の場合は step を呼ばず `BudgetExhausted` とする。start result が `Completed` の場合は budget に関係なく terminal completion として返す。

native runner の error payload は current clock state と lower error を保持する。特に clock-input failure の場合、lower F5fz error から直接 clock state を復元できないため、runner level の `StepFailed` が clock state を保持する。bare runner の result / error payload は常に `GuiBareDisplayMemoryOwner` を保持または回収できる。policy invalid と start failure は original owner を返し、F5ga step failure は `gui_bare_scheduler_real_loop_step_error_owner` を通して owner を回収する。runner level には `gui_bare_scheduler_real_loop_runner_error_owner` を用意し、caller がどの error variant でも owner を取り戻せるようにする。

F5gb は bounded runner であり、OS window loop、minifb event pump、timer wait、sleep、queue drain、DOM、Canvas、video memory transport、formal `std/gui` present host implementation、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization へは進まない。`ClockDelta`、`ExecutorOutcome`、`CompleteAck` は F5fz / F5ga の step path でのみ作られ、runner が fallback input や silent no-op を合成しない。

## F5gc std layer row tile RLE present host import scheduler start boundary

2026-06-19 の F5gc では、std layer row tile RLE present host import scheduler start として `std/gui/tile_present_host_import_scheduler_start` を追加する。これは F5cr `GuiRgba8888RowTileRlePresentHostImportRequest` を F5cw `GuiRgba8888RowTileRlePresentHostExecutionAction` へ写し、F5du `turn_start` で presenter / executor session turn を開始し、その turn state を F5ea `virtual_scheduler_turn` に渡して `GuiRgba8888RowTileRlePresentHostImportSchedulerStartReady` を作る std layer boundary である。

public authority は support、span policy、dynamic `GuiVirtualTimerState`、F5cr request だけである。success payload は original request、derived action、initial virtual scheduler state を保持する。turn start failure では original request、derived action、lower F5du error、category を保持する。support と span policy は Copy policy input であり、error recovery authority として保持しない。

`start_with_empty_timer` は active timer を持たない明示 initial GuiVirtualTimerState を作る helper である。これは timer backend unavailable の fallback や silent no-op ではない。caller が dynamic timer state を持っている場合は `start` に渡し、active timer を持たない initial state から始めたい場合だけ helper を使う。

F5gc は scheduler initial state construction までである。virtual scheduler step / drain / slice / loop、real loop driver、loop action mapping、turn driver complete、actual host import execution、timer backend、queue、platform API、DOM、Canvas、minifb、video memory、RenderTarget / DrawTarget fallback には進まない。

## F5gd Native window event pump boundary

2026-06-19 の F5gd では、`nepl-gui-native` の smoke runner から minifb input / resize polling を `poll_minifb_window_event_pump` へ集約する。Rust lib は `NativeWindowEventPumpInput` と `NativeWindowEventPumpSnapshot` を持ち、previous window size、previous left-button state、current window size、pointer sample、close state を enum / struct として返す。

`NativeWindowSize` は OS / window manager から観測した raw size なので zero dimension を許す。zero dimension は drawable surface ではなく `NativeWindowPresenterSurfaceState::Unavailable` へ写す。positive size は `Drawable { width, height }` になり、smoke runner はその width / height と同じ RGB0 buffer を再生成してから `Window::update_with_buffer` へ渡す。window resize は pixel buffer stretch ではなく、application / layout が新しい drawable size に合わせて frame を作り直す契約である。

close state は `Open`、`OsCloseRequested`、`ExitShortcutRequested` に分ける。現 smoke runner は unsaved state を持たないため OS close button と Escape shortcut のどちらでも loop を終了するが、snapshot contract は将来の rejectable close request、lifecycle、test event virtualization で区別できる形を保つ。

pointer input は `NativeWindowPointerSample::Unavailable` と `Available { x, y }` を分ける。pointer raw sample が存在しないことは通常状態であり、非有限 coordinate は `NativeWindowEventPumpError::InvalidPointerSample` として失敗する。main loop は minifb の `Key`、`MouseButton`、`MouseMode`、`is_open`、`is_key_down`、`get_mouse_down`、`get_unscaled_mouse_pos` を直接読まず、snapshot を `match` する。

F5gd は native event pump boundary だけであり、formal `std/gui` host import execution、scheduler loop、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。event pump helper は `window.update` / `update_with_buffer` を呼ばず、presentation authority は smoke runner / future backend loop に残す。

## F5ge Native backend loop step boundary

2026-06-19 の F5ge では、F5gd の event snapshot を受け取った後の native window loop state transition を `NativeWindowBackendLoop` へ移す。`main.rs` は minifb window creation、event pump adapter、title update、`window.update`、`window.update_with_buffer` だけを扱い、counter hit test、scene coordinate mapping、frame id update、resize redraw、RGB0 present buffer construction、presenter surface commit を直接行わない。

`NativeWindowBackendLoop` は `GuiDemo`、counter value、current `GuiFrame`、presenter frame id、previous observed size、previous mouse state、`NativeWindowPresenterState` を所有する。public entry `new_for_scale` は scale validation、initial frame render、initial size checked multiplication、presenter state creation、initial frame present を一括して行い、成功した場合だけ ready な loop を返す。scale 0、initial size overflow、initial surface invalid、rasterize failure、present buffer validation failure、presenter failure は `NativeWindowBackendLoopError` で分ける。

`NativeWindowBackendLoop::step` は `NativeWindowEventPumpSnapshot` を 1 件だけ処理し、`CloseRequested`、`Unavailable`、`Drawable` の typed outcome を返す。`Drawable` outcome は resize redraw evidence、pointer action、final frame evidence を分けるため、同じ snapshot に resize と counter hit が含まれても、resize frame と counter frame の両方を `frame_id,width,height` の committed evidence として保持できる。pixel borrow は outcome に含めず、final committed frame は `current_present_frame_for_window` からだけ借用する。

commit rule は次である。close は最優先で no-progress とし、previous size / mouse、presenter frame、counter を進めない。unavailable surface は observed size、mouse state、presenter surface availability だけを更新し、blank frame や fallback frame を合成しない。positive resize は new-size RGB0 buffer construction と present が成功した後だけ surface state、previous size、frame id を commit する。counter hit は pointer unavailable、outside、hit を enum で分け、hit の場合だけ checked counter increment と checked frame id を mutation 前に検査し、new frame present success の後だけ counter/current frame/frame id を進める。

F5ge は native smoke backend の loop-step state boundary であり、formal `std/gui` host import execution、scheduler loop、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。loop helper は minifb / OS handle / DOM / Canvas / video memory / stdout protocol を知らず、fallback や silent no-op を作らない。

## F5gf Native host action boundary

2026-06-19 の F5gf では、F5ge の `NativeWindowBackendLoopStepOutcome` を native host が実行する `NativeWindowHostAction` へ写す境界を追加する。`step` は引き続き pure backend state transition evidence として公開し、`step_host_action` が `CloseRequested` / `Unavailable` / `Drawable` を `Terminate` / `PumpEventsOnly` / `PresentFrame` へ変換する。

`NativeWindowHostAction` は pixel borrow を持たない。`PresentFrame` は `NativeWindowBackendLoopPresentation`、observed `NativeWindowSize`、`size_changed` を保持し、minifb へ渡す actual frame borrow は `current_present_frame_for_window` からだけ取得する。`PumpEventsOnly` は `NativeWindowSize` と `size_changed` を保持し、zero-size / unavailable surface で blank frame や fallback frame を合成しない。`Terminate` は `NativeWindowHostTerminalReason` として `OsCloseRequested` と `ExitShortcutRequested` を分ける。

`NativeWindowHostActionError` は contradictory close state を `UnsupportedCloseState` とし、F5ge の overflow、rasterize、presenter failure などは `StepFailed NativeWindowBackendLoopError` として original typed error を保持する。`main.rs` は `NativeWindowBackendLoopStepOutcome` を直接 match せず、`NativeWindowHostAction` だけを実行する。これにより formal native OS scheduler / window backend loop へ進む前に、backend state transition と host execution action の責務を分離する。

F5gf は host action selection boundary であり、formal scheduler loop、OS wait strategy、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。DOM / Canvas / video memory transport、fallback、silent no-op も導入しない。

## F5gg Native minifb window run-loop adapter boundary

2026-06-19 の F5gg では、F5gf の `NativeWindowHostAction` を消費する cfg-gated minifb window run-loop adapter を `nepl-gui-native` に追加する。F5ge / F5gf では `main.rs` が minifb window lifecycle と host action execution を持っていたが、F5gg 以降は `run_minifb_window_loop` だけが `WindowOptions`、`ScaleMode::UpperLeft`、`window.update`、`update_with_buffer` を扱う。`main.rs` は CLI option を `NativeWindowRunLoopConfig` へ変換して runner を呼ぶだけにする。

`NativeWindowRunLoopConfig` は demo、counter value、scale を保持する。`NativeWindowRunLoopExit` は terminal reason を `NativeWindowHostTerminalReason` として保持し、OS close と Escape shortcut を CLI success の中で潰さない。`NativeWindowRunLoopError` は backend initialization、window creation、event pump、host action selection、presenter frame availability、`WindowPresentFailed` を enum variant で分ける。`WindowPresentFailed` は `update_with_buffer` failure だけを表し、error を返さない `window.update` の failure を捏造しない。minifb の具体 error text は platform detail として message に閉じ込め、backend loop / event pump / host action の typed error は original error を保持する。

run-loop adapter は毎 iteration で `poll_minifb_window_event_pump` と `step_host_action` だけを呼ぶ。direct minifb input API は引き続き event pump helper に隔離し、run-loop adapter は `Key`、`MouseButton`、`MouseMode`、`is_open`、`is_key_down`、`get_mouse_down`、`get_unscaled_mouse_pos` を直接扱わない。`PumpEventsOnly` では title update と `window.update` だけを行い、frame を要求しない。`PresentFrame` では title update 後、`current_present_frame_for_window` から借用した exact-size RGB0 frame だけを `update_with_buffer` へ渡す。

F5gg は native smoke backend の minifb adapter boundary であり、formal `std/gui` host import execution、scheduler queue、timer wait、OS wait strategy、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。`set_target_fps(60)` は minifb smoke runner の busy spin 抑制だけであり、formal timer / scheduler policy ではない。fallback、silent no-op、blank frame、DOM / Canvas / video memory transport は導入しない。

## F5gh Native window host-loop core boundary

2026-06-19 の F5gh では、F5gg の minifb 固有 long loop から backend loop と host execution の core を切り出し、`NativeWindowRunLoopHost` と `run_native_window_host_loop` を追加する。これは minifb を formal host abstraction に見せかける層ではなく、`NativeWindowBackendLoop` が保持する state transition と、host が持つ event pump / title / pump-only / present operation を分ける境界である。

`run_native_window_host_loop` は `&mut NativeWindowBackendLoop` と `&mut NativeWindowRunLoopHost` を受ける。backend loop を value で消費しないため、event pump failure、host action failure、presenter frame unavailable、host present failure のいずれでも caller は backend loop state を回収できる。error は `NativeWindowHostLoopError EventError PresentError` として、host event error、`NativeWindowHostActionError`、`NativeWindowBackendLoopError`、host present error を潰さず保持する。

`NativeWindowRunLoopHost` は `poll_event_snapshot`、`set_window_title`、`pump_events_only`、`present_frame` を持つ。core loop は minifb 型、`window.update`、`update_with_buffer`、direct input API を知らない。minifb smoke backend は private `MinifbNativeWindowRunLoopHost` でこの trait を実装し、`poll_minifb_window_event_pump` と minifb presentation API をそこへ閉じる。`run_minifb_window_loop` は backend loop と minifb window / host adapter を初期化し、`run_native_window_host_loop` を呼ぶだけにする。

F5gh は native host-loop core boundary であり、formal `std/gui` host import execution、scheduler queue、timer wait、OS wait strategy、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、blank frame、DOM / Canvas / video memory transport も導入しない。

## F5gi Native window host-loop turn boundary

2026-06-19 の F5gi では、F5gh の long-running `run_native_window_host_loop` の body を `step_native_window_host_loop` という typed one-turn boundary に分ける。`NativeWindowHostLoopTurn` は `Continue` と `Exit NativeWindowRunLoopExit` だけを持ち、queue や timer state を隠した generic payload は持たない。

`step_native_window_host_loop` は initial title を設定しない。1 turn の責務は、host event snapshot の取得、`NativeWindowBackendLoop::step_host_action`、size change 時の title update、pump-only update、present frame borrow と host present、または typed exit の返却だけである。`run_native_window_host_loop` は initial title を 1 回だけ設定し、その後は `step_native_window_host_loop` を loop で呼び、`Continue` / `Exit` を match する。

この境界により、後続の formal native OS scheduler / window backend loop は long loop の内部処理を再実装せず、同じ typed turn を呼べる。ただし F5gi 自体は scheduler queue、OS wait strategy、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へ進まない。fallback、silent no-op、blank frame、DOM / Canvas / video memory transport も導入しない。

## F5gj Native window host-loop bounded runner boundary

2026-06-19 の F5gj では、F5gi の one-turn boundary を bounded に反復する `run_native_window_host_loop_bounded` を追加する。これは native smoke/window integration が infinite loop なしに finite turn count を実行し、`Exited` と `BudgetExhausted` を区別できるようにする boundary である。

`NativeWindowHostLoopRunnerState` は initial title が設定済みかどうかだけを保持する。`initialize_native_window_host_loop` は `Initialized` または `AlreadyInitialized` を返し、二度目以降の呼び出しを silent no-op にしない。`max_turn_count == 0` の bounded run は initial title を確認したうえで event poll を行わず、`BudgetExhausted completed_turns 0` を返す。

`run_native_window_host_loop_bounded` は `step_native_window_host_loop` だけで turn を進める。`Continue` は completed turn count を増やし、`Exit` は exit turn を含む count で `Exited` を返す。error は F5gi の `NativeWindowHostLoopError` をそのまま返す。F5gj は OS wait strategy、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、blank frame、DOM / Canvas / video memory transport も導入しない。

## F5gk Native window frame pacing config boundary

2026-06-19 の F5gk では、native smoke window loop の frame pacing を hidden constant から typed config へ移す。`Window::set_target_fps 60` を run-loop body に直接置くのではなく、`NativeWindowTargetFps` newtype と `NativeWindowRunLoopConfig.target_fps` を通して validated target FPS だけを minifb adapter へ渡す。

`NativeWindowTargetFps` は `1..=240` の範囲を契約とし、`0` は `Zero`、上限超過は `TooHigh max` として `NativeWindowTargetFpsError` に保持する。`NativeWindowRunLoopError::TargetFpsInvalid value reason` は raw value から config を作る public helper の fail-closed path であり、invalid FPS を silent clamp しない。CLI の `--fps` も同じ validation を使う。headless mode では frame pacing は使わないが、CLI option surface は window mode 用の設定として明記する。

F5gk は frame pacing policy の明示化までであり、formal OS wait strategy、queue / timer wait backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。fallback、silent no-op、raw `usize` FPS の minifb 直渡しも導入しない。

## F5gl Native window host-loop run policy boundary

2026-06-19 の F5gl では、native smoke window の long loop path を `NativeWindowHostLoopRunPolicy` による explicit bounded turn policy へ寄せる。`NativeWindowHostLoopTurnSlice` は `1..=4096` の validation 済み newtype であり、`0` は `Zero`、上限超過は `TooHigh max` として `NativeWindowHostLoopTurnSliceError` に保持する。default は `1` であり、現状の one-turn-per-host-update semantics を維持する。

`NativeWindowHostLoopRunPolicy` は `turn_slice` を持ち、`NativeWindowRunLoopConfig.host_loop_policy` に保持される。`run_native_window_host_loop_with_policy` は `run_native_window_host_loop_bounded` を反復し、`Exited` なら `NativeWindowRunLoopExit` を返し、`BudgetExhausted` なら同じ `NativeWindowHostLoopRunnerState` で次 slice を開始する。`run_native_window_host_loop` は default policy の wrapper であり、`run_minifb_window_loop` は `config.host_loop_policy` を渡す。

F5gl の `4096` 上限は OS wait / timer wait / FHD 60fps の保証ではなく、bounded turn budget の sanity bound である。formal OS wait strategy、queue / timer wait backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。fallback、silent no-op、`usize::MAX` による unbounded slice、sleep、queue、timer、DOM / Canvas / video memory transport も導入しない。

## F5gm Native window host-loop turn evidence boundary

2026-06-19 の F5gm では、native host-loop turn の `Continue` に実行証拠を持たせる。`NativeWindowHostLoopContinueEvidence` は `PumpedEventsOnly window_size size_changed` と `PresentedFrame presentation window_size size_changed` を持つ。これは future native OS wait strategy が surface-unavailable pump と successfully-presented frame を型で分岐するための evidence であり、wait strategy そのものではない。

`PresentedFrame` evidence の `presentation` は `NativeWindowBackendLoopPresentation` value だけであり、pixel borrow を持たない。`step_native_window_host_loop` は host present が成功した後だけ `PresentedFrame` evidence を返し、present failure では既存の `NativeWindowHostLoopError::HostPresentFailed` を返す。bounded runner と policy runner は `Continue _` として turn count だけを進め、evidence をまだ scheduler policy へ渡さない。

F5gm は turn evidence の明示化までであり、formal OS wait strategy、queue / timer wait backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。fallback、silent no-op、sleep、queue、timer、DOM / Canvas / video memory transport も導入しない。

## F5gn Native window host-loop wait decision boundary

2026-06-19 の F5gn では、F5gm の `NativeWindowHostLoopContinueEvidence` を `NativeWindowHostLoopWaitDecision` へ分類する。`WaitForHostEvent` は pump-only turn の後続 wait class、`WaitForFrameInterval` は successful presented-frame turn の後続 wait class である。

`native_window_host_loop_wait_decision` は total mapping の pure helper であり、`Result` を返さず、fallback や silent no-op を持たない。`WaitForFrameInterval` は frame-paced wait class evidence であって、実時間 sleep、timer registration、FHD 60fps 保証ではない。decision は `NativeWindowSize`、`size_changed`、必要な場合の `NativeWindowBackendLoopPresentation` だけを保持し、pixel borrow、host handle、scheduler state を保持しない。

`NativeWindowHostLoopBoundedRunResult::BudgetExhausted` は `completed_turns` と `last_wait_decision Option NativeWindowHostLoopWaitDecision` を返す。zero budget では `None`、`Continue evidence` を 1 回以上処理した場合は最後の evidence を分類した `Some decision` を返す。実 wait dispatch は F5go の責務であり、F5gn は decision の生成と保持までを標準契約にする。

F5gn は wait decision classification までであり、formal OS wait strategy、queue / timer wait backend、`Duration` 計算、`std::thread::sleep`、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。fallback、silent no-op、minifb / DOM / Canvas / video memory transport も導入しない。

## F5go Native window host-loop wait dispatch boundary

2026-06-19 の F5go では、F5gn の wait decision を host wait boundary へ渡す。`NativeWindowRunLoopHost` は `WaitError` associated type と `wait_after_budget_exhausted` を持つ。`NativeWindowHostLoopError EventError PresentError WaitError` は `HostWaitFailed WaitError` と `WaitDecisionMissing` を含み、host wait failure と missing wait evidence を他の error class から分離する。

`NativeWindowHostLoopWaitOutcome` は wait hook の outcome evidence であり、`HostEventPumpAlreadyPaced`、`FramePresentAlreadyPaced`、`FrameIntervalTimerRegistered` を持つ。minifb smoke backend は `Window::set_target_fps` を rate limit authority とし、wait hook では追加の `window.update`、`update_with_buffer`、`Duration`、`std::thread::sleep`、queue、timer を実行しない。`Window::update` と `update_with_buffer` が内部で pacing されるため、minifb wait hook は already-paced outcome を返すだけである。`FrameIntervalTimerRegistered` は external scheduler 向けの timer registration evidence であり、already-paced outcome ではない。

`run_native_window_host_loop_with_policy` は `BudgetExhausted last_wait_decision = Some decision` の場合だけ `wait_after_budget_exhausted` を呼ぶ。`last_wait_decision = None` は `NativeWindowRunLoopError::WaitDecisionMissing` へ写る fail-closed path であり、zero budget や missing evidence を silent no-op にしない。

F5go は wait dispatch boundary までであり、formal OS wait strategy、queue / timer wait backend、real timer registration、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。fallback、silent no-op、extra minifb update、DOM / Canvas / video memory transport も導入しない。

## F5gp Native window host-loop scheduler slice boundary

2026-06-19 の F5gp では、F5go まで long runner 内部に隠れていた bounded slice execution と wait dispatch を、external scheduler が 1 slice ずつ呼べる boundary として公開する。`NativeWindowHostLoopSchedulerState` は `NativeWindowHostLoopRunnerState` を所有し、initial title initialization state を slice 間で保持する。

`run_native_window_host_loop_scheduler_slice_with_policy` は policy の `turn_slice` から max turn count を取り、`run_native_window_host_loop_bounded` を 1 回だけ呼ぶ。bounded result が `Exited` なら `NativeWindowHostLoopSchedulerSliceResult::Exited exit completed_turns` を返す。bounded result が `BudgetExhausted last_wait_decision = Some decision` なら host wait hook を 1 回だけ呼び、`Waited completed_turns decision outcome` を返す。`last_wait_decision = None` は `WaitDecisionMissing` とする。

`run_native_window_host_loop_with_policy` はこの scheduler slice API を反復する wrapper へ縮小する。これにより future native OS scheduler / window backend loop は long loop を再実装せず、同じ typed slice と state を所有できる。

F5gp は scheduler slice boundary までであり、actual OS wait strategy、queue / timer wait backend、real timer registration、`Duration`、`std::thread::sleep`、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。fallback、silent no-op、extra minifb update、DOM / Canvas / video memory transport も導入しない。

## F5gq Native window host-loop wait request plan boundary

2026-06-19 の F5gq では、`NativeWindowHostLoopWaitDecision` を actual backend が消費できる `NativeWindowHostLoopWaitRequest` へ変換する契約を追加する。decision は turn evidence 由来の分類であり、request は backend wait authority へ渡す計画である。

`NativeWindowFrameIntervalRequest` は validated `NativeWindowTargetFps`、`nanos_per_frame`、`remainder_nanos_per_second` を持つ。`nanos_per_frame` は `1_000_000_000 / fps`、`remainder_nanos_per_second` は `1_000_000_000 % fps` であり、暗黙 clamp や sentinel rounding を行わない。`NativeWindowTargetFps` が `1..=240` を保証するため、zero fps と overflow はこの境界の入力型で除外される。

`NativeWindowHostLoopWaitRequest` は `WaitForHostEvent` と `WaitForFrameInterval` を持つ。host event request は event payload や queue owner を持たず、frame interval request だけが `NativeWindowFrameIntervalRequest` を持つ。F5gq は decision から backend-facing request plan を作る境界であり、F5gr 以降の `NativeWindowRunLoopHost::wait_after_budget_exhausted` は request から生成される instruction を受け取る。

`NativeWindowHostLoopSchedulerSliceResult::Waited` は `decision`、`request`、`instruction`、`outcome` を保持する。test と future scheduler は、分類元 decision、backend request plan、host instruction、host outcome を別々に検査できる。

F5gq は wait request plan boundary までであり、actual OS wait strategy、queue / timer wait backend、real timer registration、`std::time::Duration`、`std::thread::sleep`、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。fallback、silent no-op、extra minifb update、DOM / Canvas / video memory transport も導入しない。

## F5gr Native window host-loop wait strategy instruction boundary

2026-06-19 の F5gr では、F5gq の `NativeWindowHostLoopWaitRequest` を、host wait hook が消費する `NativeWindowHostLoopWaitInstruction` へ変換する契約を追加する。request は backend-facing plan であり、instruction は scheduler state を反映した actual wait backend input である。

`NativeWindowHostLoopWaitStrategyState` は frame pacing 用に直前の `NativeWindowTargetFps` と remainder accumulator を保持する。target FPS が変わった場合は accumulator を再利用せず、`0` から開始する。target FPS が同じ場合だけ `NativeWindowFrameIntervalRequest.remainder_nanos_per_second` を accumulator に足し、`0 <= accumulator < fps` を保つ。補正用の saturating、clamp、sentinel は使わない。

`NativeWindowHostLoopWaitInstruction::WaitForFrameInterval` は `wait_nanos` を持つ。`wait_nanos` は `nanos_per_frame`、または remainder 配分時の `nanos_per_frame + 1` のどちらかだけである。この値は `Duration` 変換や sleep 実行ではなく、後続の OS wait backend が消費する typed value として扱う。

`run_native_window_host_loop_scheduler_slice_with_policy_and_target_fps` は bounded run の後に request と instruction plan を作り、`NativeWindowRunLoopHost::wait_after_budget_exhausted` へ instruction を渡す。wait hook が成功した場合だけ `NativeWindowHostLoopSchedulerState.wait_strategy_state` を次状態へ進める。wait hook 失敗時は accumulator を進めず、次 slice で同じ scheduling state からやり直せる。

F5gr は wait strategy instruction boundary までであり、actual OS wait strategy、queue / timer wait backend、real timer registration、`std::time::Duration`、`std::thread::sleep`、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。fallback、silent no-op、extra minifb update、DOM / Canvas / video memory transport も導入しない。

## F5gs Native window host-loop thread wait backend boundary

2026-06-19 の F5gs では、F5gr の `NativeWindowHostLoopWaitInstruction` を native thread sleep backend へ渡す境界を追加する。これは minifb smoke backend の wait hook ではなく、formal native wait backend の最初の実行境界である。

`NativeWindowHostLoopThreadSleeper` は `sleep_for_nanos` だけを持つ injected sleeper interface である。test は scripted sleeper を使い、actual std backend は `StdNativeWindowHostLoopThreadSleeper` として `std::thread::sleep(std::time::Duration::from_nanos(u64::from(wait_nanos)))` を実行する。`Duration` と sleep はこの backend helper にだけ閉じ込め、scheduler slice、instruction planner、minifb wait hook へ漏らさない。

`execute_native_window_host_loop_thread_wait_with_sleeper` は `WaitForFrameInterval` の場合だけ sleeper を 1 回呼ぶ。実行前に `wait_nanos == nanos_per_frame` または `wait_nanos == nanos_per_frame + 1` を再検査し、不一致なら `FrameIntervalWaitNanosMismatch` を返して sleeper を呼ばない。これは public instruction が手書きで作られても F5gr の invariant を backend boundary で再確認するためである。

`WaitForHostEvent` は `HostEventWaitUnsupported` として fail closed にする。host event blocking は OS event queue / selector / message pump backend が必要であり、thread sleep、busy loop、silent no-op で代替しない。

F5gs は native thread sleep backend boundary までであり、host event queue、timer registration、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。minifb smoke backend は引き続き `Window::set_target_fps` による already-paced outcome を返し、F5gs の thread sleep backend を呼ばない。

## F5gt Native window host-loop timer registration backend boundary

2026-06-19 の F5gt では、F5gr の `NativeWindowHostLoopWaitInstruction` を native timer registration backend へ渡す境界を追加する。これは F5gs の thread sleep backend とは別の実行境界であり、formal native scheduler が frame interval wait を timer registration として扱うための contract である。

`NativeWindowHostLoopTimerRegistrar` は raw `u32` timer id を返す host boundary である。library 側は raw id をそのまま authority にせず、`raw_id > 0` を検査してから `NativeWindowHostLoopTimerRegistrationId` へ変換する。raw id `0` は `InvalidTimerRegistrationId` として `Result::Err` になる。これにより platform backend の sentinel や invalid handle を silent success にしない。

`WaitForFrameInterval` は `wait_nanos` が `nanos_per_frame` または `nanos_per_frame + 1` の場合だけ registrar を呼ぶ。不一致は `FrameIntervalWaitNanosMismatch` として fail closed にする。registrar の失敗は `RegistrarFailed` として元の error value を保持する。

`WaitForHostEvent` は `HostEventTimerRegistrationUnsupported` として fail closed にする。host event blocking は OS event queue / selector / message pump backend が必要であり、timer registration、thread sleep、busy loop、silent no-op で代替しない。

F5gt は native timer registration backend boundary までであり、host event queue、real OS timer backend、minifb wait hook への接続、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。

## F5gu Native window host-loop event queue wait backend boundary

2026-06-19 の F5gu では、F5gr の `NativeWindowHostLoopWaitInstruction` のうち `WaitForHostEvent` を event queue wait backend へ渡す境界を追加する。これは F5gs の thread sleep backend、F5gt の timer registration backend と分離した host-event execution contract である。

`NativeWindowHostLoopEventQueueWaiter` は `wait_for_host_event` だけを持つ injected backend interface である。`execute_native_window_host_loop_event_queue_wait_with_waiter` は `WaitForHostEvent` の場合だけ waiter を 1 回呼び、成功時に `HostEventReady` を返す。waiter failure は `WaiterFailed` として元の error value を保持する。

`WaitForFrameInterval` は `FrameIntervalEventQueueWaitUnsupported` として fail closed にする。event queue wait backend は frame interval wait を timer registration、thread sleep、busy loop、silent no-op で代替しない。

F5gu は event queue wait backend boundary までであり、real OS event queue / selector / message pump adapter、real OS timer backend、minifb wait hook への接続、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。

## F5gv Native window host-loop event queue normalized status adapter boundary

2026-06-19 の F5gv では、F5gu の `NativeWindowHostLoopEventQueueWaiter` に接続できる normalized status adapter boundary を追加する。これは actual OS event queue / selector / message pump ではなく、platform adapter が `nepl-gui-native` の境界用に正規化した raw status を検証する contract である。

`NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY` は host event が ready になったことだけを表す internal normalized status であり、macOS / Windows / Linux の OS API raw value そのものではない。platform-specific adapter は OS 固有の結果をこの normalized status へ変換してから返す必要がある。

`NativeWindowHostLoopEventQueueStatusAdapter` は `wait_for_host_event_raw_status` で normalized raw status を返す。`wait_native_window_host_loop_event_queue_raw_status_with_adapter` は adapter を 1 回呼び、ready status 以外を `InvalidRawStatus` として fail closed にする。adapter failure は `AdapterFailed` として元の error value を保持し、invalid status と混ぜない。

`NativeWindowHostLoopEventQueueStatusWaiter` は status adapter を F5gu の `NativeWindowHostLoopEventQueueWaiter` へ接続する wrapper である。F5gu executor が frame interval instruction を受け取った場合は `FrameIntervalEventQueueWaitUnsupported` で停止するため、status adapter は呼ばれない。

F5gv は normalized status adapter boundary までであり、real OS event queue / selector / message pump adapter、minifb wait hook、real OS timer backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。

## F5gw Native window host-loop message pump adapter boundary

2026-06-19 の F5gw では、F5gv の normalized status adapter へ message pump adapter を接続する。`NativeWindowHostLoopMessagePumpAdapter` は `pump_host_messages` を 1 回だけ実行し、成功時だけ `NativeWindowHostLoopMessagePumpStatusAdapter` が `NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY` を返す。pump failure は `PumpFailed` として元の error value を保持する。

minifb smoke backend では `MinifbNativeWindowHostLoopMessagePumpAdapter` が `window.update` を実行する。`MinifbNativeWindowRunLoopHost::wait_after_budget_exhausted` は host event wait を direct update ではなく `wait_minifb_window_host_event_message_pump` 経由で F5gu / F5gv の event queue waiter 境界へ渡す。frame interval wait は従来通り frame pacing 側の outcome として扱い、message pump adapter では処理しない。

F5gw は message pump adapter boundary までであり、real OS timer backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。message pump adapter が timer registration、thread sleep、busy loop、silent no-op、fallback へ迂回することは禁止する。

## F5gx Native window host-loop frame interval timer registration outcome boundary

2026-06-19 の F5gx では、F5gt の timer registration backend を `NativeWindowHostLoopWaitOutcome` へ接続する。timer registration 成功は future timer fire の予約を表すだけであり、frame interval wait 完了や present pacing 完了を意味しないため、`FramePresentAlreadyPaced` へは写さない。

`NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered` は `presentation`、`window_size`、`size_changed`、`wait_nanos`、`timer_registration_id` を保持する。`execute_native_window_host_loop_timer_registration_wait_with_registrar` は `WaitForFrameInterval` instruction を F5gt executor へ渡し、F5gt の successful registration outcome だけをこの wait outcome へ変換する。

`WaitForHostEvent` は `HostEventTimerRegistrationUnsupported` として fail closed にする。timer registration backend が host event wait を queue wait、message pump、thread sleep、busy loop、silent no-op で代替することは禁止する。

F5gx は timer registration outcome boundary までであり、actual timer fire / wakeup、real OS timer backend connection、minifb wait hook への接続、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。

## F5gy Native window host-loop timer fire/wakeup backend boundary

2026-06-19 の F5gy では、F5gx の `FrameIntervalTimerRegistered` を backend-observed timer fire evidence へ接続する。registration は timer を予約した証拠であり、fire / wakeup はその timer が実際に ready になった証拠であるため、両者を同じ success outcome として扱わない。

`NativeWindowHostLoopTimerFireWaiter` は `wait_for_timer_fire` に registered timer id を受け取り、backend が観測した fired raw id を返す。`execute_native_window_host_loop_timer_fire_wait_with_waiter` は `FrameIntervalTimerRegistered` の場合だけ waiter を呼び、fired raw id `0` を `InvalidFiredTimerRegistrationId`、registered id と異なる fired raw id を `FiredTimerRegistrationMismatch` として返す。

`HostEventPumpAlreadyPaced` と `FramePresentAlreadyPaced` は timer fire input ではないため、それぞれ `HostEventPumpOutcomeUnsupported` と `FramePresentOutcomeUnsupported` で拒否する。already-paced outcome を timer fire success として扱うことは禁止する。

F5gy は timer fire / wakeup backend boundary までであり、OS 固有 timer API、selector wakeup ownership、minifb wait hook への接続、scheduler resume policy、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。

## F5gz Native window host-loop timer wakeup executor boundary

2026-06-19 の F5gz では、formal native timer の registration と fire wait を一つの backend wakeup executor として合成する。ただし、registration 成功と fire 成功は別の契約であり、合成 executor はそれぞれの failure を `NativeWindowHostLoopTimerWakeError` の `RegistrationFailed` と `FireFailed` に分けて返す。

`execute_native_window_host_loop_timer_wakeup_with_backend` は `NativeWindowHostLoopWaitInstruction` を受け、まず `execute_native_window_host_loop_timer_registration_wait_with_registrar` を呼ぶ。registration が失敗した場合は waiter を呼ばず `RegistrationFailed` を返す。registration outcome が得られた場合だけ `execute_native_window_host_loop_timer_fire_wait_with_waiter` を呼び、fire failure は `FireFailed` として返す。

success は `NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired` のみである。host event wait、invalid registration id、registrar failure、invalid fired id、mismatched fired id、waiter failure はすべて enum variant として保持し、fallback、silent no-op、thread sleep、message pump、event queue substitution へ変換しない。

F5gz は scheduler resume policy が後続で消費できる typed wakeup boundary までであり、OS 固有 timer API、selector ownership、minifb wait hook 接続、real scheduler driver、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。

## F5ha Native window host-loop scheduler timer resume gate boundary

2026-06-19 の F5ha では、scheduler long runner の resume gate を追加する。`NativeWindowHostLoopSchedulerSliceResult::Waited` は「host wait hook が戻った」ことだけを表し、常に application loop を再開してよいことを意味しない。

`NativeWindowHostLoopSchedulerResumeState` は `Ready` と `WaitingForFrameIntervalTimer` を分ける。`HostEventPumpAlreadyPaced` と `FramePresentAlreadyPaced` は ready evidence として扱えるが、`FrameIntervalTimerRegistered` は timer fire を待つための pending state である。timer fire が実際に観測された場合だけ、`NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired` から `NativeWindowHostLoopSchedulerResumeReady::FrameIntervalTimerFired` を作れる。

`run_native_window_host_loop_with_policy_and_target_fps` は `Waited` outcome を resume gate に通し、`WaitingForFrameIntervalTimer` の場合は `NativeWindowHostLoopError::TimerFireResumeRequired` を返す。これにより、timer registration を wait completion と誤認して次の event poll や present へ進むことを禁止する。

F5ha は resume gate までであり、real OS timer adapter、selector ownership、minifb timer path、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。fallback、silent no-op、sleep / busy-loop による timer fire 代替は禁止する。

## F5hb Native window host-loop std deadline timer adapter boundary

2026-06-19 の F5hb では、native host-loop 用の std deadline timer adapter を追加する。adapter は `NativeWindowHostLoopTimerRegistrar` と `NativeWindowHostLoopTimerFireWaiter` の contract を同じ host-owned timer state で実装する。

`NativeWindowHostLoopDeadlineTimerAdapter` は injected clock と injected sleeper を持つ。registration では active timer が無いこと、timer id が overflow しないこと、`now_nanos + wait_nanos` が overflow しないことを検査してから active timer を作る。fire wait では active timer が存在し、requested id と active id が一致する場合だけ sleeper を呼び、sleep 成功後に active timer を消費する。

error は `NativeWindowHostLoopDeadlineTimerAdapterError` に分離する。active timer overlap、missing active timer、timer id overflow、deadline overflow、clock failure、sleeper failure、mismatched fired id を generic backend failure に潰してはいけない。sleeper failure では active timer を保持する。

std 実装は `StdNativeWindowHostLoopDeadlineTimerClock` と `StdNativeWindowHostLoopDeadlineTimerSleeper` に閉じ込める。unit test は scripted clock / sleeper を使い、実時間 sleep には依存しない。

F5hb は std deadline timer adapter boundary までであり、selector ownership、OS message loop timer、queue integration、minifb wait hook 接続、`Window::set_target_fps` の置換、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。minifb smoke backend が `FramePresentAlreadyPaced` を返す contract は維持する。

## F5hc Native window host-loop timer fired wait outcome boundary

2026-06-19 の F5hc では、timer fire completion を `NativeWindowHostLoopWaitOutcome` として運ぶ contract を追加する。

`NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered` は timer が登録されたことだけを表す pending state である。これは scheduler ready evidence ではない。`NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired` は backend が registered timer id と fired timer id の一致を確認した completion state であり、`NativeWindowHostLoopSchedulerResumeReady::FrameIntervalTimerFired` へ進められる。

`execute_native_window_host_loop_timer_wakeup_wait_with_backend` と `execute_native_window_host_loop_deadline_timer_wakeup_wait_with_adapter` は、registration failure と fire wait failure を `NativeWindowHostLoopTimerWakeError` の別段階として保持する。成功時だけ `NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired` を fired wait outcome へ写す。

F5hc は timer fired wait outcome boundary までであり、selector ownership、OS message loop timer、queue integration、minifb wait hook 接続、`Window::set_target_fps` の置換、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。fallback、silent no-op、sleep / busy-loop による timer fire 代替は禁止する。

## F5hd Native window host-loop wait owner boundary

2026-06-19 の F5hd では、host-loop wait instruction を host event queue wait path と frame interval deadline timer path に分配する owner boundary を追加する。

`NativeWindowHostLoopWaitOwner` は event queue waiter と deadline timer adapter を同じ owner に保持する。ただし 2 つの backend は互いの詳細を知らない。`execute_native_window_host_loop_wait_with_owner` が `WaitForHostEvent` を event queue waiter へ、`WaitForFrameInterval` を deadline timer wakeup wait helper へだけ渡す。

error は `NativeWindowHostLoopWaitOwnerError` に分ける。`EventQueueWaitFailed` は `NativeWindowHostLoopEventQueueWaitError` 全体を保持し、`FrameIntervalTimerWakeFailed` は `NativeWindowHostLoopDeadlineTimerWakeError` 全体を保持する。lower error を string や generic host failure に潰してはいけない。

host event path の成功は `HostEventPumpAlreadyPaced` へ写す。frame interval path の成功は `FrameIntervalTimerFired` として返す。host event path が timer clock / sleeper を呼ぶこと、frame interval path が event queue waiter を呼ぶことは禁止する。

F5hd は wait owner composition boundary までであり、selector ownership、OS message loop timer、minifb wait hook の pacing 置換、`Window::set_target_fps` の置換、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。fallback、silent no-op、busy loop による代替は禁止する。

## F5he Native minifb frame pacing authority boundary

2026-06-19 の F5he では、minifb smoke backend の frame interval wait を `NativeWindowMinifbFramePacingAuthority` に閉じる。

authority は `NativeWindowTargetFps` を保持する。`WaitForFrameInterval` instruction が持つ `NativeWindowFrameIntervalRequest.target_fps` と authority target が一致し、`wait_nanos` が `nanos_per_frame` または `nanos_per_frame + 1` の場合だけ `NativeWindowHostLoopWaitOutcome::FramePresentAlreadyPaced` を返す。不一致は `NativeWindowMinifbFramePacingAuthorityError` の target fps mismatch / wait nanos mismatch として返す。

`FramePresentAlreadyPaced` は、minifb internal `Window::set_target_fps` pacing が active authority であることを表す evidence である。wait hook が sleep、deadline timer wait、OS selector wait を行ったという意味ではない。

`run_minifb_window_loop` は `config.target_fps` から authority を作り、authority の `target_fps_usize` だけを `Window::set_target_fps` へ渡す。`set_target_fps 0` は minifb internal wait を無効化し、現在の host event wait path を tight loop にするため禁止する。future selector / message-loop timer backend を minifb に接続する場合は、minifb internal pacing と deadline timer owner の二重 authority を避ける。

F5he は minifb internal target-fps pacing authority boundary までであり、F5hd wait owner の minifb hook 接続、`Window::set_target_fps` の置換、selector ownership、OS message-loop timer、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。

## F5hf Native frame interval wait authority mode boundary

2026-06-19 の F5hf では、frame interval wait の authority を `NativeWindowFrameIntervalWaitAuthorityMode` として表す。

`NativeWindowFrameIntervalWaitAuthorityMode` は `MinifbInternalTargetFps target_fps` と `HostOwnedDeadlineTimer` を持つ。前者は minifb internal `Window::set_target_fps` pacing が wait authority であることを表し、後者は future selector / message-loop timer path が deadline timer owner を authority として使う場合の compatibility marker である。

`combine_native_window_frame_interval_wait_authority_mode` は同じ minifb target fps 同士と host-owned deadline timer 同士だけを受け入れる。minifb internal pacing と host-owned deadline timer を同時 authority にすること、または target fps が異なる minifb authority を同時に使うことは `ConflictingFrameIntervalAuthorities` として拒否する。

`validate_native_window_frame_interval_wait_authority_mode` は minifb mode の場合だけ instruction の `NativeWindowFrameIntervalRequest.target_fps` と authority target を照合する。不一致は `TargetFpsMismatch` として返す。host-owned deadline timer mode は wait instruction との compatibility check だけであり、`FramePresentAlreadyPaced`、`FrameIntervalTimerRegistered`、`FrameIntervalTimerFired` の evidence を生成しない。

`NativeWindowMinifbFramePacingAuthority` は `FramePresentAlreadyPaced` を返す前に、自分の `MinifbInternalTargetFps` mode をこの validation helper に通す。F5hf は二重 authority を拒否する safety boundary であり、selector ownership、OS message-loop timer、F5hd wait owner の minifb hook 接続、`Window::set_target_fps` の置換、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。fallback、silent no-op、busy loop による代替は禁止する。

## F5hg Native wait owner frame interval authority connection boundary

2026-06-19 の F5hg では、F5hf の authority mode を F5hd の formal wait owner へ接続する。

`NativeWindowHostLoopWaitOwner::frame_interval_wait_authority_mode` は `HostOwnedDeadlineTimer` を返す。これは formal owner path では frame interval wait の authority が deadline timer owner にあることを表す。`execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode` は explicit requested authority mode を受け取り、frame interval branch で owner authority と requested authority を `combine_native_window_frame_interval_wait_authority_mode` に通してから `validate_native_window_frame_interval_wait_authority_mode` を行う。

authority validation が成功した場合だけ `execute_native_window_host_loop_deadline_timer_wakeup_wait_with_adapter` へ進む。authority mismatch は `FrameIntervalAuthorityFailed` として返し、deadline timer registration、clock read、sleeper call、active timer mutation は起こさない。host event wait は frame interval authority を参照せず、event queue waiter だけへ渡す。

既存の `execute_native_window_host_loop_wait_with_owner` は owner 自身の `HostOwnedDeadlineTimer` authority を渡す wrapper である。F5hg は selector / message-loop timer ownership へ進む前の ownership safety connection であり、macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd、minifb wait hook の wait owner 接続、`Window::set_target_fps` の置換、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。

## F5hh Native run-loop frame interval wait backend selection boundary

2026-06-19 の F5hh では、native run-loop config が frame interval wait backend selection を持つ。

`NativeWindowRunLoopFrameIntervalWaitBackend` は `MinifbInternalTargetFps` と `HostOwnedDeadlineTimer` を持つ。default は `MinifbInternalTargetFps` である。これは現在の minifb smoke backend が `Window::set_target_fps` based pacing を authority としているためである。

`NativeWindowRunLoopFrameIntervalWaitBackend::authority_mode` は F5hf の `NativeWindowFrameIntervalWaitAuthorityMode` へ変換する。minifb runner は active authority を `MinifbInternalTargetFps target_fps` とし、requested backend authority と `combine_native_window_frame_interval_wait_authority_mode` で照合する。`HostOwnedDeadlineTimer` が requested backend の場合は `NativeWindowRunLoopError::FrameIntervalWaitBackendUnsupported` を返す。

この validation は `NativeWindowBackendLoop::new_for_scale`、minifb `Window::new`、`Window::set_target_fps`、host construction、loop execution より前に行う。unsupported backend を minifb internal pacing へ fallback してはならない。error は runner、requested backend、authority conflict reason を保持する。

F5hh は backend selection boundary であり、macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd は実装しない。minifb wait hook を F5hd wait owner / std deadline timer adapter へ接続しない。`set_target_fps 0`、fallback、silent no-op、busy loopは禁止する。

## F5hi Native host-owned deadline wait run-loop host wrapper boundary

2026-06-19 の F5hi では、native run-loop host contract と formal wait owner を接続する wrapper を追加する。

`NativeWindowHostOwnedDeadlineWaitRunLoopHost` は inner `NativeWindowRunLoopHost` と `NativeWindowHostLoopWaitOwner` を所有する。event polling、title update、pump-only、present は inner host の責務として残し、budget exhaustion 後の wait だけを owner helper に渡す。

この wrapper の `WaitError` は `NativeWindowHostLoopWaitOwnerError` であり、event queue wait failure、frame interval authority failure、deadline timer wake failure を enum として保持する。`EventError` と `PresentError` は inner host の associated type をそのまま使う。これにより wait owner の typed failure と platform host の event / present failure を混ぜずに扱える。

F5hi は future native OS backend / deterministic test backend 用の connection boundary であり、現在の minifb smoke runner には接続しない。minifb path は `Window::set_target_fps` authority を維持し、host-owned deadline timer へ fallback しない。real selector / message-loop timer backend、FHD 60fps 実測、2D compositor drain、font / stroke / shadow rasterization は後続である。

## F5hj Native interruptible deadline wait boundary

2026-06-19 の F5hj では、frame interval wait が deadline 到達または host event readiness のどちらでも wake できる boundary を追加する。

`NativeWindowHostLoopInterruptibleDeadlineWaiter` は host event wait と deadline-or-host-event wait を分ける。`NativeWindowHostLoopInterruptibleDeadlineWaitAdapter` は clock、waiter、positive timer id cursor を所有し、`execute_native_window_host_loop_interruptible_deadline_wait_with_adapter` が `NativeWindowHostLoopWaitOutcome` を返す。

`WaitForFrameInterval` は wait nanos を frame interval request と照合してから、timer id advance、clock read、deadline arithmetic、waiter call へ進む。deadline に到達した場合だけ `FrameIntervalTimerFired` を返す。host event readiness で wake した場合は `HostEventPumpAlreadyPaced` を返す。これは loop を次の polling へ戻せるという readiness evidence であり、frame deadline fired evidence ではない。candidate timer id は wait 開始前に advance するため、host event wake や frame wait failure の後も id reuse はしない。

error は host-event wait failure、frame wait nanos mismatch、timer id overflow、clock failure、deadline overflow、interruptible frame wait failure を区別する。fallback、timer-only wait への代替、thread sleep、busy loop、minifb internal pacing、synthetic fired evidenceは禁止する。F5hj は semantic interruptible wake boundary であり、macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd の actual implementation ではない。

## F5hk Native interruptible deadline wait run-loop host wrapper boundary

2026-06-19 の F5hk では、F5hj の interruptible wait adapter を `NativeWindowRunLoopHost` の wait hook として使える wrapper を追加する。

`NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost` は inner `NativeWindowRunLoopHost` と `NativeWindowHostLoopInterruptibleDeadlineWaitAdapter` を所有する。event polling、title update、pump-only、present は inner host へ委譲し、`wait_after_budget_exhausted` だけは inner host の wait hook を呼ばず `execute_native_window_host_loop_interruptible_deadline_wait_with_adapter` に渡す。

`EventError` と `PresentError` は inner host の associated type をそのまま使う。`WaitError` は `NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError` であり、host event wait failure、wait nanos mismatch、clock failure、deadline overflow、interruptible frame wait failure などを typed enum として保持する。

F5hk は future native OS backend / deterministic test backend が interruptible wait semantics を run-loop host contract から使うための接続境界である。現在の minifb smoke runner には接続しない。macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd、FHD 60fps 実測、2D compositor drain、font / stroke / shadow rasterization は後続である。fallback、thread sleep、busy loop、minifb internal pacing への代替、synthetic fired evidence は禁止する。

## F5hl Native platform wait backend selection boundary

2026-06-19 の F5hl では、real native wait backend の actual OS API を実装する前に、current platform と platform-specific wait backend の対応を typed enum と `Result` で固定する。

`NativeWindowHostLoopPlatformKind` は `Macos`、`Windows`、`Linux`、`Unsupported` を持つ。`native_window_host_loop_current_platform_kind` は `cfg(target_os = ...)` だけから current platform を返し、runtime string、environment variable、process argument、filesystem probe によって platform を推定しない。

`NativeWindowHostLoopPlatformWaitBackendKind` は `MacosRunLoopTimer`、`WindowsWaitableTimerMessageWait`、`LinuxSelectorTimerFd`、`HeadlessScripted` を持つ。`HeadlessScripted` は deterministic headless / test backend 用の future kind であり、native platform の default や fallback として成功させない。

`NativeWindowHostLoopPlatformWaitBackendSupportError` は `DefaultBackendUnsupportedPlatform`、`RequestedBackendUnsupportedPlatform`、`BackendPlatformMismatch` を持つ。validation failure は string ではなく、current platform と requested backend を typed data として返す。

`validate_native_window_host_loop_platform_wait_backend_kind_for_platform` は、macOS と `MacosRunLoopTimer`、Windows と `WindowsWaitableTimerMessageWait`、Linux と `LinuxSelectorTimerFd` の一致だけを成功にする。unsupported platform は requested backend を保持して `RequestedBackendUnsupportedPlatform` を返す。mismatch は `BackendPlatformMismatch` を返す。

`native_window_host_loop_default_platform_wait_backend_kind_for_platform` は macOS、Windows、Linux の default backend だけを返す。unsupported platform では `DefaultBackendUnsupportedPlatform` を返し、`HeadlessScripted` や minifb internal pacing へ fallback しない。

F5hl は selector / message-loop timer backend の selection contract であり、macOS AppKit / CoreFoundation run loop timer、Win32 waitable timer / message wait、Linux selector / timerfd の actual implementation ではない。minifb runner、`Window::set_target_fps` authority、thread sleep、busy loop、synthetic timer-fired evidence、DOM / Canvas / video memory transport へは接続しない。

## F5hm Native platform wait backend construction gate boundary

2026-06-20 の F5hm では、F5hl の platform/backend selection を actual backend construction の入口にするため、検証済み token と construction gate を追加する。

`NativeWindowHostLoopPlatformWaitBackendSelection` は current platform と backend kind を保持する検証済み token である。field は private であり、public constructor は置かない。caller は `validate_native_window_host_loop_platform_wait_backend_selection_for_platform` または `native_window_host_loop_default_platform_wait_backend_selection_for_platform` を通して token を得る。

`NativeWindowHostLoopPlatformWaitHostBuildError` は `BackendSupportFailed` と `BackendImplementationUnavailable` を分ける。`BackendSupportFailed` は mismatch、unsupported platform、native selection としての `HeadlessScripted` など F5hl validation failure を保持する。`BackendImplementationUnavailable` は validated selection 後に actual backend がまだ実装されていないことを platform/backend pair として保持する。

`build_native_window_host_loop_platform_wait_backend_from_selection` は actual backend を作ったことにしない。現 checkpoint では dummy host、headless scripted backend、minifb pacing、thread sleep、busy loop、synthetic timer fire を返さず、validated selection の platform/backend を保持した `BackendImplementationUnavailable` を返す。owner を受け取る builder はまだ追加しないため、construction failure で owner を失う経路も作らない。

## F5hn Native Windows waitable timer message wait raw backend boundary

2026-06-20 の F5hn では、Windows waitable timer / message wait backend の raw API boundary を追加する。これは Windows OS API を cross-platform testable contract の後ろへ閉じ込める phase であり、generic platform wait builder の behavior は F5hm の fail-closed contract を維持する。

`NativeWindowHostLoopWindowsWaitHandle` は raw handle を private field として保持する。`native_window_host_loop_windows_wait_handle_from_raw` は `0` と `-1` を `InvalidRawHandle` として拒否し、backend は validation 済み handle だけを受け取る。raw handle extraction と owned handle exposure は public API に置かず、backend は close ownership を自分の `Drop` と explicit close path だけで管理する。

`NativeWindowHostLoopWindowsDeadlinePlan` は `AlreadyReached` と `Relative100ns` を分ける。deadline が current elapsed time 以下の場合、waitable timer を arm せず `DeadlineReached` を返す。future deadline は nanosecond delta を 100ns 単位へ切り上げ、Windows relative due time の負値へ変換する。overflow は `DeadlineDelta100nsOverflow` として返し、saturating / clamp はしない。

`NativeWindowHostLoopWindowsWaitRawApi` は waitable timer 作成、relative 100ns timer arm、timer-or-message wait、message-only wait、handle close、last error retrieval を分離する。`NativeWindowHostLoopWindowsWaitBackend` はこの raw API trait と `Instant` だけを持ち、`NativeWindowHostLoopDeadlineTimerClock` と `NativeWindowHostLoopInterruptibleDeadlineWaiter` を実装する。

host event wait は `msg_wait_for_message_raw` だけを呼ぶ。frame interval wait は `set_waitable_timer_relative_100ns` の成功後に `msg_wait_for_timer_or_message_raw` を呼ぶ。timer status は `DeadlineReached`、message status は `HostEventReady`、failed status は `WaitFailed { code }`、timeout / unknown status は `UnexpectedWaitStatus` に写す。message-only wait では zero-handle message-ready status だけを success とする。

cfg-windows の `NativeWindowHostLoopWindowsWaitSysApi` は `CreateWaitableTimerW`、`SetWaitableTimer`、`MsgWaitForMultipleObjects`、`CloseHandle`、`GetLastError` を実装詳細として持つ。Windows-specific builder は validated Windows selection と raw API を受けた場合だけ backend を返し、selection support failure と raw API failure を別 error として返す。

F5hn は macOS run loop timer、Linux selector / timerfd、minifb wait hook 接続、formal FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization の実装ではない。sleep、busy loop、minifb pacing、scripted native substitute、synthetic timer fire、silent no-op による代替成功は禁止する。

## F5ho Native single-owner interruptible wait adapter boundary

2026-06-20 の F5ho では、clock と interruptible waiter を同一 backend owner に保持する adapter を追加する。これは F5hn の Windows backend を run-loop wait hook へ接続する前の ownership integration であり、waitable timer handle owner を二重化しないための境界である。

`NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter Backend` は `Backend` を 1 個だけ所有する。`Backend` は `NativeWindowHostLoopDeadlineTimerClock` と `NativeWindowHostLoopInterruptibleDeadlineWaiter` を同時に実装し、両者の `Error` type は同一でなければならない。この制約により、clock と waiter を別 object として渡す汎用 adapter とは別に、single owner backend 用の path を持つ。

single-owner executor は host event instruction では `wait_for_host_event` だけを呼ぶ。frame interval instruction では wait nanos validation、timer id overflow validation、clock read、deadline addition、timer id advance、backend wait の順で処理する。invalid wait nanos と timer id overflow は clock read / backend wait より前に fail closed する。

backend wait の結果は `HostEventReady` を `HostEventPumpAlreadyPaced`、`DeadlineReached` を `FrameIntervalTimerFired` に写す。`HostEventReady` から timer-fired evidence は生成しない。deadline wait 直前に進んだ timer id は、host event wake や backend wait error の場合も再利用しない。

`NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost` は inner host と single-owner adapter を所有する wrapper である。poll / title / pump / present は inner host へ委譲し、wait hook だけを single-owner executor に渡す。inner host の wait hook は呼ばない。

F5ho は generic platform wait builder、minifb smoke runner、`Window::set_target_fps` replacement、macOS run loop timer、Linux selector / timerfd へ接続しない。sleep、busy loop、minifb pacing、scripted native substitute、synthetic timer fire、fallback、silent no-op は禁止する。

## F5hp Native Windows platform wait support gate boundary

2026-06-20 の F5hp では、Windows wait backend を platform wait backend owner として扱い、single-owner run-loop wait hook へ接続する二段階 support gate を追加する。

`NativeWindowHostLoopPlatformWaitBackend Api` は、この checkpoint では `WindowsWaitableTimerMessageWait` variant だけを持つ。variant は `NativeWindowHostLoopWindowsWaitBackend Api` を所有し、`NativeWindowHostLoopDeadlineTimerClock` と `NativeWindowHostLoopInterruptibleDeadlineWaiter` は Windows backend へ委譲する。error は `NativeWindowHostLoopPlatformWaitBackendError` によって Windows backend error を保持する。

`build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api` は `NativeWindowHostLoopPlatformWaitBackendSelection` を再検査し、Windows + WindowsWaitableTimerMessageWait の場合だけ supplied raw API で Windows backend を構築する。MacOS / Linux の validated selection は `BackendImplementationUnavailable` として返す。mismatch、unsupported platform、native `HeadlessScripted` selection は `BackendSupportFailed` として返す。Windows raw API construction failure は `WindowsWaitBackendFailed` として保持する。

旧 F5hm の `build_native_window_host_loop_platform_wait_backend_from_selection` は API / handle owner を受け取らないため、引き続き no-owner fail-closed probe として `BackendImplementationUnavailable` を返す。暗黙の sys API 作成、dummy host、headless scripted fallback、minifb pacing、thread sleep、busy loop、synthetic timer fire は返さない。

`native_window_host_loop_platform_wait_run_loop_host_from_backend` は backend construction 成功後の infallible helper である。host と platform backend を受け取り、`NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter` と `NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost` を組み立てる。support failure や Windows backend construction failure では host owner を消費しない。

cfg-windows の `native_window_host_loop_platform_wait_backend_from_selection` は `NativeWindowHostLoopWindowsWaitSysApi` を使う backend construction helper だけを提供する。host-consuming fallible helper は作らない。F5hp は minifb runner、`Window::set_target_fps`、macOS run loop timer、Linux selector / timerfd、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へ接続しない。

## F5hq Native Windows platform wait run-loop config gate boundary

2026-06-20 の F5hq では、native run-loop configuration が wait backend selection を単一 field で保持する contract を追加する。`NativeWindowRunLoopConfig` は `wait_backend: NativeWindowRunLoopWaitBackend` を唯一の wait backend authority とし、旧 `frame_interval_wait_backend` field は持たない。旧 `NativeWindowRunLoopFrameIntervalWaitBackend` は compatibility constructor input としてだけ使い、config 内の authority にはしない。

`NativeWindowRunLoopWaitBackend` は `MinifbInternalTargetFps`、`HostOwnedDeadlineTimer`、`PlatformWait NativeWindowHostLoopPlatformWaitBackendSelection` を持つ。default は `MinifbInternalTargetFps` であり、既存 minifb smoke runner はこの default を使い続ける。`PlatformWait` は validated platform wait selection token を保持し、frame interval authority としては host-owned deadline timer として扱う。

`run_minifb_window_loop` は `config.wait_backend` を `NativeWindowBackendLoop::new_for_scale`、`Window::new`、`Window::set_target_fps` より前に検査する。`HostOwnedDeadlineTimer` と `PlatformWait` は minifb internal pacing と conflict するため、typed `FrameIntervalWaitBackendUnsupported` として拒否する。`PlatformWait` を minifb internal pacing、sleep、busy loop、headless scripted、synthetic timer fire へ fallback してはいけない。

`native_window_run_loop_platform_wait_backend_selection` は config から platform wait selection を取り出す。non-platform config では typed `NotPlatformWaitBackend` を返す。cfg-windows の `native_window_run_loop_platform_wait_backend_from_config` は config / selection から backend を構築するだけであり、host owner を受け取らない。host wrapping は、backend construction 成功後に F5hp の infallible wrapper へ渡す。

MacOS / Linux の actual backend はまだ未実装であり、typed unavailable / support failure として扱う。F5hq は minifb runner replacement、`Window::set_target_fps 0`、macOS run loop timer、Linux selector / timerfd、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization を追加しない。

## F5hr Native Windows platform wait window runner support gate

2026-06-20 の F5hr では、cfg-windows の `PlatformWait` config を actual native window run-loop runner の wait hook へ接続する。入口は `run_windows_platform_wait_window_loop` であり、既存 `run_minifb_window_loop` は置き換えない。minifb smoke runner は引き続き minifb internal target-fps pacing を所有し、`PlatformWait` config は side effect 前に拒否する。

Windows platform wait runner は、最初に `native_window_run_loop_platform_wait_backend_from_config` を実行する。non-platform config や backend construction failure は `NativeWindowRunLoopError::PlatformWaitBackendFromConfigFailed` として返し、backend construction が成功するまで `NativeWindowBackendLoop`、minifb `Window`、host adapter を作らない。

backend construction 成功後、runner は minifb `Window` を visual / event / present host としてだけ使う。`MinifbNativeWindowVisualRunLoopHost` は `poll_event_snapshot`、title update、pump、present を実装し、wait hook は `VisualHostWaitUnsupported` として fail closed にする。actual wait は `native_window_host_loop_platform_wait_run_loop_host_from_backend` が作る single-owner interruptible deadline wait hook にだけ委譲される。

この runner は `Window::set_target_fps`、`configure_minifb_window_frame_pacing`、`set_target_fps 0`、thread sleep、busy loop、headless scripted fallback、synthetic timer fire を使わない。Windows wait backend の clock / waiter failure は `NativeWindowRunLoopError::WindowsPlatformWaitHostLoopFailed` の中に typed host-loop error として保持し、string に潰さない。

CLI flag での選択、macOS / Linux actual backend、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は後続 phase とする。

## F5hs Native Windows platform wait CLI selection boundary

2026-06-20 の F5hs では、native CLI の window runner selection を `--wait-backend minifb|platform` として明示する。指定が無い場合は existing minifb smoke runner を使い、`Window::set_target_fps` による minifb internal pacing を保持する。`platform` が指定された場合だけ、validated default platform wait backend selection を `NativeWindowRunLoopConfig::new_with_platform_wait_backend_selection` に入れ、cfg-windows の `run_windows_platform_wait_window_loop` を呼ぶ。

`--wait-backend` は window mode の option であり、headless mode では明示指定を error とする。重複指定や未知の値も error とし、最後の値で上書きしない。headless mode で指定が無い場合は従来通り deterministic frame output を返す。

CLI layer は config と runner selection だけを担当し、minifb lifecycle、event pump、presenter state、OS wait API、sleep、busy loop を持たない。non-Windows platform selection は unsupported error を返し、minifb runner に自動的に切り替えない。

この checkpoint は CLI selection boundary であり、macOS run loop timer、Linux selector / timerfd、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は後続 phase とする。

## F5ht Native macOS run loop timer raw backend boundary

2026-06-20 の F5ht では、macOS run loop timer backend の raw boundary contract を追加する。標準 API は AppKit / CoreFoundation handle を public type として公開しないため、`NativeWindowHostLoopMacosRunLoopTimerHandle` は raw handle を private field として保持する。null / invalid sentinel は typed error にし、raw handle accessor や public owned-handle escape は作らない。

raw API contract は、timer creation、relative nanos scheduling、timer-or-host-event wait、host-event-only wait、timer invalidation、last error code を分ける。timer-or-event wait は `TimerFired` と `HostEventReady` を別 enum に写し、host-event-only wait では timer-fired raw status を host event ready と偽装しない。失敗 status と未知 status は、それぞれ `RunLoopWaitFailed` と `UnexpectedRunLoopStatus` として返す。

deadline conversion は monotonic origin からの elapsed nanoseconds を使い、checked arithmetic だけで relative delay を作る。deadline 到達済みは immediate `TimerFired` として返すが、sleep、busy loop、saturating / clamp、silent no-op、fallback は使わない。backend は explicit invalidation と drop cleanup の両方で handle invalidation を高々 1 回に制限する。

F5ht は raw API / fake raw API tests / source policy の boundary であり、generic `NativeWindowHostLoopPlatformWaitBackend` に `MacosRunLoopTimer(...)` owner variant を追加しない。`NativeWindowHostLoopDeadlineTimerClock` / `NativeWindowHostLoopInterruptibleDeadlineWaiter` implementation、CLI dispatch、actual macOS sys shim、native runner connection、Linux selector / timerfd、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は後続 phase とする。

## F5hu Native Linux selector timerfd raw backend boundary

2026-06-20 の F5hu では、Linux selector / timerfd backend の raw boundary contract を追加する。標準 API は Linux file descriptor を public owner として公開しないため、`NativeWindowHostLoopLinuxSelectorFd` と `NativeWindowHostLoopLinuxTimerFd` は raw fd を private field として保持する。fd `0` は有効な descriptor として扱い、invalid fd は `raw_fd < 0` だけで拒否する。raw fd accessor や public owned-fd escape は作らない。

raw API contract は、selector creation、timerfd creation、timerfd registration、relative timespec arm、timer-or-host-event wait、host-event-only wait、selector close、timerfd close、last error code を分ける。selector fd と timerfd は別 owner とし、construction failure では作成済み fd を逆順に close する。backend cleanup は timerfd と selector をそれぞれ高々 1 回だけ close する。

deadline conversion は monotonic origin からの elapsed nanoseconds を使い、checked arithmetic だけで relative timespec を作る。deadline 到達済みは immediate `TimerFired` として返す。relative timespec は seconds と nanoseconds を `/ 1_000_000_000`、`% 1_000_000_000`、`i64::try_from` で作り、sleep、busy loop、saturating / clamp、silent no-op は使わない。

timer-or-event wait は `TimerFired` と `HostEventReady` を別 enum に写す。host-event-only wait では timer-fired raw status を host event ready と偽装しない。失敗 status と未知 status は、それぞれ `SelectorWaitFailed` と `UnexpectedSelectorStatus` として返す。

F5hu は raw API / fake raw API tests / source policy の boundary であり、generic `NativeWindowHostLoopPlatformWaitBackend` に `LinuxSelectorTimerFd(...)` owner variant を追加しない。`NativeWindowHostLoopDeadlineTimerClock` / `NativeWindowHostLoopInterruptibleDeadlineWaiter` implementation、CLI dispatch、actual Linux sys shim、native runner connection、macOS actual sys shim、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は後続 phase とする。

## F5hv Native Linux selector timerfd single-owner wait trait boundary

2026-06-20 の F5hv では、F5hu の Linux selector / timerfd raw backend を single-owner interruptible deadline wait contract へ接続する。`NativeWindowHostLoopLinuxSelectorTimerFdBackend` は `NativeWindowHostLoopDeadlineTimerClock` と `NativeWindowHostLoopInterruptibleDeadlineWaiter` を実装し、F5ho の single-owner adapter から扱える。

`now_nanos` は backend が保持する monotonic origin からの elapsed nanoseconds を checked conversion で返す。frame interval wait では Linux raw wake の `TimerFired` を `DeadlineReached`、`HostEventReady` を `HostEventReady` へ写す。host-event-only wait は F5hu の event-only raw wait を使い、timer-fired status を host event として受け入れない。これにより host event wake と timer fired evidence は enum の境界で分離される。

F5hv は trait adapter boundary であり、generic `NativeWindowHostLoopPlatformWaitBackend` の Linux owner variant、Linux actual sys shim、CLI dispatch、native runner connection、minifb wait path、sleep / busy loop / fallback / silent no-op は追加しない。

## F5hw Native macOS run loop timer single-owner wait trait boundary

2026-06-20 の F5hw では、F5ht の macOS run loop timer raw backend を single-owner interruptible deadline wait contract へ接続する。`NativeWindowHostLoopMacosRunLoopTimerBackend` は `NativeWindowHostLoopDeadlineTimerClock` と `NativeWindowHostLoopInterruptibleDeadlineWaiter` を実装し、F5ho の single-owner adapter から扱える。

`now_nanos` は backend が保持する monotonic origin からの elapsed nanoseconds を checked conversion で返す。frame interval wait では macOS raw wake の `TimerFired` は `DeadlineReached`、`HostEventReady` は `HostEventReady` へ写す。host-event-only wait では timer-fired status を host event として受け入れない。これにより host event wake と timer fired evidence は enum の境界で分離される。

F5hw は trait adapter boundary であり、generic `NativeWindowHostLoopPlatformWaitBackend` の macOS owner variant、macOS actual sys shim、CoreFoundation / AppKit binding、CLI dispatch、native runner connection、minifb wait path、sleep / busy loop / fallback / silent no-op は追加しない。

## F5hx Native platform wait multi-backend owner boundary

2026-06-20 の F5hx では、Windows / macOS / Linux の raw backend owner を 1 つの typed platform wait owner enum に統合する。これは F5hl の platform selection、F5hn の Windows raw backend、F5hv の Linux wait trait 実装、F5hw の macOS wait trait 実装を接続する ownership boundary であり、actual macOS / Linux sys shim や native runner dispatch ではない。

`NativeWindowHostLoopPlatformWaitBackend WindowsApi MacosApi LinuxApi` は `WindowsWaitableTimerMessageWait`、`MacosRunLoopTimer`、`LinuxSelectorTimerFd` を持つ。各 variant は対応 backend owner を保持し、clock と interruptible deadline waiter の実装は selected backend へ委譲する。error は `NativeWindowHostLoopPlatformWaitBackendError WindowsError MacosError LinuxError` の variant として保持し、どの platform backend で失敗したかを typed data として残す。

`build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis` は selection を再検査してから、selected platform の raw API だけを使う。mismatch、unsupported platform、headless scripted は raw API construction の前に `BackendSupportFailed` として返す。selected されていない raw API を触って fallback backend、dummy backend、best-effort success を作ってはいけない。

`build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api` は Windows cfg runner 用 compatibility wrapper として残す。return type は `NativeWindowHostLoopWindowsOnlyPlatformWaitBackend WindowsApi` であり、macOS / Linux raw API の type parameter には empty enum の never raw API を使う。macOS / Linux selection はこの wrapper では `BackendImplementationUnavailable` として返す。

旧 no-owner builder `build_native_window_host_loop_platform_wait_backend_from_selection` は引き続き fail-closed probe である。API / handle owner を受け取らないため、validated real backend でも `BackendImplementationUnavailable` を返す。implicit sys API creation、headless scripted fallback、minifb pacing、thread sleep、busy loop、synthetic timer fire は禁止する。

Never raw API は `NativeWindowHostLoopNeverMacosRunLoopTimerRawApi` と `NativeWindowHostLoopNeverLinuxSelectorTimerFdRawApi` として表す。どちらも empty enum であり、trait method body は `match *self {}` のみである。panic、unreachable、dummy raw handle、dummy status、no-op success を返さない。

F5hx は platform wait owner の統合までを扱う。`#[cfg(target_os = "macos")]` actual sys shim、CoreFoundation / AppKit binding、`#[cfg(target_os = "linux")]` actual sys shim、libc / nix / epoll / poll / select / timerfd、native runner / CLI dispatch、minifb wait path、sleep、busy loop、fallback、silent no-op、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は後続 phase とする。

## F5hy Native Linux selector timerfd actual sys shim boundary

2026-06-20 の F5hy では、F5hu/F5hv の Linux selector / timerfd raw API に対して、`#[cfg(target_os = "linux")]` の actual syscall shim を追加する。sys shim は `NativeWindowHostLoopLinuxSelectorTimerFdSysApi` として `NativeWindowHostLoopLinuxSelectorTimerFdRawApi` だけを実装し、standard API や cross-platform owner enum へ `libc` / `epoll` / `timerfd` symbol を漏らさない。

selector は `epoll_create1 EPOLL_CLOEXEC`、timer は `timerfd_create CLOCK_MONOTONIC TFD_CLOEXEC|TFD_NONBLOCK` で作る。timer registration は `epoll_ctl EPOLL_CTL_ADD` で timer fd を `EPOLLIN` として登録する。relative deadline arm は checked conversion 済みの `NativeWindowHostLoopLinuxTimerFdTimespec` だけを `timerfd_settime` の `it_value` に入れ、`it_interval` は zero にする。clamp、saturating、implicit interval、sleep、busy loop は使わない。

timer-or-event wait は `epoll_wait` が timer fd readiness を返し、さらに `read` で `u64` expiration count を exactly drain でき、count が 0 でない場合だけ `TimerFired` evidence に変換できる。unknown fd、unexpected event flags、timeout、`epoll_wait` failure、`read` failure、short read、zero expiration は `FAILED` status と `last_error_code` へ保存し、synthetic timer fired evidence を作らない。

host-event-only wait は現 raw trait に host event fd registration boundary が無いため、`ENOTSUP` 相当の failure として fail closed にする。`HostEventReady` を返して host event wake を捏造しない。F5hy 単独では host event fd integration、generic platform wait helper、native runner / CLI connection、macOS actual sys shim、minifb wait path、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は後続 phase とする。

## F5hz Native Linux host event fd integration boundary

2026-06-20 の F5hz では、F5hy の Linux sys shim に host event fd owner を追加する。`NativeWindowHostLoopLinuxHostEventFd` は selector / timerfd と同じ private raw fd owner であり、fd `0` は有効、`raw_fd < 0` だけを invalid とする。public raw fd accessor は作らない。

`NativeWindowHostLoopLinuxSelectorTimerFdRawApi` は `create_host_event_fd_raw`、`register_host_event_fd_raw`、`close_host_event_fd_raw` を持ち、timer-or-event wait と host-event-only wait は host event fd owner を明示的に受け取る。backend は selector、timerfd、host event fd の 3 owner を持つ。constructor は selector、timerfd、timer registration、host event fd、host event registration の順で作り、失敗時は作成済み owner を逆順で close する。close 成功で元の失敗 code を消さない。

cfg Linux sys shim は host event fd として `eventfd 0 EFD_CLOEXEC | EFD_NONBLOCK` を使い、selector へ `EPOLLIN` で登録する。timer-or-event wait は ready fd が timerfd なら timerfd の `u64` expiration drain 後だけ `TimerFired`、host event fd なら eventfd の `u64` counter drain 後だけ `HostEventReady` を返す。host-event-only wait は host event fd readiness だけを成功にする。unknown fd、terminal flags、short read、zero count、syscall failure は typed wait failure として返す。

F5hz は host event fd owner / registration / readiness mapping までであり、eventfd への write/signal producer、generic platform wait helper の Linux sys constructor、native runner / CLI connection、minifb wait path、macOS actual sys shim、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は後続 phase とする。fallback、silent no-op、synthetic host event、timer fired evidence の偽装は禁止する。

## F5ia Native Linux platform wait support helper boundary

2026-06-20 の F5ia では、F5hz までに実装した cfg Linux `NativeWindowHostLoopLinuxSelectorTimerFdSysApi` を、generic platform wait owner の Linux-only helper へ接続する。これは Linux sys API を使って `NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd` を構築できるようにする convenience / support helper であり、runner、CLI、minifb wait path へ接続する phase ではない。

`NativeWindowHostLoopLinuxOnlyPlatformWaitBackend LinuxApi` は Windows / macOS 側を never raw API にし、LinuxApi だけを所有できる type alias とする。`build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api` は selection を先に再検査し、mismatch、unsupported platform、HeadlessScripted は raw API method を呼ぶ前に `BackendSupportFailed` として返す。Windows / macOS の validated selection は `BackendImplementationUnavailable` として返し、fallback や dummy backend にはしない。F5if 以降は Linux event source capability も explicit input として受け、Linux raw construction より前に検査する。Linux raw construction failure は `LinuxSelectorTimerFdBackendFailed` として保持する。

cfg Linux の `native_window_host_loop_platform_wait_backend_from_selection` は explicit event source capability と `NativeWindowHostLoopLinuxSelectorTimerFdSysApi::new` を Linux-only helper へ渡すだけである。eventfd write / signal producer、native runner / CLI connection、minifb wait path、macOS actual sys shim、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は後続 phase とする。fallback、silent no-op、synthetic readiness、host-consuming fallible builderは禁止する。

## F5ib Native Linux host event fd producer boundary

2026-06-20 の F5ib では、F5hz で所有・登録・drain できるようになった Linux host event fd に対して、producer 側の明示的な signal 境界を追加する。これは `NativeWindowHostLoopLinuxSelectorTimerFdBackend` が所有する private `NativeWindowHostLoopLinuxHostEventFd` へ wake request を書き込む境界であり、scheduler resume evidence や `HostEventReady` outcome を直接作る境界ではない。

`NativeWindowHostLoopLinuxSelectorTimerFdRawApi` は `signal_host_event_fd_raw` を持つ。backend は `signal_host_event` を通じてこの raw method を呼び、host event fd owner が閉じている場合は `InvalidHostEventRawFd { raw_fd: -1 }`、raw method が失敗した場合は `SignalHostEventFdFailed { code }` を返す。失敗を no-op success に変換してはいけない。

cfg Linux sys shim は `eventfd` に `u64` 値 `1` を `libc::write` で exactly `size_of::<u64>` bytes 書けた場合だけ成功にする。`EAGAIN`、short write、syscall failure は typed failure とし、event queue full や host wake request を捨てて成功扱いにしない。Never raw API は引き続き empty enum と `match *self {}` のみで全 method を実装する。

F5ib は eventfd producer boundary までであり、native runner / CLI / minifb wait path からこの producer を呼ぶ接続、macOS actual sys shim、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は後続 phase とする。fallback、silent no-op、synthetic readiness、timer fired evidence の偽装は禁止する。

## F5ic Native Linux host event signal producer handle boundary

2026-06-20 の F5ic では、F5ib の backend-owned `signal_host_event` から一段進め、blocking wait 中の backend とは別 owner が host event fd を signal できるようにする。Linux の selector / timerfd backend は `wait_for_host_event` や `wait_until_deadline_or_host_event` の間に `&mut self` を占有するため、runner 接続前に producer handle を分離しないと、外部 window / event source から安全に wake を送れない。

`NativeWindowHostLoopLinuxHostEventSignalFd` は duplicated eventfd handle の private owner である。fd `0` は有効、`raw_fd < 0` だけを invalid とし、public raw fd accessor は作らない。`NativeWindowHostLoopLinuxHostEventSignalRawApi` は host event fd の duplicate、duplicated fd への exact signal write、duplicated fd close、last error retrieval を分離する。clone / write failure は `NativeWindowHostLoopLinuxHostEventSignalProducerError` の typed variant として返す。

`NativeWindowHostLoopLinuxSelectorTimerFdBackend::create_host_event_signal_producer` は backend が保持する `NativeWindowHostLoopLinuxHostEventFd` を producer API で複製し、`NativeWindowHostLoopLinuxHostEventSignalProducer Api` を返す。backend が closed の場合は `InvalidHostEventRawFd { raw_fd: -1 }` として fail closed にする。producer は duplicated signal fd だけを所有し、`signal_host_event` 成功時も `HostEventReady` outcome や scheduler resume evidence を生成しない。

cfg Linux sys shim は `fcntl F_DUPFD_CLOEXEC` で duplicated signal fd を作る。`dup` による close-on-exec 欠落、raw fd escape、short write / `EAGAIN` の success 化は禁止する。signal は `u64` 値 `1` を `libc::write` で exactly 8 bytes 書けた場合だけ成功にし、producer close は close-once cleanup として扱う。drop cleanup failure は通常 wait contract へ混ぜない。

F5ic は producer handle boundary までであり、native runner / CLI / minifb wait path、generic dispatch から producer を呼ぶ接続、Linux minifb platform wait runner、macOS actual sys shim、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は後続 phase とする。fallback、silent no-op、pre-signal busy loop、synthetic readiness、timer fired evidence の偽装は禁止する。

## F5id Native Linux minifb observed-input signal bridge boundary

2026-06-20 の F5id では、F5ic の duplicated host event signal producer を minifb の observed input callback へ接続する最小境界を追加する。minifb `InputCallback` は minifb が event processing を行った結果として呼ばれるため、X11 / Wayland の fd readiness を直接 selector へ渡すものではない。このため F5id は Linux blocking wait runner の完成ではなく、observed keyboard / text input を eventfd wake request として送るための bridge である。

`NativeWindowHostEventSignalWaitError WaitError` は `HostEventSignalFailed` と `DelegateWaitFailed` を分ける。callback は `Result` を返せないので、producer signal failure は `MinifbNativeWindowLinuxHostEventSignalCallbackState` に first error として保存される。`NativeWindowHostEventSignalWaitGuardRunLoopHost` は `wait_after_budget_exhausted` の前にこの state を確認し、error があれば inner host の wait を呼ばず typed error を返す。これにより eventfd full / `EAGAIN` などの producer failure を silent no-op にしない。

`MinifbNativeWindowLinuxHostEventSignalInputCallback` は `add_char` と `set_key_state` で `signal_observed_input` を呼ぶ。signal success は wake request の成功だけを表し、`HostEventReady` outcome、scheduler resume evidence、timer fired evidence を生成しない。state に error が無い場合、wait guard は inner host wait へ通常どおり委譲するため、signal success だけで wait outcome を合成しない。

F5id は Linux platform wait runner、CLI dispatch、`run_linux_platform_wait_window_loop`、actual X11 / Wayland event fd selector integration、minifb wait replacement へは進まない。minifb `InputCallback` だけでは blocking wait 中に新規 OS event を起こす source にならないため、runner 有効化は実 event source fd integration または同等の externally-wakeable host event source を別 phase で扱う。fallback、silent no-op、sleep、busy loop、synthetic host event、timer fired evidence の偽装は禁止する。

## F5ie Native Linux blocking wait event source capability gate

F5ie では、Linux blocking platform wait runner が受け入れられる event source capability を型で分ける。`NativeWindowHostLoopLinuxEventSourceCapability` は `ObservedInputOnly` と `ExternallyWakeableEventSource` を持つ。`ObservedInputOnly` は minifb `InputCallback` のように、backend が既に event processing を行った結果として呼ばれる source を表す。これは blocking wait 中の readiness source ではないため、platform wait runner support では拒否する。

`validate_native_window_host_loop_linux_blocking_wait_event_source_capability` は `ObservedInputOnly` を `NativeWindowHostLoopLinuxPlatformWaitEventSourceSupportError::ObservedInputOnlyUnsupportedForBlockingWait` として返す。error は requested capability を保持する。`ExternallyWakeableEventSource` は分類値として受理するが、これは fd owner、selector registration、wait outcome、runner dispatch を作ったことを意味しない。actual X11 / Wayland fd integration または同等の externally-wakeable event source は後続 phase で扱う。

この boundary は fail-closed support gate である。`HostEventReady`、timer fired evidence、scheduler resume evidence、fallback success、best-effort success、sleep、busy loop、minifb wait replacement、`set_target_fps 0`、`run_linux_platform_wait_window_loop`、CLI dispatch は追加しない。

## F5if Native Linux platform wait backend event source config gate

F5if では、F5ie の event source capability gate を Linux platform wait backend construction helper へ接続する。`build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api` は `event_source_capability` を explicit input として受ける。validation order は selection validation、event source capability validation、Linux raw API method call の順である。

`ObservedInputOnly` が渡された場合は `NativeWindowHostLoopPlatformWaitHostBuildError::LinuxEventSourceSupportFailed` を返し、Linux selector / timerfd / eventfd の raw creation や registration を呼ばない。error は `NativeWindowHostLoopLinuxPlatformWaitEventSourceSupportError` を保持する。`ExternallyWakeableEventSource` が渡された場合だけ Linux selector / timerfd backend construction へ進む。

`build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis` も `linux_event_source_capability` を explicit input とし、selected Linux branch では同じ validation を通す。Windows / macOS branch は Linux event source capability を readiness evidence として使わない。cfg Linux `native_window_host_loop_platform_wait_backend_from_selection` も explicit event source capability を要求し、暗黙に `ExternallyWakeableEventSource` を渡す bypass を持たない。

F5if は backend construction gate であり、Linux platform wait runner、CLI dispatch、actual X11 / Wayland fd selector integration、minifb wait replacement、`HostEventReady` synthesis、timer fired evidence、scheduler resume evidence、fallback、silent no-op、sleep、busy loop は追加しない。

## F5ig Native run-loop platform wait config event source gate

F5ig では、run-loop config の platform wait 設定を bare selection から typed config へ移す。`NativeWindowRunLoopPlatformWaitBackendConfig` は selection と optional Linux event source capability を private field として保持し、constructor / accessor 経由でだけ扱う。これにより、future Linux runner が `NativeWindowRunLoopConfig` から selection だけを取り出して F5ie / F5if の event source gate を bypass する経路を作らない。

`NativeWindowRunLoopWaitBackend::PlatformWait` は `NativeWindowRunLoopPlatformWaitBackendConfig` を保持する。`new_with_platform_wait_backend_selection` は Windows compatibility の selection-only config constructor として残す。Linux capability を持つ config は `NativeWindowRunLoopPlatformWaitBackendConfig::new_with_linux_event_source_capability` で明示的に作る。

cfg Linux `native_window_run_loop_platform_wait_backend_from_config` は platform wait config を抽出し、`linux_event_source_capability` が無ければ `NativeWindowRunLoopPlatformWaitBackendConfigError::MissingLinuxEventSourceCapability` を返す。暗黙に `ExternallyWakeableEventSource` を補わない。capability がある場合だけ cfg Linux helper へ selection と capability を渡し、F5if の validation order に従う。

cfg Windows from-config は config から selection を読むだけで、Linux event source capability を readiness evidence として使わない。F5ig は config gate であり、Linux platform wait runner、CLI dispatch、actual X11 / Wayland fd integration、minifb wait replacement、`HostEventReady` synthesis、timer fired evidence、scheduler resume evidence、fallback、silent no-op、sleep、busy loop は追加しない。

## F5ih Native platform wait runner support gate

F5ih では、`NativeWindowRunLoopConfig` の platform wait config を actual window runner dispatch の前に検査する support gate を追加する。これは runner support 判定の境界であり、Linux / macOS runner を実行可能にする phase ではない。

`validate_native_window_run_loop_platform_wait_runner_support_for_platform` は config を読み、non-platform config を `Config NotPlatformWaitBackend`、current platform と selection の不一致を `BackendSupportFailed` として返す。Windows + `WindowsWaitableTimerMessageWait` は既存 Windows runner path として accepted selection を返す。

Linux では F5ig の explicit event source capability を要求する。missing capability は `MissingLinuxEventSourceCapability`、`ObservedInputOnly` は `LinuxEventSourceSupportFailed` として返す。F5it 以降、validated `ExternallyWakeableEventSource` で actual fd を持たない config は `MissingLinuxWindowEventSourceRawFd` として返し、actual fd を持つ config は `PlatformRunnerIntegrationMissing LinuxWindowEventSourceEventParsingMissing` として返す。`ExternallyWakeableEventSource` は分類値であり、actual fd registration だけでは X11 / Wayland event decoding と runner dispatch が揃わないため runner-ready ではない。

F5ik 以降、macOS は `PlatformRunnerIntegrationMissing MacosActualSysShimMissing` として返し、unsupported platform は `PlatformRunnerUnavailable` として返す。F5ih は Linux raw/sys backend construction、`run_linux_platform_wait_window_loop`、CLI dispatch、minifb wait replacement、`set_target_fps 0`、synthetic `HostEventReady`、timer fired evidence、fallback、silent no-op を追加しない。

## F5ii Native platform wait runner support gate integration

F5ii では、F5ih の support gate を Windows platform wait window runner の実 entry へ接続する。`run_windows_platform_wait_window_loop` は、backend construction、backend loop construction、window creation、visual host construction、minifb side effect より前に `validate_native_window_run_loop_platform_wait_runner_support_for_platform Windows config` を実行する。

runner support failure は `NativeWindowRunLoopError::PlatformWaitRunnerUnsupported` として返す。これは backend construction failure とは別の error stage であり、`NativeWindowRunLoopPlatformWaitRunnerSupportError` を string 化せずに保持する。support gate が成功した後だけ、`native_window_run_loop_platform_wait_backend_from_config` による actual backend construction へ進む。

F5ii は Windows runner の validation order を固定する checkpoint であり、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop`、actual X11 / Wayland fd integration、macOS actual sys shim、default minifb runner replacement、`set_target_fps 0`、synthetic `HostEventReady`、timer fired evidence、fallback、silent no-op は追加しない。

## F5ij Native platform wait CLI support gate boundary

F5ij では、native CLI の `--wait-backend platform` を library の typed runner support gate へ接続する。CLI は default platform wait selection と `NativeWindowRunLoopConfig` construction だけを行い、runner dispatch の前に `validate_native_window_run_loop_platform_wait_runner_support config` を呼ぶ。

Windows では、この validation 成功後に `run_windows_platform_wait_window_loop` を呼ぶ。runner entry 側の F5ii validation は残し、CLI validation は user-facing rejection を早めるための boundary とする。non-Windows では同じ validation を通し、default selection failure または support failure を CLI error として返す。support validation が将来成功した target で runner dispatch がまだ無い場合は、minifb fallback ではなく explicit dispatch-unavailable error を返す。

F5ij は CLI support-gate integration だけであり、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop`、actual X11 / Wayland fd integration、macOS actual sys shim、default minifb runner replacement、`set_target_fps 0`、synthetic `HostEventReady`、timer fired evidence、fallback、silent no-op は追加しない。CLI は `ExternallyWakeableEventSource` や `ObservedInputOnly` を注入せず、Linux runner readiness を偽装しない。

## F5ik Native platform wait runner missing-integration reason boundary

F5ik では、platform wait runner support gate の failure shape を精密化する。selection と capability validation は通るが actual runner readiness に必要な実体が未接続である場合、`PlatformRunnerUnavailable` ではなく `PlatformRunnerIntegrationMissing` を返す。

`NativeWindowRunLoopPlatformWaitRunnerMissingIntegration` は、Linux の `LinuxWindowEventSourceFdMissing` / `LinuxWindowEventSourceEventParsingMissing` と macOS の `MacosActualSysShimMissing` を分ける。Linux missing capability、observed-input-only、fd なし externally-wakeable config は従来通り config / event-source support error として返す。fd-present `ExternallyWakeableEventSource` だけが actual X11 / Wayland event decoding と runner dispatch missing として integration-missing になる。

F5ik は error modeling checkpoint であり、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop`、actual X11 / Wayland fd integration、macOS actual sys shim、CoreFoundation / AppKit binding、minifb wait replacement、`set_target_fps 0`、synthetic `HostEventReady`、timer fired evidence、fallback、silent no-op は追加しない。

## F5il Native Linux externally-wakeable event source owner boundary

F5il では、Linux selector / timerfd backend と external host-event signal producer を同一 owner に束ねる。これは F5ik の owner missing stage を解消するための前段であり、runner support gate を成功させる phase ではない。

`NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwner` は Linux selector / timerfd backend と host-event signal producer を private field として持つ。builder は構築済み backend と producer raw API を受け取り、backend が open であることを確認してから `create_host_event_signal_producer` を呼ぶ。closed backend は `BackendClosed`、producer 作成失敗は `HostEventSignalProducerFailed` として backend owner を返す。

owner の `signal_host_event` は producer にだけ委譲する。backend の direct `signal_host_event`、wait outcome synthesis、`HostEventReady` / timer fired evidence / scheduler-ready evidence の生成は行わない。

F5il は owner contract checkpoint であり、Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop`、actual X11 / Wayland fd integration、macOS actual sys shim、CoreFoundation / AppKit binding、minifb wait replacement、`set_target_fps 0`、sleep、busy loop、fallback、silent no-op は追加しない。

## F5im Native Linux externally-wakeable event source run-loop host boundary

F5im では、F5il の owner を backend-only path へ分解せず、`NativeWindowRunLoopHost` の wait hook として使える Linux 専用 wrapper を追加する。`NativeWindowHostLoopLinuxExternallyWakeableEventSourceRunLoopHost` は visual host と `NativeWindowHostLoopLinuxExternallyWakeableEventSourceWaitAdapter` を所有し、adapter は `NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwner` 全体を保持する。

owner から backend だけを public に取り出す API は持たない。`into_owner` は producer と backend をまとめた owner を返すだけで、`NativeWindowHostLoopLinuxSelectorTimerFdBackend` 単体を public に返さない。wait は owner 内の backend へ委譲し、signal は owner 内の producer へ委譲する。

F5im は owner-retaining wait hook checkpoint であり、Linux runner support gate は引き続き `PlatformRunnerIntegrationMissing` または config error を返す。F5it 以降、fd なし externally-wakeable config は `MissingLinuxWindowEventSourceRawFd`、fd-present config は `LinuxWindowEventSourceEventParsingMissing` で fail-closed になる。Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop`、actual X11 / Wayland event decoding、macOS actual sys shim、CoreFoundation / AppKit binding、minifb wait replacement、`set_target_fps 0`、sleep、busy loop、fallback、silent no-op、synthetic `HostEventReady` / timer fired evidence / scheduler-ready evidence は追加しない。

## F5in Native Linux platform-wait owner-ready run-loop input boundary

F5in では、F5im の owner-retaining run-loop host を Linux runner へ接続する前段として、platform-wait config と `NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwner` を同時に検査して保持する Linux 専用 input boundary を追加する。

`NativeWindowLinuxPlatformWaitRunLoopInput` は `NativeWindowRunLoopConfig` と Linux externally-wakeable owner 全体を所有する。`into_parts` は config と owner を返すだけで、backend 単体や producer 単体を public に取り出さない。これにより、producer を落とした backend-only path や、owner を失う fallible builder を runner 入口に作らない。

`native_window_linux_platform_wait_run_loop_input_for_platform` は generic runner support gate を呼ばない。generic support gate は引き続き Linux を runner-ready として扱わないためである。F5it 以降、fd なし externally-wakeable config は `MissingLinuxWindowEventSourceRawFd`、fd-present config は `LinuxWindowEventSourceEventParsingMissing` で fail-closed になる。F5in helper は下位の config / platform / backend kind / Linux event-source capability / owner-open validation だけを行い、失敗時は config と owner を error variant に保持して返す。

F5in は owner-ready input checkpoint であり、Linux runner support gate の `Ok` 化、`run_linux_platform_wait_window_loop`、CLI dispatch、actual X11 / Wayland fd integration、macOS actual sys shim、CoreFoundation / AppKit binding、minifb wait replacement、sleep、busy loop、fallback、silent no-op、synthetic `HostEventReady` / timer fired evidence / scheduler-ready evidence は追加しない。

## F5io Native Linux owner-ready input-to-host boundary

F5io では、F5in の `NativeWindowLinuxPlatformWaitRunLoopInput` を F5im の owner-retaining run-loop host へ渡す境界を追加する。これは Linux runner 本体ではなく、validated input を visual host と結合して wait hook host を作る前段である。

`native_window_host_loop_linux_platform_wait_run_loop_host_from_input` は visual host と input を受け取り、input 内 config を再検査する。検査は lower-level の platform-wait config、Linux backend kind、selection platform、Linux event-source capability、owner open state だけを使い、generic runner support gate は呼ばない。失敗時は `NativeWindowLinuxPlatformWaitRunLoopHostBuildError` として host と input を保持して返す。

成功時は `input.into_parts` で config と full owner を取り出し、config は再検査済みの authority として消費する。backend 単体や producer 単体を public に取り出さず、full owner だけを `native_window_host_loop_linux_externally_wakeable_event_source_run_loop_host_from_owner` へ渡す。

F5io は input-to-host checkpoint であり、Linux runner support gate の `Ok` 化、`run_linux_platform_wait_window_loop`、CLI dispatch、actual X11 / Wayland fd integration、macOS actual sys shim、CoreFoundation / AppKit binding、minifb wait replacement、sys API construction、sleep、busy loop、fallback、silent no-op、synthetic `HostEventReady` / timer fired evidence / scheduler-ready evidence は追加しない。

## F5ip Native Linux cfg sys host factory boundary

F5ip では、F5io の input-to-host handoff を cfg Linux sys API から組み立てる helper を追加する。これは Linux runner readiness ではなく、`NativeWindowRunLoopConfig` と visual host から Linux selector / timerfd backend、host-event signal producer、F5in input、F5io owner-retaining host へ進む construction order を固定する境界である。

testable helper は `native_window_host_loop_linux_platform_wait_run_loop_host_from_config_with_apis` とし、scripted raw API を注入できる。`PlatformWait` config、Linux backend kind、selection platform、`ExternallyWakeableEventSource` capability を raw API construction より前に検査する。検査失敗は host と config を保持する typed error として返し、backend construction failure、owner build failure、F5in input build failure、F5io host build failure はそれぞれ別 variant で下位 error を保持する。

cfg Linux wrapper `native_window_host_loop_linux_platform_wait_run_loop_host_from_config` は、2 つの独立した `NativeWindowHostLoopLinuxSelectorTimerFdSysApi` を backend owner 用と producer owner 用に注入するだけである。generic Linux runner support gate は引き続き `PlatformRunnerIntegrationMissing` を返し、CLI dispatch、`run_linux_platform_wait_window_loop`、minifb wait replacement、fallback、sleep、busy loop、synthetic readiness、timer-fired evidence fabrication は追加しない。

F5iq では、F5ip 後の actual blocker を反映するため、Linux support gate の missing reason を `LinuxWindowEventSourceFdMissing` に更新する。Linux owner / host factory は既にあるため、以後の未接続理由は owner missing ではなく、actual X11 / Wayland window event source fd integration または同等の externally-wakeable platform event source missing である。これは Linux runner を有効化する変更ではない。

## F5ir Native Linux window event source fd registration boundary

F5ir では、Linux selector / timerfd backend に external window event source fd を登録する境界を追加する。window event source fd は host-event wake 用の owned `eventfd` とは別の型であり、backend はその fd を所有しない。fd `0` は有効、負値は `InvalidWindowEventSourceRawFd` として拒否する。

backend は window event source fd を高々 1 つだけ registered token として保持する。既に登録済みの場合は `WindowEventSourceFdAlreadyRegistered` を raw API 呼び出し前に返し、state を変えない。raw registration failure は `RegisterWindowEventSourceFdFailed`、raw unregister failure は `UnregisterWindowEventSourceFdFailed` として返す。unregister failure では token を backend に戻し、subsequent cleanup / retry が可能な状態を保つ。

`try_close_handles_if_open` は registered window event source fd の unregister を owned selector / timerfd / host-event fd cleanup より前に実行する。unregister failure では owned fd を閉じず、external fd を close したという主張もしない。`close_handles_if_open` は existing cleanup compatibility path であり、observable error handling が必要な path は `try_close_handles_if_open` を使う。

cfg Linux sys shim は `epoll_ctl EPOLL_CTL_ADD` / `EPOLL_CTL_DEL` で external fd を selector に登録解除するだけである。external window event source fd の取得、X11 / Wayland event decoding、readiness status classification、Linux runner support gate の `Ok` 化、CLI dispatch、synthetic readiness、timer fired evidence は F5ir では実装しない。

## F5is Native Linux window event source readiness status boundary

F5is では、registered external window event source fd の readiness を Linux selector status として明示する。`NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_WINDOW_EVENT_SOURCE_READY` と `NativeWindowHostLoopLinuxSelectorTimerFdWake::WindowEventSourceReady` は、timerfd readiness、host-event `eventfd` readiness、window / platform event source readiness を同じ `HostEventReady` 名へ潰さないための分類である。

registered window event source fd が無い場合、deadline-or-event wait は従来通り `selector_wait_for_timer_or_event_raw` を使う。registered token がある場合だけ、`selector_wait_for_timer_host_or_window_event_raw` に selector、timerfd、host-event fd、window event source fd をすべて渡す。optional raw fd や sentinel fd は使わない。

cfg Linux sys shim は `epoll_wait` の event data が window event source fd と一致した場合、external fd を read、drain、signal、close せず `WINDOW_EVENT_SOURCE_READY` を返す。external fd の event parsing と drain は X11 / Wayland 等の owner 側の責務であり、この boundary は readiness classification だけを行う。

host-event-only wait は `HOST_EVENT_READY` だけを success とし、`WINDOW_EVENT_SOURCE_READY` は unexpected status として扱う。deadline-or-event wait では Linux selector enum に `WindowEventSourceReady` が残る。`NativeWindowHostLoopInterruptibleDeadlineWaiter` の generic wake へ写す時だけ、timer ではない event-pump wake として `HostEventReady` に変換する。これは event pump へ戻る evidence であり、timer fired evidence、scheduler completion、Linux runner readiness ではない。

F5is は readiness classification checkpoint であり、actual X11 / Wayland fd acquisition、event decoding、Linux runner support gate の `Ok` 化、CLI dispatch、minifb wait replacement、synthetic timer fired evidence は追加しない。

## F5it Native Linux window event source fd config-to-registration boundary

F5it では、Linux platform-wait config が actual externally-wakeable window event source fd を non-owning raw value として保持できるようにする。`ExternallyWakeableEventSource` は分類値であり、actual fd が無い場合は runner-ready と扱わない。

`NativeWindowRunLoopPlatformWaitBackendConfig` は `linux_window_event_source_raw_fd` を optional に持つ。既存 constructor は fd を作らない。`new_with_linux_window_event_source_raw_fd` は capability を `ExternallyWakeableEventSource` にし、raw fd を保持する。fd `0` は有効な fd として保持され、負値 fd の拒否は既存の `register_window_event_source_fd_from_raw` boundary が行う。

`native_window_host_loop_linux_platform_wait_run_loop_host_from_config_with_apis` は、Linux selection と `ExternallyWakeableEventSource` capability を検査した後、backend raw construction より前に fd の存在を要求する。fd が無い場合は `MissingLinuxWindowEventSourceRawFd` を返し、raw API を呼ばない。backend construction 後は owner construction より前に window event source fd を register する。registration failure は host、config、backend、typed registration error を保持して返す。

generic runner support gate は Linux fd-present config でも `Ok` を返さない。fd が無い externally-wakeable config は config error とし、fd がある config は `LinuxWindowEventSourceEventParsingMissing` として fail-closed にする。F5it では fd registration / readiness classification までを config builder へ接続するだけで、X11 / Wayland event decoding、runner dispatch、CLI dispatch、minifb wait replacement、synthetic timer evidence は追加しない。

## F5iu Native Linux window event source provider-owned platform config boundary

F5iu では、F5it の raw fd config を provider-owned prepared value へ進める。Linux window backend / toolkit provider が event source descriptor を返し、prepared value が `NativeWindowRunLoopPlatformWaitBackendConfig`、descriptor、provider owner を同時に保持する。

provider boundary は platform-wait backend config だけを作る。`NativeWindowRunLoopConfig` は demo、counter、scale、target FPS、run policy を持つため、F5iu helper は full run-loop config を作らない。caller が既存の explicit constructor へ prepared platform config を渡す。

selection validation は provider call より前に行う。Linux + `LinuxSelectorTimerFd` でない selection は provider を呼ばずに owner-bearing error を返す。provider は descriptor を 1 回だけ返す。descriptor fd は既存の Linux window event source fd validation を使い、fd `0` は valid、負値 fd は typed error として provider owner と descriptor を保持して返す。

prepared value は provider owner を落とさないための型であり、provider に対して Clone / Copy を提供しない。F5iu は provider-to-config ownership checkpoint であり、actual X11 / Wayland event decoding、event drain/read、Linux runner dispatch、CLI dispatch、minifb wait replacement、support gate `Ok` 化、synthetic readiness、timer evidence は追加しない。

## F5iv Native Linux window event source provider-owned run-loop handoff boundary

F5iv では、F5iu の prepared platform config を provider owner ごと full run-loop config と Linux owner-retaining run-loop host へ渡す。これは actual X11 / Wayland event decoding ではなく、provider owner を生かしたまま既存 Linux selector / timerfd owner host boundary へ接続する ownership handoff である。

full `NativeWindowRunLoopConfig` は selection + provider から暗黙に作らない。demo、counter、scale、target FPS、run policy は caller の明示入力として受け、F5iu prepared platform config の platform-wait backend config と結合する。これにより default demo / scale / FPS を helper が勝手に選ぶ経路を作らない。

prepared run-loop config と run-loop host wrapper は provider owner、descriptor、config または host を同時に保持する。constructor は private にし、F5iu prepared value を通らず任意 raw fd / descriptor / provider を組み合わせる public bypass を作らない。private constructor だけでは不十分なので、config-only / host-only escape path も禁止する。`into_config`、`into_host`、consuming `config self`、consuming `host self`、provider を返さない split は提供しない。許可するのは borrowed accessor と、config / host、descriptor、provider を同時に返す `into_parts` だけである。

host handoff helper は visual `host`、prepared run-loop config、`backend_api`、`producer_api` を入力に取り、既存 `native_window_host_loop_linux_platform_wait_run_loop_host_from_config_with_apis` を 1 回だけ呼ぶ。success では lower Linux owner-retaining host、provider owner、descriptor を同じ wrapper に入れる。failure では provider owner、descriptor、lower owner-bearing error を保持し、lower error が持つ host / config / backend recovery を失わせない。

F5iv でも Linux generic support gate は `Ok` にならない。runner dispatch、CLI dispatch、minifb wait replacement、fd read / drain / close、synthetic readiness、timer evidence、event decoding は後続に残す。

## F5iw Native Linux window event source provider-owned event pump boundary

F5iw では、F5iv で保持した provider owner を abstract event pump provider として扱い、`NativeWindowEventPumpInput` から `NativeWindowEventPumpSnapshot` を返す境界を追加する。これは X11 / Wayland binding ではなく、window event source fd readiness の後でどの owner が event snapshot authority を持つかを型で固定する checkpoint である。

event pump provider は descriptor と input を受け、typed `Result` で snapshot または provider-local error を返す。fd read / drain / close、X11 / Wayland concrete parsing、minifb event pump fallback、silent no-op は provider trait の標準 contract には含めない。

provider-owned event pump host wrapper は F5iv host wrapper を consume し、provider owner、descriptor、lower host を同時に保持する。`poll_event_snapshot` は provider に委譲し、`set_window_title`、`pump_events_only`、`present_frame`、`wait_after_budget_exhausted` は lower host に委譲する。lower host event pump と provider event pump を同時 authority にせず、host-only / provider-only escape path も作らない。

provider poll failure は wrapper error enum に写す。`poll_event_snapshot &mut self` は provider owner を value として返せないため、error は descriptor と provider-local error を持ち、wrapper 自体は lower host と provider owner を保持し続ける。conversion helper は F5iv host wrapper を consume する infallible helper とし、ここで曖昧な fallible owner recovery stage は作らない。

F5iw でも Linux support gate は `Ok` にならない。runner dispatch、CLI dispatch、`run_linux_platform_wait_window_loop`、actual X11 / Wayland sys API、minifb wait replacement、fd read / drain / close、synthetic readiness、timer evidence は後続に残す。

## F5ix Native Linux window event source normalized observation boundary

F5ix では、F5iw の provider-owned event pump contract の下に、future X11 / Wayland / toolkit decoder が共有する normalized observation to snapshot contract を追加する。これは concrete event parser ではなく、parser が得た observation を `NativeWindowEventPumpSnapshot` へ変換する標準境界である。

`NativeWindowLinuxWindowEventSourceObservation` は os close、exit shortcut、current size、mouse down、optional raw pointer を保持する。previous state は `NativeWindowEventPumpInput` が持つため、observation は previous mouse / size を持たない。transition と resize 判定は snapshot helper が input と observation を同時に見て行う。

`native_window_linux_window_event_source_snapshot_from_observation` は descriptor、input、observation を受ける。pointer finite check は既存 `build_native_window_event_pump_snapshot_from_raw` を使い、failure は descriptor 付き typed error に写す。non-finite pointer を unavailable pointer や silent success にしない。

`NativeWindowLinuxWindowEventSourceObservationProvider` は descriptor と input から observation を返す provider-local contract である。adapter は observation provider owner を保持し、F5iw の `NativeWindowLinuxWindowEventSourceEventPumpProvider` として使える。provider observation failure と snapshot construction failure は別 variant に分ける。

F5ix でも Linux support gate は `Ok` にならない。actual X11 / Wayland sys API、fd read / drain / close、runner dispatch、CLI dispatch、`run_linux_platform_wait_window_loop`、minifb wait replacement、synthetic readiness、timer evidence は後続に残す。

## F5iy Native Linux window event source observation run-loop adapter boundary

F5iy では、F5iv の provider-owned run-loop host と F5ix の observation provider adapter を接続する。これは concrete event parser ではなく、provider owner を落とさずに normalized observation provider を event pump provider として使うための run-loop adapter boundary である。

`native_window_linux_window_event_source_enable_observation_provider_event_pump` は、F5iv host wrapper を consume し、lower host、descriptor、provider owner を同時に取り出す。provider owner は F5ix の observation adapter で包まれ、F5iw の event pump run-loop host へ渡される。descriptor provider owner と observation provider owner を別 owner に分けない。

helper は infallible である。validation、backend construction、fd registration、runner support validation は F5iu / F5iv までに済んでいるため、F5iy では新しい owner recovery stage を作らない。poll 時の observation failure と snapshot construction failure は、F5ix adapter の typed error として返る。

F5iy でも Linux support gate は `Ok` にならない。actual X11 / Wayland sys API、fd read / drain / close、runner dispatch、CLI dispatch、`run_linux_platform_wait_window_loop`、minifb wait replacement、synthetic readiness、timer evidence は後続に残す。

## F5iz Native Linux window event source fd acquisition boundary

F5iz では、actual X11 / Wayland event decoding へ進む前に、Linux の window event source fd を取得して provider owner として保持する境界を追加する。これは protocol handshake や event parser ではなく、`DISPLAY` / `WAYLAND_DISPLAY` / `XDG_RUNTIME_DIR` から Unix domain socket fd を得る sys boundary と、その fd を `NativeWindowLinuxWindowEventSourceDescriptor` として返す provider boundary である。

fd acquisition は trait-injected raw API を主契約とし、cfg Linux の libc wrapper は `std::env::var`、`socket AF_UNIX SOCK_STREAM SOCK_CLOEXEC`、`connect`、`close` だけを行う薄い実装とする。Wayland は相対 `WAYLAND_DISPLAY` 名だけを受け、絶対 path や slash を含む display 名は `UnsupportedDisplayForm` として拒否する。X11 は `:N`、`:N.screen`、`unix/:N`、`unix/:N.screen` だけを受け、host / tcp display form は `UnsupportedDisplayForm` として拒否する。

owned fd は `Copy` / `Clone` を持たず、private state で open / closed を保持する。明示 `close` は typed `Result` を返し、成功後の provider descriptor request は stale raw fd ではなく `Closed` error を返す。`Drop` は cleanup best-effort であり、close success evidence として扱わない。

F5iz は acquisition / provider ownership checkpoint である。selector registration 後に provider-owned fd が backend unregister より先に close されないことを保証する final owner bundle / drop-order contract、actual X11 / Wayland protocol parsing、fd read / drain、Linux support gate の `Ok` 化、Linux runner / CLI dispatch、minifb wait replacement、synthetic readiness、timer evidence は後続に残す。

## F5ja Native Linux owned fd run-loop drop-order boundary

F5ja では、F5iz で取得した owned fd provider を runner-safe path へ渡すための final owner bundle を追加する。acquired fd path は `NativeWindowLinuxWindowEventSourceOwnedFdRunLoopHost` を通し、generic provider host を裸で使わない。wrapper は selector backend owner と owned fd provider を同じ private owner tree に保持し、public `into_parts`、provider mutable accessor、owned fd close escape を出さない。

この boundary の中心契約は drop order である。success path では lower host / selector backend が provider-owned fd より先に drop されるため、registered window event source fd は selector から unregister された後に provider が fd を close する。host build failure でも、F5iv の generic error をそのまま保持せず、lower error を provider より先に置く F5ja 専用 enum へ詰め替える。これにより、owner construction 後の failure error を drop した場合も selector unregister が provider fd close より先に起きる。

F5ja は drop-order / owner bundle checkpoint である。actual X11 / Wayland protocol parsing、fd read / drain、Linux support gate の `Ok` 化、Linux runner / CLI dispatch、minifb wait replacement、synthetic readiness、timer evidence はまだ追加しない。

## F5jb Native Linux X11 setup and event observation boundary

F5jb では、F5ja で保持した provider-owned fd を actual X11 decoder へ近づけるため、X11 setup request、setup response prefix / body drain、32 byte event packet read を trait-injected raw API の後ろへ切り出す。`NativeWindowLinuxX11EventSourceRawApi` は raw byte write/read、`last_error_code`、would-block 判定を持ち、cfg Linux sys wrapper は `send` / `recv MSG_DONTWAIT` と errno 分類だけを行う薄い実装である。

reader は setup request write progress、setup prefix progress、setup body remaining、event packet progress を内部 state として保持する。`WouldBlock` は typed error として返すが、partial setup / event bytes は保持されるため、次回 poll で同じ packet の続きを読める。X11 observation provider は descriptor provider owner と reader を同じ owner に閉じ込め、provider mutable escape や consuming `into_parts` を公開しない。setup failure、auth-required、invalid status、EOF、raw failure、unsupported event type は enum error で返し、fallback snapshot や silent no-op は作らない。

F5jb の concrete decode は X11 local Unix connection に限定する。`ConfigureNotify` は current size observation、`MotionNotify` / `ButtonPress` / `ButtonRelease` は pointer / mouse observation へ写す。Wayland、X11 authorization file lookup、window creation、event mask selection、WM_DELETE_WINDOW / ClientMessage、keyboard / IME、runner / CLI dispatch、Linux support gate の `Ok` 化、minifb wait replacement、synthetic readiness、timer evidence は後続に残す。

## F5jc Native Linux X11 authorization setup request boundary

F5jc では、F5jb の X11 setup request を authorization credential 付き encoding に拡張する。`NativeWindowLinuxX11AuthorizationCredential` は borrowed authorization protocol name / data を validation input としてだけ扱い、`NativeWindowLinuxX11SetupRequest` が encoded bytes を所有する。reader は credential borrow を保持せず、validated setup request owner だけを受け取る。

setup request builder は byte order `l`、protocol version 11.0、authorization name length、authorization data length、reserved bytes、name bytes、`pad(name)`、data bytes、`pad(data)` を生成する。name / data length は `u16` に収まる必要があり、padding と total length は checked arithmetic で検査する。失敗は `NativeWindowLinuxX11SetupRequestBuildError` の enum で返し、raw fd / raw API owner を消費しない。

F5jc でも `.Xauthority` file lookup、`XAUTHORITY` / `HOME` / env / fs / vfs access、X11 window creation、event mask selection、WM_DELETE_WINDOW / ClientMessage、keyboard / IME、Wayland decoding、Linux support gate の `Ok` 化、runner / CLI dispatch、minifb wait replacement、synthetic readiness、timer evidence は後続に残す。

## F5jd Native Linux Xauthority record parser boundary

F5jd では、caller supplied authority bytes から Xauthority record を zero-copy で parse し、caller supplied selector criteria で credential を選ぶ境界を追加する。record は family、address、display number、protocol name、protocol data を持ち、length field は MSB-first `u16` として読む。parser は offset advance を checked arithmetic にし、length field truncation、payload truncation、offset overflow を enum error として返す。

selector は `family + address + display_number` を exact match する。`FamilyLocal` を hostname なしで広く選んだり、`FamilyWild` を暗黙 fallback として扱ったりしない。`preferred_protocol_name` が指定された場合だけ protocol name を追加条件にする。selection result は `Selected credential` または `NoMatchingRecord` の typed enum とし、no-auth fallback policy は持たせない。

Rust boundary 名としては、family を `NativeWindowLinuxX11XauthorityFamily`、borrowed record を `NativeWindowLinuxX11XauthorityRecord`、caller supplied criteria を `NativeWindowLinuxX11XauthoritySelector`、result を `NativeWindowLinuxX11XauthoritySelection`、typed parse failure を `NativeWindowLinuxX11XauthorityParseError` として公開する。これらは authority bytes の所有や file lookup を持たず、credential は F5jc `NativeWindowLinuxX11AuthorizationCredential` として record slice を borrow する。

F5jd でも `.Xauthority` file lookup、`XAUTHORITY` / `HOME` / env / fs / vfs access、hostname / IP / SSH forwarding identity lookup、X11 window creation、event mask selection、WM_DELETE_WINDOW / ClientMessage、keyboard / IME、Wayland decoding、Linux support gate の `Ok` 化、runner / CLI dispatch、minifb wait replacement、synthetic readiness、timer evidence は後続に残す。

## F5je Native Linux Xauthority selector criteria boundary

F5je では、local X11 display name と caller supplied local authority address から、F5jd selector に渡す criteria を作る境界を追加する。display name parser は `NativeWindowLinuxX11DisplayNameError` を返し、fd acquisition はそれを既存の fd acquisition error へ写像するため、socket acquisition の public behavior は変えない。

accepted display form は `:N`、`:N.screen`、`unix/:N`、`unix/:N.screen` に限る。`localhost:10.0` のような host form、数字でない display number、空 screen suffix、`u32` overflow は typed error になる。display number は parsed integer から decimal bytes に戻し、`NativeWindowLinuxX11XauthoritySelectorCriteria` が fixed `[u8; 10]` に保持する。これにより、`NativeWindowLinuxX11XauthoritySelector` は temporary string ではなく criteria owner を借りる。

F5je の criteria は family を `Local` に固定し、address は caller supplied byte slice を exact match に使う。hostname / gethostname、Unix socket peer identity、TCP/IP address、SSH forwarding display policy、`FamilyWild` fallback、no-auth fallback は扱わない。`.Xauthority` file lookup、`XAUTHORITY` / `HOME` / env / fs / vfs access、X11 window creation、event mask selection、WM_DELETE_WINDOW / ClientMessage、keyboard / IME、Wayland decoding、Linux support gate の `Ok` 化、runner / CLI dispatch、minifb wait replacement、synthetic readiness、timer evidence は後続に残す。

## F5jf Native Linux Xauthority lookup path request boundary

F5jf では、Xauthority bytes を読む前に、caller supplied environment-like values から authority file path request を作る境界を追加する。Rust boundary 名としては、input を `NativeWindowLinuxX11XauthorityLookupInput`、source enum を `NativeWindowLinuxX11XauthorityPathSource`、owned plan を `NativeWindowLinuxX11XauthorityPathPlan`、failure を `NativeWindowLinuxX11XauthorityPathPlanError` とする。

`authority_file_path = Some nonempty` は `ExplicitAuthorityFile` として byte-for-byte に preserving し、home directory より優先する。`authority_file_path = Some empty` は `EmptyAuthorityFilePath` として fail closed にし、home directory default へ進まない。`authority_file_path = None` かつ `home_directory_path = Some nonempty` の場合だけ `HomeDirectoryDefault` として default file name を結合する。home が `/` で終わる場合は `.Xauthority`、そうでない場合は `/.Xauthority` を append する。

F5jf は path request boundary であり、`XAUTHORITY` / `HOME` の env acquisition、`std::env`、filesystem / VFS open / read、metadata / exists / canonicalize、file locking、Xauthority bytes parse、credential selection、setup request integration は扱わない。NUL を含む path は typed error とし、suffix append は checked length にする。fallback、silent no-op、synthetic readiness、Linux support gate の `Ok` 化、runner / CLI dispatch は後続にも混ぜない。

## F5jg Native Linux Xauthority file bytes acquisition boundary

F5jg では、F5jf の path plan を入力にして Xauthority file bytes を取得する trait-injected boundary を追加する。Rust boundary 名としては、reader trait を `NativeWindowLinuxX11XauthorityFileBytesReader`、success owner を `NativeWindowLinuxX11XauthorityFileBytes`、failure を `NativeWindowLinuxX11XauthorityFileBytesReadError` とする。

reader trait は exact `plan.path` だけを受け取り、source を再解釈したり alternate path を合成したりしない。success は raw `Vec u8` ではなく private bytes owner として返し、public surface は read-only `as_bytes` / `len` に限定する。empty file、max byte length 超過、reader failure は typed error であり、source と path を保持する。

F5jg は bytes acquisition boundary であり、`XAUTHORITY` / `HOME` の env acquisition、direct `std::fs` / `File` / `OpenOptions` / `read_to*`、metadata / exists / canonicalize、VFS adapter 実装、file locking、credential selection、setup request integration は扱わない。empty / failed read は no-auth fallback や `NoMatchingRecord` に変換しない。fallback、silent no-op、synthetic readiness、Linux support gate の `Ok` 化、runner / CLI dispatch は後続にも混ぜない。

## F5jh Native Linux Xauthority environment acquisition boundary

F5jh では、F5jf の path plan を作るための environment acquisition boundary を追加する。Rust boundary 名としては、typed variable kind を `NativeWindowLinuxX11XauthorityEnvironmentValueKind`、reader trait を `NativeWindowLinuxX11XauthorityEnvironmentReader`、failure を `NativeWindowLinuxX11XauthorityEnvironmentPathPlanError` とする。

reader trait は authority-file path variable と home-directory path variable を raw string ではなく enum で受け取り、`Option String` を返す。public helper は authority-file path variable を先に読み、present の場合は home-directory path variable を読まない。その値は `NativeWindowLinuxX11XauthorityLookupInput Some value None` へ渡し、F5jf の explicit path priority と empty explicit path fail-closed をそのまま使う。authority-file path variable が missing の場合だけ home-directory path variable を読み、F5jf の home default path plan へ進む。

environment reader failure は `EnvironmentReadFailed variable error`、F5jf path plan failure は `PathPlanFailed error` として分ける。success は既存 `NativeWindowLinuxX11XauthorityPathPlan` だけを返し、file bytes owner や credential selection result は返さない。

F5jh は environment acquisition boundary であり、direct `std::env` adapter、actual filesystem / VFS adapter、`std::fs` / `File` / `OpenOptions` / `read_to*`、metadata / exists / canonicalize、file locking、Xauthority file bytes read、record parse、credential selection、setup request integration は扱わない。fallback、silent no-op、synthetic readiness、Linux support gate の `Ok` 化、runner / CLI dispatch は後続にも混ぜない。

## F5ji Native Linux Xauthority process environment adapter boundary

F5ji では、F5jh の `NativeWindowLinuxX11XauthorityEnvironmentReader` に対する cfg Linux actual process environment adapter を追加する。Rust boundary 名としては、reader を `NativeWindowLinuxX11XauthorityProcessEnvironmentReader`、failure を `NativeWindowLinuxX11XauthorityProcessEnvironmentReadError` とする。

adapter は `AuthorityFilePath` を固定 env name `XAUTHORITY`、`HomeDirectoryPath` を `HOME` へ写す。`std::env::var` の `NotPresent` は `Ok None` として扱い、`NotUnicode` は raw `OsString` を保持する typed error にする。`std::env::var(...).ok()` により NotUnicode を missing と同一視してはならない。success path の priority、empty explicit path rejection、home default path creation は F5jh / F5jf helper に委譲する。

convenience helper は process environment reader を構築し、F5jh `native_window_linux_x11_xauthority_path_plan_from_environment` を呼んで `NativeWindowLinuxX11XauthorityPathPlan` を返すだけにする。F5ji は actual environment adapter boundary であり、filesystem / VFS adapter、file bytes read、record parse、credential selection、setup request integration、runner / CLI dispatch は扱わない。fallback、silent no-op、synthetic readiness、Linux support gate の `Ok` 化は後続にも混ぜない。

## F5jj Native Linux Xauthority filesystem file bytes adapter boundary

F5jj では、F5jg の `NativeWindowLinuxX11XauthorityFileBytesReader` に対する cfg Linux actual filesystem adapter を追加する。Rust boundary 名としては、reader を `NativeWindowLinuxX11XauthorityFilesystemFileBytesReader`、failure を `NativeWindowLinuxX11XauthorityFilesystemFileBytesReadError` とする。

adapter は F5jg から渡された exact `path` に対して `std::fs::read(path)` だけを行う。read failure は exact requested path と original `std::io::Error` を保持する typed error として返す。empty file / file too large validation は F5jg の `native_window_linux_x11_xauthority_read_file_bytes` が担当し、adapter は validation を重複実装しない。

convenience helper は filesystem reader を構築し、F5jg `native_window_linux_x11_xauthority_read_file_bytes` を呼んで `NativeWindowLinuxX11XauthorityFileBytes` を返すだけにする。F5jj は actual filesystem file bytes adapter boundary であり、VFS adapter、path normalization、metadata / exists / canonicalize、file locking、record parse、credential selection、setup request integration、runner / CLI dispatch は扱わない。fallback、silent no-op、synthetic readiness、Linux support gate の `Ok` 化は後続にも混ぜない。

## F5jk Native Linux Xauthority VFS file bytes adapter boundary

F5jk では、F5jg の `NativeWindowLinuxX11XauthorityFileBytesReader` に対する VFS source adapter を追加する。Rust boundary 名としては、virtual source を `NativeWindowLinuxX11XauthorityVfsFileBytesSource`、reader adapter を `NativeWindowLinuxX11XauthorityVfsFileBytesReader`、failure を `NativeWindowLinuxX11XauthorityVfsFileBytesReadError` とする。

adapter は F5jg から渡された exact `path` を mutable VFS source の `read_xauthority_vfs_file_bytes` にそのまま渡す。path normalization、alias lookup、alternate path synthesis、home fallback は行わない。source failure は exact requested path と source error を保持する typed error として返す。empty file / file too large validation は F5jg の `native_window_linux_x11_xauthority_read_file_bytes` が担当し、adapter は validation を重複実装しない。

convenience helper は caller supplied VFS source を借用して reader adapter を構築し、F5jg `native_window_linux_x11_xauthority_read_file_bytes` を呼んで `NativeWindowLinuxX11XauthorityFileBytes` を返すだけにする。F5jk は VFS adapter boundary であり、actual Web VFS、native resource root、direct `std::fs` / `File` / `OpenOptions` / `read_to*`、metadata / exists / canonicalize、file locking、record parse、credential selection、setup request integration、runner / CLI dispatch は扱わない。fallback、silent no-op、synthetic readiness、Linux support gate の `Ok` 化は後続にも混ぜない。

## F5jl Native Linux Xauthority credential setup request boundary

F5jl では、`NativeWindowLinuxX11XauthorityFileBytes` と `NativeWindowLinuxX11XauthoritySelectorCriteria` から exact selector selection を行い、`NativeWindowLinuxX11SetupRequest` を作る接続境界を追加する。

Rust boundary 名としては、error を `NativeWindowLinuxX11XauthorityCredentialSetupRequestError`、entry を `native_window_linux_x11_setup_request_from_xauthority` とする。error は `ParseFailed NativeWindowLinuxX11XauthorityParseError`、`NoMatchingCredential`、`SetupRequestBuildFailed NativeWindowLinuxX11SetupRequestBuildError` を持つ。

entry は `native_window_linux_x11_xauthority_select_credential file_bytes.as_bytes criteria.selector` を 1 回だけ呼び、`Selected credential` の場合だけ `native_window_linux_x11_setup_request_from_authorization credential` を呼ぶ。`NoMatchingRecord` は no-auth fallback にせず `NoMatchingCredential` として返す。parse failure と setup request build failure は lower error を保持し、string 化や generic host failure へ潰さない。

F5jl は credential selection と setup request owner construction の接続だけを担当する。`XAUTHORITY` / `HOME` env、filesystem / VFS file bytes read、hostname / display identity acquisition、raw fd / raw API owner、window creation、event mask、WM_DELETE_WINDOW、keyboard / IME、Wayland concrete decoding、runner / CLI dispatch、Linux support gate の `Ok` 化は扱わない。`AuthorizationCredential::none`、`NativeWindowLinuxX11SetupRequest::no_authorization`、fallback、silent no-op、synthetic readiness はこの境界で使わない。

## F5jm Native Linux Xauthority local authority address owner boundary

F5jm では、Xauthority `FamilyLocal` record と exact match する local authority address を typed owner として扱う境界を追加する。Rust boundary 名としては、owner を `NativeWindowLinuxX11LocalAuthorityAddress`、injected reader を `NativeWindowLinuxX11LocalAuthorityAddressReader`、error を `NativeWindowLinuxX11LocalAuthorityAddressReadError` とする。

reader は `read_x11_local_authority_address` だけを持つ。`native_window_linux_x11_local_authority_address_with_limit` は reader を 1 回だけ呼び、空 address、max byte length 超過、NUL byte を typed error で拒否する。`native_window_linux_x11_local_authority_address` は standard max byte length を使う convenience helper である。失敗は `ReadFailed`、`EmptyAddress`、`AddressTooLong`、`AddressContainsNul` に分け、empty string や wildcard record への fallback は行わない。

`native_window_linux_x11_xauthority_local_selector_criteria_from_authority_address` は `NativeWindowLinuxX11LocalAuthorityAddress` を借用し、既存の `native_window_linux_x11_xauthority_local_selector_criteria_from_display` に渡すだけにする。display number parsing、criteria-owned display number bytes、preferred protocol name matching は既存 path が保持する。

F5jm は injected local authority address owner boundary だけを担当する。actual `gethostname` / process identity acquisition、`DISPLAY` env acquisition、`XAUTHORITY` / `HOME` env、filesystem / VFS file bytes read、credential selection、setup request integration、raw fd / raw API owner、window creation、event mask、WM_DELETE_WINDOW、keyboard / IME、Wayland concrete decoding、runner / CLI dispatch、Linux support gate の `Ok` 化は扱わない。fallback、silent no-op、synthetic readiness はこの境界で使わない。

## F5jn Native Linux Xauthority process hostname address adapter boundary

F5jn では、F5jm の `NativeWindowLinuxX11LocalAuthorityAddressReader` に対する cfg Linux process hostname adapter を追加する。Rust boundary 名としては、raw API trait を `NativeWindowLinuxX11LocalAuthorityAddressRawApi`、reader を `NativeWindowLinuxX11ProcessLocalAuthorityAddressReader`、failure を `NativeWindowLinuxX11LocalAuthorityAddressProcessReadError` とする。

raw API trait は caller supplied buffer を受け取り、`gethostname` 相当の side effect だけを実行する。reader は `NATIVE_WINDOW_LINUX_X11_LOCAL_AUTHORITY_ADDRESS_MAX_BYTE_LEN + 1` bytes の buffer を確保し、raw API を 1 回だけ呼ぶ。buffer 内の最初の NUL byte までを hostname bytes とし、NUL が無い場合は typed `HostnameNotTerminated` failure として返す。空 hostname は reader 内で補正せず、F5jm helper の `EmptyAddress` へ委譲する。

cfg Linux sys adapter は `libc::gethostname` だけを呼ぶ。戻り値 failure は raw API failure として保持し、`std::env`、filesystem / VFS、DISPLAY parsing、Xauthority file lookup、credential selection、setup request construction へは進まない。convenience helper は sys adapter を reader に包み、F5jm `native_window_linux_x11_local_authority_address` を呼ぶだけにする。

F5jn は process hostname adapter boundary だけを担当する。selector criteria construction、credential selection、setup request integration、X11 raw fd / raw API owner、window creation、event mask、WM_DELETE_WINDOW、keyboard / IME、Wayland concrete decoding、runner / CLI dispatch、Linux support gate の `Ok` 化は扱わない。fallback、silent no-op、synthetic readiness はこの境界で使わない。

## F5jo Native Linux X11 top-level window create/map request owner boundary

F5jo では、X11 CreateWindow / MapWindow request bytes を typed owner として作る境界を追加する。Rust boundary 名としては、input を `NativeWindowLinuxX11TopLevelWindowCreateInput`、owner を `NativeWindowLinuxX11TopLevelWindowCreateRequest`、failure を `NativeWindowLinuxX11TopLevelWindowCreateRequestBuildError` とする。

input は window id、parent window id、position、width、height、border width、background pixel、event mask を保持する。window id と parent window id は zero と top 3 bits set を typed error として拒否し、width / height は zero を typed error として拒否する。event mask は X11 `SETofEVENT` の unused high bits を拒否する。

default event mask は F5jo request 自体が MapWindow を直後に送ることを考慮し、current F5jb decoder が non-fatal に扱える pointer / button event だけに合わせ、`ButtonPress | ButtonRelease | PointerMotion` に限定する。StructureNotify は ConfigureNotify だけでなく MapNotify なども購読するため、追加 event decode phase まで含めない。Expose もまだ decode しないため含めない。StructureNotify / Expose subscription と MapNotify / ConfigureNotify / Expose decode は後続 phase に分け、F5jo では silent no-op や fallback redraw event を作らない。

CreateWindow request は opcode `1`、depth `CopyFromParent`、class `InputOutput`、visual `CopyFromParent`、value mask `background-pixel | event-mask` とし、value-list は X11 value-mask の bit order に従って `background-pixel`、`event-mask` の順に encode する。value count は 2 なので request length は `10` units とする。MapWindow request は opcode `8`、request length `2`、同じ window id を使う。

F5jo は request owner boundary だけを担当する。actual raw fd write / read、setup observation reader integration、resource id allocation、server reply / error handling、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、Linux support gate の `Ok` 化は扱わない。fallback、silent no-op、synthetic readiness はこの境界で使わない。

## F5jp Native Linux X11 top-level window request partial-write boundary

F5jp では、F5jo の `NativeWindowLinuxX11TopLevelWindowCreateRequest` を `NativeWindowLinuxX11EventSourceObservationReader` に optional owner として保持し、X11 setup が `Ready` になった後、event packet read より前に CreateWindow / MapWindow request bytes を `write_x11_bytes_raw` へ渡す境界を追加する。

request write state は `NativeWindowLinuxX11TopLevelWindowRequestWriteState` とし、`NotConfigured`、`RequestPending`、`Ready`、`Failed` を持つ。既存 constructor は `NotConfigured` であり、window request を書かずに従来通り setup / event observation を行う。top-level request 付き constructor は setup request と request owner を同時に受け取り、`RequestPending` から開始する。

partial write は byte count を reader が保持し、would-block は `TopLevelWindowRequestWriteWouldBlock` として retryable error にする。hard write failure、zero write、overflow は typed error にし、state を `Failed` へ遷移させる。failed state で再 poll された場合は `TopLevelWindowRequestPreviouslyFailed` を返し、request write を silent retry や fallback success にしない。

F5jp は request write boundary だけを担当する。resource id allocation、setup response からの root window discovery、X11 server error / reply handling、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、StructureNotify / Expose subscription、MapNotify / ConfigureNotify / Expose decode、Wayland concrete decoding、Linux runner / CLI dispatch、Linux support gate の `Ok` 化は扱わない。fallback、silent no-op、synthetic readiness はこの境界で使わない。

## F5jq Native Linux X11 setup resource info and resource id allocation boundary

F5jq では、X11 setup success response body から client resource id space と first screen root window id を読み取る境界を追加する。Rust boundary 名としては、成功 body parser を `native_window_linux_x11_setup_resource_info_from_little_endian_success_body`、保持値を `NativeWindowLinuxX11SetupResourceInfo`、allocator を `native_window_linux_x11_resource_id_from_serial` とする。

setup success body は prefix 後の 32 byte fixed body、vendor string と 4 byte padding、pixmap format list、screen list の順に読む。`resource-id-base` と `resource-id-mask` は little-endian u32 として読み、mask zero、base/mask overlap、client resource id space の unused high bits set を typed parse error で拒否する。first root window id は server-owned id なので client resource id mask/base では検査せず、zero だけを typed parse error で拒否する。

parser は checked arithmetic で vendor padded length、pixmap format section、first screen header end を計算し、body truncation と offset overflow を enum branch として返す。empty body は historical scripted observation compatibility の `Ok(None)` とし、native GUI readiness、support gate `Ok`、synthetic root window discovery の根拠にはしない。

resource id allocator は sparse mask に対応し、serial の bit を resource-id-mask の set bit へ low-to-high に詰める。serial zero、mask bit 数を超える serial、generated id zero、generated id が base/mask invariant を満たさない場合は `NativeWindowLinuxX11ResourceIdAllocationError` を返す。mask が連続していないことを fallback や unsupported success にしない。

`NativeWindowLinuxX11EventSourceObservationReader` は setup body bytes を setup completion まで保持し、body read 完了時に parser を呼ぶ。parse failure は `SetupResourceInfoParseFailed` として setup failed state へ遷移する。success した resource info は reader accessor から読める。F5jq 自体では CreateWindow request owner の生成、window id serial owner、root window id の parent への接続は行わず、これらは F5jr で扱う。

F5jq は setup resource info parse / allocation boundary だけを担当する。actual top-level request generation from setup info、server sequence / error / reply handling、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、StructureNotify / Expose subscription、MapNotify / ConfigureNotify / Expose decode、Wayland concrete decoding、Linux runner / CLI dispatch、Linux support gate の `Ok` 化は扱わない。fallback、silent no-op、synthetic readiness はこの境界で使わない。

## F5jr Native Linux X11 setup-backed top-level window request owner boundary

F5jr では、F5jq の `NativeWindowLinuxX11SetupResourceInfo` と caller supplied serial から generated client window id を割り当て、setup success body 由来の first root window id を parent とする CreateWindow / MapWindow request owner を作る境界を追加する。Rust boundary 名としては、input を `NativeWindowLinuxX11SetupBackedTopLevelWindowCreateInput`、failure を `NativeWindowLinuxX11SetupBackedTopLevelWindowCreateRequestError`、builder を `native_window_linux_x11_top_level_window_create_request_from_setup_resource_info` とする。

`NativeWindowLinuxX11SetupBackedTopLevelWindowCreateInput` は setup resource info、window resource serial、geometry、background pixel、event mask を保持する。window id は `native_window_linux_x11_resource_id_from_serial` により client id space から生成し、zero と unused high bits を既存の client resource id 検証で拒否する。parent window id は setup success body 由来の server-owned root id なので、client id high bits 検証は適用せず、zero だけを拒否する。

F5jr の error は allocation failure と request build failure を分ける。serial zero / exhausted / generated id invariant violation は `ResourceIdAllocationFailed` とし、width zero、height zero、event mask unused bits、root id zero、request length overflow は `TopLevelWindowCreateRequestBuildFailed` として既存 build error を保持する。caller supplied `NativeWindowLinuxX11TopLevelWindowCreateInput` の public helper は従来通り parent id high bits を拒否し続ける。

F5jr は setup info から top-level request owner を作る pure boundary だけを担当する。reader への自動生成接続、server sequence / error / reply handling、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、StructureNotify / Expose subscription、Linux runner / CLI dispatch、support gate `Ok` 化、fallback、silent no-op、synthetic readiness は扱わない。

## F5js Native Linux X11 reader setup-backed top-level request generation boundary

F5js では、F5jr の pure builder を `NativeWindowLinuxX11EventSourceObservationReader` の setup completion path に接続する。reader が window resource serial、geometry、background pixel、event mask を暗黙に決めることは禁止する。caller は `NativeWindowLinuxX11SetupBackedTopLevelWindowCreatePlan` を渡し、reader は setup success body から得た `NativeWindowLinuxX11SetupResourceInfo` と plan を合成して request owner を作る。

`NativeWindowLinuxX11SetupBackedTopLevelWindowCreatePlan` は setup resource info を持たず、window resource serial、x / y、width / height、border width、background pixel、event mask だけを保持する。default event mask を使う constructor は caller が明示的に選ぶ entry であり、reader 内 hidden default ではない。plan は `create_input` により F5jr の `NativeWindowLinuxX11SetupBackedTopLevelWindowCreateInput` へ変換される。

`NativeWindowLinuxX11TopLevelWindowRequestWriteState` は `NotConfigured`、`SetupBackedBuildPending`、`RequestPending`、`Ready`、`Failed` を持つ。`SetupBackedBuildPending` は setup が `Ready` になり、かつ `setup_resource_info` が存在する場合だけ request owner build へ進む。build success 後は既存 F5jp の `RequestPending` partial-write path を再利用する。

`SetupBackedBuildPending` なのに plan が無い場合は `TopLevelWindowRequestSetupBackedPlanMissing`、setup resource info が無い場合は `TopLevelWindowRequestSetupResourceInfoMissing`、F5jr builder が失敗した場合は `TopLevelWindowRequestBuildFailed` を返し、top-level request state を `Failed` にする。failed state で再 poll された場合は `TopLevelWindowRequestPreviouslyFailed` を返す。missing plan、missing setup info、allocation failure、request build failure を skip / no-op / fallback success に変換してはならない。

F5js は reader setup state から top-level request owner を生成し、event read より前に F5jp partial-write path へ渡す境界だけを担当する。server sequence / error / reply handling、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、StructureNotify / Expose subscription、MapNotify / ConfigureNotify / Expose decode、Wayland concrete decoding、Linux runner / CLI dispatch、Linux support gate の `Ok` 化、fallback、silent no-op、synthetic readiness は扱わない。

## F5jt Native Linux X11 server error / reply header decode boundary

F5jt では、X11 event stream から読み取った 32 byte packet の raw response type を先に判定する。`packet[0] == 0` は server error、`packet[0] == 1` は server reply header として扱い、どちらも `EventTypeUnsupported` ではなく typed observation error として返す。通常 event の type 判定では従来通り send-event high bit を除くために `response_type & 0x7f` を用いるが、この mask は server error / reply 判定より後でだけ使う。

`NativeWindowLinuxX11ServerErrorPacket` は error code、sequence、bad value、minor opcode、major opcode を保持する。`NativeWindowLinuxX11ServerReplyHeader` は reply data、sequence、length units を保持する。offset は X11 fixed packet header に従い、dynamic fallback や string error conversion は行わない。

F5jt は server error / reply header の typed decode boundary だけを担当する。server sequence tracking、request / reply correlation、reply body drain、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、StructureNotify / Expose subscription、MapNotify / ConfigureNotify / Expose decode、Wayland concrete decoding、Linux runner / CLI dispatch、Linux support gate の `Ok` 化、fallback、silent no-op、synthetic readiness は扱わない。server error を top-level request write failure と推測せず、later sequence correlation phase が owner-bearing recovery を扱う。

## F5ju Native Linux X11 request sequence correlation boundary

F5ju では、F5jp/F5js が reader に保持する top-level `CreateWindow` / `MapWindow` request owner について、X11 normal request sequence を reader state として追跡する。X11 setup handshake は normal request ではないため sequence を進めない。最初の normal request sequence は `1` とし、future request に備えて 16 bit wrapping increment を使う。zero は wrap 後に到達しうる sequence として扱い、invalid sentinel にはしない。

`NativeWindowLinuxX11TopLevelWindowRequestSequencePlan` は window id、CreateWindow sequence、MapWindow sequence を保持する。CreateWindow sequence は accepted byte range が CreateWindow request byte length の終端を越えた時だけ記録する。MapWindow sequence は accepted byte range が combined top-level request byte length の終端を越えた時だけ記録する。would-block / hard failure / zero write / overflow が byte acceptance より前に返った場合は sequence を進めない。partial write が CreateWindow 境界より手前で止まった場合は何も記録せず、CreateWindow 境界だけを越えた場合は CreateWindow だけを記録する。

`NativeWindowLinuxX11EventSourceObservationError::ServerErrorReceived` は decoded `NativeWindowLinuxX11ServerErrorPacket` と `NativeWindowLinuxX11ServerErrorCorrelation` を返す。correlation は `Unmatched`、`TopLevelWindowCreate { window_id }`、`TopLevelWindowMap { window_id }` の enum とし、packet の `sequence` だけを authority にする。major opcode / minor opcode / bad value は decoded evidence として保持するが、request correlation の authority にはしない。

F5ju は server error correlation boundary だけを担当する。server reply body drain / reply correlation、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、StructureNotify / Expose subscription、MapNotify / ConfigureNotify / Expose decode、Wayland concrete decoding、Linux runner / CLI dispatch、Linux support gate の `Ok` 化、fallback、silent no-op、synthetic readiness は扱わない。

## F5jv Native Linux X11 server reply body drain boundary

F5jv では、F5jt が decode した `NativeWindowLinuxX11ServerReplyHeader` の `length_units` を使い、header に続く reply body を reader が drain してから `ServerReplyReceived` を返す。X11 reply body length は 4 byte 単位なので、byte length は `length_units * 4` として checked arithmetic で求める。overflow や `usize` conversion failure は `ServerReplyBodyLengthOverflow` として fail closed にする。

reader は pending reply header と remaining body byte count を owner state として持つ。reply body read が would-block した場合は `ServerReplyBodyReadWouldBlock` を返し、pending header と remaining byte count を保持する。次回 poll は新しい 32 byte event packet を読まず、pending body drain から再開する。read failure、EOF、overflow も header と remaining byte count を含む typed error として返し、partial state を silent clear しない。

F5jv は generic unexpected reply の stream synchronization boundary であり、reply body payload を public API として保持・parse しない。将来 InternAtom / GetProperty などの request-specific reply を扱う phase では、この drain contract を保ったまま request-specific reply body owner / parser へ接続する。F5jv は request / reply correlation、WM_DELETE_WINDOW / InternAtom / ChangeProperty、keyboard / IME、StructureNotify / Expose subscription、MapNotify / ConfigureNotify / Expose decode、Wayland concrete decoding、Linux runner / CLI dispatch、Linux support gate の `Ok` 化、fallback、silent no-op、synthetic readiness は扱わない。

## F5jw Native Linux X11 InternAtom request owner boundary

F5jw では、X11 `InternAtom` request を raw fd write から分離した typed bytes owner として作る。request は atom name bytes と `only_if_exists` bool を受け取り、opcode `16`、request length、name length、unused field、name bytes、4 byte padding を little-endian X11 request として encode する。request length は `2 + padded_name_len / 4` units とし、padding byte はすべて zero にする。

atom name は X11 protocol の counted bytes として扱う。generic `InternAtom` owner は C string ではないため NUL byte を terminator として解釈せず、NUL を含む counted name も bytes として保持する。空 name、`u16` を超える name length、total byte length overflow、request length units の `u16` 超過は `NativeWindowLinuxX11InternAtomRequestBuildError` で fail closed にする。`only_if_exists` は bool から `0` / `1` だけに encode し、raw flag byte は public input にしない。

F5jw は request bytes owner boundary だけを担当する。raw fd write/read、accepted write progress、sequence assignment、InternAtom reply parse / retain、request / reply correlation、Atom ID owner、WM_DELETE_WINDOW registration、ChangeProperty、ClientMessage decode、keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、Linux support gate の `Ok` 化、fallback、silent no-op、synthetic readiness は扱わない。ICCCM の well-known atom name policy は後続の WM protocol helper で扱い、generic InternAtom counted bytes contract と混ぜない。

## F5jx Native Linux X11 WM protocol atom InternAtom request batch boundary

F5jx では、F5jw の `NativeWindowLinuxX11InternAtomRequest` を合成し、`WM_PROTOCOLS` と `WM_DELETE_WINDOW` の `InternAtom` request batch を typed owner として作る。Rust boundary 名としては、atom kind を `NativeWindowLinuxX11WmProtocolAtomInternRequestKind`、owner を `NativeWindowLinuxX11WmProtocolAtomInternRequestBatch`、error を `NativeWindowLinuxX11WmProtocolAtomInternRequestBatchBuildError` とする。

batch owner は concatenated request bytes と request boundary offset だけを保持する。order は必ず `WM_PROTOCOLS`、`WM_DELETE_WINDOW` であり、両 request は `only_if_exists = false` で作る。これは ICCCM well-known atom を server に intern させるための request bytes であり、Atom ID が取得済みであることや window property が登録済みであることを意味しない。

F5jx は generic `InternAtom` encoding を複製せず、F5jw helper の結果を連結するだけにする。lower helper が失敗した場合は atom kind と lower error を保持し、concat byte length overflow は typed error で返す。caller supplied arbitrary atom name helper は public API にしないため、generic counted bytes policy と WM well-known name policy を混ぜない。

F5jx は request batch owner boundary だけを担当する。raw fd write/read、accepted write progress、sequence assignment、InternAtom reply parse / retain、request / reply correlation、Atom ID owner、`ChangeProperty` による `WM_PROTOCOLS` property mutation、`ClientMessage` decode、keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、Linux support gate の `Ok` 化、fallback、silent no-op、synthetic readiness は扱わない。

## F5jy Native Linux X11 top-level CreateWindow/MapWindow split request owner boundary

F5jy では、従来の `NativeWindowLinuxX11TopLevelWindowCreateRequest` に含まれていた `CreateWindow` bytes と `MapWindow` bytes を、`NativeWindowLinuxX11CreateWindowRequest` と `NativeWindowLinuxX11MapWindowRequest` として standalone owner に分離する。combined owner は互換境界として残し、standalone owner の bytes を `CreateWindow`、`MapWindow` の順に連結するだけにする。

この分離は、後続の WM protocol registration で `CreateWindow -> InternAtom batch -> InternAtom replies / Atom IDs -> ChangeProperty WM_PROTOCOLS -> MapWindow` の順序を作るための前提である。F5jy 自体は request owner split だけを担当し、current reader write path は combined owner のまま維持する。

standalone `CreateWindow` は、window id、parent window id、geometry、background pixel、event mask を既存 F5jo と同じ validation で検査する。standalone `MapWindow` は window id validation を同じ public error enum で返す。combined owner は create byte length、map byte length、total bytes、window id を参照できるが、sequence number や writer state は持たない。

F5jy は raw fd write/read、reader integration、accepted write progress、sequence assignment、InternAtom write/reply correlation、Atom ID owner、`ChangeProperty`、`ClientMessage` decode、keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、Linux support gate の `Ok` 化、fallback、silent no-op、synthetic readiness は扱わない。

## F5jz Native Linux X11 InternAtom reply packet AtomId owner boundary

F5jz では、X11 `InternAtom` reply の fixed 32 byte packet を request-specific payload として parse し、nonzero Atom ID を `NativeWindowLinuxX11AtomId` として保持する。Rust boundary 名としては、Atom ID owner を `NativeWindowLinuxX11AtomId`、reply owner を `NativeWindowLinuxX11InternAtomReply`、parse error を `NativeWindowLinuxX11InternAtomReplyParseError` とする。

parser は packet の response type が reply であることを確認し、sequence は server reply header field から保持する。`InternAtom` reply は additional reply body を持たないため `length_units == 0` を要求し、atom id は fixed reply payload の offset 8 から little-endian `u32` として読む。Atom ID 0 は `only_if_exists = true` では意味を持ち得るが、F5jz の `NativeWindowLinuxX11AtomId` は後続 WM protocol registration 用の nonzero Atom ID owner なので zero を typed error として拒否する。

F5jz は current observation reader には接続しない。現在の reader は generic `ServerReplyReceived` と body drain だけを返すため、後続 integration phase は 32 byte reply packet を generic header へ縮約する前に retain / parse する必要がある。

F5jz は raw fd write/read、reader mutation、accepted write progress、request / reply correlation、WM protocol batch sequence assignment、`ChangeProperty` による `WM_PROTOCOLS` property mutation、`ClientMessage` decode、MapWindow scheduling relocation、keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、Linux support gate の `Ok` 化、fallback、silent no-op、synthetic readiness は扱わない。

## F5ka Native Linux X11 InternAtom batch partial-write scheduling boundary

F5ka では、F5jx の `NativeWindowLinuxX11WmProtocolAtomInternRequestBatch` を `NativeWindowLinuxX11EventSourceObservationReader` の explicit optional owner state として接続する。WM protocol batch が構成された reader は、F5jy の split owner boundary に基づき `CreateWindow -> InternAtom batch -> MapWindow` の順で raw write を進める。

この boundary は MapWindow を InternAtom batch より前に送らないための最小 scheduling split を含む。batch が構成されていない reader は、従来互換の combined top-level request path を使うが、これは WM protocol registration 完了を意味しない。batch configured path と unconfigured path は state で明示的に分かれ、fallback や silent no-op として扱わない。

`NativeWindowLinuxX11WmProtocolAtomInternBatchWriteState` は `NotConfigured`、`BatchPending`、`Ready`、`Failed` を持つ。would-block は state を保持して retryable typed error を返し、hard failure、zero write、overflow は `Failed` へ進める。failed state の再 poll は previous failure として fail closed にする。

CreateWindow / MapWindow の accepted sequence は既存 top-level sequence plan が保持する。InternAtom batch については request boundary offset を使って `next_x11_request_sequence` だけを request 完了時に進める。F5ka は InternAtom reply retain / parse / correlation、Atom ID の `WM_PROTOCOLS` / `WM_DELETE_WINDOW` への meaning assignment、`ChangeProperty` による property mutation、`ClientMessage` decode、keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、support gate `Ok` 化、fallback、silent no-op、synthetic readiness を扱わない。

## F5kb Native Linux X11 InternAtom reply sequence correlation boundary

F5kb では、F5ka の InternAtom request sequence advancement を reader-owned sequence plan として保持する。Rust boundary 名としては、request sequence plan を `NativeWindowLinuxX11WmProtocolAtomInternRequestSequencePlan`、reply correlation を `NativeWindowLinuxX11WmProtocolAtomInternReplyCorrelation` とする。

sequence plan は `WM_PROTOCOLS` と `WM_DELETE_WINDOW` の request sequence を `Option u16` として保持する。batch partial write が request boundary を越えた時だけ sequence を記録し、would-block や failure で boundary 未到達の場合は記録しない。

reader は X11 reply packet を generic `ServerReplyReceived` へ縮約する前に full packet と sequence を保持し、reply body drain が必要な場合でも packet を pending state に残して stream sync を保つ。drain 完了後、sequence が `WM_PROTOCOLS` / `WM_DELETE_WINDOW` の InternAtom request と一致する packet だけを F5jz の `native_window_linux_x11_intern_atom_reply_from_packet` に渡す。

matched reply は correlation と parsed `NativeWindowLinuxX11InternAtomReply` を持つ typed observation error として返す。matched packet の parser failure は correlation と lower parser error を保持する typed error として返す。unmatched reply は既存の generic `ServerReplyReceived` のままにする。

F5kb は reply correlation boundary だけを担当する。Atom ID の `WM_PROTOCOLS` / `WM_DELETE_WINDOW` への meaning assignment、actual `ChangeProperty` registration、`WM_DELETE_WINDOW` `ClientMessage` decode、keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、support gate `Ok` 化、fallback、silent no-op、synthetic readiness は扱わない。

## F5kc Native Linux X11 WM protocol Atom meaning assignment boundary

F5kc では、F5kb の correlated `InternAtom` reply を `WM_PROTOCOLS` property Atom と `WM_DELETE_WINDOW` protocol Atom の semantic slot に割り当てる。Rust boundary 名としては、semantic slot を `NativeWindowLinuxX11WmProtocolAtomMeaning`、assignment state を `NativeWindowLinuxX11WmProtocolAtomAssignmentState`、両方が揃った completed owner を `NativeWindowLinuxX11WmProtocolAtoms` とする。

assignment state は `WM_PROTOCOLS` Atom ID と `WM_DELETE_WINDOW` Atom ID を別々の `Option` として保持する。reader は matched reply の parse が成功した場合だけ state を更新し、parse failure と unmatched generic reply では state を変えない。duplicate assignment は typed error とし、既存 slot を上書きしない。

`WmProtocolAtomInternReplyReceived` は parsed reply と completed owner の `Option` を返す。1 つ目の reply では `None`、2 つ目の reply で両 Atom ID が揃った場合だけ `Some NativeWindowLinuxX11WmProtocolAtoms` になる。この `Some` は Atom ID が揃ったことだけを表し、window property が registered されたことは表さない。

F5kc は meaning assignment boundary だけを担当する。actual `ChangeProperty` registration、`WM_DELETE_WINDOW` `ClientMessage` decode、keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、support gate `Ok` 化、fallback、silent no-op、synthetic readiness は扱わない。

## F5kd Native Linux X11 WM protocol ChangeProperty registration boundary

F5kd では、F5kc で揃った `WM_PROTOCOLS` property Atom と `WM_DELETE_WINDOW` protocol Atom を使い、top-level window に対する `ChangeProperty` request を reader-owned write path へ接続する。Rust boundary 名としては、request owner を `NativeWindowLinuxX11WmProtocolRegistrationRequest`、write state を `NativeWindowLinuxX11WmProtocolRegistrationWriteState`、sequence plan を `NativeWindowLinuxX11WmProtocolRegistrationSequencePlan` とする。

registration request は X11 `ChangeProperty` の fixed header 24 byte と 32-bit Atom data item 1 個を合わせた 28 byte request である。opcode は `18`、mode は Replace、request length は 7 units、property は `WM_PROTOCOLS` Atom、type は predefined `ATOM` Atom ID `4`、format は `32`、data length は `1`、data item は `WM_DELETE_WINDOW` Atom とする。window id と Atom ID は validated owner からだけ取得し、zero/default Atom、synthetic id、fallback id は使わない。

reader は setup-ready top-level path で `CreateWindow -> InternAtom batch -> InternAtom replies / Atom IDs -> ChangeProperty WM_PROTOCOLS -> MapWindow` の順に進める。`ChangeProperty` がまだ accepted write boundary を越えていない間は `MapWindow` を送らず、would-block では state を保持して次の poll に委ねる。hard write failure、zero write、overflow、request build failure は registration failed state として保持し、MapWindow へ進めない。

registration request の normal request sequence は、accepted write progress が 28 byte request 全体を越えた時だけ割り当てる。X11 server error correlation は top-level CreateWindow / MapWindow の plan を優先し、それに一致しない場合だけ registration sequence plan と照合する。registration request が accepted されたことは WM protocol property mutation request を送ったことだけを表し、window manager が `WM_DELETE_WINDOW` `ClientMessage` を送ることや、その event を decode できることは表さない。

F5kd は `ChangeProperty` registration write boundary だけを担当する。`WM_DELETE_WINDOW` `ClientMessage` decode、StructureNotify / Expose decode、keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、support gate `Ok` 化、fallback、silent no-op、synthetic readiness は扱わない。

## F5ke Native Linux X11 WM_DELETE_WINDOW ClientMessage decode boundary

F5ke では、F5kd の registration accepted boundary を越えた top-level window について、X11 `ClientMessage` event を `WM_DELETE_WINDOW` close request observation へ decode する。Rust boundary 名としては、accepted registration context を `NativeWindowLinuxX11RegisteredWmProtocolContext`、decode failure を `ClientMessage...` 系の `NativeWindowLinuxX11EventSourceObservationError` variant とする。

`ClientMessage` event type は raw response byte ではなく `native_window_linux_x11_event_response_type` で send-event bit を落として判定する。したがって raw response byte が `33` でも `33 | 0x80` でも `ClientMessage` として扱う。ただし close request として受けるには、registration context が存在し、format が `32`、event の window id が context の window id と一致し、message type Atom が context の `WM_PROTOCOLS` Atom と一致し、data32[0] が context の `WM_DELETE_WINDOW` Atom と一致しなければならない。

registration context は `ChangeProperty` request owner が存在するだけでは作らない。reader は `NativeWindowLinuxX11WmProtocolRegistrationWriteState::Ready` の場合だけ `NativeWindowLinuxX11RegisteredWmProtocolContext` を返す。`WaitingForAtoms`、`RequestPending`、`Failed`、`NotConfigured` では context は `None` であり、`ClientMessage` は `ClientMessageWmProtocolNotRegistered` として typed error にする。これにより Atom assignment と actual WM protocol registration accepted boundary を混同しない。

matching close は `os_close_requested = true`、`exit_shortcut_requested = false`、size と mouse state は previous input を保持し、pointer sample は `None` の observation を返す。mismatch は format、window id、message type Atom、protocol Atom ごとの typed error にする。unsupported ClientMessage を silent ignore したり、generic unsupported event へ潰したりしない。

F5ke は `ClientMessage` decode boundary だけを担当する。StructureNotify / Expose decode、keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、support gate `Ok` 化、fallback、silent no-op、synthetic readiness は扱わない。

## F5kf Native Linux X11 StructureNotify and Expose event evidence boundary

F5kf は X11 event packet を platform-neutral event kind へ写す boundary である。`NativeWindowEventPumpEventKind` は `Poll`、`CloseRequested`、`WindowResized`、`PointerMotion`、`PointerButton`、`WindowMapped`、`RedrawRequested` を分ける。`NativeWindowLinuxWindowEventSourceObservation` は event kind を保持し、`NativeWindowEventPumpSnapshot`、backend loop outcome、host action へ伝播する。

default event mask は `ButtonPress | ButtonRelease | PointerMotion | Exposure | StructureNotify` とする。これにより X11 top-level window は resize、map、expose を実際に受け取るが、受け取った event を no-op observation として潰してはならない。`ConfigureNotify` は `WindowResized`、`MapNotify` は `WindowMapped`、`Expose` は `RedrawRequested` として decode する。`Expose` も send-event bit を落とした event type で判定する。

compatibility builder は close、resize、pointer button だけを event kind として推論し、pointer sample があるだけでは `PointerMotion` を合成しない。concrete backend が motion event を読んだ場合だけ explicit builder で `PointerMotion` を渡す。

F5kf は event evidence boundary だけを担当する。keyboard / IME、Wayland concrete decoding、Linux runner / CLI dispatch、support gate `Ok` 化、fallback redraw、silent no-op、synthetic readiness は扱わない。

## F5ew Native and Bare scheduler executor one-step bridge boundary

2026-06-18 の F5ew では、Native and Bare scheduler executor one-step bridge boundary を追加する。これは backend-facing one-step bridge であり、not long-running scheduler backend である。Native は `GuiNativeSchedulerExecutorInputReady`、Bare は `GuiBareSchedulerExecutorInputReady` と borrowed F5ek policy を受ける。ready payload から original `ExecuteHostAction` と packaged `RealLoopStepInput::ExecutorOutcome` を取り出し、`LoopAction::ExecuteHostAction` と input を F5ek `real_loop_step` へ 1 回だけ渡す。戻り値は F5ek の `Result RealLoopStepResult RealLoopStepError` をそのまま返す。F5ew は host action executor、action sink / driver、support validation、clock / timer helper、queue、while loop、present、minifb、Canvas、DOM、video memory、fallback、silent no-op を実装しない。

## F5ex Native and Bare scheduler host action executor backend bridge

2026-06-18 の F5ex では、Native and Bare scheduler host action executor backend bridge を追加する。これは typed `ExecuteHostAction` payload を actual platform host import へ 1 operation だけ渡す backend-facing boundary であり、not long-running scheduler backend である。platform bridge は `ExecuteHostAction` の owner を保持したまま borrowed accessor で pending span operation を読み、operation variant ごとに begin / run / end のいずれか 1 host import を呼ぶ。raw status は `Result unit GuiError` へ写し、bare で executor ABI が提供されない場合も `Unsupported` を返す。fallback や silent no-op は禁止する。

F5ex は F5ev/F5ew と接続される。host import outcome は F5ev `scheduler_executor_input` で `RealLoopStepInput::ExecutorOutcome` に包み、F5ew `scheduler_executor_step` が F5ek `real_loop_step` へ渡す。これにより platform execution は platform module に閉じ、scheduler completion の authority は std scheduler driver 側に残る。queue、while loop、timer wait、present loop、FHD 60fps 実測、2D compositor drain、minifb / DOM / Canvas / video memory 操作、old action sink / driver、raw RenderCommand accessor、fallback はこの phase の責務ではない。

## F5ey Native Rust span operation host ABI validator checkpoint

2026-06-18 の F5ey では、F5ex native host import の Rust side boundary として、`nepl-gui-native` に scalar span operation validator と injected `NativeSpanOperationSink` を追加する。これは actual renderer ではなく、Wasm host ABI から届く begin / run / end payload を typed `NativeSpanOperation` へ変換する直前の検証境界である。status sentinel は Web video memory host ABI と同じ 0 / -1 / -2 / -3 / -4 / -5 / -6 系にそろえ、unknown sink status は `BackendFailure` に正規化する。

descriptor は sink を呼ぶ前に、positive id、frame id 一致、checked extent、`stride_bytes == width * 4`、`pixel_count == width * row_count`、`encoded_byte_count == total_run_count * 12`、`tile_count == ceil(plan_row_count / tile_rows)`、`tile_index < tile_count` をすべて満たす必要がある。run span は current row の span として `height == 1`、positive width、non-negative x / y、0..255 の RGBA channel を要求する。invalid scalar input は sink に渡さず `InvalidArgument` を返す。minifb rendering、window loop、queue、timer、video memory、DOM、Canvas、fallback、silent no-op はこの checkpoint の責務ではない。

## F5ez Native RGBA8888 framebuffer sink checkpoint

2026-06-18 の F5ez では、F5ey の typed operation を native offscreen framebuffer へ反映する `NativeRgba8888FrameBuffer` sink を追加する。pixel storage は semantic `0xRRGGBBAA` の `u32` であり、native endian byte view ではない。constructor は positive width / height、checked `width * height`、`stride_bytes == width * 4` を満たす場合だけ成功し、field は private にする。

Begin は active sequence が無い場合だけ受け、descriptor と `seen_run_count = 0` を保持する。RunSpan は active descriptor、target、`x >= 0`、`width > 0`、`height == 1`、row extent、x extent、remaining run count を検査し、全検査の後だけ pixel を書く。成功した RunSpan だけが `seen_run_count` を増やす。End は descriptor equality と `seen_run_count == descriptor.total_run_count` を要求し、短い span 列や余分な span を silent partial frame として成功させない。失敗した RunSpan / End は active state と pixels を壊さない。F5ez は minifb `update_with_buffer`、window loop、scheduler loop、video memory、DOM、Canvas、fallback、silent no-op へ進まない。

## F5fa Native RGB0 present buffer conversion checkpoint

2026-06-18 の F5fa では、completed `NativeRgba8888FrameBuffer` を native presenter 用の semantic `0x00RRGGBB` present buffer へ変換する境界を追加する。`NativeRgb0PresentBuffer` は private field と read-only accessor だけを持ち、mutable pixel access や byte slice view を公開しない。source framebuffer に active sequence が残っている場合は変換を拒否し、silent partial frame を present 成功扱いにしない。

変換は `0xRRGGBBAA` から shift / mask で channel を取り出し、caller supplied `NativeRgbColor` を explicit background として source-over alpha composition する。channel formula は `(source * alpha + background * (255 - alpha) + 127) / 255` で固定する。hidden default background、fallback background、native endian byte cast、minifb integration、window loop、scheduler loop、DOM、Canvas、video memory、fallback、silent no-op はこの checkpoint の責務ではない。

## F5fb Native presenter frame adapter checkpoint

2026-06-18 の F5fb では、`NativeRgb0PresentBuffer` を native presenter が消費できる immutable frame へ借用変換する adapter を追加する。adapter は minifb 型や OS handle を public type に含めず、checked `usize` width / height と immutable `&[u32]` pixels だけを公開する。minifb `update_with_buffer` 呼び出しは smoke runner の `main.rs` に閉じ込める。

smoke runner の既存 demo rasterizer から来る pixels は `NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo` を通す。この import は `width > 0`、`height > 0`、checked `width * height == pixels.len`、every pixel の high byte が 0 の `0x00RRGGBB` であることを要求する。high byte がある pixel は mask せず、`Result` error とする。F5fb は正式 NEPL span path、scheduler loop、queue、timer、bare runtime host import、formal `std/gui` present host import、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization へ進まない。

## F5fc Native RGB0 presenter sink checkpoint

2026-06-18 の F5fc では、validated `NativeSpanOperation` stream を `NativeRgba8888FrameBuffer` へ書き込み、complete End の成功時だけ `NativeRgb0PresentBuffer` と typed presenter frame へ進める native sink boundary を追加する。`NativeRgb0PresenterSink` は framebuffer、explicit background、last completed RGB0 buffer、last presented frame id を所有する。

Begin / RunSpan は F5ez と同じ検査を使う。End は descriptor equality と exact `seen_run_count` を満たした後、RGB0 conversion を先に行い、conversion succeeds の後だけ active sequence を閉じて last completed buffer / frame id を更新する。conversion failure、descriptor mismatch、run count mismatch、invalid run は previous completed frame を置き換えない。F5fc は window update call、scheduler loop、queue、timer、bare runtime host import、formal `std/gui` present host import、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization へ進まない。

## F5fd Native window presenter state checkpoint

2026-06-18 の F5fd では、F5fc の completed typed presenter frame と frame id を native window presenter state にコピー保持する lib-only boundary を追加する。`NativeWindowPresenterState` は surface state、last presented frame id、last frame dimensions、RGB0 pixels を private field として所有し、read-only accessor だけを公開する。

`resize_surface` は positive drawable size を `NativeWindowPresenterSurfaceState::Drawable`、zero width / height を `NativeWindowPresenterSurfaceState::Unavailable` として明示的に記録する。resize は previous frame を stretch / crop せず、次の present が成功するまで last frame をそのまま保持する。`present_sink_frame` は completed frame と completed frame id の両方を要求し、missing frame、missing frame id、validation failure、resource exhaustion、dimension overflow を distinct `NativeWindowPresenterError` として返す。state replacement は temporary buffer の validation と reservation 成功後だけ行う。

F5fd は minifb / OS window API、scheduler loop、timer、queue、bare runtime host import、formal `std/gui` present host import、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization へ進まない。

## F5fe Native window presenter smoke integration checkpoint

2026-06-18 の F5fe では、native smoke runner の window loop を `NativeWindowPresenterState` 経由の present / hit-test path へ寄せる。これにより `main.rs` が current present buffer を並列所有する状態をやめ、display と hit test の authoritative frame は `NativeWindowPresenterState::last_present_frame_required` から得る。

`NativeWindowPresenterError` は `InvalidFrameId` を持つ。`present_frame` は `frame_id > 0` を要求し、`present_buffer` と `present_sink_frame` は frame validation 後に `present_frame` へ委譲する。native smoke runner の frame id は `checked_add` で進め、overflow は error として返す。reset、wrap、saturating cast、silent reuse は行わない。

F5fe は smoke window integration だけであり、formal `std/gui` host import、scheduler loop、timer、queue、bare runtime host import、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization へ進まない。minifb / OS window API は `main.rs` だけに残し、`lib.rs` は platform-independent presenter state と validation boundary のままにする。

## F5ff Native window resize redraw checkpoint

2026-06-18 の F5ff では、native smoke runner の resize 後表示を OS scaling から exact-size redraw へ寄せる。window loop は positive resize を受けたら `NativeWindowPresenterState::resize_surface` を呼び、`checked_add` で frame id を進め、`rasterize_frame_to_surface` で current drawable surface と同じ width / height の RGB0 buffer を作り直してから `present_buffer` する。

minifb の window option は `ScaleMode::UpperLeft` を使い、`Window::update_with_buffer` 直前に `last_present_frame_required` の frame size と current window size が一致することを検査する。これにより window を広げた分だけ presenter frame の pixel resolution が増え、layout / app が resize event に応じて新しい pixel buffer を作るという将来の formal host contract と同じ向きになる。

zero-size surface は `NativeWindowPresenterSurfaceState::Unavailable` として保持する。この状態では blank frame を合成せず、positive size が戻るまで event pump だけを進める。F5ff は smoke runner の resize redraw contract の固定であり、formal native host import、scheduler loop、queue、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。

## 層構造

依存方向は次に固定する。

```text
core/gui
    ↑
alloc/gui
    ↑
std/gui
    ↑
platforms/gui/web
platforms/gui/native
platforms/gui/mobile
platforms/gui/embedded
platforms/gui/terminal
```

逆方向の依存は禁止する。

```text
core/gui -> alloc/gui          禁止
core/gui -> std/gui            禁止
core/gui -> platforms/gui/web  禁止
alloc/gui -> DOM / OS API      禁止
std/gui -> application model   原則禁止
```

## Rendering Model

標準 pipeline は次である。

```text
State + Event -> State + Effects
State -> ViewTree
ViewTree + LayoutContext -> LayoutTree
LayoutTree + RenderContext -> DrawCommand stream
DrawCommand stream -> Rasterizer / RenderTarget
Rasterizer / RenderTarget -> PixelBuffer / TextGridFrame
PixelBuffer / TextGridFrame -> SurfacePresenter
```

TUI では `RenderTarget` が text-cell command を受け取り、terminal host が ANSI / alternate screen / raw mode などへ変換する。ANSI escape sequence は `platforms/gui/terminal` または `platforms/wasix/tui` backend の実装詳細であり、`alloc/gui/widget` には漏らさない。

Web の可視 canvas は正式経路では pixel buffer を `putImageData` で表示するだけである。Canvas2D の `fillRect`、`stroke`、`fillText`、`drawImage` を標準 GUI content renderer として使う経路は持たない。CLI-only、headless、bare、unsupported surface では別の表示経路へ自動で落とさず、capability と `GuiError::Unsupported` で表す。

2D rendering engine と font rendering engine は別々の描画規則を持たない。Path、SVG、image、button skin、glyph mask、ruby、math glyph は `Paint`、`Stroke`、`Shadow`、`BlendMode`、`Clip`、`Transform2d` を共有する。詳細は `doc/neplg2/gui_2d_rendering_design.md` と `doc/neplg2/gui_font_rendering_design.md` で定義する。

## Layer 1: Pixel Drawing

`core/gui` の pixel layer は no_alloc で成立する。

```text
core/gui/geometry
    GuiPoint
    GuiSize
    GuiRect
    GuiInsets
    GuiScaleFactor

core/gui/color
    BinaryColor
    Gray8
    Rgb565
    Rgb888
    Rgba8888

core/gui/pixel
    Pixel Color

core/gui/draw_target
    DrawTarget Color Error
    FlushTarget Error
    Drawable Color Output
```

`DrawTarget` は「描けること」だけを表す。`flush` は display / framebuffer / browser / GPU surface ごとに presentation model が異なるため、`FlushTarget` として分ける。

## Layer 2: Command Rendering

Pixel-only API は embedded では有効だが、Web、native、mobile、terminal では抽象度が低すぎる。標準 API は高水準 command stream を持つ。

```text
DrawCommand:
    FillRect GuiRect GuiPaint
    StrokeRect GuiRect GuiStroke
    Line GuiPoint GuiPoint GuiStroke
    TextRun TextRunId GuiPoint TextPaint
    ImageRect ImageId GuiRect
    PushClip GuiRect
    PopClip
    PushTransform Transform2d
    PopTransform
    TextCellRun TextGridPoint TextCellRun
```

`TextCellRun` は TUI / terminal backend のための command である。terminal は pixel display ではないため、GUI と完全に同じ `FillRect` だけへ押し込めない。共通化する境界は widget / event / layout / effect / capability であり、render command は surface kind ごとの最小差を enum と capability で表す。

## Layer 3: UI Tree And App Model

Application model は次である。

```text
App Model:
    init   %fn void Model
    update %fn Model GuiEvent (Update Model)
    view   %fn Model ViewTree

Update Model:
    model   Model
    effects Vec GuiEffect
```

Widget は closure を保持しない。例えば button は action identifier を持つ。

```text
ButtonConfig:
    id WidgetId
    text str
    action ActionId
    style ButtonStyle
```

`GuiEvent::Action action` を application の `update` が受け取り、`match` で処理する。

`WidgetId` と `ActionId` は `core/gui/event` が所有する。理由は、raw pointer / keyboard / lifecycle event と同じ no_alloc event identity として扱う必要があるためである。`alloc/gui` はそれらの id を `ViewTree`、`LayoutTree`、widget descriptor の意味へ結び付ける。platform backend は raw input を typed event へ変換してよいが、application action の意味付けを platform 固有 code に閉じ込めてはいけない。

## Event Model

標準 event は全 platform を表現できる union とする。

```text
GuiEvent:
    Pointer PointerEvent
    Keyboard KeyboardEvent
    TextInput TextInputEvent
    Window WindowEvent
    Timer TimerEvent
    Lifecycle LifecycleEvent
    Accessibility AccessibilityEvent
    Action ActionId
```

Mobile では `LifecycleEvent` が必須である。Native desktop では一部だけ使う。Embedded や TUI では多くを使わない場合があるが、使わない event があることと、型体系上 event を持たないことは別である。標準 API は全 platform を表現できる event enum を持ち、実際の有無は `GuiCapabilities` で示す。

TUI は keyboard event と text input event の両方を持つ。raw key sequence は backend が `KeyboardEvent` / `TextInputEvent` へ正規化し、focus 移動や activate は `std/gui/keymap` が `KeyboardEvent` と `FocusKeyMap` から `Option FocusRouteCommand` へ変換する。`GuiEvent::Action` は `alloc/gui/routing/focus` が current focus と widget descriptor を見て発行する。application model は ANSI byte sequence や DOM key string を直接扱わない。terminal input については `platforms/gui/terminal/input` が `TerminalInputEvents` を返し、Space のように keyboard と text input の両方になりうる入力を `Option` pair として保持する。std layer は navigation key code と modifier bit の正規化 contract を持つが、ANSI / CSI byte pattern 自体は持たない。

## Routing Model

Event routing は `alloc/gui/routing` が所有する pure data 変換である。標準経路は次である。

```text
LayoutTree + GuiPoint
    -> Option WidgetId

ViewTree + WidgetId
    -> Option WidgetDescriptor

WidgetDescriptor
    -> Option GuiEvent
```

Allocator-backed tree では同じ contract を次の経路で扱う。

```text
LayoutTreeArena + GuiPoint
    -> Option WidgetId

ViewTreeArena + WidgetId
    -> Option WidgetDescriptor

WidgetDescriptor
    -> Option GuiEvent
```

Pointer hit test は `GuiRect` の half-open bounds を使う。つまり left/top は含み、right/bottom は含まない。bounded checkpoint では child が root より前面にあり、second child は first child より前面にある。`LayoutTreeArena` では arena insertion order の後方を前面として末尾から走査する。layout arena と view arena の対応は `WidgetId` identity で行い、parent index や arena storage index は routing の public identity にしない。disabled widget、または layout hit に対応する widget が view tree に存在しない場合は `Option::None` を返し、panic や silent raw event にはしない。

Focus traversal、keyboard mapping、focus routing は別契約である。`alloc/gui/focus` は bounded `ViewTree` から `FocusOrder` を作り、また allocator-backed `ViewTreeArena` を直接走査して、current focus id から next / previous focus target を返すだけで、application event は発行しない。`WidgetId` は arena storage index ではなく widget identity として比較する。`std/gui/keymap` は platform raw input から切り離された `KeyboardEvent` を `FocusRouteCommand` へ変換する。`alloc/gui/routing/focus` は解釈済みの `FocusRouteCommand` を受け取り、結果を次の enum で返す。

```text
FocusRouteCommand:
    Next
    Previous
    Activate

FocusRouteResult:
    Ignored
    MoveFocus WidgetId
    Emit GuiEvent
```

`Next` / `Previous` は focus 移動だけを `MoveFocus` として返す。`Activate` は current focus id が指す widget の action event だけを `Emit` として返す。current focus が `None`、古い `WidgetId`、disabled widget、action を持たない widget、端で移動先がない traversal は `Ignored` になる。Tab、Shift+Tab、Enter、Space の portable default mapping は `std/gui/keymap` の `FocusKeyMap` が所有する。terminal raw byte / escape sequence、DOM keyboard event、OS virtual key は platform backend が `KeyboardEvent` / `TextInputEvent` へ正規化し、platform から `FocusRouteCommand`、`GuiEvent::Action`、application-specific `ActionId` を直接作らない。

標準の keyboard focus 経路は次で固定する。

```text
platform raw input
    -> KeyboardEvent / TextInputEvent
KeyboardEvent + FocusKeyMap
    -> Option FocusRouteCommand
FocusRouteCommand + FocusOrder + ViewTree
    -> FocusRouteResult
FocusRouteResult::Emit
    -> GuiEvent::Action
```

`FocusRouteCommand` は focus intent だけを表す。application 固有の action の意味は application `update` が `GuiEvent::Action` を `match` して決める。focus state の保持と `MoveFocus` の反映は runtime または application model の上位状態が担当し、`alloc/gui/routing/focus` 自体は純粋関数として状態を変更しない。

Terminal input の現 checkpoint は 1 byte ASCII subset、3 byte ESC sequence の一部、4 byte CSI tilde sequence の一部、xterm style の bounded 6 byte modifier arrow sequence を扱う。Tab(9) は key code 9、LF(10) / CR(13) は key code 13、Space(32) は key code 32 と text input `' '`、printable ASCII(33..126) は text input のみへ正規化する。`ESC [ Z` は Shift+Tab として key code 9、modifier bit 1、text input なしの `KeyboardEvent` へ正規化する。`ESC [ A/B/C/D` は std navigation key code の ArrowUp / ArrowDown / ArrowRight / ArrowLeft へ正規化する。`ESC [ H/F` と `ESC [ 1/4 ~` は Home / End、`ESC [ 3 ~` は Delete の typed key code へ正規化する。`ESC [ 1 ; <modifier> A/B/C/D` は modifier byte `2..8` を Shift / Alt / Control bitset へ変換し、arrow key の `KeyboardEvent` として返す。範囲外 byte、不正な CSI numeric parameter、または認識済み arrow key に対する範囲外 modifier parameter は `GuiError::InvalidCommand` である。範囲内で未対応の control byte、未知の 3 byte sequence、valid shape だが未対応 final key の CSI sequence、未対応だが valid な CSI tilde numeric parameter は event なしとして扱う。Function key、IME/text-edit context による Enter / Tab の text 化、途中入力 buffering は後続 slice で実装する。

この routing は callback を呼ばず、clipboard、window、terminal、DOM、OS API に触れない。pointer capture、gesture recognition、pointer cancel、IME focus、accessibility focus、flex / grid / scroll などの本格 layout policy は `std/gui` / `platforms/gui/*` と連携する上位状態であり、現 checkpoint では未実装である。stack layout は `alloc/gui/layout/stack` の pure data policy として実装済みであり、axis / spacing、cross-axis alignment、overflow rejection を `Result` で扱う。ただし flex grow、grid placement、scroll state はまだ扱わない。TUI でも keyboard / focus routing は最終的に `FocusRouteCommand` と `GuiEvent::Action` を使い、application が raw ANSI sequence を直接 `match` する経路を作らない。

## Capability Model

Backend は capability を公開する。

```text
GuiCapabilities:
    surface_kind SurfaceKind
    color_format ColorFormat
    has_allocator bool
    has_windowing bool
    has_multi_window bool
    has_pointer bool
    has_touch bool
    has_keyboard bool
    has_text_input bool
    has_clipboard bool
    has_accessibility bool
    requires_flush bool
    max_texture_width Option i32
    max_texture_height Option i32
```

`SurfaceKind` は少なくとも次を持つ。

```text
SurfaceKind:
    WindowPixel
    OffscreenPixel
    DevicePixel
    TextGrid
    Headless
```

TUI backend は `SurfaceKind::TextGrid` を返す。Web / native window backend は `WindowPixel`、offscreen screenshot backend は `OffscreenPixel`、bare / embedded display backend は `DevicePixel`、GUI 表示を持たず virtual event replay だけを扱う backend は `Headless` を返す。旧文書や既存 checkpoint の `Pixel` は `WindowPixel` または `DevicePixel` へ分解する移行対象であり、`Command` は surface kind ではなく rasterizer へ渡す前の `DrawCommand stream` の概念として扱う。

Unsupported operation は panic や silent no-op ではなく `GuiError::Unsupported` を返す。ただし、仕様として best-effort と明記した effect だけは no-op を許す。

## Invalidation And Dirty Region

再描画に関する用語は 3 層に分ける。

```text
alloc/gui/diff
    GuiInvalidation
    retained tree の semantic invalidation

core/gui/dirty_region
    DirtyRegion
    no_alloc framebuffer / embedded 向け rect dirty contract

core/gui/dirty_region_set
    DirtyRegionSet
    no_alloc fixed-capacity rect set contract

platforms/gui/*
    DOM patch、terminal line diff、GPU surface damage、framebuffer compression
    backend implementation detail
```

`GuiInvalidation` は widget tree や layout tree のどこが古くなったかを表す。bounded `ViewTree` は slot diff を `ViewTreeDiff` として保持し、allocator-backed `ViewTreeArena` は `ViewTreeArenaDiff` として node count、shape change、content change count、単一 changed `WidgetId` を保持する。arena diff では parent index、depth、slot の `WidgetId` を tree shape / order として扱い、arena storage index を invalidation payload にしない。単一 content change は `GuiInvalidation::Widget id`、node count / shape / id 対応変更、または複数 content change は `GuiInvalidation::Tree` へ畳む。

`DirtyRegion` は `GuiRect` を使い、embedded / framebuffer backend が redraw area を allocator なしで表すための値である。現 checkpoint の `DirtyRegion` は `Empty` / `Rect` / `Full` のみを持ち、複数 rect は保持せず、Rect 同士の merge は bounding rect へ O(1) で畳む。

`DirtyRegionSet` は embedded / framebuffer 向けの fixed-capacity no_alloc rect set contract である。現 checkpoint は最大 2 個の `GuiRect` を保持し、3 個目の追加は silent no-op や panic ではなく `Full` 状態への昇格として表す。負の width / height は `GuiError::InvalidGeometry` として拒否し、x / y の負値は相対座標として許容する。zero-size rect は有効な `GuiRect` として扱い、必要なら backend 側が present 時に無視できる。`dirty_regions_push_region_checked` は `DirtyRegion` を pre-transport aggregation として取り込み、`Empty` は既存 set を返し、`Full` は `dirty_regions_full` を返し、`Rect` は `dirty_regions_push_checked` を経由して invalid rect を拒否する。これは fallback ではなく、`DirtyRegion` の明示状態を fixed-capacity set へ写す checked boundary である。DOM patch、terminal line diff、GPU surface damage compression、tile / bitmap transport は standard API の semantic diff ではなく backend / later transport detail とする。

`alloc/gui/render2d/dirty_surface` の `GuiRgba8888SoftwareSurfaceDirtyOwner` は、RGBA8888 software surface owner と `DirtyRegionSet` を同じ surface + dirty owner boundary に束ねる。dirty の更新は `dirty_regions_push_region_checked` を surface move より前に通し、失敗時は owner-bearing error で元 owner を返す。公開 API は shape / dirty の Copy metadata、`finish_surface`、free に限定し、raw surface accessor、mutable accessor、split accessor は出さない。`finish_surface` は dirty metadata を捨てる recovery / teardown API であり、transport / present / fallback ではない。

`alloc/gui/render2d/bitmap_frame` の `GuiRgba8888BitmapFrameOwner` は、dirty surface owner を formal transport 前に validated bitmap frame owner へ変換する。`frame_id > 0`、surface width / height / stride / byte_len は `gui_rgba8888_software_surface_shape` で再検証した expected metadata と一致し、dirty rect は x/y、width/height、right/bottom overflow、surface containment を通過する必要がある。失敗は `GuiRgba8888BitmapFramePrepareErrorKind` と owner-bearing `GuiRgba8888BitmapFramePrepareError` で返し、代表 kind として `SurfaceStrideMismatch`、`SurfaceByteLengthMismatch`、`DirtyRectOutOfBounds` を持つ。`finish_surface` は全 validation 成功後だけ surface owner を move する recovery / teardown boundary であり、host present、video memory host call、row byte copy、tile list、Canvas / DOM / minifb、fallback はこの layer へ入れない。

`alloc/gui/render2d/row_batch_plan` の `GuiRgba8888RowBatchPlanOwner` は、validated bitmap frame owner を formal byte payload / host present 前の row batch plan owner へ変換する。通常 application code の owner aggregate 直 constructor は compiler が拒否するが、compiler memory boundary や trusted producer から forged metadata が来てもよい前提で、`max_rows_per_batch > 0`、`frame_id > 0`、frame width / height / stride / byte_len の shape 再検証、dirty rect origin / size / right-bottom overflow / surface containment を通過した後で、dirty state を contiguous row span と batch count に畳む。`GuiRgba8888RowBatchPlanPrepareErrorKind` は `MaxRowsPerBatchInvalid`、`FrameStrideMismatch`、`DirtyRectBottomOverflow`、`DirtyRectOutOfBounds` などを持ち、prepare error は bitmap frame owner を保持する。`Empty` dirty は row_count 0 の clean-frame plan であり fallback ではない。`Two` dirty は 2 rect を覆う contiguous row span として扱い、tile list や row byte copy は作らない。`finish_frame` は row plan metadata を捨てて bitmap frame owner を返す recovery / teardown boundary であり、`finish_surface`、host present、video memory host call、Canvas / DOM / minifb、fallback はこの layer へ入れない。

`alloc/gui/render2d/row_batch_cursor` の `GuiRgba8888RowBatchCursorOwner` は、row batch cursor owner として row batch plan owner を scheduler が 1 batch ずつ進められる descriptor stream へ変換する。`gui_rgba8888_row_batch_cursor_start` は full plan invariant を再検証し、失敗時は `GuiRgba8888RowBatchCursorErrorKind::PlanInvariant` に lower `GuiRgba8888RowBatchPlanInvariantErrorKind` を保持する。`GuiRgba8888RowBatchCursorStatus` は `Ready` / `Complete` の Copy enum であり、`batch_index == batch_count` だけが complete、負値や past-end index は typed error である。`gui_rgba8888_row_batch_cursor_next_batch` は `Ready` cursor から `GuiRgba8888RowBatchDescriptor` と continuation cursor owner を返し、descriptor は frame_id / batch_index / row_start / row_count / width / height / stride_bytes / byte_len の metadata のみを持つ。caller は descriptor を読んだあと `gui_rgba8888_row_batch_cursor_batch_finish_cursor` で continuation cursor owner を取り出す。complete 用 owner terminal、drain / budget、row byte payload、tile list、host present、video memory host call、Canvas / DOM / minifb、fallback はこの layer へ入れない。

`alloc/gui/render2d/row_batch_drain` の row batch scheduler drain は、row batch cursor owner を scheduler budget 内で進めた結果を表す progress-only terminal である。`GuiRgba8888RowBatchDrainTerminal` は owner-bearing struct で、Copy enum `GuiRgba8888RowBatchDrainStatus`、continuation cursor owner、`emitted_count` を持つ。status は budget より先に判定され、complete cursor は負 budget でも `Completed` になる。ready cursor で `remaining_steps == 0` は `StepBudgetExhausted`、`remaining_steps < 0` は owner-bearing `InvalidBudget` error である。positive budget では `next_batch` 後に descriptor batch index と continuation cursor index の progress invariant を検査し、`ProgressInvariantInvalid` で止める。count 加算も checked にし、overflow は `EmittedCountOverflow` で返す。`emitted_count` は進めた batch descriptor 数であり、row payload や transport emission ではない。row bytes、tile / RLE、`Vec` collection、surface finish、host present、video memory host call、Canvas / DOM / minifb、fallback はこの layer へ入れない。

`alloc/gui/render2d/row_batch_range` の row batch range metadata boundary は、`GuiRgba8888RowBatchCursorBatchOwner` から `GuiRgba8888RowBatchRangeOwner` を作る。`GuiRgba8888RowBatchRangeOwner` は元の batch owner と Copy metadata `GuiRgba8888RowBatchRange` を保持し、range は frame_id、batch_index、row_start、row_count、width、height、stride_bytes、byte_len、`start_byte_offset`、`byte_count` を持つ。prepare はまず cursor 側の `gui_rgba8888_row_batch_cursor_batch_validate_descriptor_authority` を呼び、embedded plan invariant を再検査したうえで正規 descriptor と一致するかを確認する。authority failure は `BatchAuthorityInvalid %GuiRgba8888RowBatchCursorErrorKind` として lower kind を保持し、continuation cursor status の failure は別に `ContinuationCursorInvalid %GuiRgba8888RowBatchCursorErrorKind` として保持する。range arithmetic は `width * 4 == stride_bytes`、`height * stride_bytes == byte_len`、`row_start + row_count <= height`、`start_byte_offset + byte_count <= byte_len` を checked に検査する。byte storage 前の borrowed revalidation は stored range metadata を再計算 range と比較し、不一致なら `RangeMetadataMismatch` を返す。検査中に batch owner を消費せず、success owner の finish / free だけが continuation cursor へ戻る。row byte storage、tile / RLE、host present、video memory host call、Canvas / DOM / minifb、fallback はこの layer へ入れない。

`alloc/gui/render2d/row_byte_storage` の row byte storage boundary は、`GuiRgba8888RowBatchRangeOwner` を `GuiRgba8888RowByteStorageOwner` へ変換する。`GuiRgba8888RowByteStorageOwner` は continuation cursor、`GuiRgba8888RowBatchRange`、exact `byte_count` の copied byte storage を所有する。source storage は private sealed copy helper だけが借用し、public API は source `RegionToken`、`MemPtr`、raw storage accessor を返さない。prepare は `gui_rgba8888_row_batch_range_owner_validate_authority` で range owner authority を再検証してから allocation / copy に進み、copy が完全に成功するまで range owner を消費しない。copy error は source offset overflow、destination index invalid、projection、load、store に分け、copy failure 後の scratch cleanup 失敗は `ScratchDeallocFailed` として original copy error と分ける。read helper は copied destination byte の checked reader であり、source surface escape ではない。この layer は no tile / RLE / host present で止まり、video memory host call、Canvas / DOM / minifb、platform surface、fallback には進まない。

`alloc/gui/render2d/row_tile_plan` の row tile plan metadata boundary は、`GuiRgba8888RowByteStorageOwner` を `GuiRgba8888RowTilePlanOwner` へ変換する。`GuiRgba8888RowTilePlanOwner` は exact copied byte storage owner と Copy metadata `GuiRgba8888RowTilePlan` を保持し、`GuiRgba8888RowTilePlan` は frame_id、batch_index、row_start、row_count、width、height、stride_bytes、byte_count、tile_rows、tile_count を持つ。prepare は `gui_rgba8888_row_byte_storage_validate_authority` で continuation cursor の `batch_index - 1` から expected range を再計算し、stored range と一致することを借用で再検証する。`tile_count` は quotient / remainder による checked ceil で計算し、overflow しやすい加算式は使わない。`descriptor_at` は `&GuiRgba8888RowTilePlanOwner` を借用し、`gui_rgba8888_row_tile_plan_validate_invariants` で storage authority、range metadata、`stride_bytes == width * 4`、`row_start + row_count <= height`、`byte_count == row_count * stride_bytes`、`tile_count == ceil(row_count / tile_rows)` を再検証してから descriptor を返す。`GuiRgba8888RowTileDescriptor` の `row_start` は frame-absolute row、`byte_offset` は copied row byte storage 内の storage-relative byte offset である。この layer は no RLE / host present で止まり、byte payload split、RLE encode、video memory host call、Canvas / DOM / minifb、platform surface、fallback には進まない。

`alloc/gui/render2d/row_tile_payload` の row tile payload view boundary は、`GuiRgba8888RowTilePlanOwner` と tile index を `GuiRgba8888RowTilePayloadOwner` へ変換する。これは owned payload buffer ではなく、existing copied row storage 上に `GuiRgba8888RowTileDescriptor` を重ねた tile-scoped byte payload view である。prepare は `gui_rgba8888_row_tile_plan_descriptor_at &plan tile_index` を通して descriptor authority と bounds を再検証し、失敗時は `DescriptorInvalid` と original tile plan owner を owner-bearing error に保持する。`byte_at` は tile-relative index を `0 <= index < descriptor.byte_count` で検査し、`descriptor.byte_offset + index` を checked add で storage-relative index へ変換してから `gui_rgba8888_row_byte_storage_byte_at` を呼ぶ。lower storage read failure は `StorageReadFailed` に包む。`gui_rgba8888_row_tile_plan_storage_ref` は raw `RegionToken` / `MemPtr` を返さず、typed borrowed authority として `&GuiRgba8888RowByteStorageOwner` だけを返す。この layer は no RLE / host present で止まり、追加 allocation、追加 copy、RLE encode、video memory host call、Canvas / DOM / minifb、platform surface、fallback には進まない。

`alloc/gui/render2d/row_tile_rle` の row tile RLE cursor boundary は、`GuiRgba8888RowTilePayloadOwner` を `GuiRgba8888RowTileRleCursorOwner` へ変換し、tile 内の RGBA8888 pixel run を streaming で返す。`GuiRgba8888RowTileRleRun` は `pixel_offset`、`pixel_count`、`Rgba8888 color` を持つ Copy metadata であり、cursor / step / step error は payload owner または continuation cursor owner を保持する owner-bearing value なので Clone / Copy を実装しない。`cursor_start` は payload byte count が正で 4 byte RGBA8888 pixel に整列していることを検査し、失敗時は payload owner を start error に保持する。`cursor_status` は `Ready` / `Complete` の Copy enum であり、負 index と past-end index は typed error である。`cursor_next_run` は complete cursor に対して silent no-op を返さず、`CursorComplete` owner-bearing error を返す。pixel read は `pixel_index * 4` と channel offsets `+1` / `+2` / `+3` を checked arithmetic で検査し、payload read failure は `PayloadReadFailed %GuiRgba8888RowTilePayloadReadErrorKind` に包む。この layer は streaming-only であり、encoded RLE buffer、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback には進まない。

`alloc/gui/render2d/row_tile_rle_drain` の row tile RLE drain boundary は、`GuiRgba8888RowTileRleCursorOwner` を scheduler budget 内で進め、`GuiRgba8888RowTileRleDrainTerminal` または `GuiRgba8888RowTileRleDrainError` を返す。`GuiRgba8888RowTileRleDrainTerminal` は `status`、continuation cursor、`emitted_run_count` を保持する owner-bearing value であり、Clone / Copy を実装しない。`status` は Copy enum の `Completed` または `StepBudgetExhausted` である。drain は complete status を budget より先に判定し、complete cursor は負 budget でも `Completed`、Ready cursor の負 budget は `InvalidBudget` owner-bearing error、Ready cursor の zero budget は step を実行せず `StepBudgetExhausted` になる。positive budget では discard する run metadata の `pixel_offset` と `pixel_count` を continuation cursor の `next_pixel_index` と照合し、`emitted_run_count` を checked arithmetic で増やす。この layer は encoded RLE buffer、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback には進まない。

`alloc/gui/render2d/row_tile_rle_count` の row tile RLE count boundary は、F5cd の drain が返す slice-local `emitted_run_count` を future encoded RLE transport の exact capacity evidence として累積する。`GuiRgba8888RowTileRleCountOwner` は `GuiRgba8888RowTileRleCursorOwner` と `accumulated_run_count` を保持し、`gui_rgba8888_row_tile_rle_count_step_budget` は lower `gui_rgba8888_row_tile_rle_drain_budget` だけに委譲する。`GuiRgba8888RowTileRleCountStepStatus` は `Pending` / `Completed` の Copy enum であり、`GuiRgba8888RowTileRleCountOwner`、`GuiRgba8888RowTileRleCountStep`、`GuiRgba8888RowTileRleCountError` は owner-bearing value なので Clone / Copy を実装しない。`count_start` は Ready cursor だけを受け入れ、Complete cursor は過去に消費された run count evidence を持たないため `InitialCursorComplete` で拒否する。lower drain error は `DrainFailed %GuiRgba8888RowTileRleDrainErrorKind` に包み、`accumulated_run_count + emitted_run_count` の overflow は `AccumulatedRunCountOverflow` とする。overflow では cursor が既に進んでいる可能性があるため fake continuation owner を返さず、error に recoverable cursor と prior `accumulated_run_count` を保持する。この layer は encoded RLE buffer、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。

`alloc/gui/render2d/row_tile_rle_count_completed` の row tile RLE completed count boundary は、F5ce の count owner を formal transport allocation 前の completed evidence へ昇格する。`GuiRgba8888RowTileRleCountCompletedOwner` は `GuiRgba8888RowTileRleCountOwner` と `total_run_count` を保持し、`gui_rgba8888_row_tile_rle_count_completed_prepare` は cursor status を先に検査する。status error は `CursorInvalid %GuiRgba8888RowTileRleStepErrorKind`、Ready cursor は `CountNotCompleted`、Complete cursor で total run count が 0 以下なら `TotalRunCountInvalid` を返す。error は original count owner を保持し、caller が recover / free を選ぶ。completed module は count owner 内部に直接依存せず、`gui_rgba8888_row_tile_rle_count_owner_cursor_status` などの borrowed helper を通す。この layer は RLE 再走査、drain、payload byte read、encoded RLE buffer、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。

`alloc/gui/render2d/row_tile_rle_encode_seed` の row tile RLE encode seed boundary は、F5cf の completed count evidence を formal encoded transport 前の payload seed へ変換する。`GuiRgba8888RowTileRleEncodeSeedOwner` は `GuiRgba8888RowTilePayloadOwner` と exact `total_run_count` を保持し、`gui_rgba8888_row_tile_rle_encode_seed_prepare` は total run count が 0 以下なら `TotalRunCountInvalid` として original completed owner を保持する owner-bearing error を返す。成功時は `completed -> count -> cursor -> payload` の順に owner を閉じ、payload seed と total run count だけを残す。この layer は cursor restart、RLE 再走査、drain、payload byte read、encoded RLE buffer、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。cursor restart error、encoded buffer allocation、tile transport ABI は後続 phase の owner boundary として定義する。

`alloc/gui/render2d/row_tile_rle_encode_cursor` の row tile RLE encode cursor boundary は、F5cg の payload seed を formal encoded writer 前の ready cursor owner へ変換する。`GuiRgba8888RowTileRleEncodeCursorOwner` は `GuiRgba8888RowTileRleCursorOwner` と exact `total_run_count` を保持し、`gui_rgba8888_row_tile_rle_encode_cursor_start` は seed の total count を読んでから payload owner を finish し、`gui_rgba8888_row_tile_rle_cursor_start` を 1 回だけ呼ぶ。F5cg で total count は検査済みなので、この boundary は invalid count branch を追加しない。restart failure は `CursorStartFailed %GuiRgba8888RowTileRleStartErrorKind` とし、lower `GuiRgba8888RowTileRleStartError` と total count を owner-bearing error に保持する。`cursor_start` は正で RGBA8888 に整列した payload を next pixel index 0 の cursor にするため、成功結果を ready cursor として扱い、この layer では `cursor_status` を呼ばない。この layer は RLE 再走査、drain、`cursor_next_run`、payload byte read、encoded RLE buffer、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。

`alloc/gui/render2d/row_tile_rle_writer_plan` の row tile RLE writer plan boundary は、F5ch の ready cursor owner を formal encoded writer 前の capacity plan owner へ変換する。`GuiRgba8888RowTileRleWriterPlanOwner` は `GuiRgba8888RowTileRleCursorOwner`、exact `total_run_count`、exact `encoded_byte_count` を保持する。encoded RLE transport の 1 run は `pixel_offset i32`、`pixel_count i32`、`Rgba8888` 4 bytes の固定 12 bytes とし、`encoded_byte_count = total_run_count * 12` を checked multiplication で検査する。capacity boundary として `total_run_count > 0` を再検査し、0 以下は `TotalRunCountInvalid`、overflow は `EncodedByteCountOverflow` として original `GuiRgba8888RowTileRleEncodeCursorOwner` を保持する owner-bearing error を返す。success path だけが ready owner を finish して cursor owner を writer plan owner へ移す。この layer は cursor status 再検査、RLE 再走査、drain、`cursor_next_run`、payload byte read、encoded buffer allocation、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。

`alloc/gui/render2d/row_tile_rle_storage` の row tile RLE encoded storage boundary は、F5ci の writer plan owner を future encoded writer 用の owned byte storage へ変換する。`GuiRgba8888RowTileRleStorageOwner` は `GuiRgba8888RowTileRleCursorOwner`、exact `total_run_count`、exact `encoded_byte_count`、`RegionToken u8` storage を保持する。F5cj の `storage_prepare` は allocation / reservation only boundary であり、RLE run writer ではない。`GuiRgba8888RowTileRleStoragePrepareErrorKind` は `EncodedByteCountInvalid`、`TotalRunCountInvalid`、`EncodedByteCountOverflow`、`EncodedByteCountMismatch`、`AllocationFailed` を持ち、prepare の全 failure path は original `GuiRgba8888RowTileRleWriterPlanOwner` を保持する owner-bearing error を返す。prepare は encoded byte count 正値、total run count 正値、`total_run_count * 12` の checked recompute、stored byte count との一致、exact byte allocation の順に進み、allocation が成功してからだけ writer plan owner を finish する。この prepare layer は `cursor_next_run`、drain、payload byte read、encoded byte write、`Vec`、raw storage accessor、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。

F5ck の run writer cursor boundary は、`GuiRgba8888RowTileRleStorageOwner` を `GuiRgba8888RowTileRleWriteCursorOwner` へ変換し、1 step ごとに固定 12 byte record を書く。record は `pixel_offset i32 little-endian`、`pixel_count i32 little-endian`、`Rgba8888 r,g,b,a` である。writer は consuming `cursor_next_run` を使わず、borrowed `gui_rgba8888_row_tile_rle_cursor_peek_run` で run metadata を得て、12 byte write がすべて成功してから consuming `gui_rgba8888_row_tile_rle_cursor_advance_by_run` で cursor を進める。store / projection / advance failure は owner-bearing error で unchanged `written_run_count` / `written_byte_count` を返す。partial bytes が target slot に存在しても、その slot は uncommitted であり public reader はない。completion は `written_run_count == total_run_count` と lower cursor `Complete` の両方を要求し、ready cursor なら invariant error とする。この writer boundary は payload byte reader、encoded byte reader、host present、video memory host call、Canvas / DOM / minifb、platform surface、fallback、silent no-op には進まない。

`alloc/gui/render2d/row_tile_rle_encoded` の row tile RLE sealed encoded owner boundary は、F5ck の writer cursor が complete した storage だけを formal tile / bitmap transport 前の sealed owner として扱う。`GuiRgba8888RowTileRleEncodedOwner` は lower cursor、total run count、encoded byte count、private storage を保持する。seal は encoded byte count、total run count、`total_run_count * 12`、written run count range、written byte count range、`written_run_count * 12 == written_byte_count`、`written_run_count == total_run_count`、`written_byte_count == encoded_byte_count`、lower cursor `Complete` の順に検査する。未完了 writer は `WriterNotComplete`、lower cursor がまだ `Ready` なら `CursorNotComplete` で拒否し、すべての failure は original write cursor owner を保持する。sealed owner は byte reader、storage pointer accessor、host present、video memory host call、platform API、fallback を提供しない。

`alloc/gui/render2d/row_tile_rle_packet` の row tile RLE packet owner boundary は、F5cl の sealed encoded owner を formal tile / bitmap transport 前の validated descriptor owner へ変換する。`GuiRgba8888RowTileRlePacketOwner` は `GuiRgba8888RowTileRleEncodedOwner` と `GuiRgba8888RowTileRlePacketDescriptor` を同じ owner boundary に束ねる。descriptor は frame id、batch index、tile index、plan row start、plan row count、tile row start、tile row count、width、height、stride bytes、tile rows、tile count、pixel count、total run count、encoded byte count を持つ Copy metadata である。`plan_row_count` は後続の std layer row tile RLE present-frame owner が tile count を再導出するための authority であり、tile 自身の `row_count` と混同してはならない。prepare は encoded count、`total_run_count * 12`、cursor completion、payload descriptor authority、`pixel_count * 4 == descriptor_byte_count`、`width * 4 == stride_bytes`、`row_count * stride_bytes == descriptor_byte_count`、row extent、derived tile count、tile index range を checked arithmetic で検査する。payload descriptor authority failure は `PayloadDescriptorInvalid` として lower authority error を包む。failure は original sealed encoded owner を owner-bearing error に保持し、success path だけが sealed owner を packet owner へ move する。この layer は byte reader、raw storage、host present、video memory host call、platform API、fallback、silent no-op を提供しない。

`std/gui/tile_present` の std layer row tile RLE present-frame owner は、packet owner と `SurfaceId` / `FrameId` を同じ owner boundary に束ねる。`GuiRgba8888RowTileRlePresentDescriptor` は `surface`、`frame`、`packet` descriptor copy を持つ Copy metadata であり、`GuiRgba8888RowTileRlePresentFrameOwner` が actual `GuiRgba8888RowTileRlePacketOwner` を保持する。prepare は `SurfaceId` / `FrameId` raw value、packet frame id と frame id の一致、positive geometry、plan row extent、tile row extent、`width * 4 == stride_bytes`、`plan_row_count` と `tile_rows` から再導出した tile count、tile index range、`row_count * width == pixel_count`、`total_run_count * 12 == encoded_byte_count` を checked arithmetic で再検査する。failure は packet owner を `GuiRgba8888RowTileRlePresentFramePrepareError` に保持し、success path だけが packet owner を present-frame owner へ move する。この phase は `GuiSurfacePresentCommand` を拡張せず、host import、byte reader、raw storage、video memory host call、platform API、fallback、silent no-op を提供しない。Web / native / headless presenter はこの owner を消費する後続 phase で定義する。

`alloc/gui/render2d/row_tile_rle_packet_record` の row tile RLE packet typed record reader は、F5cn の後続で presenter が必要とする最小の typed read boundary である。`GuiRgba8888RowTileRlePacketRecordReadErrorKind` は total run count、encoded byte count、record index、record byte offset、raw projection / load、decoded i32、channel、run extent の失敗を enum として分ける。この module だけが quarantined typed record reader として private `RegionToken u8` を借用し、12 byte record を `GuiRgba8888RowTileRleRun` に戻す。public API は `gui_rgba8888_row_tile_rle_packet_record_at &packet record_index` だけを typed run reader として公開し、raw pointer、byte slice、storage accessor は公開しない。`row_tile_rle_packet` と `row_tile_rle_encoded` の no-reader contract は維持され、例外はこの typed record reader module に閉じる。

`std/gui/tile_present_run_cursor` の std layer row tile RLE present run cursor は、`GuiRgba8888RowTileRlePresentRunCursorOwner` が `GuiRgba8888RowTileRlePresentFrameOwner`、`next_record_index`、`total_run_count` を保持する presenter-neutral owner boundary である。start は present descriptor から `total_run_count * 12 == encoded_byte_count` を再検査し、failure では original present owner を `GuiRgba8888RowTileRlePresentRunCursorStartError` に保持する。step は `record_index == total_run_count` を explicit `Completed` として返し、`record_index > total_run_count` は `RecordIndexPastEnd` owner-bearing error にする。record read failure は `PacketRecordReadFailed %GuiRgba8888RowTileRlePacketRecordReadErrorKind` に包む。この cursor は host import、raw memory、video memory host call、platform API、fallback、silent no-op を提供しない。Web / native / headless の host import はこの cursor を消費する後続 phase とする。

`std/gui/tile_present_command_cursor` の std layer row tile RLE present command cursor は、F5co の run cursor を presenter-facing frame command stream へ昇格する。`GuiRgba8888RowTileRlePresentCommandCursorOwner` は lower run cursor owner、present descriptor copy、phase を保持し、owner-bearing value なので Clone / Copy を実装しない。command は `GuiRgba8888RowTileRlePresentCommand::BeginFrame`、`Run`、`GuiRgba8888RowTileRlePresentCommand::EndFrame` であり、public step は one typed output per public step を守る。`BeginPending` は BeginFrame、`RunPending` の lower `RunReady` は Run、lower `Completed` は同じ public step で EndFrame を返して phase を Completed に進める。Completed phase は terminal Completed を返す。lower start / step failure は F5co error kind と category を包み、present owner または command cursor owner を失わない。この command cursor does not bypass F5co。packet record reader、packet storage、`RegionToken`、`MemPtr`、host import、video memory host call、platform API、fallback、silent no-op には進まない。actual Web / native / bare / headless presenter はこの command stream を消費する後続 phase で定義する。

## Text Model

Text は platform 差が大きいため 3 層に分ける。

```text
core/gui
    TextRunId
    TextPaint
    FontId
    FontMetrics
    TextBounds
    TextMeasureRequest
    TextMeasureResult
    TextMeasurer

alloc/gui/text
    TextBuffer
    TextLayout
    LineBreak
    TextStyle
    CachedTextLayout

std/gui/text_measure
    HostFont
    FontLoadEffect

std/gui/ime
    ImeBridge
    ImeRequest
    ImeState
```

Layout は text measurement に依存するため、`LayoutContext` に `TextMeasurer` contract を注入する。この contract は `core/gui` 側の data / trait として定義し、`alloc/gui/layout` が `std/gui` に依存しないようにする。

Formal GUI text measurement は host browser / OS API の測定結果を authority にしない。正式経路は `FontResourceRequest -> GuiFontFace -> ScaledFont -> ShapedRun -> RenderedTextMetrics` であり、font file parsing、metrics、shaping、glyph rasterization から layout 用の寸法を得る。`std/gui/text_measure` の host wrapper は legacy smoke、terminal cell measurement、mock test、移行期 compatibility のための境界として扱い、formal GUI renderer の寸法決定には使わない。

`alloc/gui/layout` は browser global、OS font API、terminal escape sequence を直接呼ばない。font loading、IME、complex shaping などの side effect は `std/gui` または `platforms/gui/*` に閉じ込める。ただし font metrics / shaping / rasterization の authority は `alloc/gui/font` と `alloc/gui/text` の data contract に置く。

TUI text measurement は terminal cell width を返す `TextMeasurer` 実装として扱う。現行の `platforms/wasix/tui/text/width.nepl` にある表示幅近似は、将来 `platforms/gui/terminal` 側の measurer へ移す。

Outline font rendering、font resource loading、ruby / furigana、Japanese vertical writing、math inline layout との接続は `doc/neplg2/gui_font_rendering_design.md` で扱う。Formal GUI text renderer は `fonts/HackGenConsoleNF-Regular.ttf` を resource path として読み、font metrics、shaped run、positioned glyph、glyph rasterization を同じ font face から導く。Browser / OS text measurement は authority にせず、unsupported feature は typed error として扱う。

## Mobile Lifecycle Contract

Mobile backend は native desktop の一種として扱わない。少なくとも次の状態遷移を event contract として表す。

```text
MobileLifecycleState:
    NotStarted
    ForegroundActive
    ForegroundInactive
    Background
    Suspended
    SurfaceLost
```

`LifecycleEvent` は `Started`、`Suspended`、`Resumed`、`Backgrounded`、`Foregrounded`、`LowMemory` に加えて、surface recreation と IME state の失効を表せる extension point を持つ。surface が失われた後の draw / present は panic ではなく `GuiError::SurfaceUnavailable` を返す。runtime は `RequestRedraw`、timer、IME composition、accessibility focus を lifecycle state と照合して処理する。

## Accessibility Model

Visual tree と semantic tree は分ける。

```text
Visual tree:
    ViewTree
    LayoutTree
    DrawCommand

Semantic tree:
    AccessibilityTree
    SemanticNode
    Role
    Label
    State
    Action
```

Accessibility は drawing の副産物にしない。`DrawCommand` から button の意味は復元できないため、widget layer が semantic tree を生成する。

## Host ABI

Host boundary は WIT-like な schema として設計する。ただし、最初の Web Playground backend は TypeScript / JavaScript shim で同じ ABI shape を実装してよい。

```wit
package neknaj:gui;

interface surface {
    type window-id = u32;
    type surface-id = u32;

    request-redraw: func(window: window-id) -> result<_, gui-error>;
    begin-frame: func(window: window-id) -> result<frame-id, gui-error>;
    push-command: func(frame: frame-id, command: draw-command) -> result<_, gui-error>;
    end-frame: func(frame: frame-id) -> result<_, gui-error>;
    present-commands: func(window: window-id, commands: list<draw-command>) -> result<_, gui-error>;
}

interface events {
    poll-event: func() -> option<gui-event>;
}

interface capabilities {
    get-capabilities: func() -> gui-capabilities;
}

interface text {
    measure-text: func(run: text-run) -> result<text-metrics, text-error>;
}

interface ime {
    set-ime-state: func(window: window-id, state: ime-state) -> result<_, gui-error>;
}

interface accessibility {
    update-tree: func(window: window-id, tree: accessibility-tree) -> result<_, gui-error>;
}

world gui-app {
    import surface;
    import events;
    import capabilities;
    import text;
    import ime;
    import accessibility;

    export init: func();
    export update: func(event: gui-event) -> result<update-result, gui-error>;
    export render-frame: func(window: window-id) -> result<_, gui-error>;
}
```

`present-commands` は `Vec DrawCommand` を使える backend 向けの convenience path である。embedded / no_alloc target は `begin-frame`、`push-command`、`end-frame` の streaming path を使い、command list allocation を要求されない。

TUI terminal backend は同じ world の surface implementation として `TextGrid` surface を提供する。Raw mode、TTY state、ANSI cursor movement、alternate screen は backend detail である。

## Current Display Smoke Backends

2026-06-02 checkpoint では、正式な `neknaj:gui` host ABI へ到達する前の表示 smoke backend として、次を実装している。

```text
web/src/gui-preview
    Web Playground の floating GUI window layer、typed command DTO、runtime bridge、host frame decoder、NEPL stdout legacy smoke transport、bitmap video memory presenter

nepl-gui-native
    pure framebuffer renderer と resizable minifb window smoke backend
```

`examples/gui_mandelbrot.nepl`、`examples/gui_life.nepl`、`examples/gui_counter.nepl`、`examples/gui_calculator.nepl`、`examples/gui_scientific_calculator.nepl`、`examples/gui_paint.nepl`、`examples/gui_breakout.nepl` は、NEPL 側で application model、typed event update、render command frame を作る。現 checkpoint では `platforms/gui/web/stdout_protocol.nepl` を通して Web Playground host へ frame stream を出すが、これは formal host surface ABI へ到達する前の legacy smoke transport であり、same app code contract の正式 path ではない。stdout helper は platform backend detail として `GuiWebTextAlign` enum と `Result unit GuiError` を返す checked API を持ち、invalid geometry を panic や silent no-op にしない。text label を持つ button は `GuiWebButtonConfig` と `gui_web_stdout_button` で `fill_rect`、`text_run`、`action_rect` の順序を一箇所に集約し、example 側は app 固有の `ActionId`、label、色、geometry だけを渡す。Mandelbrot は Preview / HD / Detail action で sample grid と logical surface size を切り替え、HD / Detail mode は 1280x720 logical frame の raster 部分を `rgba-row` payload で描画する。さらに `--video-memory-once` は 32x18 の有限 surface を formal Web video memory row host import へ出す opt-in 検査 path であり、stdout protocol、command frame、TS simulation へ fallback しない。`--video-memory-progressive-once` は同じ preview surface を row batch ごとの rect dirty publish で更新する有限検査 path であり、`--video-memory-progressive-test` は CI から同じ implementation を呼ぶ alias である。`--video-memory-progressive-loop-test` は `GuiEvent::Timer` の matching timer id だけで row batch を 1 つ進める有限検査 path である。Life は next step、animate toggle、cell pixel size、HD view を扱う。Counter、四則電卓、関数電卓は button の `ActionId` を update で解釈する。Paint は button action だけでなく full `GuiWebEvent` の pointer position を model update に使う。Breakout は button action と timer tick で model を進める。各 button 領域は `NEPLG2_GUI_ACTION_RECT` で `ActionId` hit target として出力される。Web Playground の Run 経路では、この NEPL stdout frame stream が floating GUI window を開く。TypeScript はこれらの example を simulation せず、stdout frame decode と backend presentation だけを担当する。

Native smoke backend は macOS AppKit、Windows Win32、Linux Wayland / X11 の window lifecycle 調査を踏まえ、OS window manager が与える resize / close / event pump を受ける形へ寄せている。`WindowOptions.resize = true`、`ScaleMode::UpperLeft`、`set_target_fps 60`、current window size 監視、resize 後の exact-size RGB0 buffer redraw、letterbox-aware hit test、`NativeSurfaceState::Unavailable` を使い、固定 size framebuffer 前提の click mapping と OS scaling による stretched presentation を避ける。調査内容と native backend contract は `doc/neplg2/gui_native_platform_behavior.md` に分けて記録する。これはまだ正式な `std/gui::GuiHost.present` 実装ではなく、minifb と native handle は標準 API の public type へ出さない。

Web Playground の表示 smoke は editor の panel layout の上に独立した DOM layer を置き、`GuiFloatingWindowManager` が minimize、maximize / restore、drag move、edge / corner resize、dock restore を扱う。これは native window と同等の基本操作を browser 上で検査するための backend UI であり、標準 API の window model ではない。`GuiFloatingWindowManager` の move state、source、window mode、dock state は discriminated union で表す。`minimized` mode は previous mode を保持するため、maximized window を minimize / restore しても original restore rect は失われない。top bar の `GUI` button と editor header の `G` button は user-facing 導線から外し、NEPL execution が stdout protocol を出した時だけ window を開く。host event / queue status は GUI window content に挟まず、折りたたみ式の `GuiWindowDebugPanel` へ分離する。通常の window body は host frame canvas だけを含む。host frame の title は window titlebar の表示責務であり、canvas renderer は同じ title を content 内へ再描画しない。debug panel は通常 window より低い補助 z-layer に置き、collapsed 時は toggle 以外の pointer capture を持たず、`aria-live` を off にして queue 更新を main GUI live region の読み上げ対象にしない。`window-manager.ts` と `panel.ts` が `null` / `undefined` / non-null assertion に頼らないこと、かつ debug/status DOM を window content に戻さないことを source policy regression で固定する。

Host frame の描画 data は `web/src/gui-preview/commands.ts` の `fill-rect` / `rgba-row` / `text-run` command union、`rgba8888` 相当の color struct、command frame、`action-rect` input target で表す。`rgba-row` は legacy smoke transport で HD raster の row payload を bounded command count で運ぶための現 checkpoint の command であり、Canvas や DOM 型を public DTO に入れない。旧 `renderer.ts` による Mandelbrot / Life / Counter の TS scene simulation は削除済みであり、Run 経路の GUI 表示は現 checkpoint では NEPL stdout protocol によって駆動する。`panel.ts` は host-frame surface として、NEPL 実行が出した command frame だけを描画する。`host-bridge.ts` は unknown input を `GuiWebHostResult` の `ok` / `err` union で decode し、invalid frame、invalid command、invalid rect、invalid color、invalid text、invalid input target、unsupported command を typed error として返す。`runtime-bridge.ts` は presenter missing、invalid install target、invalid frame state、host decode error、invalid video memory frame、video memory open / present failure を `GuiWebRuntimeResult` で返し、global `neplGuiHost.presentCommands`、`beginFrame` / `pushCommand` / `endFrame` streaming path、`presentVideoMemory`、`closeWindow` を floating window presenter へ接続する。`presentVideoMemory` は `windowId`、`title`、`SharedArrayBuffer` を持つ video memory frame だけを受け付け、`ArrayBuffer`、typed array、numeric id、string handle、transferable object を typed error として拒否する。stdout protocol や command frame path への自動迂回は持たない。`panel.ts` は `none` / `command-frame` / `video-memory` の state を分け、同じ `SharedArrayBuffer` identity の opened surface を再利用する。video memory presentation は `ImageData` と `putImageData` だけで行い、surface size と drawable size が一致しない場合も CSS scale、Canvas transform、`drawImage` による伸縮を行わず、top-left に 1:1 で提示する。window resize は `WindowEventKind::Resized` として application に渡し、application 側が新しい pixel buffer size を決める。`web/src/runtime/worker.ts` は Web-only host import module `nepl_gui_web` に `video_memory_create_surface`、`video_memory_acquire_write_slot`、`video_memory_write_slot_bytes`、`video_memory_write_rgba8888_row`、`video_memory_discard_write_slot`、`video_memory_publish_slot`、`video_memory_present_surface`、`video_memory_close_surface` を持つ。`surface_id` と `frame_id` は worker-local opaque positive integer であり、`SharedArrayBuffer`、DOM handle、Canvas handle、ArrayBuffer transfer object、JS object handle、string handle は NEPL/Wasm へ渡さない。`video_memory_present_surface` は typed `gui_video_memory_present` worker message と ack `SharedArrayBuffer` で main thread presenter の実結果を待ってから status を返す。`platforms/gui/web/surface.nepl` は raw negative status を module private helper で `Result` / `GuiError` へ写す。`web/src/gui-preview/stdout-protocol.ts` は stdout fd=1 の line protocol だけを typed command frame と typed animation timer request へ decode し、`NEPLG2_GUI_RGBA_ROW` を `rgba-row` command、`NEPLG2_GUI_ACTION_RECT` を frame-local input target として読む。chunk split、invalid frame state、invalid color、invalid rgba row、invalid action rect、invalid animation timer を discriminated error で表す。frame 内 parse error は partial frame を破棄し、壊れた frame を present しない。Web checkpoint の presentation hot path は video memory surface と `putImageData` only presenter に寄せる。DrawCommand stream / tile / bitmap / row / RLE を直接渡す正式 host import ABI は残件である。`web/src/gui-preview/input-bridge.ts` は DOM / Canvas に依存しない typed queue として `GuiWebInputEvent::action`、`GuiWebInputEvent::pointer`、`GuiWebInputEvent::keyboard`、`GuiWebInputEvent::text-input`、`GuiWebInputEvent::window`、`GuiWebInputEvent::timer` を保持し、listener へも typed event だけを通知する。DOM `KeyboardEvent` の key string は `panel.ts` で std key code contract と Unicode scalar value へ正規化され、application code へ DOM string は渡らない。Space は keyboard event と text input event の両方として queue し、composition 中、Meta shortcut、multi-scalar text は現 checkpoint では queue しない。pointer down / up は DOM `button` から changed button を正規化し、pointer move は DOM `buttons` bitmask から現在押下中の button state を正規化する。これにより paint のような app は hover move と primary drag を `PointerButton` で区別できる。floating host frame window は resize 時に `WindowEventKind::Resized` を worker queue へ渡す。stdout animation timer request は window id と timer id を持ち、Shell が browser timer を管理して `TimerEvent` を worker queue へ渡す。close button は現 checkpoint では拒否可能 close request ではなく host lifecycle signal として扱い、window を削除した後で active worker を interrupt する。terminal stop / process finish は `neplGuiHost.closeWindow` presenter path で host-frame window を削除し、active timer も停止する。`web/src/gui-preview/shared-event-queue.ts` は SharedArrayBuffer の full event queue と legacy action projection queue を分ける。full event queue は action / pointer / keyboard / text input / window / timer の kind、window id、action id、pointer milli-position、pointer kind、pointer id、button、keyboard kind、key code、modifier bit、text scalar value、window kind、window size、timer id、timer tick を worker へ渡す。record slot length は 8 のまま固定し、event kind ごとに payload slot を再利用する。action-only queue は `poll_action_id` / `wait_action_id` 互換 path が pointer / keyboard / text input / window / timer event を consume しないための projection である。queue は bounded だが、producer は `event-queue-full` / `action-queue-full` を返さない。容量に達した場合は古い unread record を明示的に押し出し、新しい input を受け入れる。full event poll と legacy action-only poll を同じ app run で混用すると action projection queue に残る event があるため、互換 path は action-only app 用である。`web/src/runtime/worker.ts` は Web-only host import module `nepl_gui_web` の `poll_action_id` / `wait_action_id` に加えて、`poll_event_kind` / `wait_event_kind` と last-event field accessors を提供する。`platforms/gui/web/input.nepl` は raw sentinel を public API に出さず、`gui_web_wait_action_result` で unsupported host を `GuiError::Unsupported`、timeout を `Option::None`、action を `Option::Some ActionId` として正規化する。さらに `gui_web_wait_event_result` / `gui_web_poll_event_result` は `Result Option GuiWebEvent GuiError` を返し、現 checkpoint の action record を `GuiEvent::Action`、pointer down / move / up / cancel record を `GuiEvent::Pointer`、keyboard record を `GuiEvent::Keyboard`、text scalar record を `GuiEvent::TextInput`、window resize record を `GuiEvent::Window`、timer record を `GuiEvent::Timer` として保持する。text scalar は `char_from_i32_result` で検証し、surrogate や範囲外を `GuiError::InvalidCommand` にする。window kind と size も raw value を `WindowEventKind` と `GuiSize` へ正規化し、未知 kind や 0 以下の size は `GuiError::InvalidCommand` にする。timer id と tick は正の id と 0 以上の tick として検証し、不正な record は `GuiError::InvalidCommand` にする。`web/src/terminal/shell.ts` は active run が present した window id だけを queue 対象にし、stale window の input event が別 app に混入しないようにする。空 poll の busy spin を避けるため、interactive app は `wait_action_id` または `wait_event_kind` の Atomics wait path を使う。

`video_memory_discard_write_slot` は未公開 write frame の所有権を `Writing -> Free` に戻すためだけの Web backend import である。成功時は dirty metadata を消し、published / presented epoch は進めない。frame が存在しない、既に publish / discard 済み、または resize generation が古い場合は typed status を返し、stdout protocol、command frame、別 surface への fallback は行わない。

`video_memory_write_rgba8888_row` は formal row payload の最小 writer である。`write_slot_bytes` と違い app は byte offset を渡さず、origin、pixel width、source pointer だけを渡す。`width <= 0`、surface 範囲外、`width * 4` と一致しない source byte length は typed error で拒否し、clamp / truncate / zero-byte no-op は行わない。row write は pixel plane だけを更新し、dirty metadata、slot epoch、published epoch、presented epoch は publish path へ残す。

`examples/gui_video_memory_rows.nepl` は focused NEPL example として、`ByteBuilder` / `ByteBuf` owner で row bytes を構築し、borrowed `MemPtr u8` を `gui_web_video_memory_write_rgba8888_row` へ渡す。これは stdout `rgba-row` を使わない formal row host import の source contract を示すための例である。現行 CI の `run_test.js` は default `nepl_gui_web` video memory host import を unsupported stub として持つため、通常 doctest では host capability missing を `Result` に写す境界を壊さない。この example の happy path は fake positive `nepl_gui_web` host import harness が通常 path の NEPL/Wasm 実行として検査する。Mandelbrot も `--video-memory-once` で有限 preview model の RGBA8888 row payload を同じ host import path へ出す。Mandelbrot の legacy stdout interactive path は `WindowEventKind::Resized` を application update で受け、drawable pixel size から responsive model を作れる。`--video-memory-resize-once` は finite formal video memory resize/recreate checkpoint であり、typed resize event の後に old surface を close して new surface を create / render / present / close する。`--video-memory-loop` は formal video memory surface を保持し、typed event を待ち続ける loop checkpoint である。resize event では old surface close 成功後だけ new surface を create / render / present し、focus / unfocus / non-window event では current surface を維持し、close request では current surface を close して正常終了する。`--video-memory-loop-test` の wait count は CI の停止条件であり scheduler policy ではない。Mandelbrot の progressive video memory path は row batch ごとに dirty rect を publish する finite checkpoint であり、batch end は sample height で clamp する。Timer event driven progressive loop checkpoint は matching timer id の event だけを batch 進行条件にし、timer id 不一致、empty event、focus event では batch を進めない。FHD 60 fps readiness、formal tiled transport、formal timer registration ABI、real scheduler policy は後続 slice である。

pointer move と window state coalescing は順序保存 contract として扱う。`panel.ts` は pointermove を `requestAnimationFrame` 単位で最新の move へまとめ、`input-bridge.ts` の stored queue は直前に保持された同じ window id、pointer id、button の `PointerEventKind::Move` だけを最新座標へ置き換える。`shared-event-queue.ts` は write tail 直前の unread slot が同一 pointer move または同一 window kind の state record である場合だけ最新値へ置換でき、queue 全体の未読 slot は走査しない。timer tick も同じく直前の unread slot が同一 window id / timer id の `TimerEvent` である場合だけ最新 tick へ置換でき、action や pointer / keyboard / text / window event をまたいで更新してはいけない。`Down` / `Up` / `Cancel`、action、keyboard、text input、close lifecycle signal をまたいで古い move / window / timer record を更新してはいけない。

ただし、この checkpoint はまだ NEPLg2 program から `DrawCommand` stream や tile / bitmap / row / RLE payload を JS / native host へ直接 export する全体正式 ABI ではない。現在の Web example は legacy stdout protocol で command frame を出す path を残すが、これは正式 backend へ到達できない時の代替実行経路ではない。Web video memory surface については `nepl_gui_web` host import で create / write / discard / publish / present できる初期経路を持つ。`CanvasRenderingContext2D`、DOM element、stdout transport、SharedArrayBuffer queue、minifb は backend implementation detail であり、`core/gui`、`alloc/gui`、`std/gui` の public type には入れない。HD example の現状は high-resolution logical surface の raster 部分を legacy transport の `rgba-row` payload で描く段階であり、1280x720 の全 pixel を個別 command として stdout へ流す契約ではない。Mandelbrot の legacy stdout interactive path は host-frame resize event を `GuiWebEvent` として読み、1 drawable pixel per sample の model を作れる。Mandelbrot の finite video memory resize path は old surface close と resized surface recreate を検査する。Mandelbrot の formal video memory event loop path は open surface を維持し、typed window resize event で old surface を close して resized surface を recreate する。Mandelbrot の progressive video memory path は row batch ごとの dirty rect publish と timer event driven batch progression を検査する。次の段階で NEPL/Wasm が生成した command stream から formal host import ABI を呼び、native 側も正式 `GuiHost.present` へ寄せる。Web 側 input は action event、pointer down / move / up / cancel、keyboard down / up、single-scalar text input、host-frame window resized、timer tick を full event queue と `GuiEvent` wrapper へ接続済みである。close button と terminal stop は現 checkpoint では host lifecycle cleanup として実装済みであり、拒否可能 close request は formal host ABI 後に扱う。残件は IME composition / multi-scalar text、window focus / unfocus の発火 policy、rejectable close request、lifecycle variant の poll ABI、run/session/window/timer id の正式化、formal timer registration ABI、formal tiled rendering、real scheduler policy、Life の任意解像度 board storage、Paint の persistent stroke / bitmap storage、DrawCommand / tile presentation の formal host import ABI である。

## Public Module Contract

最小契約は次である。

```text
core/gui:
    GuiPoint
    GuiSize
    GuiRect
    GuiInsets
    GuiScaleFactor
    WidgetId
    ActionId
    BinaryColor
    Gray8
    Rgb565
    Rgb888
    Rgba8888
    Pixel
    DrawTarget
    FlushTarget
    DirtyRegion
    DirtyRegionSet
    DrawCommand
    RenderTarget
    GuiEvent
    GuiCapabilities
    GuiError

alloc/gui:
    ViewTree
    WidgetId semantics and re-export
    ActionId semantics and re-export
    LayoutTree
    Routing
    Theme
    TextBuffer
    TextLayout
    AccessibilityTree
    App Model
    Update Model
    GuiEffect
    MockGuiHost
    SnapshotTest

std/gui:
    GuiHost
    WindowId
    SurfaceId
    Runtime
    FocusKeyMap
    KeyboardEvent to FocusRouteCommand mapping
    HostTextMeasurer
    ResourceHandle
    Clipboard
    Timer
    ImeBridge
    ErrorDisplay

platforms/gui:
    web host
    native host
    mobile host
    embedded host
    terminal host
    terminal input normalization
```

## Current Implementation Status

契約と現状実装は分けて扱う。2026-06-02 時点の checkpoint は次である。

| Layer | Contract | Current implementation |
|---|---|---|
| `core/gui/geometry` | `GuiPoint`、`GuiSize`、`GuiRect`、`GuiInsets`、`GuiScaleFactor` | constructor / accessor / basic arithmetic を実装済み |
| `core/gui/color` | `BinaryColor`、`Gray8`、`Rgb565`、`Rgb888`、`Rgba8888`、`GuiColor` | constructor / accessor を実装済み |
| `core/gui/event` | `GuiEvent`、`ActionId`、`WidgetId`、pointer / keyboard / lifecycle data | initial data contract を実装済み |
| `core/gui/error` | enum-based GUI/TUI error data | `Unsupported`、`InvalidGeometry`、`ResourceExhausted`、`InvalidCommand` を実装済み。`ResourceMissing`、`TextMeasureFailed`、`FontError`、`SurfaceUnavailable` など formal font / mobile lifecycle extension error は未実装 |
| `core/gui/text_measure` | `TextMeasurer` contract、request/result、font id | borrowed `&Self` contract と fixed-cell `MockTextMeasurer` を実装済み。host wrapper は legacy / terminal / mock 境界として `std/gui` / platform 側で継続し、formal GUI font measurement は `GuiFontFace` based engine へ移す |
| `core/gui/draw_target` | `DrawTarget`、`FlushTarget`、pixel-level drawing | `MockDrawTarget` と O(1) contract test を実装済み。iterator stream と rasterizer は未実装 |
| `core/gui/render_target` | streaming `RenderTarget` | `MockRenderTarget` を実装済み。command list / typed rasterizer は未実装 |
| `core/gui/dirty_region` | no_alloc `DirtyRegion`、checked rect constructor、O(1) merge | `Empty` / `Rect` / `Full` と bounding rect merge を実装済み |
| `core/gui/dirty_region_set` | no_alloc fixed-capacity rect set | 最大 2 rect の `DirtyRegionSet`、overflow to Full、checked push を実装済み。generic capacity と backend damage compression は未実装 |
| `alloc/gui/app` | callback-free app model、`ViewNode`、`GuiEffect`、`Update` | leaf view、button config、redraw/title effect、bounded `GuiEffectBatch` を実装済み。将来の `Vec GuiEffect` へ置換する境界は `Update.effects` に固定 |
| `alloc/gui/layout` | `LayoutContext`、constraints、text measurement injection、measure/place result、arena layout connector、stack layout policy | `TextMeasurer` 注入、constraint validation、fixed text measure、place-at helper、`ViewTreeArena` を `LayoutTreeArena` へ変換する arena order の縦積み connector、parent-local sibling offset を使う vertical / horizontal stack layout policy を実装済み。stack は `StackLayoutPolicy` の axis / spacing / cross-axis alignment / overflow policy を `Result` で扱い、`Allow` は現状互換の配置、`Reject` は parent bounds 外の配置を `GuiError::InvalidGeometry` とする。途中失敗時の `LayoutTreeArena` owner は内部で解放する。flex / grid / scroll、text buffer と arena node の対応付けは未実装 |
| `alloc/gui/widget` | callback-free widget descriptor、action event、semantic lowering、measure bridge | button / label descriptor、`ActionId` event 生成、semantic node 生成、layout measure bridge、focusable accessor を実装済み |
| `alloc/gui/tree` | retained `ViewTree` / `LayoutTree`、allocator-backed arena、focus target query | root + 2 child の bounded tree、capacity error、first focusable id、focusable count に加え、parent index / depth を持つ `ViewTreeArena` / `LayoutTreeArena`、owner-recovery 付き arena child insertion、arena focus count / first focusable query を実装済み。arena を使った pointer routing、diff / invalidation、初期 linear layout connector、stack layout policy は接続済み |
| `alloc/gui/focus` | platform 非依存 focus order / next / previous traversal | bounded `ViewTree` から `FocusOrder` を作り、allocator-backed `ViewTreeArena` は unbounded `FocusOrder` へ落とさず直接走査して、current id から next / previous focus target を `Option WidgetId` で返す実装を追加済み。wrap policy と route command integration は未実装 |
| `alloc/gui/routing` | `LayoutTree` hit test、`WidgetId` lookup、widget action lowering、focus command routing | bounded root + 2 child の pointer action routing、`LayoutTreeArena` の末尾優先 hit test、`ViewTreeArena` の `WidgetId` lookup、arena pointer action lowering、`FocusRouteCommand` / `FocusRouteResult` を実装済み。pointer capture、gesture、stateful pointer routing は未実装 |
| `alloc/gui/diff` | retained tree diff / invalidation data contract | bounded `ViewTree` の slot diff、allocator-backed `ViewTreeArenaDiff`、`GuiInvalidation::Clean` / `Widget` / `Tree` への畳み込みを実装済み。terminal line diff、DOM patch、dirty rect compression は platform 側で未実装 |
| `alloc/gui/text` | platform 非依存 `TextBuffer` / checked edit storage / measured text layout data | `TextBufferId`、`TextBuffer`、checked insert / replace / delete を `Result TextBuffer GuiError` で実装済み。`TextLayout` は injected `TextMeasurer` だけを使って測定し、byte length、char count、fallback cell count、width / height / baseline、max width を保持する。`CachedTextLayout` は buffer id、run id、font id、max width、byte length、char count から deterministic cache key を作る。line break、text hash / revision based invalidation、complex shaping cache は未実装 |
| `alloc/gui/theme` | typed theme scheme / color role / metric role、fallible color/metric helper | `GuiColor` palette、`ThemeMetrics` validation、`Option FontId`、text-cell style helper を実装済み。full typography / component style は未実装 |
| `alloc/gui/accessibility` | semantic node / role / state / action tree | bounded semantic tree の初期 slice を実装済み。host accessibility bridge は `std/gui` / platform 側で継続 |
| `std/gui` | host/runtime/window/timer/text/IME/accessibility/error display/keymap contract | typed data contract、legacy / mock / terminal 用 core `TextMeasurer` host wrapper、`GuiEffectBatch -> GuiRuntimeCommandBatch` 解釈、capability unsupported error、`FocusKeyMap` による Tab / Shift+Tab / Enter / Space から `FocusRouteCommand` への変換、std navigation key code と modifier bit accessor を実装済み。formal GUI text measurement は `GuiFontFace` based engine へ移す。platform 実行は未実装。raw input normalization は `platforms/gui/*` 側で継続 |
| `platforms/gui/terminal` | terminal as `SurfaceKind::TextGrid` backend and terminal input normalization | `TerminalProfile` と core `TextCellRun` based frame、1 byte ASCII subset、`ESC [ Z`、`ESC [ A/B/C/D/H/F`、`ESC [ 1/3/4 ~`、`ESC [ 1 ; <modifier> A/B/C/D` から `TerminalInputEvents` への正規化を実装済み。custom capability と grid size は `Result` で検証し、TextGrid 以外や負 size を拒否する。ANSI / TTY present、Function key などの追加 CSI sequence、途中入力 buffering は未実装 |
| `web/src/gui-preview` | Web Playground display smoke backend | editor panel layout の上に floating GUI window layer を描画し、NEPL legacy stdout protocol で出力された Counter / Mandelbrot / Life / calculator / scientific calculator / paint / breakout frame と host-decoded frame を typed command DTO 経由で表示する。Web checkpoint の bitmap video memory path は `SharedArrayBuffer`、`ImageData`、`putImageData` only presenter で表示し、`nepl_gui_web` video memory host import で create / write / discard / publish / present できる。DrawCommand stream と tile / bitmap / RLE payload の formal presentation ABI は引き続き残件として扱う。old `renderer.ts` / `gui-preview` pane / TS example simulation は削除済みであり、window manager は host-frame source / move / window mode / dock 状態を union で表す。`commands.ts` は Canvas / DOM 型、`null | undefined`、optional metric field を持たず、`fill-rect` / `rgba-row` / `text-run` command union を持つ。`panel.ts` は `none` / `command-frame` / `video-memory` の state を分け、command-frame では `renderGuiPreviewFrameToCanvas`、video-memory では `presentNewestGuiVideoMemoryFrameToCanvas` だけを呼ぶ。同じ `SharedArrayBuffer` identity の opened surface は再利用し、surface size と drawable size が違っても CSS scale、Canvas transform、`drawImage` による伸縮はしない。`host-bridge.ts` は unknown input を `GuiWebHostResult` で decode し、Canvas / DOM 型や throw に依存しない。`runtime-bridge.ts` は global `neplGuiHost.presentCommands`、`beginFrame` / `pushCommand` / `endFrame` streaming path、`presentVideoMemory`、`closeWindow`、typed presenter registration、typed frame state error、typed video memory frame error、`takeInputEvents` / `resetInputEvents` を持ち、DOM / Canvas 型に依存しない。`presentVideoMemory` は `SharedArrayBuffer` だけを受け、`ArrayBuffer`、typed array、numeric id、string handle、transferable object を拒否する。stdout protocol や command frame path への自動迂回は持たない。`stdout-protocol.ts` は NEPL 実行 stdout fd=1 の line protocol を typed frame と animation timer request に変換し、`NEPLG2_GUI_RGBA_ROW` を row payload command、`NEPLG2_GUI_ACTION_RECT` を input target として扱い、frame 内 parse error では partial frame を破棄する。`input-bridge.ts` は action / pointer / keyboard / text-input / window / timer event を typed queue に保持し、`shared-event-queue.ts` は high-frequency pointer move / window state / adjacent timer tick を coalesce し、容量到達時は古い unread record を押し出して producer に overflow error を返さない。Shell は active-run window id で filter し、stdout timer request を active timer として管理する。Counter は legacy action projection path を維持し、Mandelbrot / Life / calculator / scientific calculator / paint / breakout は `gui_web_wait_event_result` で full event queue を読み、NEPL 側 update loop を継続する。Mandelbrot の HD / Detail mode は 1280x720 logical frame の raster 部分を legacy `rgba-row` payload として描画し、Life の HD mode は現 checkpoint では bounded sample rectangle stream を使う。Paint は pointer event を model に反映し、Breakout は `GuiEvent::Timer` で animation を進める。terminal stop / process finish は `closeWindow` で host-frame window を削除し、window close button は active worker を interrupt し、active timer は run lifecycle とともに停止する。`platforms/gui/web/input.nepl` は full event poll を `Result Option GuiWebEvent GuiError` として公開し、action、pointer down / move / up / cancel、keyboard down / up、single-scalar text input、host-frame window resized、timer tick を `GuiEvent` へ正規化する。formal host import ABI の代替経路としては扱わず、presentation formal ABI、tile / bitmap / RLE payload、IME composition、window focus / unfocus の発火 policy、rejectable close request、lifecycle event は未実装 |

この表にない Web / native / mobile / embedded backend、flex / grid / scroll layout policy、text buffer と arena node の対応付け、stateful pointer capture / gesture、Web / native / mobile raw keyboard normalization、terminal の Function key などの追加 ANSI / CSI sequence と途中入力 buffering、text line break / text hash based cache invalidation、resource loading、real host presentation、formal host import ABI 上の tile / bitmap / row / RLE transport、persistent paint canvas、arbitrary-size Life board は未実装である。

std layer row tile RLE present host-command record は F5cq の checkpoint である。`std/gui/tile_present_host_command` は F5cp の public step descriptor accessor と step result accessor だけを使い、`GuiRgba8888RowTileRlePresentHostCommandRecord` と `GuiRgba8888RowTileRlePresentHostCommandStepResult` を作る。record shape は `BeginFrame descriptor`、`RunRecord run_record`、`EndFrame descriptor` であり、run record は descriptor と run を保持し、does not flatten to kind plus optional run。これは enum / match による静的検査で不正状態を表現不能にするためである。この layer does not bypass F5cp。F5co run cursor、packet record reader、raw storage、host import、platform API、fallback には直接触れない。

std layer row tile RLE present run-span boundary は F5df の checkpoint である。`std/gui/tile_present_run_span` は F5cq の `GuiRgba8888RowTileRlePresentHostCommandRunRecord` を消費し、tile-local linear pixel offset を `GuiRgba8888RowTileRlePresentRunRowSpan` の stream へ分解する。row span は x、y、width、color だけを持ち、高さは accessor が常に 1 を返すため、platform rect や renderer rect へ依存しない。start は width、height、row_start、row_count、tile_rows、pixel_count、run offset、run count、run end を checked arithmetic と enum error で検査し、invalid cursor を作らない。step は run を `local_row = offset / width`、`x = offset % width`、`y = row_start + local_row` で 1 行以内に切り、remaining が 0 の場合は explicit Completed を返す。空 span、silent no-op、unsupported host への fallback はない。F5df は does not call platform import。F5da-F5de action / driver、F5cs virtual drain、F5cp / F5co lower cursor、packet record reader、raw storage、queue、scheduler、DOM / Canvas / minifb、video memory、DrawTarget / RenderTarget、fallback に触れない。

std layer row tile RLE present host import request は F5cr の checkpoint である。`std/gui/tile_present_host_import` は F5cq の `GuiRgba8888RowTileRlePresentHostCommandRecord` だけを消費し、`GuiRgba8888RowTileRlePresentHostImportRequest` に包む。request target は `GuiRgba8888RowTileRlePresentHostImportTarget` の `Window WindowId`、`Offscreen`、`Device` に限定する。Headless is not a presentation target。headless / text grid は `GuiError::Unsupported` とし、fallback target を選ばない。RGBA8888 row tile RLE 専用の境界なので `ColorFormat::FormatRgba8888` 以外の capability も `GuiError::Unsupported` として、この mismatch を platform backend に持ち越さない。

std layer row tile RLE present virtual drain は F5cs の checkpoint である。`std/gui/tile_present_virtual_drain` は headless / test が F5cq host-command record を観測するための境界であり、presentation target ではないため does not consume F5cr。`GuiRgba8888RowTileRlePresentVirtualDrain` は Begin / Run / End の phase、optional surface / frame、expected / seen count を保持し、RunRecord では `run_pixel_offset == seen_pixel_count` を要求する。これにより total count だけでは見逃す gap / overlap / reorder を std layer で拒否できる。error は `GuiRgba8888RowTileRlePresentVirtualDrainErrorKind` と直前 drain state を保持し、silent no-op や fallback presenter へ逃げない。

std layer row tile RLE present schedule boundary は F5ct の checkpoint である。`std/gui/tile_present_schedule` は F5cq host-command record stream を platform host dispatch の前で deterministic slice budget に区切る。`GuiRgba8888RowTileRlePresentScheduleState` は F5cs virtual drain state と slice-local command / pixel counters だけを保持し、stream validation は F5cs virtual drain に委譲する。`Yield means exact slice budget` であり、valid record を消費した後に command budget または pixel budget へちょうど到達したときだけ `Yield` を返す。over-budget is a typed error であり、budget 超過、single RunRecord の pixel budget 超過、checked arithmetic overflow、lower F5cs failure は previous schedule state を持つ error で返す。この layer は queue、timer、F5cr request、host import call、raw packet storage、platform API、Canvas / DOM / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present scheduled dispatch boundary は F5cu の checkpoint である。`std/gui/tile_present_dispatch` は F5ct before F5cr の順序で、schedule validation / budget decision の後に host import request value を作る。`GuiRgba8888RowTileRlePresentDispatchState` は `GuiRgba8888RowTileRlePresentScheduleState` だけを持つ。success path は `RequestReady request plus post phase` であり、request と `Continue` / `Yield` / `Completed` の post phase を同じ ready value に入れる。これにより exact-budget record と EndFrame record の request delivery を落とさない。F5ct error と F5cr error は previous dispatch state を返し、F5cr error では同じ step で得た updated schedule state を採用しない。この layer は F5cs direct call、F5cp / F5co cursor、raw packet storage、queue、timer、host import execution、platform API、Canvas / DOM / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present dispatch loop outcome boundary は F5cv の checkpoint である。`std/gui/tile_present_dispatch_loop` は F5cu の `RequestReady request plus post phase` を、platform executor の host outcome と接続するための one-shot pending value に包む。`GuiRgba8888RowTileRlePresentDispatchLoopPendingRequest` は previous state、next state、request、post phase を同時に保持し、Clone / Copy を持たない。`complete_request consumes pending` ため、同じ host outcome を二重に完了して next state を複数回 publish する replay を型上の所有権境界で避ける。host outcome が Err の場合は previous state を持つ typed error を返し、Ok の場合だけ post phase に従って Continue / Yield / Completed の completion に next state を入れる。この layer は F5cu だけを authority とし、F5ct / F5cr / F5cs の direct call、host import execution、queue、timer、scheduler、raw storage、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host execution action boundary は F5cw の checkpoint である。`std/gui/tile_present_host_execution` は F5cr の `GuiRgba8888RowTileRlePresentHostImportRequest` を `GuiRgba8888RowTileRlePresentHostExecutionAction` に写す。action は flat target x record action であり、Window / Offscreen / Device と BeginFrame / RunRecord / EndFrame の直積を enum variant として持つ。Window variant は `WindowId` と descriptor / run record を payload struct に保持し、Offscreen / Device は variant 名で target を保持する。F5cw は F5cr request accessor と F5cq record / run record shape だけを authority とし、does not execute host imports。actual Web / native / bare executor の失敗はこの action ではなく `Result unit GuiError` として F5cv `complete_request` に戻す。この layer は F5cv / F5cu / F5ct / F5cs direct call、F5cr request constructor、raw storage、host execution API、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation boundary は F5dg の checkpoint である。`std/gui/tile_present_host_span_operation` は F5cw `GuiRgba8888RowTileRlePresentHostExecutionAction` を actual Web / native / bare presenter が 1 operation ずつ消費できる `GuiRgba8888RowTileRlePresentHostSpanOperation` stream に写す。cursor は `GuiRgba8888RowTileRlePresentHostSpanOperationCursor` で、phase は `SinglePending operation`、`RunPending target run_span_cursor`、`Completed` の 3 種だけである。Begin / End action は SinglePending operation として 1 回だけ発行され、次の step は explicit Completed を返す。Run action は start で F5df run-span cursor を 1 回だけ作り、step ごとに F5df `run_span_step` を最大 1 回だけ呼んで WindowRunSpan / OffscreenRunSpan / DeviceRunSpan に target-qualified mapping する。F5df start / step error は action または cursor context を保持する enum error に包む。F5dg は actual host import execution、F5da-F5de action driver、F5cs virtual drain、F5cp / F5co lower cursor、packet record / raw storage、platform API、DOM / Canvas / minifb、video memory、DrawTarget / RenderTarget、queue、scheduler、fallback、silent no-op を提供しない。

std layer row tile RLE present scheduled span operation boundary は F5dh の checkpoint である。`std/gui/tile_present_scheduled_span_operation` は F5dg operation stream を platform presenter へ渡す前に deterministic slice budget で区切る。これは F5ct record scheduler の再利用ではない。F5ct は F5cq record 単位で full RunRecord を 1 cost とするため、F5dg が Run を複数 row span に分解した後の span operation scheduling とは authority が異なる。F5dh の `GuiRgba8888RowTileRlePresentScheduledSpanOperationState` は F5dg cursor と slice-local operation / pixel counters だけを保持し、F5cs / F5ct / F5cu を直接呼ばない。Begin / End は operation cost 1 and pixel cost 0、RunSpan は operation cost 1 and `span.width * span.height` pixel cost である。valid operation を消費した後に exact budget へ到達した場合だけ `Yield` を返し、operation、post phase、next state は `OperationReady` に同居するため exact-budget operation は失われない。over-budget、single span pixel budget 超過、checked arithmetic overflow、lower F5dg failure は typed error として previous state を保持する。`resume_slice` は F5dg cursor を保持して slice counters だけ reset する。この layer は actual host import execution、record scheduler、action driver、raw storage、queue、timer、platform API、DOM / Canvas / minifb、video memory、DrawTarget / RenderTarget、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation attempt boundary は F5di の checkpoint である。`std/gui/tile_present_host_span_operation_attempt` は F5dh `GuiRgba8888RowTileRlePresentScheduledSpanOperationReady` と actual Web / native / bare / headless presenter が返す caller supplied outcome を対応づける。`GuiRgba8888RowTileRlePresentHostSpanOperationAttempt` は attempted operation と `Result unit GuiError` だけを保持し、std layer は `Result::Ok unit` や synthetic failure を作らない。`attempt_step` は support before equality の順序を守る。support は F5cy `GuiRgba8888RowTileRlePresentHostExecutorSupport` enum を target support set として読むが、F5cy `require_supported` や action equality helper へは戻らない。operation equality は 9 variants すべてで variant と target を比較し、Window variants は `window_id_raw`、Begin / End は descriptor、RunSpan は x / y / width / height / RGBA channel を public accessor で比較する。unsupported target と mismatched attempt は `GuiError::Unsupported` / `GuiError::InvalidCommand` category を持ち、scheduled ready と attempt を失わない typed error として返す。Yield phase is data only であり、F5di は resume、queue、timer、scheduler、actual platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation completion boundary は F5dj の checkpoint である。`std/gui/tile_present_host_span_operation_completion` は F5di の `GuiRgba8888RowTileRlePresentHostSpanOperationAttemptStep` を AttemptStep only の入力として受け、caller supplied outcome と ready phase を completion value へ写す。`GuiRgba8888RowTileRlePresentHostSpanOperationCompletion` は `Continue state` / `Yield state` だけを持つ。F5dh `Completed` は operation を持たない terminal なので、per-operation completion does not create Completed。host outcome failure does not publish state であり、`Err host_error` は `GuiRgba8888RowTileRlePresentHostSpanOperationCompletionHostFailed` に host error、scheduled ready、attempt、category `Some host_error` を保持する。F5dj は F5di の association validation を再実行せず、F5dh step / start / resume、F5cs / F5ct / F5cu、F5cy action validation、F5cw action equality、F5da-F5de action driver、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter step boundary は F5dk の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_step` は support set、F5dh ready、presenter supplied attempt を受け、F5di before F5dj の順序で 1 operation の戻り道を固定する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterStep` は F5dj の `GuiRgba8888RowTileRlePresentHostSpanOperationCompletionStep` を保持する success value である。F5di rejection は `AttemptRejected` として support、ready、attempt、lower F5di error、lower category を保持し、F5dj rejection は `CompletionRejected` として attempt step、lower F5dj error、lower category を保持する。F5dk does not execute host imports し、success / failure outcome を合成せず、actual Web / native / bare / headless presenter が作った attempt だけを扱う。Completed、F5dh start / step / resume、F5dg start / step、F5cw action validation、F5da-F5de action driver、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op は提供しない。

std layer row tile RLE present host span operation presenter loop boundary は F5dl の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_loop` は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterLoopState` を LoopState として定義し、support、F5dh policy、scheduled state を同じ value に束ねる。`start` は F5dh start を 1 回だけ呼び、`request` は F5dh step を 1 回だけ呼ぶ。F5dh `OperationReady` は support / policy / ready を持つ presenter request へ写し、F5dh operation-less terminal は loop `Completed` として返す。`complete` は F5dk presenter step を 1 回だけ呼び、F5dk success branch でだけ F5dj completion step を読み、Continue / Yield を support / policy / scheduled state 付き LoopState へ再包装する。F5dl does not execute host imports し、actual presenter attempt を合成せず、F5dh `resume_slice`、F5di / F5dj direct call、F5dg start / step、F5cs / F5ct / F5cu、F5da-F5de action driver、F5cy / F5cw validation、queue、timer、real scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter outcome boundary は F5dm の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_outcome` は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterOutcomeRequest` を non-Copy request bridge として定義し、F5dl request と F5dh ready operation accessor から得た expected operation を保持する。actual presenter はこの request を borrow して operation を読み、host outcome を得た後で `OutcomeRequest` を value として消費し、F5di attempt constructor へ caller supplied `Result unit GuiError` を 1 回だけ渡して `OutcomeAttempt` を作る。`complete` は `OutcomeAttempt` を value として消費し、F5dl complete を 1 回だけ呼び、lower error は request、attempt、F5dl category accessor 由来 category とともに typed error へ保持する。F5dm does not execute host imports し、F5di validation、F5dk presenter step、F5dj completion step、F5dh start / step / resume、F5dg、action drivers、queue、timer、real scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、loop `Completed` creation を提供しない。

std layer row tile RLE present host span operation presenter driver boundary は F5dn の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_driver` は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterDriverState` を non-Copy driver state として定義し、actual Web / native / bare / headless presenter loop が F5dl と F5dm を別々に扱わず、driver start / request / complete contract だけを扱えるようにする。`start` は F5dl start を 1 回だけ呼んで F5dl loop state を DriverState へ包む。`request` は DriverState を value として消費して F5dl request を 1 回だけ呼び、F5dl `Request` の場合だけ F5dm outcome request を作る。F5dl terminal `Completed` は driver `Completed` へ写すだけで、F5dm outcome request は作らない。`complete` は `OutcomeRequest` と caller supplied outcome を value として受け、F5dm outcome attempt と F5dm outcome complete を 1 回ずつ呼び、F5dl Continue / Yield を次の DriverState へ再包装する。F5dn does not execute host imports し、F5dl complete direct call、F5di constructor / validation direct call、F5dh start / step / resume direct call、action drivers、queue、timer、real scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `GuiError` creation を提供しない。

std layer row tile RLE present host span operation presenter executor boundary は F5do の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor` は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorRequest` を non-Copy request として定義し、F5dn OutcomeRequest、OutcomeRequest 内の F5dl request から読んだ support、expected span operation を同じ value に束ねる。executor が返す `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorAttempt` は executed span operation と caller supplied outcome を持つ。request creation は OutcomeRequest 由来 support だけで unsupported operation を検査し、unsupported の場合も F5dn complete へ合成 outcome を流さない。complete は request の expected span operation と attempt の reported span operation を payload まで比較し、一致した場合だけ F5dn complete を 1 回呼ぶ。F5do does not execute host imports し、F5dl complete direct call、F5dm outcome attempt / complete direct call、F5di constructor / validation direct call、F5cw action mapping、action drivers、queue、timer、real scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `GuiError` outcome creation を提供しない。

std layer row tile RLE present host span operation presenter executor loop boundary は F5dp の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_loop` は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorLoopState` を non-Copy loop state として定義し、F5dn DriverState を保持する。`start` は F5dn start を 1 回だけ呼び、`request` は LoopState を value として消費して F5dn request を 1 回だけ呼ぶ。F5dn `Completed` branch は loop `Completed` へ写すだけで F5do を呼ばない。F5dn `Operation` branch は F5do executor request を 1 回だけ呼ぶ。`complete` は F5do executor complete を 1 回だけ呼び、F5dn DriverCompletion を Continue / Yield の LoopCompletion へ再包装する。F5dp is not actual Web / native / bare / headless execution であり、real scheduler policy でもない。F5dn complete direct call、F5dm / F5dl / F5di / F5dh / F5dk / F5dj direct call、F5cw action mapping、action drivers、queue、timer、real scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `GuiError` outcome creation を提供しない。

std layer row tile RLE present host span operation presenter executor attempt driver boundary は F5dq の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_attempt_driver` は actual Web / native / bare / headless presenter executor が返した executor supplied attempt を F5dp executor loop completion へ戻す。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorAttemptDriverStep` は completion-only success value であり、F5dp complete に消費された request / attempt を再保持しない。failure は category と lower F5dp error だけを持ち、lower F5dp error を recovery authority とする。F5dq は F5dp complete wrapper であり、F5do direct call、F5dn / F5dm / F5dl / F5di / F5dh / F5dk / F5dj direct call、old action path、virtual executor、virtual drain、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `Result::Err GuiError` outcome creation を提供しない。

std layer row tile RLE present host span operation presenter executor session boundary は F5dr の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session` は actual Web / native / bare / headless presenter loop が ready state、executor pending request、completion result を sentinel / null なしで保持できる session contract を定義する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionState` は `Ready` または `Completed` であり、`Completed` request は lower loop を呼ばず terminal `Completed` result を返す。`Ready` state だけが F5dp request を 1 回呼び、operation request は `SessionPending` に移る。`session_complete` は pending request と executor attempt を value として消費し、F5dq attempt driver step を 1 回だけ呼び、Continue / Yield を Ready session state に写す。F5dr は actual execution、real scheduler policy、F5dp complete direct、F5do / F5dn / F5dm / F5dl / F5di / F5dh / F5dk / F5dj direct call、old action path、virtual executor、virtual drain、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit` / `Result::Err GuiError` outcome creation を提供しない。

std layer row tile RLE present host execution report boundary は F5cx の checkpoint である。`std/gui/tile_present_host_execution_report` は F5cw の `GuiRgba8888RowTileRlePresentHostExecutionAction` と executor outcome を action context and executor outcome を失わない report に束ねる。`GuiRgba8888RowTileRlePresentHostExecutionReport` は action と `GuiRgba8888RowTileRlePresentHostExecutionReportKind` を保持し、kind は `Succeeded` または `Failed GuiError` である。report construction は executor-supplied `Result unit GuiError` を data に写すだけなので新しい failure mode を作らない。F5cx は not actual execution and not pending completion であり、F5cv / F5cu / F5ct / F5cs / F5cp / F5co、F5cr request constructor、raw storage、host API、platform API、queue、timer、scheduler、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。caller は `report_outcome` で元の `Result unit GuiError` を取り出し、F5cv `complete_request` へ渡せる。

std layer row tile RLE present host executor boundary は F5cy の checkpoint である。`std/gui/tile_present_host_executor` は actual Web / native / bare executor の手前で、executor target support と returned report association を検査する。`GuiRgba8888RowTileRlePresentHostExecutorSupport` は Window、Offscreen、Device とその非空の組み合わせだけを表す enum で、空 support を表現しない。`GuiRgba8888RowTileRlePresentHostExecutorError` は `UnsupportedAction` / `ReportActionMismatch`、category、expected action、reported action option を保持する typed error である。`validate_report_for_action` は support validation の後、F5cx report が持つ action と expected action の full action identity を比較する。full action identity は variant、window、surface、frame、packet metadata、run offset、run count、RGBA channel を含む。matching action の failed report は association として valid なので、この layer では拒否しない。actual host import execution、F5cv pending completion、F5cu / F5ct / F5cs / F5cp / F5co、F5cr request construction、raw storage、host API、platform API、queue、timer、scheduler、DOM / Canvas / minifb、video memory、fallback、silent no-op は提供しない。

std layer row tile RLE present host report loop bridge boundary は F5cz の checkpoint である。`std/gui/tile_present_host_report_loop_bridge` は F5cv pending request、F5cw action decoding、F5cx `report_outcome`、F5cy `validate_report_for_action` を接続する。`GuiRgba8888RowTileRlePresentHostReportLoopBridgeError` は `ExecutorValidationFailed` または `LoopCompletionFailed` の lower error、category、loop state を保持する。contract は validation before completion であり、support / full action identity 検査に失敗した場合は F5cv `complete_request` を呼ばず、pending previous state を error に返す。validation success の場合だけ F5cx `report_outcome` を F5cv `complete_request` に渡し、pending value を消費する。matching action の failed report は F5cv `HostImportExecutionFailed` へ進み、wrong action report は completion 前に `ExecutorValidationFailed` で止まる。この layer は actual host import execution、F5cu / F5ct / F5cs / F5cp / F5co、F5cr request construction、raw storage、host API、platform API、queue、timer、scheduler、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host execution driver boundary は F5da の checkpoint である。`std/gui/tile_present_host_execution_driver` は F5cv の one-shot pending request を、actual Web / native / bare / headless executor が読む action と completion 用 pending value の組へ束ねる。`GuiRgba8888RowTileRlePresentHostExecutionDriverPending` は `GuiRgba8888RowTileRlePresentDispatchLoopPendingRequest` と `GuiRgba8888RowTileRlePresentHostExecutionAction` を保持し、pending を所有するため Clone / Copy を持たない。`prepare` は pending request accessor から request を読み、F5cw action を 1 回だけ導出して original pending value と一緒に保持する。executor は `pending_action` だけを読み、実行結果は `Result unit GuiError` として `complete_outcome` に返す。`complete_outcome` は stored action を読んでから pending value を取り出し、F5cx report を作って F5cz bridge へ渡す。F5da は F5cv `complete_request`、F5cy validation、F5cr request construction を直接呼ばず、actual platform API、DOM / Canvas / minifb、video memory、queue、timer、scheduler、F5cu / F5ct / F5cs / F5cp / F5co、raw storage、fallback、silent no-op を提供しない。

std layer row tile RLE present virtual host executor boundary は F5db の checkpoint である。`std/gui/tile_present_virtual_executor` は F5da の one-shot driver pending を deterministic headless / test executor で消費し、actual Web / native / bare executor と同じ F5cw action shape を使う。`GuiRgba8888RowTileRlePresentVirtualExecutor` は F5cy support と F5cs virtual drain を保持する。`execute` は F5da pending action を読み、F5cy `require_supported` を drain mutation より前に実行する。support rejection では F5da `complete_outcome` を one-shot cleanup として呼ぶが、virtual drain は更新せず `SupportRejected` を返す。support success の場合だけ F5cw action を F5cq host-command record へ total mapping し、F5cs virtual drain に流す。drain failure でも F5da `complete_outcome Err` で pending を消費し、recovery executor は original executor のまま `DrainFailed` を返す。driver completion が期待と矛盾した場合は `InconsistentCompletion` で返す。F5db は fallback ではなく actual platform presenter でもない。F5cv direct completion、F5cz direct bridge、F5cr request construction、actual platform API、DOM / Canvas / minifb、video memory、queue、timer、scheduler、F5cu / F5ct / F5cp / F5co、raw storage、fallback、silent no-op を提供しない。

std layer row tile RLE present host action sink boundary は F5dc の checkpoint である。`std/gui/tile_present_host_action_sink` は actual Web / native / bare presenter から返る executor-supplied outcome を `GuiRgba8888RowTileRlePresentHostActionSinkStep` として action と一緒に保持する。`gui_rgba8888_row_tile_rle_present_host_action_sink_step` は F5cy `require_supported` を先に呼び、unsupported target では typed `UnsupportedAction` を返す。supported action の場合だけ caller が渡した `Result unit GuiError` を step に入れる。F5dc does not manufacture success であり、std layer は `Result::Ok unit` を作って actual execution を成功扱いにしない。F5dc は F5da pending を所有せず、F5da completion、F5cx report、F5cz bridge、F5cr request construction、F5cu / F5ct / F5cs / F5cp / F5co、raw storage、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host action sink driver boundary は F5dd の checkpoint である。`std/gui/tile_present_host_action_sink_driver` は F5dc の action sink step と F5da の one-shot driver completion を接続する。`gui_rgba8888_row_tile_rle_present_host_action_sink_driver_step` は driver pending から action を借用で読み、caller が渡した executor-supplied outcome を F5dc step に渡す。F5dc rejection では completion を呼ばず、`SinkRejected` の owner-bearing error として original driver pending を返す。F5dc success では同じ outcome を F5da `complete_outcome` に渡し、`GuiRgba8888RowTileRlePresentHostActionSinkDriverStep` として sink step と completion を返す。driver completion failure では pending は既に消費済みなので、F5da driver error と sink step だけを `DriverCompletionFailed` に保持する。F5dd does not manufacture executor outcome。std layer で `Result::Ok unit` や synthetic `Result::Err` を作らず、actual executor の `Result unit GuiError` をそのまま流す。F5dd は F5cv direct completion、F5cz direct bridge、F5cx report construction、F5cr request construction、F5db virtual executor、F5cu / F5ct / F5cs / F5cp / F5co、raw storage、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host action attempt driver boundary は F5de の checkpoint である。`std/gui/tile_present_host_action_attempt_driver` は actual Web / native / bare executor が返した action attempt と F5da driver pending の action identity を比較してから、F5dd action sink driver へ outcome を渡す。`GuiRgba8888RowTileRlePresentHostActionAttempt` は attempted action と executor-supplied outcome だけを持つ Copy value である。`gui_rgba8888_row_tile_rle_present_host_action_attempt_driver_step` は driver pending から expected action を借用で読み、attempt action と F5cy full action equality で比較する。一致しない場合は F5dd を呼ばず、`AttemptActionMismatch` の owner-bearing error として expected action、attempted action、`GuiError::InvalidCommand` category、original driver pending を返す。一致した場合だけ、attempt outcome を F5dd `gui_rgba8888_row_tile_rle_present_host_action_sink_driver_step` に委譲する。F5de does not manufacture executor outcome。`Result::Ok unit` や synthetic `Result::Err` を作らず、attempt に含まれる `Result unit GuiError` だけを流す。F5de は F5dc direct call、F5cv direct completion、F5cz direct bridge、F5cx report construction、F5cr request construction、F5db virtual executor、raw storage、queue、timer、scheduler、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter executor session turn boundary は F5ds の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn` は actual Web / native / bare / headless scheduler が F5dr session state または executor pending request のどちらを所有しているかを、1 turn 分の typed state として保持する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnState` は `Session` と `Pending` だけで構成し、no separate Completed turn state を持たない。terminal completion の authority は F5dr session state と F5dr session request に残すため、F5ds は F5dr `SessionState::Ready` / `Completed` を直接見ない。`turn_poll` は owner-consuming API であり、`Pending` は executor へそのまま移し、`Session` だけが F5dr session request を 1 回呼ぶ。`turn_complete` は F5dr session complete だけを 1 回呼び、Continue / Yield を `Session` turn state へ包み直す。この boundary は real scheduler policy、queue、timer、actual platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic outcome creation を提供しない。

std layer row tile RLE present host span operation presenter executor session turn step boundary は F5dt の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_step` は F5ds poll / complete result を、future Web / native / bare / headless driver が同じ enum で扱える transient step result に正規化する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnStepResult` は `Execute`、`Continue`、`Yield`、`Completed` を持つ。`Execute` は pending request を所有し、`Continue` と `Yield` は F5ds turn state を所有する。`Completed` は transient Completed result であり、persistent completed state ではない。start is setup authority なので `turn_step_start` は F5ds `turn_start` を 1 回だけ呼び、step result ではなく turn state を返す。`turn_step_poll` と `turn_step_complete` はそれぞれ F5ds `turn_poll` / `turn_complete` だけを 1 回呼び、結果を `TurnStepResult` へ写す。この boundary は lower F5dr / F5dp / F5dq direct call、old action path、raw storage、actual platform API、DOM / Canvas / minifb、video memory、queue、timer、real scheduler、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter executor session turn driver boundary は F5du の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_driver` は F5dt `Execute` result を actual Web / native / bare / headless executor が outcome-only に扱える `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnDriverPending` へ包む。driver pending は F5dr session pending request を所有し、Clone / Copy されない。`turn_driver_pending_operation` は pending を消費せず、F5ds borrowed request accessor と F5do `executor_request_operation` から borrowed expected operation を読む。`turn_driver_complete` は caller supplied outcome と borrowed expected operation から F5do `executor_attempt` を 1 回だけ作り、F5dt `turn_step_complete` へ 1 回だけ委譲する。executor から operation を受け取らないため、この layer で prevents operation mismatch を contract として固定する。この boundary は real scheduler、queue、timer、platform API、DOM / Canvas / minifb、video memory、raw storage、fallback、silent no-op、synthetic `Result::Ok unit`、synthetic `Result::Err GuiError::` を提供しない。

std layer row tile RLE present host span operation presenter executor session turn scheduler decision boundary は F5dv の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_scheduler` は F5du driver step result を target-neutral scheduler decision へ写す。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnSchedulerDecision` は `Execute`、`ContinueNow`、`ScheduleOneShot`、`Completed` で構成する。`ScheduleOneShot` は actual timer backend ではなく、validated delay と turn state を束ねた scheduled state である。policy constructor と `scheduler_decide` はどちらも `yield_delay_ms >= 0` を検査し、invalid policy は owner-bearing policy error として original driver step、`PolicyInvalid` kind、`GuiError::InvalidCommand` category を返す。F5dv は F5du start / poll / complete / pending operation helper、timer API、queue、real scheduler backend、platform API、DOM / Canvas / minifb、video memory、raw storage、DrawTarget / RenderTarget、fallback、silent no-op、synthetic outcome creation を提供しない。

std layer row tile RLE present host span operation presenter executor session turn timer request boundary は F5dw の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_timer` は F5dv scheduler decision を target-neutral timer request へ写す。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnTimerReady` は `Execute`、`ContinueNow`、`ScheduleTimer`、`Completed` を持つ。`ScheduleTimer` は owner-bearing timer pending であり、pending は F5dv scheduled state と std `TimerRequest` を所有する。timer policy は `WindowId` と `TimerId` を持ち、policy constructor と interpret はどちらも `timer_id_raw > 0` を検査する。`TimerRequest` は policy と scheduled delay の検査後にだけ作り、one-shot timer request として `repeating false` を固定する。invalid policy と invalid scheduled delay は original scheduler decision を保持する owner-bearing interpret error になる。timer completion は pending request timer id、incoming `TimerEvent` timer id、tick を検査し、成功時だけ scheduled turn state を回収して F5dv `ContinueNow` decision を返す。F5dw は actual timer backend registration、queue、real scheduler backend、platform API、DOM / Canvas / minifb、video memory、raw storage、DrawTarget / RenderTarget、fallback、silent no-op、synthetic outcome creation を提供しない。

std layer row tile RLE present host span operation presenter executor session turn virtual timer bridge は F5dz の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer` は F5dw の owner-bearing timer pending と F5dy の `GuiVirtualTimerState` を `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerPending` にまとめ、deterministic headless / offscreen test で scheduled turn を再開する。schedule は F5dw pending から borrowed `TimerRequest` を読み、`gui_virtual_timer_schedule` を 1 回だけ呼ぶ。advance は `gui_virtual_timer_advance` を 1 回だけ呼び、event がなければ pending と next virtual timer state を返す。`GuiEvent::Timer` の場合だけ F5dw `turn_timer_complete` を 1 回だけ呼び、成功時は scheduler decision を返す。schedule failure は original F5dw pending、original virtual state、lower `GuiError` を保持する。advance failure は original combined pending と lower `GuiError` を保持する。unexpected event は F5dw pending、advance-after virtual timer state、event を保持する。timer complete failure は F5dw complete error と advance-after virtual timer state を保持する。F5dz は real scheduler loop、actual timer backend、queue、platform API、DOM / Canvas / minifb、video memory、DrawTarget / RenderTarget、fallback、silent no-op、loop drain を提供しない。

std layer row tile RLE present host span operation presenter executor session turn virtual scheduler state boundary は F5ea の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler` は F5dv scheduler decision、F5dw timer request、F5dz virtual timer bridge を deterministic scheduler state として接続する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState` は `Turn`、`WaitingTimer`、`Execute`、`Completed` を持つ。`GuiVirtualTimerState` は static policy ではなく dynamic state であり、`Turn`、`Execute`、`Completed` の payload または F5dz `WaitingTimer` pending に保持する。`ContinueNow` は reusable decision ではなく pollable `Turn` phase へ写す。`ScheduleTimer` だけが F5dz schedule を呼ぶ。F5dz advance が `Ready` decision を返した場合は、one-shot timer が完了済みであることを `gui_virtual_timer_empty` として明示し、同じ decision boundary に戻す。F5ea は actual scheduler loop、timeslice policy、event queue、loop drain、platform timer backend、DOM / Canvas / minifb、video memory、DrawTarget / RenderTarget、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter executor session turn virtual scheduler single step boundary は F5eb の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step` は F5ea state を 1 回だけ進める。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerStepResult` は `Advanced`、`BlockedWaitingTimer`、`BlockedExecute`、`Completed` を持つ。Turn path は F5du driver poll、F5dv scheduler decide、F5ea timer decide の順序を固定し、それぞれを 1 回だけ呼ぶ。poll failure と scheduler decision failure は current `GuiVirtualTimerState` と lower error を保持する。timer decision failure は F5ea の lower owner-bearing error を保持する。`WaitingTimer`、`Execute`、`Completed` はその場で backend、queue、event loop、executor を呼ばず、`BlockedWaitingTimer`、`BlockedExecute`、`Completed` として外部 loop authority へ返す。F5eb は actual scheduler loop、timeslice policy、event queue、loop drain、platform timer backend、DOM / Canvas / minifb、video memory、DrawTarget / RenderTarget、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter executor session turn virtual scheduler bounded drain boundary は F5ec の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain` は F5eb step result を bounded に消費する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainPolicy` は F5eb step policy と `max_advance_count` だけを持つ。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainResult` は `BudgetExhausted`、`BlockedWaitingTimer`、`BlockedExecute`、`Completed` を持つ。`max_advance_count` は policy construction と drain entry で 0 以上に検査する。0 の場合は step を呼ばず `BudgetExhausted` として state と remaining count を返す。`Advanced` だけが budget を消費し、`BlockedWaitingTimer`、`BlockedExecute`、`Completed` は budget を消費せず typed terminal として返る。`StepFailed` は F5eb lower error だけを保持し、original state を重複保持しない。F5ec は timer advance、executor completion、actual scheduler loop、timeslice backend、platform API、DOM / Canvas / minifb、video memory、DrawTarget / RenderTarget、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter executor session turn virtual scheduler transition boundary は F5ed の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition` は F5ec drain terminal を `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransition` へ写す。Transition enum は `YieldSlice`、`AwaitTimer`、`ExecuteHostAction`、`Done` を持つ。`BudgetExhausted` は `YieldSlice`、`BlockedWaitingTimer` は `AwaitTimer`、`BlockedExecute` は `ExecuteHostAction`、`Completed` は `Done` に対応する。F5ed は F5ec payload struct を public transition payload として保持せず、accessor で取り出した state / pending / execute / completed と `remaining_count` を transition-owned payload へ詰め替える。`remaining_count` は正規化、減算、再計算をせず、後続 scheduler authority が budget 消費状況を判断するための値としてそのまま保持する。F5ed は timer advance、executor completion、F5ec drain 再実行、F5eb step、actual scheduler loop、queue、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter executor session turn virtual scheduler slice boundary は F5ee の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice` は F5ec bounded drain と F5ed transition を 1 work slice の public boundary として接続する。Policy は F5ec drain policy と `yield_delay_ms` だけを保持し、`yield_delay_ms` は policy construction と slice entry の両方で 0 以上に検査する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSliceResult` は `YieldSlice`、`AwaitTimer`、`ExecuteHostAction`、`Done` を持つ。`YieldSlice` は state、`remaining_count`、`yield_delay_ms` を保持し、`AwaitTimer`、`ExecuteHostAction`、`Done` はそれぞれ pending / execute / completed authority と `remaining_count` を保持する。F5ee は public slice entry で F5ec drain を 1 回だけ呼び、成功時だけ F5ed transition mapping を 1 回だけ呼ぶ。F5ec / F5ed payload struct は slice payload として保持せず、slice-owned payload へ詰め替える。Drain failure は lower F5ec error だけを `DrainFailed` に保持する。F5ee は F5eb step 直接呼び出し、timer advance、executor completion、real scheduler loop、queue、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop boundary は F5ef の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop` は F5ee `virtual_scheduler_slice` を real scheduler loop / headless app-loop が match できる loop-owned result へ詰め替える唯一の入口である。Policy は F5ee slice policy だけを保持する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopResult` は `Yield`、`AwaitTimer`、`ExecuteHostAction`、`Done` を持つ。`Yield` は state、`remaining_count`、`yield_delay_ms` を保持し、他の payload は pending / execute / completed authority と `remaining_count` を保持する。F5ef public step は F5ee `virtual_scheduler_slice` を 1 回だけ呼び、F5ee payload struct を public loop payload として保持せず、loop-owned result へ詰め替える。Failure は lower-only slice error として F5ee slice error だけを保持する。F5ef は F5ec / F5ed / F5eb / F5ea を直接呼ばず、timer advance、executor completion、actual while loop、queue drain、backend timer、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop action boundary は F5eg の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action` は F5ef `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopResult` を、後続の real scheduler loop / headless app-loop authority が消費する `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopAction` へ total mapping で写す。`loop_action_from_result` の public 入口は F5ef result だけを入力にし、`Yield` / `AwaitTimer` / `ExecuteHostAction` / `Done` を `YieldToClock` / `AwaitTimerAdvance` / `ExecuteHostAction` / `Complete` へ explicit match で詰め替える。Action payload は F5ef payload struct を保持せず、state / pending / execute / completed authority、`remaining_count`、`yield_delay_ms` を action-owned payload として保持する。Mapping は total なので F5eg 自体は error `Result` を作らない。timer advance、executor completion、real scheduler loop、queue drain、native / bare / headless real backend、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op は後続 authority の責務であり、F5eg は提供しない。

std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop timer advance boundary は F5eh の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance` は F5eg `AwaitTimerAdvance` payload を consumed authority として受け、F5ea `virtual_scheduler_advance_timer` を 1 回だけ呼ぶ。Public `loop_timer_advance` entry は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionAwaitTimerAdvance`、`TurnTimerPolicy`、`delta_ms` だけを入力にし、general `LoopAction` や F5eg `loop_action_from_result` は受けない。`remaining_count` は pending owner を消費する前に読み、成功時は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopTimerAdvanceCompleted` として次の scheduler state と original `remaining_count` を返す。失敗時は lower F5ea `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerAdvanceError` と original `remaining_count` を `AdvanceFailed` に保持する。F5eh は executor completion、yield-to-clock handling、real scheduler loop、queue drain、native / bare / headless real backend、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop executor complete boundary は F5ei の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete` は F5eg `ExecuteHostAction` payload を consumed authority として受け、caller supplied `Result unit GuiError` を F5du `turn_driver_complete` へ 1 回だけ渡す。成功した driver step は F5dv `scheduler_decide` へ 1 回だけ渡し、その decision は保存していた `timer_state` とともに F5ea `virtual_scheduler_decide` へ 1 回だけ渡す。Public `loop_executor_complete` entry は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorCompletePolicy`、`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionExecuteHostAction`、caller supplied outcome だけを入力にする。policy は scheduler policy と timer policy だけを保持し、timer state、queue、backend handle、host handle を保持しない。`remaining_count` と `timer_state` は pending owner を消費する前に読み、成功時は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorCompleteCompleted` として次の scheduler state と original `remaining_count` を返す。失敗時は `DriverCompleteFailed`、`SchedulerDecisionFailed`、`TimerDecisionFailed` に lower error と original `remaining_count` を保持し、F5du / F5dv 由来の失敗では `category` と `timer_state` も保持する。F5ei は executor result を合成せず、`Result::Ok unit` や `GuiError` を作らない。F5ei は yield-to-clock handling、complete handling、real scheduler loop、queue drain、native / bare / headless real backend、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を提供しない。

std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop yield complete boundary は F5ej の checkpoint である。`std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete` は F5eg `YieldToClock` payload と caller supplied `delta_ms` を入力にする deterministic clock-delta authority であり、actual real scheduler loop そのものではない。Public `loop_yield_complete_yield_advance` entry は `remaining_count` と `yield_delay_ms` を state owner consumption 前に読み、`delta_ms < 0` を `DeltaInvalid`、`yield_delay_ms < 0` を `YieldDelayInvalid` として owner-bearing `Result` error へ写す。`0 <= delta_ms < yield_delay_ms` の場合だけ `sub yield_delay_ms delta_ms` を実行し、same state / same `remaining_count` / reduced delay を持つ `YieldPending` を `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYieldCompleteYieldAdvanceResult` として返す。`delta_ms >= yield_delay_ms` の場合は state owner を `YieldReady` として返す。Public `loop_yield_complete_complete_ack` entry は F5eg `Complete` payload から `remaining_count` を completed owner consumption 前に読み、terminal completed payload へ変換する。F5ej は timer advance、executor complete、scheduler decision、actual real scheduler loop、headless app-loop integration、native / bare real timer backend、queue drain、backend handle、platform API、DOM / Canvas / minifb、video memory、fallback、silent no-op を持たない。

Web formal one-shot timer request backend boundary は F5dx の checkpoint である。`platforms/gui/web/timer` は F5dw の target-neutral `TimerRequest` を Web host import `nepl_gui_web.request_timer` へ渡す platform boundary であり、raw negative status を typed `GuiError` へ写す。window id と timer id は positive integer、`interval_ms < 0` は invalid、`interval_ms == 0` は clear request である。`repeating true` は repeating timer、`repeating false` は one-shot timer として accepted であり、Web Shell はそれぞれ `setInterval` と `setTimeout` を使う。Timer state は browser handle、window id、timer id、interval、repeating mode、tick を保持し、clear は stored mode に応じて `clearInterval` / `clearTimeout` を使い分ける。One-shot timer は clear-before-enqueue ordering を守り、`GuiEvent::Timer` を shared input queue に渡す前に active timer entry を消す。F5dx は Web platform boundary だけを実装し、std / core / alloc に DOM、Canvas、browser handle、stdout fallback、polling fallback、scheduler loop、native / bare / headless backend を持ち込まない。

## TUI Migration Contract

既存 TUI は `features/tui` と `platforms/wasix/tui` に直接 helper が露出している。これを段階的に次へ移す。

```text
features/tui
    legacy-compatible facade
    -> features/gui + terminal profile helper

platforms/wasix/tui
    existing WASIX terminal backend
    -> platforms/gui/terminal/wasix

platforms/wasix/tui/text
    display width / wrap helpers
    -> std/gui/text_measure terminal measurer + alloc/gui/text layout helpers

platforms/wasix/tui/input
    raw byte / ANSI keyboard helper
    -> platforms/gui/terminal/input + std/gui/keymap + alloc/gui/routing/focus

platforms/wasix/tui/buffer
    raw line buffer and diff present
    -> RenderTarget TextGrid + terminal host present

platforms/wasix/tui/ansi
    ANSI output helper
    -> terminal backend implementation detail
```

移行中は既存 import path を壊さない。`features/tui` は compatibility facade とし、内部実装を GUI substrate へ寄せる。新しい application は `features/gui` / `alloc/gui` / `std/gui` を使い、terminal target では terminal backend を選ぶ。

## 参考

- `embedded-graphics` の `DrawTarget` は最下層 drawing abstraction の参考にする。
- WebAssembly Component Model / WIT は host interface schema の参考にする。
- Zenn 設計指針の platform 依存隔離、`Option` / `Result` / enum、契約と現状実装の分離、試作段階でも雑設計を残さない方針を正の制約として扱う。
- NEPLg2 の既存 `core` / `alloc` / `std` / `platforms` 分離と NEPLg2.1 prefix 型式移行を正の制約として扱う。
## F5ek std layer row tile RLE present host span operation presenter executor session turn virtual scheduler real loop step boundary

F5ek は、F5eg `LoopAction` を actual real scheduler loop / headless app-loop が扱う直前の std layer dispatch contract として定義する。ここでは backend、queue、timer sleep、platform API を実行しない。`LoopAction` と explicit input を照合し、正しい組み合わせだけを F5ej / F5eh / F5ei の typed authority へ 1 回だけ渡す。

F5ek policy は `scheduler_policy` と `timer_policy` のみを持つ。`LoopExecutorCompletePolicy` を policy 内へ保持しないため、timer policy authority は二重化しない。Execute branch は F5ei `loop_executor_complete_with_policy_refs` を使い、timer branch と同じ `timer_policy` を借用する。

`RealLoopStepInput` は `ClockDelta`、caller supplied `ExecutorOutcome`、explicit `CompleteAck` を持つ。入力種別が action と合わない場合は action owner と input owner を持つ mismatch error を返す。fallback、silent no-op、executor outcome 合成は禁止する。
