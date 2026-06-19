# NEPLg2 GUI/TUI 標準ライブラリ実装計画

作成日: 2026-06-01

## 目的

`doc/neplg2/gui_standard_library_spec.md` に基づき、GUI と TUI を共通 UI substrate として実装する。既存 TUI は保守対象として残すのではなく、terminal backend として段階的に再設計・再実装する。

この計画は `plan.md` を変更せず、NEPLg2.1 の現行 `stdlib/` 上で進める。

## 現状

- 明示的な GUI 標準ライブラリは存在し、`core/gui`、`alloc/gui`、`std/gui`、`platforms/gui/terminal` の初期 checkpoint まで進んでいる。
- 現在の実装は bounded data contract と flat arena tree の初期 checkpoint を優先した段階であり、arena を使う focus traversal、pointer routing、diff / invalidation、arena order の linear layout connector、parent-local stack layout policy、stack cross-axis alignment、overflow rejection は実装済みである。Web Playground には floating GUI window layer、backend-neutral command DTO、TypeScript 側 host frame decode / present 境界、`presentCommands` と `beginFrame` / `pushCommand` / `endFrame` / `closeWindow` runtime bridge、`presentVideoMemory` runtime bridge、SharedArrayBuffer video memory surface、`ImageData` + `putImageData` only presenter、Web-only `nepl_gui_web` video memory host import、NEPL stdout legacy smoke transport、ActionId input target decode、typed input queue、coalescing / saturating SharedArrayBuffer event queue、Web-only `nepl_gui_web` input host import、Counter / Life / Mandelbrot / calculator / scientific calculator / paint / breakout の NEPL-side wait/update/render loop、Mandelbrot HD 用 stdout `rgba-row` payload、host-frame window resized event poll、timer event poll、close button / terminal stop の相互 lifecycle cleanup、native には OS window manager の resize / close / event pump behavior を反映した minifb optional window runner と、F5ex host import scalar payload を検証済み `NativeSpanOperation` へ変換して injected sink へ渡す Rust side validator、typed span operation を semantic `0xRRGGBBAA` offscreen framebuffer へ反映する checked sink、completed framebuffer を explicit background 付き semantic `0x00RRGGBB` present buffer へ変換する境界、RGB0 present buffer を minifb 非依存の typed presenter frame へ借用する adapter、complete span sequence だけを last completed RGB0 presenter frame へ更新する native sink boundary、resize と last frame ownership を分離する native window presenter state boundary、native smoke window loop が `NativeWindowPresenterState` を display / hit-test authority として使う integration checkpoint、resize 時に current drawable surface と同じ size の RGB0 frame を redraw/present する checkpoint、typed `ExecuteHostAction` から pending span operation identity を読み F5ev scheduler ready payload と一緒に保持する native presenter operation identity input boundary を追加済みである。flex / grid / scroll layout、stateful pointer routing、正式 DrawCommand / tile presentation host import ABI、formal tile / bitmap / row / RLE payload、lifecycle event poll、mobile backend はまだ未実装である。
- F5fh では native formal presenter session boundary として、Rust lib-only `NativeWindowPresenterSession` を追加済みである。session は `NativeRgb0PresenterSink` と `NativeWindowPresenterState` を所有し、Begin / RunSpan は typed not-presented outcome、End 成功だけを typed presented outcome として返す。sink failure と presenter failure は enum error で分離し、失敗時は previous presenter frame を保持する。minifb / OS window loop、actual scheduler backend、timer、queue、formal `std/gui` host import 接続、bare runtime host import はこの checkpoint では未実装である。
- F5fi では native presenter session host helper boundary として、F5ey/F5ex 由来の scalar begin / run / end validation path を `NativeWindowPresenterSession` へ接続する Rust lib-only helper を追加済みである。validation failure は `NativeWindowPresenterSessionHostError::ValidationFailed`、session failure は `SessionFailed` として分離し、Begin / RunSpan は `NotPresented`、End 成功だけ `Presented` になる。
- F5fj では native presenter session host import boundary として、`platforms/gui/native/scheduler_host_executor` の formal NEPL `#extern` 名を `window_presenter_session_begin` / `window_presenter_session_run` / `window_presenter_session_end` に差し替え済みである。これは F5fi Rust helper に対応する native-only host import contract であり、bare runtime host import、actual scheduler backend、timer、queue、minifb loop はまだ未実装である。
- F5fk では bare display presenter session host import boundary として、`platforms/gui/bare/scheduler_host_executor` の formal NEPL `#extern` 名を `display_presenter_session_begin` / `display_presenter_session_run` / `display_presenter_session_end` に差し替え済みである。これは bare display / offscreen / device surface presenter session の host import contract であり、bare actual display driver、polling input、actual scheduler backend、timer、queue、present loop はまだ未実装である。
- F5fl では bare display framebuffer adapter boundary として、`platforms/gui/bare/framebuffer` の pure validation state machine を追加済みである。Begin / RunSpan / End の順序、target、surface、frame、shape、row-major progress、incomplete end、descriptor contract、active seen count invariant を host import 前に検査し、validation failure と host failure を typed error で保持する。
- F5fm では bare display storage adapter boundary として、`platforms/gui/bare/display_storage` の typed effect ledger を追加済みである。`GuiBareFramebufferStepApplied` をそのまま信用せず、storage が保持する canonical framebuffer state から operation を再検証し、stale / replay / forged state は enum error で拒否する。actual display driver、raw memory、present loop、fallback、silent no-op は後続 slice へ残す。
- F5fn では bare display memory write plan boundary として、`platforms/gui/bare/display_memory` の checked byte write plan を追加済みである。`GuiBareDisplayMemoryState` は canonical `GuiBareDisplayStorageState` を保持し、public `GuiBareDisplayStorageStepApplied` / effect をそのまま信用せず、canonical storage state から再適用した expected state / effect と一致する場合だけ進む。span write は `height * stride_bytes`、`y * stride_bytes`、`x * 4`、`width * 4`、`byte_start`、`byte_end` を checked arithmetic で計算し、overflow / OOB を enum error で fail-closed にする。present は descriptor complete count を確認する。actual display driver、raw byte buffer ownership、host import、present loop、fallback、silent no-op は後続 slice へ残す。
- F5fo では bare display driver outcome ledger boundary として、`platforms/gui/bare/display_driver` の pure ledger を追加済みである。`GuiBareDisplayDriverState` は canonical memory state として `GuiBareDisplayMemoryState` を保持し、public `GuiBareDisplayMemoryStepApplied` をそのまま信用せず、canonical memory state から `gui_bare_display_memory_apply` を再適用した expected state / action と supplied state / action を照合する。`BeginAccepted`、`SpanWriteAccepted`、`FramePresentAccepted` は checked action evidence と exact match した場合だけ state を進め、`DriverRejected` は lower `GuiError` を保持した `Result` error として返す。actual hardware write、raw byte buffer ownership、host import、present loop、fallback、silent no-op は後続 slice へ残す。
- F5fp では bare display driver host import boundary として、`platforms/gui/bare/display_driver_host_import` を追加済みである。`gui_bare_display_driver_host_import_step` は host import の前に F5fo ledger preflight を通し、preflight success action の variant に対応する `display_driver_begin` / `display_driver_span_write` / `display_driver_frame_present` だけを 1 回呼び、status `0` だけを accepted outcome として F5fo ledger へ再適用する。`-1` は `DriverRejected GuiError::Unsupported`、unknown negative と positive non-zero は `DriverRejected GuiError::BackendFailure` として fail-closed にする。span write import は byte_start / byte_len / byte_end / surface byte count / run and pixel range / color evidence を host へ渡す。raw byte buffer readback、byte echo verification、polling input、long-running scheduler backend、timer queue、present loop、fallback、silent no-op は後続 slice へ残す。
- F5fq では bare display driver byte echo verification boundary として、`platforms/gui/bare/display_driver_byte_echo` を追加済みである。public `GuiBareDisplayDriverStepApplied` は Copy value として偽造できるため public entry には取らず、`GuiBareDisplayDriverState`、`GuiBareDisplayMemoryStepApplied`、`GuiBareDisplayDriverOutcome`、`GuiBareDisplayDriverByteEcho` から内部で F5fo `gui_bare_display_driver_apply` を呼ぶ。canonical step が `SpanWrite` / `SpanWriteAccepted` の場合だけ、accepted byte range と RGBA8888 typed channel enum `Red` / `Green` / `Blue` / `Alpha` に echo byte を照合する。raw display memory ownership、bulk byte readback、actual driver adapter、polling input、long-running scheduler backend、timer queue、present loop、fallback、silent no-op は後続 slice へ残す。
- F5fr では bare raw display memory ownership boundary として、`platforms/gui/bare/display_memory_owner` を追加済みである。owner は canonical `GuiBareDisplayDriverState`、private `RegionToken u8`、surface byte count、lifetime verified byte count、packet-local verified byte count を保持し、public write は supplied `GuiBareDisplayDriverByteEchoVerified` を authority にせず owner state から F5fq を再実行する。single-byte store / readback success の後だけ owner state と byte count を進め、owner-bearing error は Clone / Copy にしない。
- F5fs では bare display memory span write/readback boundary として、`platforms/gui/bare/display_memory_span_readback` を追加済みである。owner 内 canonical driver state から F5fo apply を再実行し、canonical `SpanWrite` / `SpanWriteAccepted` の byte range 全体を owner raw memory に store し、full readback が成功した後だけ owner state、lifetime verified byte count、packet-local verified byte count を進める。F5fs evidence は span readback であり frame ready / present ready evidence ではない。
- F5ft では bare actual display driver adapter boundary として、`platforms/gui/bare/display_driver_adapter` を追加済みである。adapter は owner 内 driver state から F5fp host import step を呼び、returned host accepted ledger step だけを authority にする。SpanWrite は F5fs full span write/readback success まで owner を進めず、Begin は packet-local verified byte count を reset し、Present は host accepted ledger step completed evidence に限定して owner state を進める。span readback failure after host accepted は owner-bearing error で owner を回収するが、external host side effect rollback や whole surface readiness は主張しない。
- F5fu では bare presented packet readiness evidence boundary として、`platforms/gui/bare/display_present_readiness` を追加済みである。public entry は `GuiBareDisplayDriverAdapterCompleted` を value として消費し、`FramePresentHostAccepted` / `FramePresentAccepted`、owner canonical driver phase Idle、`last_present` 一致、`packet_verified_byte_count == pixel_count * 4` を検査して packet-local readiness evidence を返す。これは row-tile RLE packet の readiness であり、whole surface aggregation、hardware flush completion、long-running scheduler backend、timer queue、present loop、DOM、Canvas、minifb、video memory transport、fallback、silent no-op は後続 slice へ残す。
- F5fw では bare display hardware flush accepted boundary として、`platforms/gui/bare/display_flush_completion` を追加済みである。public entry は sealed `GuiBareDisplayWholeSurfacePacketReadinessCompleted` を value として消費し、preflight 後に `nepl_gui_bare.display_hardware_flush` を 1 回呼ぶ。成功は host accepted status であり、physical scanout completion や scheduler completion ではない。
- F5fx では bare display operation-to-driver-adapter bridge として、`platforms/gui/bare/display_operation_driver_bridge` を追加済みである。public entry は owner + `GuiRgba8888RowTileRlePresentHostSpanOperation` だけを受け取り、owner canonical state から framebuffer / storage / memory validation を通して `gui_bare_display_driver_adapter_step` へ渡す。lower validation failure は host import 前に original owner を回収し、adapter failure は adapter recovered owner と lower adapter error を保持する。host side effect rollback、long-running scheduler backend、timer queue、present loop、DOM、Canvas、minifb、video memory transport、fallback、silent no-op は扱わない。
- F5fy では bare display presenter input boundary として、`platforms/gui/bare/display_presenter_input` を追加済みである。public entry は `GuiBareDisplayMemoryOwner` と typed `ExecuteHostAction` だけを受け、action 消費前に pending operation を borrowed に読み、F5fx bridge を 1 回だけ呼ぶ。success と category 付き bridge failure は `gui_bare_scheduler_executor_input` で scheduler ready へ戻し、category が無い bridge failure は scheduler ready を作らず original action と lower bridge error を保持する。direct host import、long-running scheduler backend、timer queue、present loop、DOM、Canvas、minifb、video memory transport、fallback、silent no-op、rollback claim は扱わない。
- F5fz では native scheduler real-loop action step boundary として、`platforms/gui/native/scheduler_real_loop_step` を追加する。public entry は native policy、backend clock state、F5el `NeedInput` だけを受け、Yield / Timer は native clock helper success payload の action / input を F5ek へ渡し、Execute は `gui_native_scheduler_host_executor_step` に委譲して host import failure を `Result unit GuiError` outcome として F5ek/F5el path へ戻し、Complete branch だけが `CompleteAck` を作る。F5ek success 後は F5el `real_loop_driver_after_step` を 1 回だけ呼ぶ。これは native-only の not long-running scheduler backend であり、bare owner path、queue、timer wait、present loop、minifb loop、DOM、Canvas、video memory transport、fallback、silent no-op は扱わない。
- F5ga では bare scheduler real-loop action step boundary として、`platforms/gui/bare/scheduler_real_loop_step` を追加する。public entry は bare policy、backend clock state、`GuiBareDisplayMemoryOwner`、F5el `NeedInput` だけを受け、Yield / Timer は bare clock helper success payload の action / input を F5ek へ渡し、Execute は F5fy display presenter input owner path へ委譲する。success と category 付き `BridgeFailedReady` は recovered owner を F5ew `gui_bare_scheduler_executor_step` へ戻し、`BridgeFailedMissingCategory` では `GuiError` を捏造せず direct error にする。F5ek / F5el failure は branch、clock state、recovered owner、lower error を保持する。これは bare-only の not long-running scheduler backend であり、queue、timer wait、present loop、direct host import、DOM、Canvas、minifb、video memory transport、fallback、silent no-op は扱わない。
- F5gb では Native / Bare scheduler bounded real-loop runner boundary として、`platforms/gui/native/scheduler_real_loop_runner` と `platforms/gui/bare/scheduler_real_loop_runner` を追加する。policy は platform real-loop step policy と `max_step_count` だけを保持し、F5el driver policy は F5fz / F5ga policy accessor から借用する。`max_step_count < 0` は typed `PolicyInvalid`、`max_step_count == 0` は F5el start 後の `NeedInput` を `BudgetExhausted` とし、Completed は terminal result として返す。native error は runner level で clock state を保持し、bare result / error は owner を保持または `gui_bare_scheduler_real_loop_step_error_owner` で回収する。これは bounded real backend loop checkpoint であり、OS window loop、minifb event pump、timer wait、sleep、queue drain、formal `std/gui` present host implementation、DOM、Canvas、video memory transport、fallback、silent no-op は扱わない。

## Phase F5gd: Native window event pump boundary

F5gd では `nepl-gui-native` の OS window observation を `NativeWindowEventPumpSnapshot` へ集約する。`NativeWindowSize` は observed size として zero dimension を許し、zero width / height は `NativeWindowPresenterSurfaceState::Unavailable` へ写す。positive drawable size では smoke runner が same width / height の RGB0 buffer を再生成し、pixel buffer stretch ではなく redraw / present で扱う。

close state は `Open`、`OsCloseRequested`、`ExitShortcutRequested` を分ける。現 smoke runner では OS close と Escape shortcut のどちらも process 終了へ写すが、future native backend / lifecycle / test event virtualization のため event pump boundary では同一化しない。

minifb input API を `poll_minifb_window_event_pump` に閉じる。main.rs は `Key`、`MouseButton`、`MouseMode`、`is_open`、`is_key_down`、`get_mouse_down`、`get_unscaled_mouse_pos` を直接扱わず、snapshot を `match` する。`poll_minifb_window_event_pump` は `window.update` / `update_with_buffer` を呼ばず、presentation authority は smoke runner / future backend loop に残す。

この phase は native event pump boundary だけであり、formal `std/gui` host import execution、scheduler loop、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。

## Phase F5ge: Native backend loop step boundary

F5ge では、F5gd の event pump snapshot を受けた後の native smoke backend state transition を `NativeWindowBackendLoop` へ移す。`main.rs` は minifb window creation、event pump adapter、title update、`window.update`、`window.update_with_buffer` だけを扱い、counter hit test、scene coordinate mapping、frame id update、resize redraw、RGB0 present buffer construction、presenter surface commit を直接行わない。

`NativeWindowBackendLoop::new_for_scale` は scale validation、initial frame render、checked initial size、presenter state creation、initial present を一括して行う。`event_pump_input` は previous observed size / previous mouse state を返し、`step` は `NativeWindowEventPumpSnapshot` を 1 件だけ処理して `CloseRequested` / `Unavailable` / `Drawable` を返す。`Drawable` は resize redraw evidence、pointer action、final frame evidence を持ち、resize と counter hit が同じ snapshot に入っても両方の committed presentation evidence を保持する。

commit rule は F5ge の中心契約である。close は no-progress、unavailable は observed size / mouse / surface availability だけを更新して blank frame を作らない。positive resize は rasterize / present が成功した後だけ surface state / previous size / frame id を commit する。counter hit は pointer unavailable、outside、hit を enum で分け、hit の場合だけ checked counter increment と checked frame id を mutation 前に検査し、present success 後だけ counter/current frame/frame id を進める。

この phase は native smoke backend の loop-step boundary であり、formal `std/gui` host import execution、scheduler loop、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。loop helper は minifb / DOM / Canvas / video memory / stdout protocol を知らず、fallback や silent no-op を作らない。

## Phase F5gf: Native host action boundary

F5gf では、F5ge の backend state transition outcome を native host execution action へ写す `NativeWindowHostAction` を追加する。`step` は pure backend transition evidence として残し、`step_host_action` が `CloseRequested` / `Unavailable` / `Drawable` を `Terminate` / `PumpEventsOnly` / `PresentFrame` へ変換する。

`Terminate` は `NativeWindowHostTerminalReason` により OS close と Escape shortcut を分ける。`PumpEventsOnly` は zero-size / unavailable surface 中に `window.update` だけを実行する action であり、blank frame や fallback frame を作らない。`PresentFrame` は `NativeWindowBackendLoopPresentation` と observed `NativeWindowSize` を持つが pixel borrow は持たず、actual pixel borrow は `current_present_frame_for_window` からだけ取得する。

`main.rs` は `NativeWindowBackendLoopStepOutcome` を直接 match せず、`NativeWindowHostAction` だけを match する。これにより future formal native OS scheduler / window backend loop は backend state transition と host-side execution decision を分けて扱える。

この phase は host action boundary だけであり、formal scheduler loop、queue、timer wait、OS wait strategy、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback や silent no-op も導入しない。

## Phase F5gg: Native minifb window run-loop adapter boundary

F5gg では、F5gf の `NativeWindowHostAction` を消費する cfg-gated minifb window run-loop adapter として `run_minifb_window_loop` を追加する。F5ge / F5gf で `main.rs` が持っていた minifb window lifecycle と host action execution はこの phase で supersede され、`main.rs` は `NativeWindowRunLoopConfig` を作って runner を呼ぶだけになる。

`run_minifb_window_loop` は `WindowOptions`、`ScaleMode::UpperLeft`、window title update、`window.update`、`update_with_buffer` を所有する。direct minifb input API は `poll_minifb_window_event_pump` に閉じ、run-loop adapter は `Key`、`MouseButton`、`MouseMode`、`is_open`、`is_key_down`、`get_mouse_down`、`get_unscaled_mouse_pos` を直接読まない。`WindowPresentFailed` は `update_with_buffer` failure だけを表し、`window.update` failure という存在しない error を作らない。

source policy は `run_minifb_window_loop` slice に minifb lifecycle API を許可する一方、`main.rs` では minifb / `WindowOptions` / `ScaleMode` / `window.update` / `update_with_buffer` を禁止する。run-loop slice でも queue、timer、sleep、`Duration`、`setTimeout`、`setInterval` は禁止し、`set_target_fps(60)` は smoke runner の busy spin 抑制としてだけ扱う。

この phase は minifb adapter boundary であり、formal native OS scheduler loop、queue、timer wait、OS wait strategy、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback や silent no-op も導入しない。

## Phase F5gh: Native window host-loop core boundary

F5gh では、F5gg の minifb window run-loop から `NativeWindowRunLoopHost` と `run_native_window_host_loop` を切り出す。core loop は `&mut NativeWindowBackendLoop` と host trait を受け、event snapshot polling、`step_host_action`、title update、pump-only、present を実行する。backend loop を value で消費しないため、host event failure、host action failure、presenter frame unavailable、host present failure のいずれでも caller が backend state を回収できる。

minifb 固有の `Window`、`set_title`、`window.update`、`update_with_buffer` は private `MinifbNativeWindowRunLoopHost` に閉じる。`run_minifb_window_loop` は backend loop と minifb window を初期化し、host adapter を作って `run_native_window_host_loop` を呼ぶだけになる。direct input API は引き続き `poll_minifb_window_event_pump` に閉じ、core loop と adapter のどちらも `Key`、`MouseButton`、`MouseMode`、`is_open`、`is_key_down`、`get_mouse_down`、`get_unscaled_mouse_pos` を直接読まない。

source policy は core loop slice と minifb host adapter slice を分ける。core loop slice は minifb / `window.update` / `update_with_buffer` を禁止し、minifb host adapter slice は direct input API、queue、timer、fallback、silent no-op を禁止する。

この phase は host-loop core boundary であり、formal native OS scheduler loop、queue、timer wait、OS wait strategy、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback や silent no-op も導入しない。

## Phase F5gi: Native window host-loop turn boundary

F5gi では、F5gh の long loop body を `step_native_window_host_loop` に分ける。`NativeWindowHostLoopTurn` は `Continue` と `Exit NativeWindowRunLoopExit` だけを持ち、scheduler queue や timer wait を隠す payload は持たない。

`step_native_window_host_loop` は initial title を設定せず、host event snapshot 1 件、`step_host_action` 1 件、`Terminate` / `PumpEventsOnly` / `PresentFrame` の実行 1 件だけを扱う。`run_native_window_host_loop` は initial title を 1 回だけ設定し、その後は `step_native_window_host_loop` の `Continue` / `Exit` だけを match する。

source policy は long loop runner slice と one-turn core slice を分ける。long loop runner に `poll_event_snapshot`、`step_host_action`、`NativeWindowHostAction::`、`current_present_frame_for_window`、host pump / present が戻らないことを検査し、one-turn core では minifb、direct input API、queue、timer、sleep、DOM / Canvas / video memory、fallback、silent no-op を禁止する。

この phase は future formal native OS scheduler / window backend loop の typed turn boundary であり、formal scheduler queue、timer wait、OS wait strategy、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback や silent no-op も導入しない。

## Phase F5gj: Native window host-loop bounded runner boundary

F5gj では、F5gi の one-turn function を bounded に反復する Rust smoke/native layer の runner を追加する。`NativeWindowHostLoopRunnerState` は initial title 済みかどうかだけを保持し、`initialize_native_window_host_loop` は `NativeWindowHostLoopInitialization::Initialized` / `NativeWindowHostLoopInitialization::AlreadyInitialized` を返す。二度目以降の初期化は unit の silent no-op にしない。

`run_native_window_host_loop_bounded` は `max_turn_count` の範囲で `step_native_window_host_loop` だけを呼ぶ。`max_turn_count == 0` では initial title を確認したうえで event poll を行わず、`BudgetExhausted completed_turns 0` を返す。`Continue` は completed turn count を増やし、`Exit` は exit turn を含む count で `Exited` を返す。

source policy は initializer、bounded runner、long loop runner、one-turn core の slice を分ける。bounded runner は `poll_event_snapshot`、`step_host_action`、`NativeWindowHostAction::`、present frame borrow、host pump / present を直接持たず、`step_native_window_host_loop` だけを turn progress authority にする。queue、timer、sleep、DOM / Canvas / video memory、fallback、silent no-op も禁止する。

この phase は future native scheduler の cooperative timeslice boundary であり、formal scheduler queue、timer wait、OS wait strategy、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback や silent no-op も導入しない。

- `examples/gui_counter.nepl`、`examples/gui_life.nepl`、`examples/gui_mandelbrot.nepl`、`examples/gui_calculator.nepl`、`examples/gui_scientific_calculator.nepl`、`examples/gui_paint.nepl`、`examples/gui_breakout.nepl` は GUI substrate の application update と render command stream を確認しつつ、現 checkpoint では `platforms/gui/web` の stdout legacy smoke transport で Web Playground host へ frame を出力する。これは正式な same app code contract ではなく、formal host surface ABI へ移行する対象である。Counter は action projection 互換 path を維持し、それ以外の interactive example は full `GuiWebEvent` polling を使う。text label を持つ button の stdout emission は `GuiWebButtonConfig` と `gui_web_stdout_button` へ集約し、example 側の重複した `fill_rect -> text_run -> action_rect` 手書きを戻さない。
- GUI/TUI の executable NEPLg2 code、stdlib doctest、`tests/stdlib/gui_*.n.md`、headless GUI examples は、括弧付き call を使わず、中間 `let` と pipeline で式境界を明示する方針に揃えた。prose の `O(1)` や WIT sketch は対象外である。
- 既存の近い資産は `features/tui` と `platforms/wasix/tui` である。
- `platforms/wasix/tui` は TTY ABI、ANSI 出力、text width、box line、line buffer、diff present を持つが、raw terminal helper と UI concept が同じ層に混ざっている。
- Web Playground は browser 上の editor / terminal と、その panel layout の上に重ねる floating GUI window layer を持つ。`host-bridge.ts` は unknown input の `present-commands` 風 frame を typed frame へ decode できる。`runtime-bridge.ts` は floating GUI manager を presenter として登録し、global `neplGuiHost.presentCommands`、`beginFrame` / `pushCommand` / `endFrame`、`presentVideoMemory`、`closeWindow` から typed Result 境界で frame lifecycle を渡せる。`presentVideoMemory` は `windowId`、`title`、`SharedArrayBuffer` だけを受け、`ArrayBuffer`、typed array、numeric id、string handle、transfer object は `invalid-video-memory-frame` として拒否する。Panel は command frame と video memory surface の state を分け、同じ `SharedArrayBuffer` identity の opened surface を再利用する。Surface size と window drawable size が違う場合も CSS scale や `drawImage` による伸縮はせず、top-left 1:1 presentation と resize event で扱う。`web/src/runtime/worker.ts` は `nepl_gui_web.video_memory_create_surface` / `acquire_write_slot` / `write_slot_bytes` / `write_rgba8888_row` / `discard_write_slot` / `publish_slot` / `present_surface` / `close_surface` を持ち、worker-local opaque id と `Result` に写される negative status だけを NEPL/Wasm へ返す。`present_surface` は typed `gui_video_memory_present` worker message と ack `SharedArrayBuffer` により main thread presenter の実結果を待つ。さらに `platforms/gui/web/stdout_protocol.nepl` と `web/src/gui-preview/stdout-protocol.ts` により、Web Playground の `Run` で実行された NEPL program の stdout frame stream が floating GUI window を開く。stdout helper は `GuiWebTextAlign` enum と `Result unit GuiError` で invalid geometry を返し、TypeScript parser は stdout fd=1 のみを protocol として扱い、frame 内 parse error では partial frame を破棄する。button hit target は `NEPLG2_GUI_ACTION_RECT` として出力され、timer request は window id と timer id を持つ `NEPLG2_GUI_ANIMATE_MS` として出力される。Web 側は active run が表示した window の `GuiWebInputEvent::action` / `pointer` / `keyboard` / `text-input` / `window resized` / `timer` を typed queue から SharedArrayBuffer queue へ渡す。worker の `nepl_gui_web.wait_action_id` import と `platforms/gui/web/input.nepl` の `Result Option ActionId GuiError` wrapper は、full event queue とは別の action projection queue を読む互換 action path として残す。`nepl_gui_web.wait_event_kind` / last-event field import と `Result Option GuiWebEvent GuiError` wrapper は、action、pointer down / move / up / cancel、keyboard down / up、single-scalar text input、host-frame window resized、timer tick を `GuiEvent` として NEPL app 側へ戻せる。close button は現 checkpoint では拒否可能 event ではなく host lifecycle signal として active worker を interrupt し、terminal stop / process finish は presenter の `closeWindow` で host-frame window を削除する。正式な DrawCommand / tile presentation host import ABI、IME composition / multi-scalar text、window focus / unfocus の発火 policy、rejectable close request、lifecycle variant、session id の正式化はまだ未実装である。

`video_memory_write_rgba8888_row` は `write_slot_bytes` より高水準な row payload 境界である。application は `y * stride + x * 4` の byte offset を計算せず、origin、pixel width、source pointer だけを渡す。worker と video memory surface helper は `width > 0`、surface bounds、`width * 4` と source byte length の一致を検査し、不一致は typed error として返す。row write は pixel plane だけを更新し、dirty metadata、slot epoch、published epoch、presented epoch は publish path の authority として残す。

`examples/gui_video_memory_rows.nepl` は formal row host import の focused NEPL example である。row bytes は `ByteBuilder` / `ByteBuf` owner で構築し、借用 `MemPtr u8` だけを `gui_web_video_memory_write_rgba8888_row` へ渡す。stdout `rgba-row`、command frame fallback、raw extern、`write_slot_bytes` には戻らない。現行 CI の通常 doctest は `nepl_gui_web` video memory import を unsupported stub として持つため、positive fake host import harness は opt-in focused regression として通常 path の NEPL/Wasm 実行を検査する。`examples/gui_mandelbrot.nepl` は `--video-memory-once` で 32x18 preview model の RGBA8888 row payload を同じ fake host harness で検査できる。legacy stdout interactive path は resize event を application update に取り込み、drawable pixel size から 1 pixel per sample の responsive model を作れる。`--video-memory-resize-once` は finite formal video memory resize/recreate checkpoint として、初期 surface を present した後に typed window resize event を 1 件読み、old surface を close して resized surface を create / render / present / close する。`--video-memory-loop` は formal video memory surface を保持して typed event を待つ loop checkpoint であり、resize event では old surface close 成功後に resized surface を recreate し、focus / unfocus / non-window event では surface を維持し、close request では current surface を close して終了する。`--video-memory-loop-test` の wait count は CI の停止条件であり scheduler policy ではない。`--video-memory-progressive-once` は row batch ごとの dirty rect publish を検査する finite checkpoint であり、`--video-memory-progressive-test` は同じ実装を実行する CI alias である。`--video-memory-progressive-loop-test` は既存 `GuiEvent::Timer` を使い、matching timer id の event 1 件につき 1 row batch だけ進める finite checkpoint である。timer id 不一致、empty event、focus event は batch を進めない。FHD 60 fps 実測、formal tiled transport、formal timer registration ABI、real scheduler policy は後続 slice であり、今回の loop checkpoint と timer-driven progressive row-batch checkpoint を全面移行とは扱わない。

## 根本課題

既存 TUI は terminal 向け helper としては有用だが、GUI と共通化できる抽象境界が不足している。

```text
現状:
    features/tui
        -> platforms/wasix/tui
            -> ansi/text/box/buffer/tty

問題:
    - ActionId / GuiEvent / GuiEffect がない
    - terminal raw input と application event が分離されていない
    - text-cell surface と GUI render command が共通 model を持たない
    - line buffer diff present が backend detail ではなく public helper になっている
    - accessibility / lifecycle / capability の型境界がない
```

解決後の形は次である。

```text
Application:
    Model + GuiEvent -> Update Model
    Model -> ViewTree

Common substrate:
    ViewTree -> LayoutTree -> DrawCommand stream

Backends:
    Web bitmap pixel surface
    Native window surface
    Mobile host view
    Embedded DrawTarget
    Terminal TextGrid surface
```

## Phase 1: core/gui no_alloc 基盤

追加する module:

```text
stdlib/core/gui.nepl
stdlib/core/gui/prelude.nepl
stdlib/core/gui/geometry.nepl
stdlib/core/gui/color.nepl
stdlib/core/gui/pixel.nepl
stdlib/core/gui/text_measure.nepl
stdlib/core/gui/draw_target.nepl
stdlib/core/gui/render_target.nepl
stdlib/core/gui/dirty_region.nepl
stdlib/core/gui/dirty_region_set.nepl
stdlib/core/gui/capability.nepl
stdlib/core/gui/error.nepl
stdlib/core/gui/event.nepl
stdlib/core/gui/render_command.nepl
```

最初の checkpoint では trait-based drawing の実 backend へ深く踏み込まず、型・constructor・軽量 helper・mock target・doctest を先に固定する。ただし、embedded を最低制約にするため、`core/gui` が `alloc` / `std` / `platforms` へ依存しないことは Phase 1 で検査する。検査は focused doctest だけにせず、`nodesrc/test_stdlib_gui_layering_policy.js` で core/platform 依存方向と terminal TextGrid 型の再定義禁止も固定する。

検証:

```powershell
node nodesrc/tests.js -i tests/stdlib/gui_core.n.md --no-tree -o tmp/gui-core-phase1.json -j 1 --dist web/dist --assert-io
node nodesrc/run_source_policy_regressions.js --warn-only
node nodesrc/issues.js check --dir issues
git diff --check
```

## Phase 2: TUI 共通化の型橋渡し

追加または変更する module:

```text
stdlib/features/gui.nepl
stdlib/features/tui.nepl
stdlib/platforms/gui/terminal.nepl
stdlib/platforms/gui/terminal/capability.nepl
stdlib/platforms/gui/terminal/text_grid.nepl
stdlib/platforms/gui/terminal/frame.nepl
```

目的:

- `features/gui` を新しい UI substrate の public facade にする。
- `features/tui` は compatibility facade として残し、内部で terminal backend へ寄せる。
- terminal は `SurfaceKind::TextGrid` capability を持つ backend として定義する。
- terminal 固有の cols / rows は `TerminalProfile` に置き、共通 capability と text-cell command は `core/gui` の型を再利用する。custom capability を受ける helper は `Result` を返し、`SurfaceKind::TextGrid` 以外を拒否する。
- 既存 `line_top` / `line_box` / `buffer_present_diff` などを一気に消さず、terminal backend の compatibility helper として段階的に移す。
- この Phase は terminal backend の型境界を先に作るための橋渡しであり、最初の complete backend は Web Playground のままとする。

2026-06-01 checkpoint では、terminal backend は `TerminalProfile`、`TextGridCapability`、`TerminalFrame` と core `TextCellRun` の橋渡しまでを実装済みである。`TextGridRenderTarget` や real `GuiHost.present` はまだ実装していない。旧 `features/tui` / `platforms/wasix/tui` の raw line buffer diff は compatibility path に残し、共通 substrate の public diff contract にはしない。

検証:

```powershell
node nodesrc/tests.js -i tests/stdlib/features_tui.n.md -i tests/stdlib/gui_core.n.md --no-tree -o tmp/gui-tui-bridge.json -j 1 --dist web/dist --assert-io
node nodesrc/test_stdlib_wasix_tui_no_unsafe_unwraps.js
node nodesrc/issues.js check --dir issues
git diff --check
```

## Phase 3: alloc/gui application model

追加する module:

```text
stdlib/alloc/gui.nepl
stdlib/alloc/gui/app.nepl
stdlib/alloc/gui/app/types.nepl
stdlib/alloc/gui/widget.nepl
stdlib/alloc/gui/widget/types.nepl
stdlib/alloc/gui/tree.nepl
stdlib/alloc/gui/tree/types.nepl
stdlib/alloc/gui/layout.nepl
stdlib/alloc/gui/layout/types.nepl
stdlib/alloc/gui/layout/arena.nepl
stdlib/alloc/gui/layout/stack.nepl
stdlib/alloc/gui/theme.nepl
stdlib/alloc/gui/theme/types.nepl
stdlib/alloc/gui/accessibility.nepl
stdlib/alloc/gui/focus.nepl
stdlib/alloc/gui/focus/types.nepl
stdlib/alloc/gui/routing.nepl
stdlib/alloc/gui/routing/types.nepl
stdlib/alloc/gui/routing/focus.nepl
stdlib/alloc/gui/diff.nepl
stdlib/alloc/gui/diff/types.nepl
stdlib/alloc/gui/text.nepl
stdlib/alloc/gui/text/types.nepl
stdlib/alloc/gui/test.nepl
```

最初は `WidgetId`、`ActionId`、`ViewNode`、`GuiEffect`、`Update`、`LayoutContext`、widget descriptor、retained tree、focus traversal、event routing、diff / invalidation、text buffer、theme、semantic tree、mock replay helper を小さく入れる。Closure callback、DOM、terminal raw code、OS handle は入れない。

2026-06-01 checkpoint では、`Update.effects` の境界を `GuiEffectBatch` に変更し、`alloc/gui/layout` の `TextMeasurer` 注入、`alloc/gui/widget` の button / label descriptor、`alloc/gui/tree` の bounded retained tree と parent index / depth を持つ `ViewTreeArena` / `LayoutTreeArena`、`alloc/gui/theme` の typed palette / metrics、`alloc/gui/accessibility` の semantic tree 初期 slice まで実装した。arena child insertion は owner-recovery error payload で tree owner と rejected child を返す。`GuiEffectBatch` は現時点では capacity 2 の bounded data であり、`alloc` collection 側の owner contract が安定した段階で `Vec GuiEffect` へ置き換える。

Focus traversal は `alloc/gui/focus` の platform 非依存 data contract として扱う。順方向 / 逆方向、wrap の有無、現在 focus が tree に存在しない場合の結果、disabled widget の除外を `Option` / enum で表し、host focus や accessibility focus の反映は `std/gui` 以降へ渡す。2026-06-01 の arena focus checkpoint では、bounded `FocusOrder` に任意長 arena を押し込めず、`ViewTreeArena` を直接走査して `focus_next_in_arena` / `focus_previous_in_arena` が `Option WidgetId` を返すようにした。

Focus routing は traversal とは別に `alloc/gui/routing/focus` へ置く。`FocusRouteCommand::Next` / `Previous` は focus movement だけを返し、`Activate` は current focus id の widget action だけを `GuiEvent::Action` として返す。戻り値は `FocusRouteResult::Ignored` / `MoveFocus WidgetId` / `Emit GuiEvent` で分ける。Tab、Shift+Tab、Enter、Space の portable default mapping は `std/gui/keymap` が `KeyboardEvent` と `FocusKeyMap` から `Option FocusRouteCommand` へ変換する。Arrow key や modifier bit の std contract は `std/gui/keymap` に置くが、ANSI escape sequence、DOM keyboard event、OS virtual key は platform backend が `KeyboardEvent` へ正規化し、application は raw key sequence を直接扱わない。

Event routing は `alloc/gui/routing` の pure data contract として扱う。pointer routing は `LayoutTree` hit test で `WidgetId` を得て、`ViewTree` の widget data から `GuiEvent::Action` を導出する。現 checkpoint は bounded root + 2 child、flat arena tree storage、arena focus next / previous traversal、arena pointer hit test / action lowering、half-open `GuiRect`、second child topmost、arena insertion order topmost、disabled widget suppression、layout hit だけで view widget が無い場合の `Option::None`、focus command routing、std keymap、terminal 1 byte input normalization、`ESC [ Z` Shift+Tab normalization、`ESC [ A/B/C/D` arrow key normalization、`ESC [ H/F` と `ESC [ 1/3/4 ~` の Home / End / Delete normalization、`ESC [ 1 ; <modifier> A/B/C/D` xterm modifier arrow normalization までを固定する。pointer capture、gesture、Web / native / mobile raw keyboard normalization、terminal の追加 ANSI / CSI sequence、途中入力 buffering は後続で実装する。TUI では keyboard / focus routing から同じ `FocusRouteCommand` / `GuiEvent::Action` を生成し、raw ANSI input を application が直接扱わないようにする。

Arena layout connector は `alloc/gui/layout/arena` に置く。現 checkpoint の `layout_view_tree_arena_linear` は `LayoutContext`、borrowed `ViewTreeArena`、`LayoutHint`、`LayoutConstraints` だけを使い、owner-consuming な `LayoutTreeArena` を返す。node は arena insertion order で y 方向へ配置し、parent index と depth は `ViewTreeArena` の構造を保つ。invalid constraints は `GuiError::InvalidGeometry`、空 tree や欠落 node は `GuiError::InvalidCommand` を返し、構築途中で失敗した `LayoutTreeArena` owner は connector が内部で解放する。

Stack layout policy は `alloc/gui/layout/stack` に置く。現 checkpoint の `layout_view_tree_arena_stack` は `StackLayoutPolicy` の `StackAxis`、spacing、`StackCrossAlignment`、`StackOverflowPolicy` を pure data として受け取り、同じ parent を持つ previous sibling の extent と spacing から parent-local offset を計算する。`Start` / `Center` / `End` は cross-axis position を決め、`Stretch` は vertical stack なら child width、horizontal stack なら child height を parent cross size にそろえる。overflow policy の `Allow` は現状互換の配置を維持し、clip / scroll は後続 layer に委ねる。`Reject` は parent bounds 外へ出る配置を `GuiError::InvalidGeometry` として拒否する。`ViewTreeArena` は borrow-only、成功時だけ `LayoutTreeArena` owner を返す。negative spacing、負 constraints、min > max constraints は `GuiError::InvalidGeometry`、壊れた parent index / 欠落 node は `GuiError::InvalidCommand` とし、途中で作った `LayoutTreeArena` owner は内部で解放する。flex grow、grid placement、scroll state、text buffer と arena node の対応付けは後続で実装する。

Diff / invalidation は `alloc/gui/diff` に置く。ここでは dirty widget / tree / layout などの共通 data contract だけを持ち、terminal line buffer diff、DOM patch、framebuffer dirty rect compression は `platforms/gui/*` の実装詳細にする。現 checkpoint では bounded `ViewTreeDiff` に加え、allocator-backed `ViewTreeArenaDiff` が node count、shape change、content change count、単一 changed `WidgetId` を保持する。arena owner は消費せず、parent index / depth / slot `WidgetId` の変化は `GuiInvalidation::Tree`、単一 content change は `GuiInvalidation::Widget id`、複数 content change は `GuiInvalidation::Tree` へ畳む。

Text buffer と minimal layout cache data は `alloc/gui/text` が所有する。`core/gui::TextRunId` は buffer snapshot 内の安定参照 id とし、`std/gui` / platform は測定、IME、font loading を担当する。現 checkpoint では `TextLayout` が injected `TextMeasurer` の結果、byte length、char count、fallback cell count、max width を保持し、`CachedTextLayout` が buffer id、run id、font id、max width、byte length、char count から deterministic key を作る。`core/gui` に `String` 実体を持たせず、terminal backend に text buffer ownership を漏らさない。line break、text hash / revision based invalidation、complex shaping cache は後続で実装する。

Bitmap font による Web GUI 再実装が安定した後、outline font rendering は `doc/neplg2/gui_font_rendering_design.md` の計画に従って進める。`web/src/fonts/HackGenConsoleNF-Regular.ttf` は `fonts/HackGenConsoleNF-Regular.ttf` resource として VFS / filesystem / embedded blob へ載せ、layout engine と rendering engine は同じ `GuiFontFace` / `ScaledFont` / metrics を使う。Ruby、furigana、日本語縦書き、math inline object は text layout engine の正式対象とし、未対応機能は hidden fallback ではなく typed unsupported error にする。

`core/gui/dirty_region` は embedded / framebuffer 向け no_alloc redraw contract として扱う。現 checkpoint は `DirtyRegion::Empty` / `Rect` / `Full` と O(1) bounding rect merge までを固定する。

`core/gui/dirty_region_set` は no_alloc fixed-capacity multiple rect contract として扱う。現 checkpoint は最大 2 rect を保持し、3 個目の追加は `Full` 状態へ昇格する。generic capacity、backend-specific damage compression、DOM patch、terminal line diff は `platforms/gui/*` の実装詳細へ置く。

検証:

```powershell
node nodesrc/tests.js -i tests/stdlib/gui_app.n.md --no-tree -o tmp/gui-app-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_layout.n.md --no-tree -o tmp/gui-layout-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_widget.n.md --no-tree -o tmp/gui-widget-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_tree.n.md --no-tree -o tmp/gui-tree-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_focus.n.md --no-tree -o tmp/gui-focus-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_routing.n.md --no-tree -o tmp/gui-routing-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_focus_routing.n.md --no-tree -o tmp/gui-focus-routing-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_diff.n.md --no-tree -o tmp/gui-diff-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_text.n.md --no-tree -o tmp/gui-text-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_theme.n.md --no-tree -o tmp/gui-theme-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/tests.js -i tests/stdlib/gui_accessibility.n.md --no-tree -o tmp/gui-accessibility-phase3.json -j 1 --dist web/dist --assert-io
node nodesrc/issues.js check --dir issues
git diff --check
```

## Phase 4: std/gui host contract

追加する module:

```text
stdlib/std/gui.nepl
stdlib/std/gui/host.nepl
stdlib/std/gui/runtime.nepl
stdlib/std/gui/window.nepl
stdlib/std/gui/timer.nepl
stdlib/std/gui/text_measure.nepl
stdlib/std/gui/keymap.nepl
stdlib/std/gui/ime.nepl
stdlib/std/gui/accessibility_host.nepl
stdlib/std/gui/error_display.nepl
```

目的:

- `GuiHost`、`WindowId`、`SurfaceId`、`TimerId`、`HostTextMeasurer`、`ImeBridge`、`AccessibilityHost` の標準境界を定義する。
- application は `GuiHost` を直接呼ばず、`GuiEffect` を runtime が解釈する。
- capability unsupported を `GuiError::Unsupported` として返す。
- `TextMeasurer` contract は `core/gui` に置く。`std/gui/text_measure` の host wrapper は legacy smoke、mock、terminal cell measurement、移行期 compatibility に限定し、formal GUI text measurement は `FontResourceRequest -> GuiFontFace -> ScaledFont -> ShapedRun -> RenderedTextMetrics` の font engine へ移す。
- `FocusKeyMap` は `KeyboardEvent` を `FocusRouteCommand` へ O(1) で変換し、`alloc/gui/routing/focus` に platform raw key code を漏らさない。

## Phase 5: Web Playground backend

対象:

```text
stdlib/platforms/gui/web/**
web/src/gui/**
nodesrc/gui_*_test_runner.js
tests/gui_playground/**
```

目的:

- TypeScript host bridge が `GuiEvent`、`DrawCommand`、`TextMeasurer` を変換する。
- JS `null` / `undefined` は backend 境界で `Option` / `Result` に変換する。
- Web bitmap pixel surface を `RenderTarget` から rasterize した pixel buffer presenter として実装する。
- CLI で headless smoke test を走らせる。

2026-06-02 checkpoint では、Web Playground workspace の old `gui-preview` pane と TS example scene renderer を削除し、editor の panel layout の上に重なる independent floating GUI window layer だけを表示経路として残した。window は minimize、maximize / restore、drag move、edge / corner resize、dock restore を持つ。`window-manager.ts` の transient state は `idle` / `drag` / `resize`、source は `host-frame`、window mode は `normal` / `minimized previousMode` / `maximized restoreRect`、dock button は `none` / `mounted` 形式の union で表し、maximize 中に minimize しても original restore rect を失わない。top bar の `GUI` button と editor header の `G` button は削除済みであり、Web Playground の `Run` で実行された NEPL program が stdout legacy smoke transport を出した時だけ host-frame window を開く。host event / queue status は window body に挟まず、折りたたみ式 `GuiWindowDebugPanel` で別表示する。host frame の title は floating window titlebar で表示し、renderer は app content だけを描く。debug panel は通常 window 操作を優先する低い z-layer とし、collapsed 時は toggle だけが pointer event を受け、`aria-live` を off にして queue 更新を main GUI live region の読み上げに混ぜない。古い workspace snapshot に `gui-preview` pane が残っている場合は `panel-layout.ts` の normalize で editor leaf に戻す。`window-manager.ts` と `panel.ts` には `null` / `undefined` / non-null assertion に依存しない source policy と、debug/status DOM を window content に戻さない regression を追加した。

`web/src/gui-preview/commands.ts` は `fill-rect` / `rgba-row` / `text-run` の command DTO、`rgba8888` 相当の color struct、text align enum、command frame、`action-rect` input target を持つ。`rgba-row` は legacy smoke transport で raster row を bounded command count で運ぶ現 checkpoint の payload であり、formal presentation ABI の代替ではない。`web/src/gui-preview/renderer.ts` は削除済みであり、Run 経路の Counter / Mandelbrot / Life / calculator / scientific calculator / paint / breakout 表示は現 checkpoint では `examples/gui_*.nepl` が出す stdout protocol だけで駆動する。`panel.ts` は host-frame surface であり、NEPL 実行結果の command frame だけを描画する。`panel.ts` は `GuiPreviewDebugSink` へ frame / input queue record を渡せるが、window content には status text を作らない。`web/src/gui-preview/host-bridge.ts` は unknown input を `GuiWebHostResult` の `ok` / `err` union で decode し、invalid color byte、unsupported command、invalid frame shape、invalid rgba row、invalid input target を typed error として返す。`web/src/gui-preview/runtime-bridge.ts` は presenter の未登録、global install 先の不正、streaming frame state、host decode error を `GuiWebRuntimeResult` の `ok` / `err` union で扱い、Playground 初期化時に floating GUI manager を global `neplGuiHost.presentCommands`、`beginFrame` / `pushCommand` / `endFrame` / `discardFrame`、`closeWindow` へ登録する。あわせて `neplGuiHost.takeInputEvents` / `resetInputEvents` を公開し、Web 側で queue された `GuiWebInputEvent::action` / `pointer` / `keyboard` / `text-input` / `window` / `timer` を Result として取り出せる。`web/src/gui-preview/input-bridge.ts` は listener にも typed event だけを通知し、`shared-event-queue.ts` は SharedArrayBuffer の full event queue と legacy action projection queue を分ける。full event queue は action / pointer / keyboard / text input / window / timer event の kind / window id / payload を worker へ渡し、action projection queue は `wait_action_id` 互換 path が non-action event を consume しないために使う。keyboard は focusable host frame surface が active host frame を表示している時だけ queue され、DOM key string は std key code と modifier bit、または Unicode scalar value へ正規化される。Space は keyboard と text input の両方、composition 中、Meta shortcut、multi-scalar text は未対応として queue しない。host-frame window は resize 時に `WindowEventKind::Resized` を queue し、stdout の animation timer request は `TimerEvent` として queue する。close button は現 checkpoint では veto 可能 event ではなく host lifecycle signal として扱い、window を削除した後に Shell listener が active worker を interrupt する。terminal stop / process finish は `neplGuiHost.closeWindow` presenter path で host-frame window を削除する。`web/src/runtime/worker.ts` は Web-only host import module `nepl_gui_web` の `poll_action_id` / `wait_action_id`、`poll_event_kind` / `wait_event_kind`、last-event field accessors に加えて、video memory surface の `create_surface` / `acquire_write_slot` / `write_slot_bytes` / `write_rgba8888_row` / `discard_write_slot` / `publish_slot` / `present_surface` を提供する。`platforms/gui/web/input.nepl` と `platforms/gui/web/surface.nepl` は raw sentinel を `Result` / `GuiError` へ正規化する。`web/src/terminal/shell.ts` は現在の run が stdout frame または video memory present で表示した window id だけを input queue 対象にし、stale window の input event を実行中 app へ混入させない。Counter は button click を action projection path で扱い、Mandelbrot / Life / calculator / scientific calculator / paint / breakout は full `GuiWebEvent` polling で action、pointer、timer を NEPL 側 event loop に戻して `update` と `render` を再実行する。Mandelbrot の HD / Detail mode は 1280x720 logical frame の raster 部分を `rgba-row` payload で描き、Life の HD mode は現 checkpoint では bounded sample rectangle stream で描く。これは legacy transport で扱える高解像度表示 checkpoint であり、DrawCommand / tile formal host import ABI と native `GuiHost.present` の HD raster contract はまだ未実装である。`web/src/gui-preview/stdout-protocol.ts` は stdout fd=1 の line protocol だけを typed command frame と typed animation timer request へ decode し、`NEPLG2_GUI_RGBA_ROW` を typed row payload、`NEPLG2_GUI_ACTION_RECT` を input target として保持し、frame 内 parse error では partial frame を fail-closed に破棄する。`web/src/terminal/shell.ts` が stdout chunk を parser に通して host frame を present し、run-wasm 開始/終了で GUI event queue と timer を reset / inactive 化する。Formal Web video memory presentation は bitmap pixel buffer を visible canvas へ `putImageData` するだけの presenter へ移行している。DOM / Canvas / SharedArrayBuffer は Web frontend の backend detail であり、`core/gui`、`alloc/gui`、`std/gui` の public type には入れない。

2026-06-13 resize checkpoint では、floating window の resize は app content を CSS / Canvas viewport で伸縮しない。`canvas-renderer.ts` は logical viewport を left 0 / top 0 / scale 1 に固定し、devicePixelRatio は backing bitmap rasterize scale にだけ使う。Window manager は outer frame rect ではなく `GuiPreviewPanel.drawableSurfaceCssSize` が返す drawable surface size を `WindowEventKind::Resized` に入れる。App / layout engine はその event を受け、次の frame width / height と pixel buffer を生成して present する。次 frame が届くまでは古い frame を左上 1:1 で表示し、余白は surface background とする。

GUI の最低性能目標は FHD 60 fps とする。現 checkpoint の legacy command frame renderer は正式 video memory surface ではないが、same-size frame ごとに大きな `Uint8ClampedArray` と `ImageData` を作り直さない。Canvas size が変わらない限り bitmap buffer と `ImageData` を再利用し、resize event によって app が新 frame を出すまで同じ presentation contract の中で処理する。

action record は full event queue と legacy action projection queue へ独立に書き込む。queue は bounded だが、producer は `event-queue-full` / `action-queue-full` を返さない。容量に達した場合は古い unread record を明示的に押し出し、新しい input を受け入れる。これにより action-only app が full event queue を読まない場合や、full-event app が legacy projection queue を読まない場合でも、Web UI が即時 overflow error で停止しない。

pointer move と window state は coalescing として実装する。`panel.ts` は pointermove を `requestAnimationFrame` 単位で最新 move へまとめ、`input-bridge.ts` の stored queue は直前に保持された同じ window id、pointer id、button の `move` または同じ window id / window kind の resize / focus state だけを最新値へ置き換える。`shared-event-queue.ts` は write tail 直前の unread slot が同一 pointer move または同一 window kind の state record である場合だけ最新値へ置換できる。queue 全体の未読 slot は走査しない。`down` / `up` / `cancel`、action、keyboard、text input、close lifecycle signal をまたいで古い move / window record を更新してはいけない。

次の Web checkpoint では NEPL/Wasm runtime が DrawCommand stream や tile / bitmap / row / RLE payload を正式 host import ABI 経由で渡す接続を追加する。video memory surface の create / write / discard / publish / present import は初期経路として接続済みである。Mandelbrot の formal video memory event loop は surface を保持し、typed resize event で full-resolution redraw と surface recreate を行う初期 checkpoint まで接続済みである。Mandelbrot の progressive path は finite row-batch dirty rect checkpoint と timer event driven batch progression checkpoint まで接続し、batch 末尾は sample height で clamp する。あわせて `GuiWebEvent` の IME composition / multi-scalar text、window focus / unfocus の発火 policy、rejectable close request、lifecycle variant へ広げ、session id / window id / timer id の正式化、formal timer registration ABI、Mandelbrot formal tiled rendering、real scheduler policy、Life の arbitrary-size board storage、Paint の persistent canvas storage を NEPL app の update loop と host ABI に接続する。現在の stdout protocol は、その接続後に正式 path から参照されない legacy smoke fixture として隔離する。

## Phase 6: Terminal backend replacement

対象:

```text
stdlib/platforms/gui/terminal/**
stdlib/platforms/wasix/tui/**
tests/stdlib/features_tui.n.md
tests/stdlib/gui_terminal_input.n.md
nodesrc/tui_regression.js
```

目的:

- `platforms/wasix/tui` の raw storage / ANSI / TTY helper を terminal backend implementation detail に押し下げる。
- `buffer_new` / `buffer_present_diff` の raw handle API を `TextGridRenderTarget` / `TerminalFrame` / `GuiHost.present` へ置き換える。
- 現 checkpoint の `TerminalFrame` は単一 `TextCellRun` frame 境界である。`TextGridRenderTarget` は diff / present 実装時に追加し、terminal-specific line diff は `platforms/gui/terminal` に閉じる。
- `platforms/gui/terminal/input.nepl` は terminal raw byte、3 byte ESC sequence、4 byte CSI tilde sequence、bounded 6 byte CSI modifier sequence を `TerminalInputEvents` へ正規化する。`ESC [ Z` は Shift+Tab として key code 9、modifier bit 1 へ正規化し、`ESC [ A/B/C/D` と `ESC [ 1 ; <modifier> A/B/C/D` は std navigation key code と modifier bitset へ正規化する。`ESC [ H/F` と `ESC [ 1/3/4 ~` は Home / End / Delete の typed key code へ正規化するが、`FocusRouteCommand` や `ActionId` は作らず、`std/gui/keymap` と `alloc/gui/routing/focus` の責務を保つ。
- `features/tui` の利用者向け path は壊さず、内部を新 substrate に差し替える。

互換維持:

- `features_tui.n.md` は当面維持する。
- 新規 test は `tests/stdlib/gui_terminal.n.md` に追加し、旧 TUI helper と新 TextGrid backend の対応を固定する。
- input normalization は `tests/stdlib/gui_terminal_input.n.md` に分け、Tab / LF / CR / Space / printable ASCII / invalid byte / unsupported control byte / Shift+Tab ESC sequence / arrow key ESC sequence / Home / End / Delete CSI sequence / xterm modifier arrow CSI sequence / unknown sequence / invalid numeric parameter / invalid modifier parameter を固定する。

## Phase 7: Embedded backend

目的:

- Phase 1 で固定した `core/gui` no_alloc contract を real-style backend で再検査する。
- `MockDrawTarget`、`DirtyRegion`、`DirtyRegionSet` を用意する。現 checkpoint の `DirtyRegionSet` は fixed capacity 2 の O(1) contract であり、generic capacity や backend-specific compression は後続で追加する。
- Optional `FlushTarget` を確認する。

## Phase 8: Native / Mobile backend

Native:

- window / surface / clipboard / timer / cursor を `std/gui` host contract に接続する。
- File dialog、menu、tray、drag-and-drop は extension module に分ける。

2026-06-02 checkpoint では、workspace member `nepl-gui-native` を追加し、pure framebuffer renderer と minifb window runner を分けて実装した。CI / headless 環境では `cargo test -p nepl-gui-native --lib` が framebuffer 変換と metric contract だけを検査する。実 window は target-specific optional dependency の `window` feature で有効化し、`cargo run -p nepl-gui-native --features window -- mandelbrot` のように明示実行する。

2026-06-02 native platform behavior checkpoint では、macOS AppKit、Windows Win32、Linux Wayland / X11 の window lifecycle を調べ、smoke runner を固定 size window から OS window manager が与える resize / close / event pump を受ける構造へ寄せた。`WindowOptions.resize = true`、`ScaleMode::UpperLeft`、`set_target_fps 60`、current window size の監視、resize 後の exact-size RGB0 buffer redraw、letterbox-aware hit test、zero-size surface の `Unavailable` model を追加し、close button または Escape で process が正常終了するようにした。調査内容と backend contract は `doc/neplg2/gui_native_platform_behavior.md` に分けて記録する。

この crate も現時点では正式な `std/gui::GuiHost` 実装ではない。次の native checkpoint では `std/gui` の host contract が固まった後、`nepl-gui-native` の framebuffer renderer を `platforms/gui/native` 側の `present` 実装へ寄せる。

Mobile:

- `LifecycleEvent`、touch id、IME composition、safe area、orientation、accessibility bridge を backend として接続する。
- Keyboard event だけで text input を扱わない。

## Documentation Rules

- 仕様と現状実装を分けて書く。
- platform 差は `GuiCapabilities` と target notes で説明する。
- doc comment は日本語で書き、module 先頭に目的、実装、注意、計算量を置く。
- callback を避ける理由、TUI を backend として扱う理由、unsupported operation を `Result` にする理由は繰り返し明示する。

## Test Policy

最小 test 群:

```text
tests/stdlib/gui_core.n.md
    geometry arithmetic
    color constructors
    event/capability enum smoke
    text-grid surface kind

tests/stdlib/gui_app.n.md
    ActionId based update replay
    GuiEffect batch construction
    ViewNode snapshot

tests/stdlib/gui_layout.n.md
    TextMeasurer injection
    constraint clamp
    invalid constraint error
    ViewTreeArena to LayoutTreeArena linear connector
    invalid arena layout constraints
    StackLayoutPolicy vertical sibling spacing
    stack nested parent-local offset
    invalid stack policy error
    stack vertical center alignment
    stack vertical stretch alignment
    stack horizontal end alignment
    stack reject overflow error

tests/stdlib/gui_widget.n.md
    Button action event generation
    disabled action suppression
    semantic node generation

tests/stdlib/gui_tree.n.md
    bounded ViewTree child insertion
    ViewTreeArena nested insertion and owner-recovery error
    first focusable WidgetId
    LayoutTree child insertion
    LayoutTreeArena nested insertion

tests/stdlib/gui_focus.n.md
    next / previous focus traversal
    disabled widget exclusion
    missing current focus result

tests/stdlib/gui_routing.n.md
    LayoutTree hit test
    button action lowering
    disabled / outside suppression
    child z-order

tests/stdlib/gui_focus_routing.n.md
    focus route command movement
    focused widget action emission
    disabled / non-action / stale focus ignored

tests/stdlib/gui_keymap.n.md
    default Tab / Shift+Tab / Enter / Space mapping
    KeyUp and unknown key ignored
    custom FocusKeyMap contract

tests/stdlib/gui_diff.n.md
    widget data diff
    tree shape invalidation
    child widget invalidation id
    ViewTreeArena single widget content invalidation
    ViewTreeArena shape / multiple content change invalidation

tests/stdlib/gui_text.n.md
    TextBuffer storage
    checked insert / replace / delete
    TextRunId mapping boundary

tests/stdlib/gui_theme.n.md
    typed color role lookup
    metric validation
    Option FontId

tests/stdlib/gui_accessibility.n.md
    Semantic node state
    bounded semantic tree insertion

tests/stdlib/gui_terminal.n.md
    TextGrid capability
    terminal frame model
    legacy TUI facade bridge

tests/stdlib/gui_terminal_input.n.md
    terminal byte to KeyboardEvent / TextInputEvent normalization
    Space as both key and text
    ESC [ Z as Shift+Tab keyboard normalization
    invalid byte and unsupported control byte handling

tests/stdlib/gui_web_input.n.md
    Web host input empty queue as Result Ok None
    GuiWebEvent wrapper contract for action, pointer, keyboard, text input, and window record polling

tests/stdlib/gui_dirty_region.n.md
    Empty / Rect / Full merge
    bounding rect union
    invalid geometry error

tests/stdlib/gui_dirty_region_set.n.md
    fixed two rect storage
    overflow to Full
    invalid geometry error
```

Web backend が入った後:

```text
node nodesrc/cli.js -i tests/gui_playground --gui-playground-tests -o json=tmp/gui-playground.json
```

## Checkpoint Commit Rule

各 phase は小さく commit する。

1. docs only
2. `core/gui` types + tests
3. `features/gui` + terminal capability bridge
4. `alloc/gui` app model + tests
5. `alloc/gui` layout/widget/accessibility slice + tests
6. TUI backend replacement checkpoint

各 commit 前に、その checkpoint に対応する focused test、`node nodesrc/issues.js check --dir issues`、`git diff --check` を通す。
