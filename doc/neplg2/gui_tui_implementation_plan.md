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

## Phase F5gk: Native window frame pacing config boundary

F5gk では、native smoke window loop の frame pacing を `NativeWindowTargetFps` と `NativeWindowRunLoopConfig.target_fps` に出す。target FPS は `NATIVE_WINDOW_RUN_LOOP_MIN_TARGET_FPS = 1`、`NATIVE_WINDOW_RUN_LOOP_MAX_TARGET_FPS = 240`、`NATIVE_WINDOW_RUN_LOOP_DEFAULT_TARGET_FPS = 60` の typed config とし、`NativeWindowTargetFpsInvalidReason::Zero` / `TooHigh max` により invalid value を明示する。既存 `NativeWindowRunLoopConfig::new` は default 60 を使い、custom FPS は `new_with_target_fps` または raw value 用 helper の validation を通す。

`run_minifb_window_loop` は validation 済みの `target_fps.as_usize` だけを `Window::set_target_fps` に渡す。`set_target_fps 60` の hidden constant、`config.target_fps` の raw 直渡し、invalid FPS clamp、fallback、silent no-op は禁止する。CLI に `--fps N` を追加するが、headless mode では frame pacing を使わないことを usage に明記する。

この phase は frame pacing policy の明示化であり、formal OS wait strategy、queue / timer wait backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。

## Phase F5gl: Native window host-loop run policy boundary

F5gl では、F5gj の bounded runner と F5gk の frame pacing config をつなぎ、native smoke window の long loop が direct unbounded step loop を持たないようにする。`NativeWindowHostLoopTurnSlice` は `1..=4096` の bounded turn budget であり、default は `1` とする。これは現状の one-turn-per-host-update semantics を保つための policy であり、OS wait strategy、timer wait、performance guarantee を表すものではない。

`NativeWindowHostLoopRunPolicy` は validation 済み `turn_slice` を持ち、`NativeWindowRunLoopConfig.host_loop_policy` に保持される。`run_native_window_host_loop_with_policy` は `run_native_window_host_loop_bounded` だけを繰り返し、`Exited` なら terminal reason を返し、`BudgetExhausted` なら同じ runner state で次 slice に進む。`run_native_window_host_loop` は default policy を渡す wrapper になり、`run_minifb_window_loop` は `config.host_loop_policy` を使う。

source policy は policy type、range constants、default policy delegation、`run_minifb_window_loop -> run_native_window_host_loop_with_policy`、policy runner -> bounded runner を固定する。`usize::MAX`、sleep、queue、timer、fallback、silent no-op、DOM / Canvas / video memory transport は導入しない。

この phase は native host-loop run policy の明示化であり、formal OS wait strategy、queue / timer wait backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。

## Phase F5gm: Native window host-loop turn evidence boundary

F5gm では、`NativeWindowHostLoopTurn::Continue` を単なる継続 signal から、host turn の実行証拠を持つ value に拡張する。F5gl で bounded run policy は入ったが、`Continue` が pump-only と present-frame を潰していると、future native OS wait strategy が frame-paced wait と surface-unavailable pump behavior を型で分岐できない。

`NativeWindowHostLoopContinueEvidence` は `PumpedEventsOnly window_size size_changed` と `PresentedFrame presentation window_size size_changed` を持つ。`PresentedFrame` の `presentation` は `NativeWindowBackendLoopPresentation` の value evidence であり、pixel borrow は持たない。`PresentedFrame` evidence は host present が成功した後だけ返し、present error 時には evidence を返さない。

bounded runner と policy runner はこの evidence をまだ消費せず、`Continue _` として turn count だけを進める。これは future wait decision 用の evidence 境界であり、formal OS wait strategy、queue / timer wait backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、sleep、queue、timer、DOM / Canvas / video memory transport も導入しない。

## Phase F5gn: Native window host-loop wait decision boundary

F5gn では、F5gm の `NativeWindowHostLoopContinueEvidence` を future scheduler の入力分類として `NativeWindowHostLoopWaitDecision` に写像する。これは OS wait strategy の実装ではなく、bounded runner が最後の継続 turn を「host event を待つべき class」または「frame interval を待つべき class」として保持できるようにする境界である。

`NativeWindowHostLoopWaitDecision` は `WaitForHostEvent window_size size_changed` と `WaitForFrameInterval presentation window_size size_changed` を持つ。どちらも value-only evidence であり、pixel borrow、host handle、scheduler state は持たない。`native_window_host_loop_wait_decision` は `PumpedEventsOnly` を `WaitForHostEvent`、`PresentedFrame` を `WaitForFrameInterval` へ全域的に写す pure helper であり、`Result`、fallback、silent no-op を使わない。

`run_native_window_host_loop_bounded` は `BudgetExhausted completed_turns last_wait_decision` を返す。zero budget では `None`、`Continue evidence` を処理した turn では最後の evidence から得た wait decision を `Some` にする。実 wait dispatch は次 checkpoint の F5go で扱い、F5gn では分類結果の生成と保持までを固定する。

この phase は wait decision classification までであり、formal OS wait strategy、queue / timer wait backend、`Duration` 計算、`std::thread::sleep`、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、minifb / DOM / Canvas / video memory transport も導入しない。

## Phase F5go: Native window host-loop wait dispatch boundary

F5go では、F5gn の `NativeWindowHostLoopWaitDecision` を `NativeWindowRunLoopHost::wait_after_budget_exhausted` に渡し、policy runner が bounded slice の間で host wait boundary を必ず通るようにする。`NativeWindowRunLoopHost` は `WaitError` associated type を持ち、wait hook failure は `NativeWindowHostLoopError::HostWaitFailed` として event pump failure、present failure、host action failure から分離する。

`NativeWindowHostLoopWaitOutcome` は wait hook の outcome evidence であり、`HostEventPumpAlreadyPaced`、`FramePresentAlreadyPaced`、`FrameIntervalTimerRegistered` を持つ。minifb の `Window::update` と `update_with_buffer` は `set_target_fps` による rate limit を内部で通るため、minifb adapter の wait hook は追加の `update_with_buffer`、`std::thread::sleep`、`Duration`、queue、timer を実行せず、すでに pace 済みであることだけを typed outcome として返す。`FrameIntervalTimerRegistered` は timer registration evidence であり、frame pacing 完了を意味しない。これにより event pump の二重実行、frame pacing の二重適用、timer registration 成功の wait completion 偽装を避ける。

`run_native_window_host_loop_with_policy` は `BudgetExhausted last_wait_decision = Some decision` で host wait hook を呼ぶ。`last_wait_decision = None` は `NativeWindowHostLoopError::WaitDecisionMissing` として fail closed にし、zero budget / missing wait evidence を silent no-op として飲み込まない。

この phase は wait dispatch boundary までであり、formal OS wait strategy、queue / timer wait backend、real timer registration、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、extra minifb update、DOM / Canvas / video memory transport も導入しない。

## Phase F5gp: Native window host-loop scheduler slice boundary

F5gp では、F5go の long runner 内部にあった bounded slice execution と wait dispatch を、formal native scheduler が 1 slice ずつ呼べる API へ切り出す。

`NativeWindowHostLoopSchedulerState` は `NativeWindowHostLoopRunnerState` を所有し、initial title initialization を slice 間で保持する。`NativeWindowHostLoopSchedulerSliceResult` は `Exited` と `Waited` を分け、`Waited` では completed turn count、wait decision、wait request、wait outcome を保持する。`run_native_window_host_loop_scheduler_slice_with_policy` は bounded run と wait dispatch の 1 cycle を実行し、`run_native_window_host_loop_with_policy` はこの scheduler slice API を反復する wrapper になる。

この phase は external scheduler-facing slice boundary までであり、actual OS wait strategy、queue / timer wait backend、real timer registration、`Duration`、`std::thread::sleep`、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、extra minifb update、DOM / Canvas / video memory transport も導入しない。

## Phase F5gq: Native window host-loop wait request plan boundary

F5gq では、F5gp の wait decision を backend-facing wait request plan へ変換する。`NativeWindowHostLoopWaitRequest` は host event wait と frame interval wait を区別し、frame interval wait は validated `NativeWindowTargetFps` から計算した `NativeWindowFrameIntervalRequest` を持つ。

`NativeWindowFrameIntervalRequest` は `nanos_per_frame` と `remainder_nanos_per_second` を保持し、暗黙 rounding、clamp、sentinel を使わない。F5gq は decision から request plan を作る境界であり、F5gr 以降の `NativeWindowRunLoopHost::wait_after_budget_exhausted` は request から生成される instruction を受け取る。

この phase は wait request plan boundary までであり、actual OS wait strategy、queue / timer wait backend、real timer registration、`Duration`、`std::thread::sleep`、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、extra minifb update、DOM / Canvas / video memory transport も導入しない。

## Phase F5gr: Native window host-loop wait strategy instruction boundary

F5gr では、F5gq の wait request plan を `NativeWindowHostLoopWaitInstruction` へ変換する。`NativeWindowHostLoopWaitStrategyState` は scheduler slice 間で frame pacing target FPS と remainder accumulator を持つ。target FPS が変わった場合は accumulator を reset し、同じ target FPS の場合だけ remainder を継続する。

frame interval instruction の `wait_nanos` は `nanos_per_frame` または `nanos_per_frame + 1` に限定する。accumulator invariant は `0 <= remainder < fps` とし、saturating、clamp、sentinel、zero-fill fallback は使わない。host event instruction は event payload、queue owner、poll result を持たず、window size と `size_changed` evidence だけを持つ。

`NativeWindowRunLoopHost::wait_after_budget_exhausted` は request ではなく instruction を受け取る。scheduler slice は wait hook が成功した後だけ `NativeWindowHostLoopSchedulerState` の wait strategy state を次状態へ進める。wait hook 失敗時に accumulator を進めないことで、error path が frame pacing state を silent に消費しない。

この phase は wait strategy instruction boundary までであり、actual OS wait strategy、queue / timer wait backend、real timer registration、`Duration`、`std::thread::sleep`、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、extra minifb update、DOM / Canvas / video memory transport も導入しない。

## Phase F5gs: Native window host-loop thread wait backend boundary

F5gs では、F5gr の wait instruction を native thread sleep backend に渡す実行境界を追加する。これは minifb smoke backend の wait hook ではない。minifb は `Window::set_target_fps` によって `update` / `update_with_buffer` 内部で pace されるため、F5gs thread wait を minifb hook へ接続すると二重 pacing になる。

`NativeWindowHostLoopThreadSleeper` は injected sleeper interface であり、test は scripted sleeper、std native helper は `StdNativeWindowHostLoopThreadSleeper` を使う。`execute_native_window_host_loop_thread_wait_with_sleeper` は frame interval instruction の `wait_nanos` を再検査し、`nanos_per_frame` または `nanos_per_frame + 1` の場合だけ sleeper を 1 回呼ぶ。不一致は `FrameIntervalWaitNanosMismatch` で fail closed にする。

host event wait は `HostEventWaitUnsupported` を返す。OS event queue / selector / message pump backend がない状態で host event wait を thread sleep や busy loop に変換しない。

この phase は native thread wait backend boundary までであり、host event queue、timer registration、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、extra minifb update、DOM / Canvas / video memory transport も導入しない。

## Phase F5gt: Native window host-loop timer registration backend boundary

F5gt では、F5gr の wait instruction を native timer registration backend へ渡す実行境界を追加する。これは thread sleep backend ではなく、formal native scheduler が frame interval wait を timer registration として表すための別境界である。

`NativeWindowHostLoopTimerRegistrar` は injected backend interface であり、host backend から raw `u32` timer id を返す。`execute_native_window_host_loop_timer_registration_with_registrar` は frame interval instruction の `wait_nanos` を再検査し、`nanos_per_frame` または `nanos_per_frame + 1` の場合だけ registrar を 1 回呼ぶ。registrar が返した raw id は `NativeWindowHostLoopTimerRegistrationId` へ変換する前に正値であることを検査する。raw id `0` は `InvalidTimerRegistrationId` として fail closed にする。

host event wait は `HostEventTimerRegistrationUnsupported` を返す。OS event queue / selector / message pump backend がない状態で host event wait を timer registration、thread sleep、busy loop、silent no-op に変換しない。

この phase は native timer registration backend boundary までであり、host event queue、selector、message pump、real OS timer backend、minifb wait hook への接続、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、extra minifb update、DOM / Canvas / video memory transport も導入しない。

## Phase F5gu: Native window host-loop event queue wait backend boundary

F5gu では、F5gr の wait instruction のうち `WaitForHostEvent` だけを event queue wait backend へ渡す実行境界を追加する。これは F5gs の thread sleep backend、F5gt の timer registration backend と対になる host-event wait contract であり、frame interval wait を扱わない。

`NativeWindowHostLoopEventQueueWaiter` は injected backend interface であり、`wait_for_host_event` は `NativeWindowSize` と `size_changed` evidence を受け取る。`execute_native_window_host_loop_event_queue_wait_with_waiter` は host event instruction の場合だけ waiter を 1 回呼び、成功時に `HostEventReady` を返す。waiter failure は `WaiterFailed` として元の error value を保持する。

frame interval wait は `FrameIntervalEventQueueWaitUnsupported` を返す。event queue wait backend は timer registration、thread sleep、busy loop、silent no-op へ変換しない。

この phase は event queue wait backend boundary までであり、real OS event queue / selector / message pump adapter、real OS timer backend、minifb wait hook への接続、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、extra minifb update、DOM / Canvas / video memory transport も導入しない。

## Phase F5gv: Native window host-loop event queue normalized status adapter boundary

F5gv では、F5gu の `NativeWindowHostLoopEventQueueWaiter` に接続できる raw status adapter 境界を追加する。ここで扱う raw status は OS API の値そのものではなく、platform adapter が `nepl-gui-native` の境界用に正規化して返す internal normalized status である。

`NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY` は host event が ready になったことだけを表す。`NativeWindowHostLoopEventQueueStatusAdapter` は `wait_for_host_event_raw_status` で normalized raw status を返す。`wait_native_window_host_loop_event_queue_raw_status_with_adapter` は adapter を 1 回呼び、ready status 以外を `InvalidRawStatus` として fail closed にする。adapter 自身の失敗は `AdapterFailed` として元の error value を保持する。

`NativeWindowHostLoopEventQueueStatusWaiter` は status adapter を F5gu の `NativeWindowHostLoopEventQueueWaiter` へ接続する wrapper である。これにより F5gu executor 経由で host event wait を実行できるが、frame interval instruction は F5gu executor が先に reject するため status adapter を呼ばない。

この phase は normalized status adapter boundary までであり、real OS event queue / selector / message pump adapter、minifb wait hook、real OS timer backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、extra minifb update、DOM / Canvas / video memory transport も導入しない。

## Phase F5gw: Native window host-loop message pump adapter boundary

F5gw では、F5gv の normalized status adapter に接続する message pump adapter 境界を追加する。これは OS 固有 event queue / selector / message pump の実行結果を、`NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY` と typed error へ写す platform adapter boundary である。

`NativeWindowHostLoopMessagePumpAdapter` は `pump_host_messages` を 1 回だけ実行し、成功時だけ `NativeWindowHostLoopMessagePumpStatusAdapter` が F5gv の ready status を返す。pump failure は `PumpFailed` として保持する。minifb smoke backend では `MinifbNativeWindowHostLoopMessagePumpAdapter` が `window.update` を実行し、`wait_minifb_window_host_event_message_pump` が F5gu / F5gv の event queue waiter 経由で host wait outcome へ戻す。

この phase は message pump adapter boundary までであり、real OS timer backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。frame interval wait は引き続き frame pacing 側の責務であり、message pump adapter が timer registration、thread sleep、busy loop、silent no-op へ変換することは禁止する。fallback、DOM / Canvas / video memory transport も導入しない。

subagent review:

- Darwin the 2nd に F5gw 計画を渡し、F5gv の normalized status boundary へ接続すること、OS 固有 status / failure を typed Result / enum に分離すること、real OS timer backend を別 slice に残すこと、source policy の観点で確認させた。結果は `PLAN_APPROVED` である。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gx: Native window host-loop frame interval timer registration outcome boundary

F5gx では、F5gt の timer registration backend を `NativeWindowHostLoopWaitOutcome` へ接続する。ただし timer registration 成功は wait completion ではないため、`FramePresentAlreadyPaced` には写さない。

`NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered` は `presentation`、`window_size`、`size_changed`、`wait_nanos`、`timer_registration_id` を保持する。`execute_native_window_host_loop_timer_registration_wait_with_registrar` は `WaitForFrameInterval` instruction だけを F5gt executor へ渡し、成功時だけこの outcome を返す。`WaitForHostEvent` は F5gt の `HostEventTimerRegistrationUnsupported` を保持し、message pump path に残す。

minifb smoke backend の wait hook は F5gx path へ接続しない。minifb の frame interval は引き続き `Window::set_target_fps` / `update_with_buffer` の pacing authority により `FramePresentAlreadyPaced` として扱う。F5gx は external scheduler が timer registration evidence を受け取るための boundary であり、actual timer fire、thread sleep、busy loop、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、wait completion の偽装は禁止する。

subagent review:

- Darwin the 2nd の初回 plan review は、timer registration 成功を `FramePresentAlreadyPaced` へ写す計画を `PLAN_BLOCKED` とした。
- 修正版では `FrameIntervalTimerRegistered` outcome を追加し、timer registration evidence と pacing completion を型で分離する方針に変更した。再 review は `PLAN_APPROVED` である。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gy: Native window host-loop timer fire/wakeup backend boundary

F5gy では、F5gx の `FrameIntervalTimerRegistered` outcome を受け取り、backend が返す fired timer id を検証して `FrameIntervalTimerFired` evidence へ進める。timer registration 成功と timer fire 成功を混ぜず、registration id と fired id の完全一致だけを待機完了 evidence として扱う。

`NativeWindowHostLoopTimerFireWaiter` は `wait_for_timer_fire` で registered timer id を受け取り、backend が観測した fired raw id を返す。`execute_native_window_host_loop_timer_fire_wait_with_waiter` は `HostEventPumpAlreadyPaced` と `FramePresentAlreadyPaced` を unsupported として waiter を呼ばずに拒否する。`FrameIntervalTimerRegistered` の場合だけ waiter を 1 回呼び、fired raw id `0` は `InvalidFiredTimerRegistrationId`、registered id と異なる raw id は `FiredTimerRegistrationMismatch` として fail closed にする。

この phase は timer fire / wakeup backend boundary までであり、OS 固有 timer API、selector wakeup ownership、minifb wait hook 接続、thread sleep、busy loop、scheduler resume policy、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、mismatched id の成功扱いは禁止する。

subagent review:

- Darwin the 2nd に F5gy 計画を渡し、registered timer id と fired timer id の照合境界にすること、already-paced outcome を timer fire input として成功扱いしないこと、source policy の観点で確認させた。結果は `PLAN_APPROVED` である。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5gz: Native window host-loop timer wakeup executor boundary

F5gz では、F5gx の timer registration wait 境界と F5gy の timer fire wait 境界を、1 回の backend wakeup executor として合成する。registration 成功と fire 成功の責務は混ぜず、どちらの段階で失敗したかを `NativeWindowHostLoopTimerWakeError` で保持する。

`NativeWindowHostLoopTimerWakeError` は `RegistrationFailed NativeWindowHostLoopTimerRegistrationError` と `FireFailed NativeWindowHostLoopTimerFireError` を持つ。`execute_native_window_host_loop_timer_wakeup_with_backend` は、まず `execute_native_window_host_loop_timer_registration_wait_with_registrar` を呼び、registration error では waiter を呼ばずに `RegistrationFailed` を返す。registration outcome が得られた場合だけ `execute_native_window_host_loop_timer_fire_wait_with_waiter` を呼び、fire error を `FireFailed` として返す。

この phase は scheduler が後続で利用できる typed wakeup boundary を作る checkpoint であり、OS 固有 timer API、selector wakeup ownership、minifb wait hook 接続、scheduler resume policy、thread sleep、busy loop、message pump、event queue substitution、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。fallback、silent no-op、registration error 後の waiter 呼び出し、direct `register_timer_nanos` / `wait_for_timer_fire` の再実装は禁止する。

subagent review:

- Darwin the 2nd に F5gz 計画を渡し、registration/fire の error 分離、helper 合成に限定すること、source policy の観点を確認させた。結果は `PLAN_APPROVED` である。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5ha: Native window host-loop scheduler timer resume gate boundary

F5ha では、scheduler の long runner が `NativeWindowHostLoopSchedulerSliceResult::Waited` を無条件に次 loop へ進める状態を止める。`FrameIntervalTimerRegistered` は timer reservation evidence であり、timer fire evidence ではないため、long runner は `TimerFireResumeRequired` として fail closed する。

`NativeWindowHostLoopSchedulerResumeState` は `Ready NativeWindowHostLoopSchedulerResumeReady` と `WaitingForFrameIntervalTimer` を持つ。`native_window_host_loop_scheduler_resume_state_from_wait_outcome` は `HostEventPumpAlreadyPaced` と `FramePresentAlreadyPaced` を ready に写し、`FrameIntervalTimerRegistered` を waiting に写す。`native_window_host_loop_scheduler_resume_ready_from_timer_fire` は `NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired` だけを ready evidence へ写す。

`run_native_window_host_loop_with_policy_and_target_fps` は `Waited` の outcome を resume gate に通し、ready の場合だけ次 slice へ進む。`WaitingForFrameIntervalTimer` の場合は `NativeWindowHostLoopError::TimerFireResumeRequired` を返し、次の event poll、present、sleep、queue、busy loop へ進まない。

この phase は resume gate までであり、OS 固有 timer API、selector wakeup ownership、minifb wait hook 接続、real timer wait、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5ha 計画を渡し、`Waited` の無条件継続を止める root-cause slice であること、timer fire wait 実装へ踏み込まないこと、source policy の観点を確認させた。結果は `PLAN_APPROVED` である。
- 実装後に subagent review を受け、指摘があれば修正する。

## Phase F5hb: Native window host-loop std deadline timer adapter boundary

F5hb では、F5gt / F5gy / F5gz で用意した timer registration / fire wait contract を、native std deadline timer adapter の状態として実装する。`NativeWindowHostLoopDeadlineTimerAdapter` は clock と sleeper を注入され、positive timer id、active timer、deadline nanos を所有する。

adapter は active timer overlap、active timer missing、registration id overflow、deadline arithmetic overflow、clock failure、sleeper failure、mismatched fire id をそれぞれ enum error として返す。成功時だけ active timer を消費し、sleeper failure では active timer を保持する。これにより timer fire completion を synthetic outcome や silent no-op で作らず、後続 selector / message loop backend が同じ contract を実装できる。

std 実装は `StdNativeWindowHostLoopDeadlineTimerClock` と `StdNativeWindowHostLoopDeadlineTimerSleeper` に閉じ込める。実時間 sleep はこの adapter slice のみで許可し、unit test は scripted clock / sleeper により deterministic に検査する。

この phase は std deadline timer adapter boundary までであり、selector wakeup ownership、OS message loop timer、minifb wait hook への接続、`Window::set_target_fps` の置換、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。minifb smoke backend は引き続き `FramePresentAlreadyPaced` を返す。

subagent review:

- 初回 plan review では Darwin the 2nd から `CHANGES_REQUESTED` が出た。minifb は `Window::set_target_fps` が pacing authority であるため、この slice で minifb wait hook へ timer / sleep を接続しないこと、selector ownership と std deadline wait adapter を分けること、overlap / missing / mismatch / id overflow / deadline overflow を enum error にすること、test は injected clock / sleeper で行うことが指摘された。
- 実装はこの指摘に合わせ、F5hb を std deadline timer adapter boundary に絞る。

## Phase F5hc: Native window host-loop timer fired wait outcome boundary

F5hc では、`NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired` を `NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired` として host-loop scheduler に返せるようにする。これにより、selector / message loop timer backend が後続で実装された時に、registration-only state と fire-completed state を同じ wait outcome contract で区別できる。

`FrameIntervalTimerRegistered` は `WaitingForFrameIntervalTimer` のまま維持する。`FrameIntervalTimerFired` は `NativeWindowHostLoopSchedulerResumeReady::FrameIntervalTimerFired` に写し、long runner が次の event poll へ進める ready evidence として扱う。

generic helper と deadline timer adapter helper は、registration failure と fire wait failure を `NativeWindowHostLoopTimerWakeError` の別 variant として保持する。成功時だけ timer fired evidence を wait outcome へ写す。fallback、silent no-op、busy loop、thread sleep、message pump、event queue substitution は使わない。

この phase は timer fired wait outcome boundary までであり、selector wakeup ownership、OS message loop timer、minifb wait hook への接続、`Window::set_target_fps` の置換、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5hc 計画を渡し、F5hb 後の root-cause slice として `APPROVED` を得た。required constraints は、registered は waiting のままにすること、fired だけが resume すること、error separation を維持すること、minifb を触らないこと、source policy で registered / fired の両 path を検査することである。

## Phase F5hd: Native window host-loop wait owner boundary

F5hd では、host-loop wait instruction を host event queue wait backend と frame interval deadline timer backend のどちらへ渡すかを所有する composition boundary を追加する。これは selector / message-loop timer 実装ではなく、F5gu/F5gv/F5gw host event path と F5hb/F5hc timer path を 1 つの typed owner で分岐する checkpoint である。

`NativeWindowHostLoopWaitOwner` は event queue waiter と deadline timer adapter を所有する。`NativeWindowHostLoopWaitOwnerError` は `EventQueueWaitFailed` と `FrameIntervalTimerWakeFailed` を分け、lower error を丸ごと保持する。`execute_native_window_host_loop_wait_with_owner` は `WaitForHostEvent` を event queue waiter へ、`WaitForFrameInterval` を deadline timer wakeup wait helper へだけ渡す。

host event path の成功は `HostEventPumpAlreadyPaced` に写し、frame interval path の成功は `FrameIntervalTimerFired` をそのまま返す。unit test と source policy は、host event path が timer clock / sleeper に触れないこと、frame interval path が event queue waiter に触れないこと、両 error stage が潰れないことを固定する。

この phase は wait owner composition boundary までであり、selector ownership、OS message loop timer、minifb wait hook の pacing 置換、`Window::set_target_fps` の置換、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。event queue backend が timer を代替すること、timer backend が queue wait を代替すること、fallback / silent no-op / busy loop は禁止する。

subagent review:

- Darwin the 2nd に F5hd 計画を渡し、F5hb/F5hc 後の root-cause slice として `PLAN_APPROVED` を得た。required constraints は、lower error 全体を wrapper に保持すること、host event と frame interval の branch exclusivity を test すること、minifb を変更しないことである。

## Phase F5he: Native minifb frame pacing authority boundary

F5he では、minifb smoke backend が持つ `Window::set_target_fps` based pacing を typed authority として明示する。これは F5hd wait owner を minifb wait hook へ接続する phase ではない。現在の minifb host event wait は `window.update` による message pump adapter を使い、frame interval wait は minifb の internal target-fps pacing authority が `FramePresentAlreadyPaced` を返す。

`NativeWindowMinifbFramePacingAuthority` は validated `NativeWindowTargetFps` を保持する。`WaitForFrameInterval` instruction の `frame_interval.target_fps` が authority target と一致し、`wait_nanos` が `nanos_per_frame` または remainder carry 分の `nanos_per_frame + 1` の場合だけ `FramePresentAlreadyPaced` を返す。不一致は `FrameIntervalTargetFpsMismatch` または `FrameIntervalWaitNanosMismatch` として fail closed にする。

`run_minifb_window_loop` は `minifb_native_window_frame_pacing_authority config.target_fps` で authority を作り、`configure_minifb_window_frame_pacing` が `authority.target_fps_usize` を `window.set_target_fps` へ渡す。`set_target_fps 0` は minifb 内部 wait を無効化するため、この phase では禁止する。`set_target_fps 60` や raw `config.target_fps` の直渡しも禁止する。

この phase は minifb internal target-fps pacing authority の明文化までであり、selector ownership、OS message-loop timer、F5hd wait owner の minifb hook 接続、`Window::set_target_fps` の置換、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。future deadline timer owner と minifb internal pacing は同時に frame interval wait authority になってはいけない。

subagent review:

- Darwin the 2nd の初回 plan review は、`set_target_fps 0` による即時置換を `CHANGES_REQUESTED` とした。現在の `WaitForHostEvent` は `window.update` を通るため、minifb internal pacing を無効化すると tight loop になるためである。
- 改訂 plan では、F5he を minifb internal target-fps pacing authority boundary に絞り、typed `NativeWindowTargetFps` を保持する authority helper、`FramePresentAlreadyPaced` の authority 経由化、`set_target_fps 0` 禁止、deadline timer owner 未接続を条件として `PLAN_APPROVED` を得た。

## Phase F5hf: Native frame interval wait authority mode boundary

F5hf では、frame interval wait の authority が minifb internal target-fps pacing と host-owned deadline timer のどちらなのかを typed mode として表す。これは selector / message-loop timer ownership の実装ではなく、そこへ進む前に二重 authority を拒否する safety boundary である。

`NativeWindowFrameIntervalWaitAuthorityMode` は `MinifbInternalTargetFps target_fps` と `HostOwnedDeadlineTimer` を持つ。`combine_native_window_frame_interval_wait_authority_mode` は同じ minifb target fps 同士と host-owned deadline timer 同士だけを受け入れ、minifb と host-owned の組み合わせ、または target fps が異なる minifb mode を `ConflictingFrameIntervalAuthorities` として拒否する。

`validate_native_window_frame_interval_wait_authority_mode` は minifb mode の場合だけ instruction の `frame_interval.target_fps` と authority target を照合する。host-owned deadline timer mode は、すでに作られた wait instruction を timer owner が消費するための compatibility check であり、`FramePresentAlreadyPaced`、`FrameIntervalTimerRegistered`、`FrameIntervalTimerFired` のどれも生成しない。

`NativeWindowMinifbFramePacingAuthority` は自分の authority mode を返し、`FramePresentAlreadyPaced` を返す前にこの validation helper を通る。F5hf でも minifb wait hook は F5hd wait owner / std deadline timer adapter へ接続しない。`set_target_fps 0`、fallback、silent no-op、busy loop は禁止する。

subagent review:

- Darwin the 2nd に F5hf 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、mode type を pure / non-platform-specific にすること、host-owned mode validation が wait evidence を生成しないこと、minifb authority path が新 helper を実際に使うこと、minifb と host-owned deadline timer の conflict を双方向で拒否すること、docs が selector/message-loop 実装ではないと明記することである。

## Phase F5hg: Native wait owner frame interval authority connection boundary

F5hg では、F5hf の authority mode を F5hd の formal wait owner の frame interval branch に接続する。これは macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd の実装ではない。selector / message-loop timer ownership へ進む前に、formal owner path が `HostOwnedDeadlineTimer` authority であることを明示し、minifb internal pacing authority が混入した場合に timer registration より前で fail closed にする boundary である。

`NativeWindowHostLoopWaitOwner` は `frame_interval_wait_authority_mode` として `HostOwnedDeadlineTimer` を返す。`execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode` は frame interval branch で、owner の active authority と requested authority を `combine_native_window_frame_interval_wait_authority_mode` に通し、その後 `validate_native_window_frame_interval_wait_authority_mode` を通る。この検査に成功した場合だけ `execute_native_window_host_loop_deadline_timer_wakeup_wait_with_adapter` を呼ぶ。

authority mismatch は `NativeWindowHostLoopWaitOwnerError::FrameIntervalAuthorityFailed` として返し、deadline timer registration、clock read、sleeper call、active timer mutation は起こさない。host event wait は frame interval authority を参照せず、event queue waiter だけへ進む。既存の `execute_native_window_host_loop_wait_with_owner` は owner 自身の `HostOwnedDeadlineTimer` mode を渡す wrapper として残す。

F5hg でも minifb wait hook は F5hd wait owner / std deadline timer adapter へ接続しない。`set_target_fps 0`、fallback、silent no-op、busy loop は禁止する。

subagent review:

- Darwin the 2nd に F5hg 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、deadline timer registration / clock read / sleeper call / active timer mutation より前に authority を検査すること、minifb authority rejection で timer state が無変更であることを test すること、host event wait が authority を参照しないこと、既存 helper を explicit-authority helper の wrapper にすること、minifb path を変更しないことである。

## Phase F5hh: Native run-loop frame interval wait backend selection boundary

F5hh では、native run-loop config が frame interval wait backend authority を明示的に持つ。`NativeWindowRunLoopFrameIntervalWaitBackend` は `MinifbInternalTargetFps` と `HostOwnedDeadlineTimer` を持ち、default は minifb smoke backend の現在の authority である `MinifbInternalTargetFps` とする。

`NativeWindowRunLoopFrameIntervalWaitBackend::authority_mode` は F5hf の `NativeWindowFrameIntervalWaitAuthorityMode` へ変換する。minifb runner は `validate_minifb_window_run_loop_frame_interval_wait_backend` により、active authority を minifb internal target-fps pacing として扱い、requested backend authority と `combine_native_window_frame_interval_wait_authority_mode` で照合する。`HostOwnedDeadlineTimer` が minifb runner に指定された場合は `NativeWindowRunLoopError::FrameIntervalWaitBackendUnsupported` として fail closed にする。

この validation は `NativeWindowBackendLoop::new_for_scale`、`Window::new`、`Window::set_target_fps`、host construction、loop execution より前に行う。unsupported backend を minifb internal pacing へ fallback しない。これにより future selector / message-loop deadline timer backend を config に追加しても、minifb smoke path と二重 authority にならない。

F5hh は backend selection boundary までであり、macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd は実装しない。minifb wait hook を F5hd wait owner / std deadline timer adapter へ接続しない。`set_target_fps 0`、fallback、silent no-op、busy loop は禁止する。

subagent review:

- Darwin the 2nd に F5hh 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、config backend を bool / string にしないこと、default を minifb internal target-fps pacing にすること、minifb runner で `HostOwnedDeadlineTimer` を window 作成前に拒否すること、unsupported backend を fallback しないこと、typed runner / requested backend / authority reason を error に保持すること、source policy で validation ordering と minifb path の deadline timer 禁止を固定することである。

## Phase F5hi: Native host-owned deadline wait run-loop host wrapper boundary

F5hi では、F5hg の `NativeWindowHostLoopWaitOwner` を `NativeWindowRunLoopHost` contract の wait hook として使う wrapper を追加する。これは future native OS backend や deterministic test backend が、host event wait と frame interval timer wait を formal owner に委譲するための境界である。

`NativeWindowHostOwnedDeadlineWaitRunLoopHost` は inner `NativeWindowRunLoopHost` と `NativeWindowHostLoopWaitOwner` を所有する。`poll_event_snapshot`、`set_window_title`、`pump_events_only`、`present_frame` は inner host へ委譲し、`wait_after_budget_exhausted` だけを `execute_native_window_host_loop_wait_with_owner` へ渡す。`EventError` と `PresentError` は inner host の型を保持し、`WaitError` は `NativeWindowHostLoopWaitOwnerError` のまま返す。文字列化、fallback、silent no-op は行わない。

F5hi でも minifb runner はこの wrapper を使わない。minifb smoke backend は F5hh の通り `Window::set_target_fps` を pacing authority とし、host-owned deadline timer backend へ暗黙接続しない。macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は後続で扱う。

subagent review:

- Darwin the 2nd に F5hi 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、wrapper wait が inner host の wait hook を呼ばず `execute_native_window_host_loop_wait_with_owner` だけを使うこと、non-wait operation が inner host error 型を保持して委譲されること、wait error を typed owner error として保持すること、host-event wait と frame-interval wait の owner dispatch を test すること、minifb path に wrapper / wait owner / deadline timer adapter を混入させないことである。

## Phase F5hj: Native interruptible deadline wait boundary

F5hj では、real selector / message-loop timer backend へ進む前に、frame interval wait が deadline だけでなく host event readiness でも wake できる semantic boundary を追加する。既存の deadline timer adapter は timer-only であり、そのまま OS backend へ接続すると入力 event が frame deadline まで遅延しうるため、host event と deadline のどちらで起きたかを enum で保持する。

`NativeWindowHostLoopInterruptibleDeadlineWaiter` は host event だけを待つ `wait_for_host_event` と、deadline または host event readiness を待つ `wait_until_deadline_or_host_event` を持つ。`NativeWindowHostLoopInterruptibleDeadlineWaitAdapter` は clock、waiter、positive timer id cursor を所有し、`execute_native_window_host_loop_interruptible_deadline_wait_with_adapter` が `NativeWindowHostLoopWaitOutcome` へ写す。

`WaitForFrameInterval` は `wait_nanos == nanos_per_frame || nanos_per_frame + 1` を clock read / id advance / waiter call より前に検査する。deadline に到達した場合だけ `FrameIntervalTimerFired` を返す。host event readiness で wake した場合は `HostEventPumpAlreadyPaced` を返し、これは「host event readiness により loop が polling へ戻れる」ことを表す。timer fired evidence ではない。candidate timer id は wait 開始前に advance されるため、host event で中断された場合も raw id は再利用しない。

error は host-event wait failure、frame wait nanos mismatch、timer id overflow、clock failure、deadline overflow、interruptible frame wait failure を enum variant として分ける。fallback、timer-only wait への代替、thread sleep、busy loop、minifb internal pacing、synthetic fired evidence は禁止する。

F5hj でも minifb runner はこの adapter を使わない。macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd の platform-specific 実装は後続で扱う。

subagent review:

- Darwin the 2nd に F5hj 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、explicit executor/helper が `NativeWindowHostLoopWaitOutcome` を返すこと、frame wait の wait-nanos validation が clock/id/waiter side effect より前であること、host event wake を timer fired と偽装しないこと、distinct error stages を保つこと、fallback / thread sleep / busy loop / minifb internal pacing / synthetic fired evidence を禁止すること、source policy で new boundary と minifb path の分離を固定することである。

## Phase F5hk: Native interruptible deadline wait run-loop host wrapper boundary

F5hk では、F5hj の interruptible deadline wait adapter を `NativeWindowRunLoopHost` の wait hook として使える wrapper を追加する。wrapper は inner `NativeWindowRunLoopHost` と interruptible wait adapter を所有し、event polling、title update、pump-only、present は inner host に委譲する。budget exhaustion 後の wait だけは inner wait hook を呼ばず、`execute_native_window_host_loop_interruptible_deadline_wait_with_adapter` へ渡す。

F5hk は future native OS backend / deterministic test backend が interruptible wait semantics を run-loop host contract から使うための接続境界である。minifb runner には接続しない。macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は後続で扱う。

subagent review:

- Darwin the 2nd に F5hk 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、F5hi を変更せず別 wrapper として追加すること、wait hook が interruptible helper だけを呼ぶこと、inner wait hook を呼ばないことを test すること、deadline reached / host event ready の evidence を区別すること、typed wait error を保持すること、minifb path に新 wrapper を混ぜないことである。

## Phase F5hl: Native platform wait backend selection boundary

F5hl では、F5hk の wrapper を actual OS backend へ接続する前に、current platform と platform-specific wait backend の対応を typed enum と `Result` で固定する。macOS は run loop timer、Windows は waitable timer / message wait、Linux は selector / timerfd を、それぞれ別 backend kind として表す。

`NativeWindowHostLoopPlatformKind` は `Macos`、`Windows`、`Linux`、`Unsupported` を持つ。`native_window_host_loop_current_platform_kind` は `cfg(target_os = ...)` だけで current platform を決め、runtime string、environment probing、filesystem probing を使わない。

`NativeWindowHostLoopPlatformWaitBackendKind` は `MacosRunLoopTimer`、`WindowsWaitableTimerMessageWait`、`LinuxSelectorTimerFd`、`HeadlessScripted` を持つ。`HeadlessScripted` は deterministic headless / test backend 用の future kind であり、native backend default や native platform validation の fallback として成功させない。

validation は current platform と requested backend の一致だけを success とし、unsupported platform や mismatch は typed error として返す。default backend selection は macOS、Windows、Linux だけに定義し、unsupported platform では `Result::Err` にする。

F5hl は backend selection contract であり、macOS AppKit / CoreFoundation run loop timer、Win32 waitable timer / message wait、Linux selector / timerfd の actual implementation ではない。minifb runner、`Window::set_target_fps` authority、thread sleep、busy loop、fallback、silent no-op、synthetic timer fire は導入しない。

subagent review:

- Darwin the 2nd に F5hl 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、`HeadlessScripted` を default / native fallback にしないこと、standard spec も同期すること、typed enum と `cfg` selection を source policy で固定すること、runtime string / env probing / fallback / silent no-op を拒否すること、全 mismatch pair と unsupported platform を test することである。

## Phase F5hm: Native platform wait backend construction gate boundary

F5hm では、F5hl の platform/backend selection を actual native wait host construction の入口へ接続する。ただし、この phase では macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd の actual implementation はまだ作らない。目的は、未検証の raw backend kind や unsupported backend から platform wait host を作る経路を閉じ、実装未接続の状態を typed error として外へ返すことである。

`NativeWindowHostLoopPlatformWaitBackendSelection` は current platform と backend kind の検証済み pair を保持する。selection token の field は private とし、public constructor は置かない。token は `validate_native_window_host_loop_platform_wait_backend_selection_for_platform` または `native_window_host_loop_default_platform_wait_backend_selection_for_platform` からだけ得る。これにより、future builder は raw enum pair ではなく検証済み token を受け取る。

`NativeWindowHostLoopPlatformWaitHostBuildError` は `BackendSupportFailed NativeWindowHostLoopPlatformWaitBackendSupportError` と `BackendImplementationUnavailable { platform, backend }` を分ける。mismatch、unsupported platform、`HeadlessScripted` の native selection は support failure として validation 段階で止める。validated selection まで進んだ real backend についてだけ、actual implementation が未接続であることを `BackendImplementationUnavailable` で返す。

`build_native_window_host_loop_platform_wait_backend_from_selection` は selection を消費して dummy host、scripted host、sleep adapter、minifb pacing adapter を返さない。現 checkpoint では actual backend を作ったことにせず、selection の platform/backend を保持した unavailable error を返す。owner を受け取る builder はまだ追加しないため、construction failure で host owner を失う経路も作らない。

この phase は construction gate までであり、実 OS backend の代替として headless scripted、minifb、thread sleep、busy loop、synthetic timer fire を返さない。`NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost` の dummy adapter 接続、minifb smoke runner 接続、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は含めない。

subagent review:

- Darwin the 2nd に F5hm 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、selection token の field を private にすること、unsupported construction で owner を消費しないこと、validated selection 後だけ `BackendImplementationUnavailable` を返すこと、`HeadlessScripted` / minifb / sleep / busy loop / synthetic timer fire の backend を作らないこと、source policy と test で fixed contract にすることである。

## Phase F5hn: Native Windows waitable timer message wait raw backend boundary

F5hn では、F5hm の construction gate の後続として、Windows の waitable timer / message wait backend を actual OS 呼び出しから切り離した raw API 境界として実装する。目的は、Windows の handle validation、deadline-to-relative-time conversion、wait status mapping、message-only wait、timer-or-message wait を Linux CI でも deterministic に検査できる contract にすることである。

`NativeWindowHostLoopWindowsWaitHandle` は raw handle を private field として保持し、`0` と `-1` を invalid handle として拒否する。public code は raw `isize` や owned handle を backend 外へ取り出さない。raw handle の読み取りは backend module 内部に閉じ、public API には handle open state の query だけを置く。

`NativeWindowHostLoopWindowsDeadlinePlan` は `AlreadyReached` と `Relative100ns i64` を分ける。deadline が `now` 以下なら raw API call を行わず deadline reached とする。future deadline は nanosecond delta を 100ns 単位へ切り上げ、Windows relative due time として負の `i64` に変換する。overflow は saturating / clamp せず typed error として返す。

`NativeWindowHostLoopWindowsWaitRawApi` は `create_waitable_timer_raw`、`set_waitable_timer_relative_100ns`、`msg_wait_for_timer_or_message_raw`、`msg_wait_for_message_raw`、`close_handle_raw`、`last_error_code` を持つ。backend はこの trait だけに依存し、cfg-windows の sys shim と scripted test API を同じ contract で扱う。

`NativeWindowHostLoopWindowsWaitBackend` は `NativeWindowHostLoopDeadlineTimerClock` と `NativeWindowHostLoopInterruptibleDeadlineWaiter` を実装する。host event wait は message-only wait を呼び、timer wait や synthetic readiness を使わない。frame interval wait は waitable timer を relative 100ns deadline で arm してから timer-or-message wait を行い、timer signaled を `DeadlineReached`、message ready を `HostEventReady` へ写す。`WAIT_FAILED` は `last_error_code` を保持した typed error にし、timeout や未知 status は unexpected status として返す。

cfg-windows の `NativeWindowHostLoopWindowsWaitSysApi` は `windows-sys` の `CreateWaitableTimerW`、`SetWaitableTimer`、`MsgWaitForMultipleObjects`、`CloseHandle`、`GetLastError` だけをこの境界に閉じ込める。target-specific dependency は Windows target だけに置き、non-Windows の build / test は scripted raw API で同じ contract を検査する。

この phase では、generic `build_native_window_host_loop_platform_wait_backend_from_selection` は F5hm の fail-closed behavior を維持する。Windows-specific builder は validated Windows selection と raw API を受けた場合だけ backend を作り、selection support failure と raw API construction failure を別 error として返す。minifb runner、macOS run loop timer、Linux selector / timerfd、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は後続で扱う。

subagent review:

- Darwin the 2nd に F5hn 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、cross-platform raw API boundary を `windows-sys` から独立させること、message-only wait を immediate ok / polling にしないこと、`AlreadyReached` は plan が deadline <= now を証明した時だけ返すこと、Windows sys shim を cfg に閉じ込めること、generic builder の unavailable behavior を維持すること、sleep / minifb / busy wait / scripted native substitute / synthetic timer fire を禁止することである。

## Phase F5ho: Native single-owner interruptible wait adapter boundary

F5ho では、F5hn の Windows wait backend を run-loop wait hook へ接続する前段として、clock と waiter を同一 owner に保持する interruptible wait adapter 境界を追加する。既存の `NativeWindowHostLoopInterruptibleDeadlineWaitAdapter Clock Waiter` は clock と waiter を別 field として持つため、Windows の waitable timer handle owner にはそのまま使わない。

`NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter Backend` は `Backend` を 1 個だけ所有する。`Backend` は `NativeWindowHostLoopDeadlineTimerClock` と `NativeWindowHostLoopInterruptibleDeadlineWaiter` の両方を実装し、両 trait の `Error` type が同一であることを型で固定する。これにより、Windows backend の timer handle、clock origin、message wait state を二重化しない。

single-owner executor は F5hj の semantic contract を維持する。host event instruction は backend の `wait_for_host_event` だけを呼び、clock read、timer id allocation、frame wait を行わない。frame interval instruction は invalid `wait_nanos` と timer id overflow を clock read より前に拒否し、clock read、deadline calculation、`next_raw_id` advance、backend wait の順で進む。

`HostEventReady` wake は `HostEventPumpAlreadyPaced` に写し、timer-fired evidence を生成しない。`DeadlineReached` wake だけが `FrameIntervalTimerFired` を返す。deadline wait 直前に進めた timer id は、backend wait error や `HostEventReady` でも再利用しない。

`NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost` は inner host と single-owner adapter を所有する。event polling、title update、pump-only、present は inner host へ委譲し、budget exhaustion 後の wait だけを single-owner executor へ渡す。inner host の wait hook は呼ばない。

この phase では、F5hn Windows backend を generic platform wait builder、`run_minifb_window_loop`、`Window::set_target_fps` replacement へはまだ接続しない。macOS run loop timer、Linux selector / timerfd、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization も後続で扱う。sleep、busy loop、minifb、scripted native substitute、synthetic timer fire、fallback、silent no-op は禁止する。

subagent review:

- Darwin the 2nd に F5ho 計画を渡し、`PLAN_APPROVED` を得た。required constraints は、Backend の clock/waiter 同一 error type を型で固定すること、`HostEventReady` から timer-fired evidence を作らないこと、invalid wait / id overflow を clock/read/wait より前に fail closed すること、deadline wait 直前だけ `next_raw_id` を進めて再利用しないこと、OS handle や raw internals を露出しないこと、minifb runner / generic platform builder / `set_target_fps` へ接続しないことである。

## Phase F5hp: Native Windows platform wait support gate boundary

F5hp では、F5hn の Windows wait backend と F5ho の single-owner run-loop wait hook を接続する support gate を追加する。目的は、Windows backend を platform wait backend owner として扱えるようにしつつ、backend construction failure で run-loop host owner を失わない二段階 contract を固定することである。

`NativeWindowHostLoopPlatformWaitBackend Api` は、この checkpoint では `WindowsWaitableTimerMessageWait NativeWindowHostLoopWindowsWaitBackend Api` だけを持つ。MacOS / Linux の backend はまだ実装されていないため、この enum に dummy variant、headless scripted variant、minifb variant、sleep variant、busy-loop variant は追加しない。`NativeWindowHostLoopPlatformWaitBackendError` は Windows backend error を保持し、string や generic backend failure に潰さない。

`build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api` は validated selection を再検査し、Windows + WindowsWaitableTimerMessageWait の場合だけ supplied raw API から Windows backend を作る。MacOS / Linux の validated selection は `BackendImplementationUnavailable` として返し、mismatch / unsupported は `BackendSupportFailed` として返す。旧 F5hm の `build_native_window_host_loop_platform_wait_backend_from_selection` は API / handle owner を受け取らない no-owner fail-closed probe として残し、actual backend を暗黙に作らない。

run-loop host への接続は fallible builder に含めない。backend construction 成功後にだけ、`native_window_host_loop_platform_wait_run_loop_host_from_backend` が host と platform backend を受け取り、`NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter` と `NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost` を infallible に組み立てる。このため support failure、Windows raw API failure、MacOS / Linux unavailable で host owner を消費しない。

cfg-windows helper は `NativeWindowHostLoopWindowsWaitSysApi` を使う backend construction helper だけを提供する。host-consuming fallible helper は作らない。minifb runner、`Window::set_target_fps` replacement、macOS run loop timer、Linux selector / timerfd、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は後続で扱う。sleep、busy loop、headless fallback、synthetic timer fire、fallback、silent no-op は禁止する。

subagent review:

- Darwin the 2nd の初回 plan review は `PLAN_CHANGES`。fallible helper が host を受け取ると build failure で host owner を失うため、backend construction と host wrapping を二段階に分けるよう指摘された。
- revised plan では、fallible backend builder と infallible wrapper helper に分離し、`PLAN_APPROVED` を得た。required constraints は、selection を raw API handle 作成前に再検査すること、support failure / backend unavailable / Windows raw API failure を分離すること、旧 no-owner builder を fail-closed probe として残すこと、dummy / headless / minifb / sleep variant を追加しないこと、minifb path が platform backend owner / wrapper に触れないこと、host-consuming fallible builder を作らないことである。

## Phase F5hq: Native Windows platform wait run-loop config gate boundary

F5hq では、F5hp の Windows platform wait backend support gate を native run-loop configuration の wait backend selection へ接続する。ただし、この checkpoint では minifb runner を platform wait runner へ置き換えない。目的は、run-loop config が minifb internal pacing と platform wait backend を同じ field で排他的に表し、二重 authority を作らないことである。

`NativeWindowRunLoopConfig` の wait backend authority は `NativeWindowRunLoopWaitBackend` を単一 source of truth とする。default は `MinifbInternalTargetFps` のままであり、既存 smoke runner の behavior は変えない。旧 `NativeWindowRunLoopFrameIntervalWaitBackend` は compatibility input として残し、constructor で `NativeWindowRunLoopWaitBackend` へ変換するだけにする。config field として `frame_interval_wait_backend` を再導入してはいけない。

`NativeWindowRunLoopWaitBackend::PlatformWait selection` は、validated `NativeWindowHostLoopPlatformWaitBackendSelection` を保持し、frame interval authority としては host-owned deadline timer と同じ扱いになる。minifb runner は `config.wait_backend` を window / backend creation / `Window::set_target_fps` より前に検査し、`PlatformWait` と `HostOwnedDeadlineTimer` を typed conflict として拒否する。`PlatformWait` が指定されても minifb pacing へ fallback しない。

platform wait backend construction は host owner を受け取らない。`native_window_run_loop_platform_wait_backend_selection` は config から selection を取り出し、non-platform config なら typed `NotPlatformWaitBackend` を返す。cfg-windows の `native_window_run_loop_platform_wait_backend_from_config` は config / selection から backend を作るだけで、host wrapping は F5hp の infallible wrapper に委ねる。

この phase では、macOS run loop timer、Linux selector / timerfd、minifb runner の platform wait replacement、`Window::set_target_fps 0`、sleep / busy loop / headless fallback、synthetic timer fire は追加しない。

subagent review:

- Darwin the 2nd の初回 plan review は `PLAN_CHANGES`。`NativeWindowRunLoopConfig` に新旧二つの wait authority field を置くと、minifb validation が片方だけを見る退行を作れるため、new enum を config の単一 source of truth にするよう指摘された。
- revised plan では、`wait_backend` を唯一の config field とし、旧 frame interval enum を compatibility input に限定する方針で `PLAN_APPROVED` を得た。required constraints は、minifb runner が `config.wait_backend` を side effect より前に検査すること、`PlatformWait` が minifb へ fallback しないこと、config-to-platform backend construction が host を消費しないこと、source policy で旧 config field 再導入と validation order を固定することである。

## Phase F5hr: Native Windows platform wait window runner support gate

F5hr では、F5hq で config に入った Windows platform wait backend を、actual native window run-loop runner の wait hook へ接続する。ただし、既存 `run_minifb_window_loop` を platform wait runner に置き換えず、Windows 専用の別入口 `run_windows_platform_wait_window_loop` として追加する。default smoke runner は引き続き minifb internal `Window::set_target_fps` pacing を使い、`PlatformWait` を受け取った場合は typed conflict として拒否する。

`run_windows_platform_wait_window_loop` は最初に `native_window_run_loop_platform_wait_backend_from_config config` を呼ぶ。non-platform config、selection mismatch、Windows waitable timer construction failure は typed `PlatformWaitBackendFromConfigFailed` として返し、この時点では `NativeWindowBackendLoop::new_for_scale`、minifb `Window::new`、visual host construction を行わない。これにより fallible backend construction が host owner を消費しない。

Windows runner は backend construction 成功後にだけ `NativeWindowBackendLoop` と minifb `Window` を作る。minifb `Window` は visual / event / present host としてだけ使い、`Window::set_target_fps`、`set_target_fps 0`、`configure_minifb_window_frame_pacing` は呼ばない。wait/pacing owner は `NativeWindowHostLoopPlatformWaitBackend` を包んだ single-owner interruptible deadline wait hook だけである。

F5hr では `MinifbNativeWindowVisualRunLoopHost` を追加し、既存 `MinifbNativeWindowRunLoopHost` から分離する。visual host は `poll_event_snapshot`、title update、pump、present だけを担当し、`wait_after_budget_exhausted` は `VisualHostWaitUnsupported` を返す fail-closed hook にする。platform wait wrapper は inner visual host の wait hook を呼ばず、`NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter` へ wait を渡す。

`NativeWindowRunLoopError` は platform backend construction failure と Windows platform wait host-loop failure を typed variant として保持し、platform wait backend error を string 化しない。minifb `update_with_buffer` error は既存の present error boundary として `String` のままだが、Windows wait backend の OS status、deadline conversion、clock / waiter failure は enum evidence のまま保持する。

この phase では CLI flag 接続、macOS run loop timer、Linux selector / timerfd、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization、headless scripted fallback は追加しない。

subagent review:

- Darwin the 2nd の plan review は初回 `REQUIRED_CHANGES`。既存 `MinifbNativeWindowRunLoopHost` を platform wait runner に再利用すると、値の中に minifb pacing authority と `FramePresentAlreadyPaced` を返せる wait hook が残り、wait/pacing owner が曖昧になると指摘された。
- revised plan では、minifb visual / event / present 専用 host を分け、visual host の wait hook を typed unsupported にする方針で実装開始可となった。CLI 接続は scope 外でよく、library runner、source policy、unit test までを F5hr の範囲にする。

## Phase F5hs: Native Windows platform wait CLI selection boundary

F5hs では、F5hr の Windows platform wait runner を CLI から明示的に選べるようにする。`nepl-gui-native` は `--wait-backend minifb|platform` を受け取り、未指定時は従来通り `run_minifb_window_loop` と minifb internal `Window::set_target_fps` pacing を使う。`platform` 指定時だけ、validated default platform wait backend selection を `NativeWindowRunLoopConfig::new_with_platform_wait_backend_selection` に入れ、cfg-windows の `run_windows_platform_wait_window_loop` へ渡す。

CLI option は window mode 専用である。`--wait-backend` が headless mode で明示指定された場合、重複して指定された場合、または `minifb` / `platform` 以外の値が与えられた場合は error として拒否する。指定が無い headless mode は従来通り headless frame output を返す。

`main.rs` は runner selection と config construction だけを担当する。minifb lifecycle、event pump、presenter state、OS wait API、sleep、busy loop、message pump、timer registration は lib 側 runner / host の責務であり、CLI 側には置かない。non-Windows platform selection は unsupported error を返し、minifb runner へ切り替えない。

この phase は CLI selection boundary だけであり、macOS run loop timer、Linux selector / timerfd、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization は後続 slice に分ける。

## Phase F5ht: Native macOS run loop timer raw backend boundary

F5ht では、macOS run loop timer backend の actual owner integration へ進む前に、raw API 境界、typed handle、deadline conversion、status mapping、handle invalidation の contract を固定する。これは AppKit / CoreFoundation の system shim を直接呼ぶ実装ではなく、fake raw API による library-level contract checkpoint である。

`NativeWindowHostLoopMacosRunLoopTimerHandle` は raw handle を private field として保持し、null / invalid sentinel は `InvalidRawHandle` または timer creation failure として `Result` で返す。公開 API は raw handle accessor や owned raw handle escape を持たない。backend は `Drop` と explicit invalidation の両方で close / invalidate を高々 1 回に制限し、drop cleanup failure は recovery せず cleanup best-effort として扱う。

deadline は monotonic origin からの nanoseconds で扱い、`checked_sub` と `u64::try_from` だけで relative timer delay を作る。deadline 到達済みは `TimerFired` として immediate wake にし、clamp、saturating arithmetic、sleep、busy loop、synthetic host event は使わない。timer-or-event wait の raw status は `TimerFired` と `HostEventReady` を別 enum に写し、host event only wait で timer fired raw status が返った場合は `UnexpectedRunLoopStatus` にする。

この phase では、generic `NativeWindowHostLoopPlatformWaitBackend` へ `MacosRunLoopTimer(...)` owner variant を追加しない。`NativeWindowHostLoopDeadlineTimerClock` / `NativeWindowHostLoopInterruptibleDeadlineWaiter` の実装、CLI selection、native platform runner dispatch、actual macOS sys shim、Linux selector / timerfd、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization は後続 slice に分ける。

subagent review:

- Darwin the 2nd の plan review は `PLAN_CHANGES`。macOS backend を generic platform owner へ統合するには actual macOS sys shim と run-loop ownership の検証が不足するため、F5ht は raw API / status / deadline / handle ownership 境界だけに絞るよう指摘された。
- revised plan では、fake raw API tests と source policy で raw boundary を固定し、generic platform owner、CLI、runner dispatch への接続を後続へ残す方針にした。

## Phase F5hu: Native Linux selector timerfd raw backend boundary

F5hu では、Linux selector / timerfd backend の actual sys shim と platform wait owner integration へ進む前に、selector fd owner、timerfd owner、`NativeWindowHostLoopLinuxSelectorTimerFdRawApi` raw API 境界、relative timespec conversion、timer-or-event status mapping、close-once cleanup の contract を固定する。これは `epoll` / `poll` / `select` / `timerfd_*` を直接呼ぶ実装ではなく、fake raw API による library-level contract checkpoint である。

selector fd と timerfd は別 owner として保持する。Linux では fd `0` が有効な descriptor になり得るため、invalid sentinel は `raw_fd < 0` だけにする。public API は raw fd accessor や owned raw fd escape を持たず、constructor failure と raw wait failure は typed error として返す。

raw API contract は、selector creation、timerfd creation、timerfd registration、relative timespec arm、timer-or-host-event wait、host-event-only wait、selector close、timerfd close、last error code を分ける。construction failure では作成済み fd を逆順に close し、normal cleanup では timerfd と selector をそれぞれ高々 1 回だけ close する。

deadline は monotonic origin からの nanoseconds で扱い、到達済み deadline は `TimerFired` として immediate wake にする。relative timespec は seconds と nanoseconds を `/ 1_000_000_000`、`% 1_000_000_000`、`i64::try_from` で checked conversion し、clamp、saturating arithmetic、sleep、busy loop、synthetic host event は使わない。timer-or-event wait は `TimerFired` と `HostEventReady` を別 enum に写し、host-event-only wait で timer fired raw status が返った場合は unexpected status として拒否する。

この phase では、generic `NativeWindowHostLoopPlatformWaitBackend` へ `LinuxSelectorTimerFd(...)` owner variant を追加しない。`NativeWindowHostLoopDeadlineTimerClock` / `NativeWindowHostLoopInterruptibleDeadlineWaiter` の実装、CLI selection、native platform runner dispatch、actual Linux sys shim、macOS actual sys shim、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization は後続 slice に分ける。

subagent review:

- Darwin the 2nd の plan review は `PLAN_APPROVED`。ただし、fd `0` を有効として扱い invalid は `raw_fd < 0` に限定すること、selector fd と timerfd の ownership を混ぜず close-once を保つこと、timespec conversion を checked にすること、`TimerFired` と `HostEventReady` を分けること、actual sys shim / generic owner / trait impl / CLI / runner dispatch は scope 外にすることが required constraint とされた。

## Phase F5hv: Native Linux selector timerfd single-owner wait trait boundary

F5hv では、F5hu の Linux selector / timerfd raw backend を `NativeWindowHostLoopDeadlineTimerClock` と `NativeWindowHostLoopInterruptibleDeadlineWaiter` に接続し、F5ho の single-owner interruptible deadline wait adapter で扱える境界を追加する。これは raw backend を run-loop wait owner contract へ接続する最小 slice であり、actual Linux syscall shim や generic platform wait enum 統合へは進まない。

`NativeWindowHostLoopLinuxSelectorTimerFdBackend` は monotonic origin からの checked elapsed nanoseconds を `now_nanos` として返す。host-event-only wait は F5hu の event-only raw wait をそのまま使い、timer-fired status を host event として扱わない。frame interval wait は existing raw `TimerFired` を `NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached` に、`HostEventReady` を `HostEventReady` に写す。timer fired evidence は single-owner executor の deadline reached branch だけで作られ、host event ready を timer fired と偽装しない。

この phase では、`NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd(...)`、`build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api`、`#[cfg(target_os = "linux")]` actual sys shim、`libc` / `nix` / `epoll` / `poll` / `select` / `timerfd_*`、CLI / native runner dispatch、minifb wait path、sleep、busy loop、synthetic timer fire、fallback、silent no-op は追加しない。

subagent review:

- Darwin the 2nd の plan review は `APPROVED`。generic platform owner 統合へ飛ぶ前に F5ho single-owner adapter との semantic 接続を固定する root-cause slice として妥当であり、generic enum variant / actual Linux sys shim / runner 接続は後続へ残すよう確認された。

## Phase F5hw: Native macOS run loop timer single-owner wait trait boundary

F5hw では、F5ht の macOS run loop timer raw backend を `NativeWindowHostLoopDeadlineTimerClock` と `NativeWindowHostLoopInterruptibleDeadlineWaiter` に接続し、F5ho の single-owner interruptible deadline wait adapter で扱える境界を追加する。これは raw backend を run-loop wait owner contract へ接続する最小 slice であり、actual macOS sys shim や generic platform wait enum 統合へは進まない。

`NativeWindowHostLoopMacosRunLoopTimerBackend` は monotonic origin からの checked elapsed nanoseconds を `now_nanos` として返す。host-event-only wait は F5ht の event-only raw wait をそのまま使い、timer-fired status を host event として扱わない。frame interval wait は macOS raw wake の `TimerFired` を `NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached` に、`HostEventReady` を `HostEventReady` に写す。timer fired evidence は single-owner executor の deadline reached branch だけで作られ、host event ready を timer fired と偽装しない。

この phase では、generic `NativeWindowHostLoopPlatformWaitBackend::MacosRunLoopTimer(...)` owner variant、`build_native_window_host_loop_platform_wait_backend_from_selection_with_macos_api`、`#[cfg(target_os = "macos")]` actual sys shim、CoreFoundation / AppKit binding、CLI / native runner dispatch、minifb wait path、sleep、busy loop、fallback、silent no-op は追加しない。

subagent review:

- Darwin the 2nd の plan review は `APPROVED`。F5hv と対称の trait-connection slice として妥当であり、typed boundary、no fallback、platform separation を維持するよう確認された。required constraint として、`TimerFired -> DeadlineReached` と `HostEventReady -> HostEventReady` の写像、event-only wait で timer-fired status を `UnexpectedRunLoopStatus` として保持すること、generic owner / actual macOS sys shim / CLI / minifb / sleep / fallback を scope 外に残すことが確認された。

## Phase F5hx: Native platform wait multi-backend owner boundary

F5hx では、F5hp の Windows-only platform wait owner と、F5hv / F5hw の Linux / macOS single-owner wait trait 実装を、1 つの typed platform wait owner enum に統合する。これは raw API を受け取った場合にだけ selected backend を構築する ownership boundary であり、actual Linux syscall shim、actual macOS sys shim、native runner / CLI dispatch の追加ではない。

`NativeWindowHostLoopPlatformWaitBackend WindowsApi MacosApi LinuxApi` は `WindowsWaitableTimerMessageWait`、`MacosRunLoopTimer`、`LinuxSelectorTimerFd` の 3 variant を持つ。各 variant は対応する raw backend owner を 1 個だけ所有し、`NativeWindowHostLoopDeadlineTimerClock` と `NativeWindowHostLoopInterruptibleDeadlineWaiter` は selected backend へ委譲する。error は `NativeWindowHostLoopPlatformWaitBackendError WindowsError MacosError LinuxError` の variant として保持し、platform error を string や generic failure に潰さない。

`build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis` は validated selection を再検査し、Windows selection なら Windows raw API だけ、macOS selection なら macOS raw API だけ、Linux selection なら Linux raw API だけを使って backend を構築する。selection mismatch、unsupported platform、headless scripted は raw API construction より前に `BackendSupportFailed` として拒否する。selected されていない raw API は fallback / dummy / no-op として呼ばない。

Windows-only compatibility helper `build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api` は残す。この helper は return type を `NativeWindowHostLoopWindowsOnlyPlatformWaitBackend WindowsApi` に限定し、macOS / Linux selection は `BackendImplementationUnavailable` のまま返す。これは Windows cfg runner 用の互換入口であり、macOS / Linux raw API を暗黙に作ったことにしない。

旧 F5hm の `build_native_window_host_loop_platform_wait_backend_from_selection` は no-owner fail-closed probe として残す。API / handle owner を受け取らないため、validated real backend でも `BackendImplementationUnavailable` を返し、headless scripted、minifb pacing、thread sleep、busy loop、synthetic timer fire へ fallback しない。

Windows-only helper の type parameter には uninhabited raw API `NativeWindowHostLoopNeverMacosRunLoopTimerRawApi` と `NativeWindowHostLoopNeverLinuxSelectorTimerFdRawApi` を使う。これらは empty enum であり、trait method body は `match *self {}` だけにする。panic、dummy handle、dummy status、silent no-op、fallback status を返さない。

この phase では、`#[cfg(target_os = "linux")]` actual Linux sys shim、`#[cfg(target_os = "macos")]` actual macOS sys shim、CoreFoundation / AppKit binding、libc / nix / epoll / poll / select / timerfd、native runner / CLI dispatch、minifb wait path、sleep、busy loop、fallback、silent no-op、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization は追加しない。

subagent review:

- Darwin the 2nd の plan review は `PLAN_APPROVED`。required constraint として、multi-backend owner enum は selected backend だけを構築すること、Windows-only compatibility wrapper と no-owner fail-closed probe を残すこと、Never raw API は empty enum + `match *self {}` に限定すること、actual macOS / Linux sys shim と runner dispatch へは進まないことが確認された。

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

## Native platform wait continuation phases

### Phase F5hy: Native Linux selector timerfd actual sys shim boundary

目的:

- F5hu/F5hv の Linux selector / timerfd raw API を、Linux の actual `epoll` / `timerfd` syscall shim へ接続する。
- sys shim は `#[cfg(target_os = "linux")]` に閉じ、非 Linux build や標準 API の public name に `libc`、`epoll`、`timerfd` を漏らさない。
- timer fired evidence は timer fd readiness と `u64` expiration drain の両方を満たした場合だけ作る。

実装:

- `NativeWindowHostLoopLinuxSelectorTimerFdSysApi` を追加し、`NativeWindowHostLoopLinuxSelectorTimerFdRawApi` だけを実装する。
- selector creation、timerfd creation、timerfd registration、relative timer arm、timer-or-event wait、close、last error code を syscall shim 内へ分ける。
- host-event-only wait は host event fd registration が未設計であるため `ENOTSUP` 相当の failure として fail closed にする。F5hy 単独では host event fd owner を作らない。

検証:

- `nodesrc/test_native_gui_platform_behavior.js` で core/fake Linux backend slice と cfg Linux sys shim slice を分ける。
- core/fake slice では `libc` / `epoll` / `timerfd` を禁止し、sys shim slice では `epoll_create1`、`timerfd_create`、`epoll_ctl`、`timerfd_settime`、`epoll_wait`、`read`、`close` の存在と failure normalization を検査する。
- Windows host では Linux target が無い場合、source policy と通常 Rust tests を主検証にし、Linux target compile は CI / Linux 環境で確認する。

非目標:

- host event fd integration、generic platform wait helper の Linux sys constructor、native runner / CLI connection、macOS actual sys shim、minifb wait path、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- `EINTR` retry loop、thread sleep、busy loop、fallback、silent no-op、synthetic host event / timer fired evidence は導入しない。

### Phase F5hz: Native Linux host event fd integration boundary

目的:

- F5hy の Linux selector / timerfd sys shim に host event fd owner を追加し、host-event-only wait が synthetic readiness ではなく actual host event fd readiness だけで成功するようにする。
- Linux-specific primitive は cfg Linux sys shim へ閉じ、core/fake raw backend slice には `libc`、`eventfd`、`epoll` を漏らさない。
- event producer、generic platform helper、runner / CLI 接続へは進まず、host event fd owner / registration / readiness mapping だけを固定する。

実装:

- `NativeWindowHostLoopLinuxHostEventFd`、`native_window_host_loop_linux_host_event_fd_from_raw`、private raw helper を追加する。fd `0` は有効、負値だけ invalid とする。
- `NativeWindowHostLoopLinuxSelectorTimerFdRawApi` に host event fd creation / registration / close と、host event fd を受け取る wait method を追加する。
- backend は selector、timerfd、host event fd を所有し、constructor failure cleanup は逆順で行う。close-once cleanup は host event fd、timerfd、selector の順に行う。
- cfg Linux sys shim は `eventfd 0 EFD_CLOEXEC | EFD_NONBLOCK` を作成し、selector へ `EPOLLIN` で登録する。
- timer-or-event wait は timerfd readiness と timer drain だけを `TimerFired`、eventfd readiness と eventfd drain だけを `HostEventReady` とする。host-event-only wait は eventfd readiness だけを成功にする。

検証:

- Rust unit tests で fd `0` / negative validation、host event creation failure、host event registration failure、event-only wait の host event fd identity、timer-or-event wait の timer / host event fd identity、close-once cleanup を検査する。
- source policy で raw API method、constructor order、cleanup order、cfg Linux `eventfd` usage、timer/event readiness mapping、fallback / silent no-op / synthetic evidence 禁止を固定する。
- 可能な環境では `cargo check -p nepl-gui-native --lib --target x86_64-unknown-linux-gnu` を実行し、Linux cfg sys shim が型検査されることを確認する。

非目標:

- eventfd write / signal producer、generic platform wait helper の Linux sys constructor、native runner / CLI connection、macOS actual sys shim、minifb wait path、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- thread sleep、busy loop、fallback、silent no-op、synthetic host event、timer fired evidence の偽装は導入しない。

### Phase F5ia: Native Linux platform wait support helper boundary

目的:

- F5hz の cfg Linux sys shim を、F5hx の generic platform wait owner に Linux-only support helper として接続する。
- Windows helper と対称に、Linux selection だけが actual Linux sys API から backend を構築できる入口を作る。
- no-owner generic builder は fail-closed probe として残し、runner / CLI / minifb wait path へはまだ接続しない。

実装:

- `NativeWindowHostLoopLinuxOnlyPlatformWaitBackend LinuxApi` を追加し、Windows / macOS type parameter には never raw API を使う。
- `build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api` を追加する。selection validation を raw API method より前に行い、Linux + `LinuxSelectorTimerFd` の場合だけ existing Linux backend builder を呼ぶ。F5if 以降は event source capability も explicit input とし、Linux raw API 呼び出し前に検査する。
- Windows / macOS の validated selection は `BackendImplementationUnavailable`、mismatch / unsupported / HeadlessScripted は `BackendSupportFailed`、Linux raw construction failure は `LinuxSelectorTimerFdBackendFailed` として返す。
- cfg Linux `native_window_host_loop_platform_wait_backend_from_selection` は explicit event source capability と `NativeWindowHostLoopLinuxSelectorTimerFdSysApi::new` を Linux-only helper へ渡す。

検証:

- Rust unit tests で Linux helper の success、Windows / macOS selection の unavailable、support failure before raw method calls、Linux raw construction failure を検査する。
- source policy で Linux-only alias、never Windows raw API、cfg Linux helper、runner / CLI / minifb 未接続、fallback / silent no-op / synthetic evidence 禁止を固定する。
- `cargo check -p nepl-gui-native --lib --target x86_64-unknown-linux-gnu` により cfg Linux helper が型検査されることを確認する。

非目標:

- eventfd write / signal producer、native runner / CLI connection、minifb wait path、macOS actual sys shim、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- thread sleep、busy loop、fallback、silent no-op、synthetic host event、timer fired evidence の偽装、host-consuming fallible builder は導入しない。

### Phase F5ib: Native Linux host event fd producer boundary

目的:

- F5hz の host event fd owner / readiness mapping に対して、producer 側の explicit signal 境界を追加する。
- eventfd write は cfg Linux sys shim の raw API 実装へ閉じ、backend は private owner と typed error だけを見る。
- producer success を `HostEventReady` outcome や scheduler resume evidence に直結せず、runner / CLI / minifb 接続前の contract として固定する。

実装:

- `NativeWindowHostLoopLinuxSelectorTimerFdRawApi` に `signal_host_event_fd_raw` を追加する。
- `NativeWindowHostLoopLinuxSelectorTimerFdBackendError::SignalHostEventFdFailed` を追加する。
- `NativeWindowHostLoopLinuxSelectorTimerFdBackend::signal_host_event` は、closed host event fd を `InvalidHostEventRawFd { raw_fd: -1 }` で拒否し、raw failure を `SignalHostEventFdFailed { code: last_error_code }` として返す。
- cfg Linux `NativeWindowHostLoopLinuxSelectorTimerFdSysApi` は `u64` 値 `1` を `libc::write` で exactly 8 bytes 書いた場合だけ成功にする。short write、`EAGAIN`、syscall failure は success にしない。
- never raw API と scripted raw API を同じ method set に更新する。

検証:

- Rust unit tests で signal success の host event fd identity、raw failure の error code preservation、closed backend rejection、scripted raw API の signal call record を検査する。
- source policy で raw API method、typed error、cfg Linux `libc::write` 使用、exact `u64` write、unit test 名、runner / CLI / minifb 未接続、fallback / silent no-op / synthetic readiness 禁止を固定する。
- `cargo check -p nepl-gui-native --lib --target x86_64-unknown-linux-gnu` で cfg Linux sys shim が型検査されることを確認する。

非目標:

- native runner / CLI connection、minifb wait path、generic dispatch からの producer 呼び出し、macOS actual sys shim、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- eventfd full / `EAGAIN` を成功扱いする fallback、silent no-op、synthetic host event、timer fired evidence の偽装は導入しない。

### Phase F5ic: Native Linux host event signal producer handle boundary

目的:

- F5ib の backend-owned producer を、blocking wait 中の backend とは別 owner から使える producer handle 境界へ分離する。
- Linux platform wait runner / CLI dispatch を有効にする前に、external window / event source が wait backend を所有せず host event fd を wake できる contract を固定する。
- producer signal success を `HostEventReady` outcome や scheduler resume evidence に直結させず、wake request だけを表す。

実装:

- `NativeWindowHostLoopLinuxHostEventSignalFd` を duplicated eventfd handle の private owner として追加する。fd `0` は有効、負値だけ invalid とする。
- `NativeWindowHostLoopLinuxHostEventSignalRawApi` を追加し、host event fd の clone、duplicated signal fd への exact write、duplicated signal fd close、last error retrieval を分ける。
- `NativeWindowHostLoopLinuxHostEventSignalProducerError` を追加し、closed backend、invalid duplicated fd、clone failure、signal failure を typed error として保持する。
- `NativeWindowHostLoopLinuxSelectorTimerFdBackend::create_host_event_signal_producer` は backend の host event fd を producer API で duplicate し、`NativeWindowHostLoopLinuxHostEventSignalProducer Api` を返す。backend closed は `InvalidHostEventRawFd { raw_fd: -1 }` として拒否する。
- cfg Linux `NativeWindowHostLoopLinuxSelectorTimerFdSysApi` は `fcntl F_DUPFD_CLOEXEC` で producer fd を作り、signal は `u64` 値 `1` の exact write だけを success にする。
- producer owner は explicit close と Drop の両方で close-once cleanup を行う。cleanup failure は通常 wait result と混ぜない。

検証:

- Rust unit tests で signal fd の fd `0` valid / negative invalid、producer duplication、clone failure、closed backend rejection、signal failure、close-once を検査する。
- source policy で private signal fd owner、producer raw API、typed error、`F_DUPFD_CLOEXEC`、exact write、unit test 名、runner / CLI / minifb 未接続、fallback / silent no-op / synthetic readiness 禁止を固定する。
- `cargo check -p nepl-gui-native --lib --target x86_64-unknown-linux-gnu` で cfg Linux producer shim が型検査されることを確認する。

非目標:

- native runner / CLI connection、minifb wait path、generic dispatch から producer を呼ぶ接続、Linux minifb platform wait runner、macOS actual sys shim、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- pre-signal busy loop、synthetic `HostEventReady`、timer fired evidence の偽装、`dup` による close-on-exec 欠落、raw fd escape、eventfd full / `EAGAIN` を成功扱いする fallback は導入しない。

### Phase F5id: Native Linux minifb observed-input signal bridge boundary

目的:

- F5ic の duplicated host event signal producer を、minifb が既に観測した keyboard / text input callback から wake request として呼べるようにする。
- minifb `InputCallback` は `window.update` / `update_with_buffer` による event processing 中に呼ばれるため、blocking wait を起こす完全な Linux event source ではない。この phase は observed-input signal bridge だけを固定し、Linux platform wait runner / CLI dispatch へは接続しない。
- callback は `Result` を返せないため、signal failure を silent no-op にせず共有 state に保存し、wait boundary で typed error として表面化する。

実装:

- `NativeWindowHostEventSignalWaitError WaitError` を追加し、host event signal failure と delegate wait failure を別 variant として保持する。
- `NativeWindowHostEventSignalErrorState` と `NativeWindowHostEventSignalWaitGuardRunLoopHost` を追加する。wait guard は `wait_after_budget_exhausted` の先頭で signal state を確認し、error があれば delegate wait を呼ばずに `HostEventSignalFailed` を返す。error が無い場合だけ inner host wait へ委譲する。
- cfg Linux + window の `MinifbNativeWindowLinuxHostEventSignalCallbackState` は F5ic producer を所有し、最初の signal error だけを保存する。以後の callback は同じ error を上書きしない。
- cfg Linux + window の `MinifbNativeWindowLinuxHostEventSignalInputCallback` は `add_char` と `set_key_state` で `signal_observed_input` を呼ぶ。これは observed input に対する wake request であり、`HostEventReady` outcome や timer fired evidence を生成しない。

検証:

- Rust unit tests で、signal state error が delegate wait より先に返ること、signal state が clean な場合は delegate wait failure がそのまま typed variant で返ることを検査する。
- cfg Linux + window tests で、minifb input callback が producer signal を呼ぶこと、signal failure は first error として 1 回だけ記録されることを検査する。
- source policy で F5id docs、wait guard、callback state、callback test、runner / CLI 非接続、synthetic readiness / timer evidence / fallback 禁止を固定する。

非目標:

- Linux platform wait runner、CLI dispatch、`run_linux_platform_wait_window_loop`、actual X11 / Wayland fd selector integration、minifb wait replacement、`set_target_fps 0`、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- minifb callback signal success を `HostEventReady` outcome として扱わない。callback bridge は eventfd producer request だけであり、blocking wait readiness の証明ではない。
- sleep、busy loop、fallback、silent no-op、synthetic host event、timer fired evidence の偽装は導入しない。

### Phase F5ie: Native Linux blocking wait event source capability gate

目的:

- F5id の observed-input signal bridge を、Linux blocking platform wait runner の readiness source と誤認しないための support gate を追加する。
- minifb `InputCallback` は already observed input の callback であり、blocking wait 中に OS event readiness を直接提供する source ではない。この性質を `ObservedInputOnly` として型に残す。
- actual X11 / Wayland fd integration または同等の externally-wakeable source は後続 phase とし、この phase では分類値と fail-closed validation だけを追加する。

実装:

- `NativeWindowHostLoopLinuxEventSourceCapability` を追加し、`ObservedInputOnly` と `ExternallyWakeableEventSource` を分ける。
- `NativeWindowHostLoopLinuxPlatformWaitEventSourceSupportError` を追加し、blocking platform wait が `ObservedInputOnly` を要求された場合は `ObservedInputOnlyUnsupportedForBlockingWait` として typed error を返す。
- `validate_native_window_host_loop_linux_blocking_wait_event_source_capability` を追加する。`ObservedInputOnly` は拒否し、`ExternallyWakeableEventSource` は分類値としてだけ受理する。受理は fd owner、selector registration、wait outcome、runner dispatch を意味しない。

検証:

- Rust unit tests で、`ObservedInputOnly` が blocking wait support では拒否されること、`ExternallyWakeableEventSource` が分類値として受理されることを検査する。
- source policy で、F5ie gate が `HostEventReady`、timer fired evidence、fd creation / registration、runner dispatch、minifb wait replacement、fallback / silent no-op を生成しないことを固定する。
- GUI spec / native behavior notes / `todo.md` / `note.n.md` に F5ie scope と後続残件を記録する。

非目標:

- Linux platform wait runner、CLI dispatch、`run_linux_platform_wait_window_loop`、actual X11 / Wayland fd selector integration、minifb wait replacement、`set_target_fps 0`、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- `ExternallyWakeableEventSource` を受理しても、その場で `HostEventReady` outcome、scheduler resume evidence、timer fired evidence、selector registration を作らない。
- callback-only source を fallback / best effort として blocking wait source に昇格しない。

### Phase F5if: Native Linux platform wait backend event source config gate

目的:

- F5ie の event source capability gate を Linux platform wait backend construction helper の入口へ接続し、future runner が helper を通るだけで observed-input-only source を拒否できるようにする。
- `build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api` と cfg Linux `native_window_host_loop_platform_wait_backend_from_selection` の両方で event source capability を explicit input にし、暗黙に `ExternallyWakeableEventSource` を渡す bypass を作らない。
- multi-backend raw API builder の Linux branch でも同じ gate を使い、selected Linux backend construction の全入口で raw API 呼び出し前に event source capability を検査する。

実装:

- `NativeWindowHostLoopPlatformWaitHostBuildError::LinuxEventSourceSupportFailed` を追加し、`NativeWindowHostLoopLinuxPlatformWaitEventSourceSupportError` を保持する。
- `build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api` は `event_source_capability` を受け取り、selection validation の後、Linux raw API method 呼び出しの前に `validate_native_window_host_loop_linux_blocking_wait_event_source_capability` を実行する。
- `build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis` は `linux_event_source_capability` を受け取り、Linux branch に入った場合だけ同じ validation を通す。Windows / macOS branch は Linux capability を readiness evidence として使わない。
- cfg Linux `native_window_host_loop_platform_wait_backend_from_selection` は `event_source_capability` を explicit input とし、caller が source capability を指定しないまま Linux backend を作れないようにする。

検証:

- Rust unit tests で、Linux-only helper と multi-backend raw API builder の両方について `ObservedInputOnly` が `LinuxEventSourceSupportFailed` になり、Linux raw API method call count が 0 のままであることを検査する。
- Rust unit tests で、`ExternallyWakeableEventSource` の場合は従来通り Linux selector / timerfd backend を構築できることを検査する。
- source policy で旧 two-argument Linux helper call、no-capability cfg Linux helper、runner / CLI dispatch、synthetic readiness、timer fired evidence、fallback / silent no-op を禁止する。

非目標:

- Linux platform wait runner、CLI dispatch、`run_linux_platform_wait_window_loop`、actual X11 / Wayland fd selector integration、minifb wait replacement、`set_target_fps 0`、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- `ExternallyWakeableEventSource` を backend construction success 以外の readiness evidence として扱わない。selector fd creation / registration は backend owner construction であり、runner readiness や scheduler resume evidence ではない。
- Windows / macOS branch の behavior は変えない。

### Phase F5ig: Native run-loop platform wait config event source gate

目的:

- F5if で Linux backend construction helper が explicit event source capability を要求するようになったため、`NativeWindowRunLoopConfig` 側にも platform wait config と Linux event source capability の保持場所を作る。
- `NativeWindowRunLoopWaitBackend::PlatformWait` を bare selection ではなく typed config にし、future Linux runner が selection だけで F5ie / F5if gate を bypass できないようにする。
- Windows CLI compatibility のため `new_with_platform_wait_backend_selection` は selection-only config を作る入口として残す。ただし cfg Linux from-config path では capability 未指定を typed error にする。

実装:

- `NativeWindowRunLoopPlatformWaitBackendConfig` を追加し、selection と optional Linux event source capability を private field として保持する。
- config は `new`、`new_with_linux_event_source_capability`、`selection`、`linux_event_source_capability` accessor からだけ組み立てる。
- `native_window_run_loop_platform_wait_backend_config` を追加し、既存の selection extractor は config から selection を読む compatibility helper にする。
- `MissingLinuxEventSourceCapability` を `NativeWindowRunLoopPlatformWaitBackendConfigError` に追加する。
- cfg Linux `native_window_run_loop_platform_wait_backend_from_config` は platform wait config 抽出後に explicit Linux capability を要求し、未指定なら `MissingLinuxEventSourceCapability` を返す。capability がある場合だけ F5if の cfg Linux helper へ渡す。
- cfg Windows from-config は config から selection を読むだけで、Linux capability を readiness evidence として扱わない。

検証:

- Rust unit tests で default minifb config、selection-only platform config、Linux capability missing error、explicit Linux event source capability accessor を検査する。
- `cargo check --target x86_64-unknown-linux-gnu` で cfg Linux from-config path が compile できることを検査する。
- source policy で private config fields、constructor / accessor、cfg Linux missing capability gate、no implicit `ExternallyWakeableEventSource`、runner / CLI dispatch / synthetic readiness / fallback 禁止を固定する。

非目標:

- Linux platform wait runner、CLI dispatch、`run_linux_platform_wait_window_loop`、actual X11 / Wayland fd selector integration、minifb wait replacement、`set_target_fps 0`、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- `new_with_platform_wait_backend_selection` を rename / remove しない。
- `ExternallyWakeableEventSource` を config に入れただけで readiness evidence、scheduler resume evidence、timer fired evidence を作らない。

### Phase F5ih: Native platform wait runner support gate

目的:

- F5ig の typed platform wait config を actual runner dispatch の前段で検査し、現在実行可能な platform wait runner と未接続 platform を型で分ける。
- Windows platform wait runner は既存の actual runner path として受理する。
- Linux は F5ie / F5if / F5ig の event source capability gate を通しても、actual X11 / Wayland fd integration または同等の externally-wakeable source owner / selector registration が未接続であるため、runner-ready とは扱わない。
- macOS は raw boundary / trait boundary までで actual sys shim / runner が未接続であるため typed unavailable にする。

実装:

- `NativeWindowRunLoopPlatformWaitRunnerSupportError` を追加し、config error、backend support error、Linux event-source support error、Linux externally-wakeable integration missing、platform runner unavailable を分ける。
- `validate_native_window_run_loop_platform_wait_runner_support_for_platform` を追加する。`NativeWindowRunLoopConfig` から platform wait config を抽出し、selection と current platform を検査する。
- Windows + `WindowsWaitableTimerMessageWait` は selection を返す。
- Linux + missing capability は `MissingLinuxEventSourceCapability`、`ObservedInputOnly` は `LinuxEventSourceSupportFailed` を返す。F5iq では、validated `ExternallyWakeableEventSource` は `PlatformRunnerIntegrationMissing LinuxWindowEventSourceFdMissing` として返した。F5it 以降の current contract では、fd なし externally-wakeable config は `MissingLinuxWindowEventSourceRawFd` として返し、fd-present config は `PlatformRunnerIntegrationMissing LinuxWindowEventSourceEventParsingMissing` として返す。
- macOS は F5ik 以降 `PlatformRunnerIntegrationMissing MacosActualSysShimMissing` として返し、unsupported platform は `PlatformRunnerUnavailable` を返す。

検証:

- Rust unit tests で Windows accepted、non-platform config rejection、macOS unavailable、Linux missing capability、Linux observed-input-only、Linux externally-wakeable integration missing、cross-platform selection rejection を検査する。
- source policy で F5ih gate が Linux raw/sys backend construction、`run_linux_platform_wait_window_loop`、CLI dispatch、minifb wait replacement、`set_target_fps 0`、synthetic `HostEventReady`、timer fired evidence、fallback / silent no-op に進まないことを固定する。

非目標:

- Linux platform wait runner、CLI dispatch、actual X11 / Wayland fd selector integration、minifb wait replacement、macOS actual sys shim、`set_target_fps 0`、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- `ExternallyWakeableEventSource` を runner readiness、scheduler resume evidence、timer fired evidence として扱わない。
- backend construction helper や raw/sys API を support gate から呼ばない。

### Phase F5ii: Native platform wait runner support gate integration

目的:

- F5ih の support gate を、現在実行可能な Windows platform wait window runner の入口へ接続する。
- runner が window、backend loop、visual host、minifb side effect を作る前に、`NativeWindowRunLoopConfig` がその runner で実行可能かを typed error として検査する。
- support validation と actual backend construction を別 stage に保ち、runner support failure を backend construction failure や string error に潰さない。

実装:

- `NativeWindowRunLoopError::PlatformWaitRunnerUnsupported` を追加し、`NativeWindowRunLoopPlatformWaitRunnerSupportError` をそのまま保持する。
- `run_windows_platform_wait_window_loop` は function entry 直後に `validate_native_window_run_loop_platform_wait_runner_support_for_platform Windows config` を呼ぶ。
- support gate 成功後にだけ `native_window_run_loop_platform_wait_backend_from_config` を呼び、さらにその後にだけ `NativeWindowBackendLoop::new_for_scale`、minifb `Window::new`、visual host construction へ進む。

検証:

- Rust unit tests で non-platform config と cross-platform config が `PlatformWaitRunnerUnsupported` として返り、backend construction stage へ defer されないことを検査する。
- source policy で Windows platform runner の順序を support gate、backend-from-config、backend loop/window creation の順に固定する。
- source policy で minifb frame pacing、`set_target_fps`、sleep、busy loop、Linux runner / raw sys construction、fallback、silent no-op、synthetic evidence を Windows platform runner に追加しないことを固定する。

非目標:

- Linux runner / CLI dispatch、`run_linux_platform_wait_window_loop`、actual X11 / Wayland fd integration、macOS actual sys shim、default minifb runner の置換、`set_target_fps 0`、synthetic `HostEventReady`、timer fired evidence、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- support validation を CLI 側へ移さない。CLI は selection / config construction だけを担当する。

### Phase F5ij: Native platform wait CLI support gate boundary

目的:

- `nepl-gui-native --wait-backend platform` の CLI selection を、Windows 専用の固定文字列 rejection ではなく、library の typed runner support gate へ通す。
- CLI は platform wait config construction と runner selection だけを担当し、actual runner readiness は `validate_native_window_run_loop_platform_wait_runner_support` に委譲する。
- non-Windows target でも minifb へ fallback せず、default selection failure または runner support failure を CLI error として返す。

実装:

- window / non-wasm 共通の `platform_wait_window_run_loop_config` を追加し、`native_window_host_loop_default_platform_wait_backend_selection` と `NativeWindowRunLoopConfig::new_with_platform_wait_backend_selection` だけで config を作る。
- window / non-wasm 共通の `validate_platform_wait_window_runner_support` を追加し、runner dispatch より前に `validate_native_window_run_loop_platform_wait_runner_support config` を呼ぶ。
- Windows branch は support gate helper 成功後に `run_windows_platform_wait_window_loop` を呼ぶ。library runner 側の support gate は authoritative boundary として残す。
- non-Windows branch は同じ helper を通し、support validation が現在の未接続 platform で失敗すれば typed reason を CLI error にする。将来 support validation が成功したのに runner dispatch が未実装なら explicit dispatch-unavailable error を返す。

検証:

- source policy で shared config builder、support-gate helper、Windows runner call before-dispatch validation、non-Windows branch の shared validation、Linux capability injection 禁止を固定する。
- Rust bin tests で platform wait config builder が `PlatformWait` backend を作ることを検査する。
- `cargo test -p nepl-gui-native --features window --bin nepl-gui-native`、native library tests、source policy、Linux target check で regression を検査する。

非目標:

- `run_linux_platform_wait_window_loop`、Linux CLI dispatch、actual X11 / Wayland fd integration、macOS actual sys shim、minifb wait replacement、`set_target_fps 0`、synthetic `HostEventReady`、timer fired evidence、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- CLI で `ExternallyWakeableEventSource` や `ObservedInputOnly` を注入しない。Linux runner readiness を CLI 側で偽装しない。
- support validation を CLI だけへ移さない。library runner entry の validation は残す。

### Phase F5ik: Native platform wait runner missing-integration reason boundary

目的:

- platform wait selection と capability validation は通るが actual runner readiness に必要な実体が未接続である場合を、generic unavailable ではなく typed missing integration reason として返す。
- Linux の `ExternallyWakeableEventSource` は分類値であり、actual X11 / Wayland window event source fd integration または同等の externally-wakeable platform event source が無い限り runner-ready ではないことを enum で固定する。
- macOS は raw / trait boundary まで存在するが actual sys shim / run-loop ownership が無いため、unsupported platform とは別の missing integration として表す。

実装:

- `NativeWindowRunLoopPlatformWaitRunnerMissingIntegration` を追加し、Linux の missing integration と `MacosActualSysShimMissing` を分ける。F5iq 時点の Linux variant は `LinuxWindowEventSourceFdMissing` だった。F5it 以降の current contract では、fd なし externally-wakeable config は config error、fd-present config は `LinuxWindowEventSourceEventParsingMissing` である。
- `NativeWindowRunLoopPlatformWaitRunnerSupportError::PlatformRunnerIntegrationMissing` を追加し、validated selection と missing reason を保持する。
- Linux + missing capability は `Config MissingLinuxEventSourceCapability`、`ObservedInputOnly` は `LinuxEventSourceSupportFailed` のまま維持する。validated `ExternallyWakeableEventSource` だけを integration-missing に写す。
- Windows + `WindowsWaitableTimerMessageWait` はこれまで通り accepted selection を返す。unsupported platform は `PlatformRunnerUnavailable` のままにする。

検証:

- Rust unit tests で Linux externally-wakeable と macOS が `PlatformRunnerIntegrationMissing` を返すことを検査する。
- source policy で new enum / variant、旧 Linux 専用 integration-missing variant の削除、support gate が backend / sys / window / minifb / fallback を呼ばないことを固定する。

非目標:

- `run_linux_platform_wait_window_loop`、Linux CLI dispatch、actual X11 / Wayland fd integration、macOS actual sys shim、CoreFoundation / AppKit binding、minifb wait replacement、`set_target_fps 0`、synthetic `HostEventReady`、timer fired evidence、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- `ExternallyWakeableEventSource` を runner readiness、scheduler resume evidence、timer fired evidence として扱わない。

### Phase F5il: Native Linux externally-wakeable event source owner boundary

目的:

- F5ik の `LinuxExternallyWakeableEventSourceOwnerMissing` を解消する前段として、Linux selector / timerfd backend と external host-event signal producer を同一 owner に束ねる contract を追加する。
- `ExternallyWakeableEventSource` を分類値のままにせず、actual readiness source へ進むためには backend owner と wake producer owner が同時に存在する必要があることを型で表す。
- producer 作成に失敗した場合、backend owner を失わず typed error で返す。

実装:

- `NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwner` を追加し、`NativeWindowHostLoopLinuxSelectorTimerFdBackend` と `NativeWindowHostLoopLinuxHostEventSignalProducer` を private field として保持する。
- builder は既に構築済みの Linux selector / timerfd backend と producer raw API を受け取り、backend が open であることを確認してから `create_host_event_signal_producer` を呼ぶ。
- backend が閉じている場合は `BackendClosed`、producer 作成失敗は `HostEventSignalProducerFailed` として backend owner と typed lower error を返す。
- owner の `signal_host_event` は producer にだけ委譲し、backend 側の direct signal や wait outcome synthesis は行わない。

検証:

- Rust unit tests で owner builder が backend と producer を保持し、producer duplicate が host event fd に対して行われることを検査する。
- Rust unit tests で owner の signal が producer raw API だけを使い、backend direct signal を使わないことを検査する。
- Rust unit tests で producer 作成失敗と closed backend が backend owner を返すことを検査する。
- source policy で owner fields、builder の `create_host_event_signal_producer` 呼び出し、owner-bearing error、Linux runner / CLI / minifb / sleep / fallback / synthetic evidence 非導入を固定する。

非目標:

- Linux platform wait runner support gate を `Ok` にしない。
- `run_linux_platform_wait_window_loop`、Linux CLI dispatch、actual X11 / Wayland fd integration、macOS actual sys shim、CoreFoundation / AppKit binding、minifb wait replacement、`set_target_fps 0`、synthetic `HostEventReady`、timer fired evidence、scheduler-ready evidence、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- minifb `InputCallback` を blocking wait readiness source として扱わない。

### Phase F5im: Native Linux externally-wakeable event source run-loop host boundary

目的:

- F5il owner を backend と producer に public 分解せず、owner 全体を保持したまま `NativeWindowRunLoopHost` の wait hook として使える境界を追加する。
- Linux selector / timerfd backend による wait と external signal producer の所有を同じ wrapper に閉じ、producer を落として backend だけを既存 single-owner wrapper へ渡す経路を閉じる。
- wait hook は backend-observed readiness だけを outcome へ写し、signal request は producer にだけ委譲する。

実装:

- `NativeWindowHostLoopLinuxExternallyWakeableEventSourceWaitAdapter` を追加し、`next_raw_id` と `NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwner` を保持する。
- `execute_native_window_host_loop_linux_externally_wakeable_event_source_wait_with_adapter` を追加し、host event wait と frame interval wait を owner 内 backend へ委譲する。
- `NativeWindowHostLoopLinuxExternallyWakeableEventSourceRunLoopHost` を追加し、visual host operation は inner host、wait operation は owner-retaining adapter へ委譲する。
- owner の public `into_parts` を削除し、backend 単体を owner から public に取り出せないようにする。

検証:

- Rust unit tests で owner-retaining run-loop host が host event wait を backend へ渡し、inner host の wait hook を呼ばないことを検査する。
- Rust unit tests で frame interval wait 後も producer が保持され、signal 可能であることを検査する。
- source policy で public owner split、backend-only wrapper への逃げ道、Linux runner / CLI / minifb / fallback / synthetic evidence 非導入を固定する。

非目標:

- Linux platform wait runner support gate を `Ok` にしない。
- `run_linux_platform_wait_window_loop`、Linux CLI dispatch、actual X11 / Wayland fd integration、macOS actual sys shim、CoreFoundation / AppKit binding、minifb wait replacement、`set_target_fps 0`、sleep、busy loop、fallback、silent no-op、synthetic `HostEventReady`、timer fired evidence、scheduler-ready evidence、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- owner-retaining adapter を generic platform wait backend selection や minifb default runner へ接続しない。

### Phase F5in: Native Linux platform-wait owner-ready run-loop input boundary

目的:

- Linux runner 実装の前段として、validated platform-wait config と externally-wakeable owner 全体を同じ input object に保持する。
- generic support gate の Linux `PlatformRunnerIntegrationMissing` behavior を維持したまま、runner 専用の下位 validation 境界を分離する。
- fallible path で config と owner を失わず、backend-only / producer-dropping path を public API にしない。

実装:

- `NativeWindowLinuxPlatformWaitRunLoopInputBuildError` を追加し、config failure、platform unavailable、wrong current platform、backend support failure、Linux event-source support failure、closed owner を分ける。すべての failure は config と owner を保持して返す。
- `NativeWindowLinuxPlatformWaitRunLoopInput` を追加し、config と full owner を private field として保持する。`into_parts` は config と full owner だけを返す。
- `native_window_linux_platform_wait_run_loop_input_for_platform` を追加し、config extraction、current platform、Linux backend kind、selection platform、Linux capability、owner open state を順に検査する。
- current-platform wrapper は `native_window_host_loop_current_platform_kind` を渡すだけにし、generic runner support gate や backend construction helper は呼ばない。

検証:

- Rust unit tests で open owner accepted、missing Linux capability、observed-input-only、wrong current platform、closed owner を検査する。
- source policy で full owner retention、generic support gate 非呼び出し、runner / CLI / minifb / sys API / fallback / synthetic readiness 非導入を固定する。

非目標:

- Linux platform wait runner support gate を `Ok` にしない。
- `run_linux_platform_wait_window_loop`、Linux CLI dispatch、actual X11 / Wayland fd integration、macOS actual sys shim、CoreFoundation / AppKit binding、minifb wait replacement、sleep、busy loop、fallback、silent no-op、synthetic `HostEventReady`、timer fired evidence、scheduler-ready evidence、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- owner-ready input を generic platform wait backend selection や minifb default runner へ接続しない。

### Phase F5io: Native Linux owner-ready input-to-host boundary

目的:

- F5in の owner-ready input を F5im の owner-retaining run-loop host へ渡す明示的な boundary を追加する。
- visual host owner と Linux owner-ready input を失敗時にも回収できる `Result` にし、runner 実装前の host wrapping contract を固定する。
- generic support gate の Linux `PlatformRunnerIntegrationMissing` behavior を維持したまま、input-to-host handoff だけを進める。

実装:

- `NativeWindowLinuxPlatformWaitRunLoopHostBuildError` を追加し、config failure、backend support failure、Linux event-source support failure、closed owner を分ける。すべての failure は host と input を保持して返す。
- `native_window_host_loop_linux_platform_wait_run_loop_host_from_input` を追加し、platform-wait config、Linux backend kind、selection platform、Linux capability、owner open state を再検査する。
- success path は `input.into_parts` で config と full owner を取り出し、full owner だけを `native_window_host_loop_linux_externally_wakeable_event_source_run_loop_host_from_owner` へ渡す。

検証:

- Rust unit tests で accepted input が visual host operation を inner host へ委譲し、wait を Linux owner backend へ渡すことを検査する。
- Rust unit tests で F5in input 作成後に owner が close された場合、`OwnerClosed` が host と input を返すことを検査する。
- source policy で generic support gate 呼び出し、runner / CLI / minifb / sys API、fallback / sleep / synthetic readiness、backend-only / producer-only extraction を禁止する。

非目標:

- Linux platform wait runner support gate を `Ok` にしない。
- `run_linux_platform_wait_window_loop`、Linux CLI dispatch、actual X11 / Wayland fd integration、macOS actual sys shim、CoreFoundation / AppKit binding、minifb wait replacement、sys API construction、sleep、busy loop、fallback、silent no-op、synthetic `HostEventReady`、timer fired evidence、scheduler-ready evidence、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- input-to-host helper を generic platform wait backend selection や minifb default runner へ接続しない。

### Phase F5ip: Native Linux cfg sys host factory boundary

目的:

- F5io の input-to-host handoff を、cfg Linux sys API を注入する host factory として組み立てる。
- runner / CLI dispatch へ進む前に、config validation、sys backend construction、owner construction、F5in input construction、F5io handoff の順序を型とテストで固定する。
- validation failure、backend failure、owner failure、input failure、host handoff failure を string 化せず、host / config / lower owner-bearing error を保持して返す。

実装:

- `NativeWindowLinuxPlatformWaitRunLoopHostFromConfigBuildError` を追加し、config、backend support、Linux event-source support、backend build、owner build、input build、host build の failure stage を分ける。
- `native_window_host_loop_linux_platform_wait_run_loop_host_from_config_with_apis` を追加し、scripted backend API / producer API を注入できるようにする。
- helper は `PlatformWait` config、Linux backend kind、selection platform、Linux event-source capability を raw API construction より前に検査し、成功時だけ Linux selector / timerfd backend、externally-wakeable owner、F5in input、F5io run-loop host を順に作る。
- cfg Linux wrapper `native_window_host_loop_linux_platform_wait_run_loop_host_from_config` は、backend owner 用と producer owner 用に別々の `NativeWindowHostLoopLinuxSelectorTimerFdSysApi::new` を注入するだけにする。

検証:

- Rust unit tests で injected helper の success path が owner-retaining host を作り、wait が Linux owner backend を通ることを検査する。
- Rust unit tests で config validation failure が raw API construction 前に止まり、backend build failure と owner build failure が host / config / lower owner を保持することを検査する。
- source policy で validation before raw construction、F5in / F5io helper 経由、cfg sys wrapper の inject-only contract、generic support gate / runner / CLI / minifb / fallback / synthetic readiness 非導入を固定する。

非目標:

- Linux platform wait runner support gate を `Ok` にしない。
- `run_linux_platform_wait_window_loop`、Linux CLI dispatch、actual X11 / Wayland fd integration、macOS actual sys shim、CoreFoundation / AppKit binding、minifb wait replacement、support-gate readiness、sleep、busy loop、fallback、silent no-op、synthetic `HostEventReady`、timer fired evidence、scheduler-ready evidence、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。
- helper を generic platform wait backend selection や minifb default runner へ接続しない。

### Phase F5iq: Native Linux window event source fd missing reason boundary

目的:

- F5il から F5ip で Linux externally-wakeable owner、owner-retaining wait hook、owner-ready input、input-to-host handoff、cfg sys host factory まで接続済みになったため、generic support gate の missing reason を実態に合わせる。
- Linux support gate は引き続き fail closed とし、未接続理由を owner missing ではなく actual X11 / Wayland window event source fd integration または同等の externally-wakeable platform event source missing として表す。
- host-event `eventfd` producer と window / platform event source fd を混同しない。

実装:

- `NativeWindowRunLoopPlatformWaitRunnerMissingIntegration` の Linux variant を `LinuxWindowEventSourceFdMissing` に更新し、既存の `NativeWindowHostLoopLinuxEventSourceCapability` を保持する。
- `validate_native_window_run_loop_platform_wait_runner_support_for_platform` は Linux validated `ExternallyWakeableEventSource` に対して引き続き `PlatformRunnerIntegrationMissing` を返す。
- source policy で旧 `LinuxExternallyWakeableEventSourceOwnerMissing` 名を拒否し、support gate が runner / CLI / minifb / fallback / sleep / synthetic readiness へ進まないことを固定する。

検証:

- Rust unit tests で Linux externally-wakeable support gate failure が `LinuxWindowEventSourceFdMissing` になることを検査する。
- source policy で new variant、old variant rejection、runner / CLI / sys / minifb / fallback 非導入を検査する。

非目標:

- `run_linux_platform_wait_window_loop`、Linux CLI dispatch、actual X11 / Wayland fd integration、macOS actual sys shim、minifb wait replacement、sleep、busy loop、fallback、silent no-op、synthetic `HostEventReady`、timer fired evidence、scheduler-ready evidence は実装しない。

### Phase F5ir: Native Linux window event source fd registration boundary

目的:

- F5iq の `LinuxWindowEventSourceFdMissing` の後続として、actual X11 / Wayland 等から得た window event source fd を Linux selector / timerfd backend に登録できる境界を追加する。
- host-event wake 用 `eventfd` と window / platform event source fd を型で分ける。window event source fd は backend が所有せず、登録解除だけを行い、close は呼ばない。
- duplicate registration、負値 fd、raw registration failure、raw unregister failure を `Result` と enum error で表し、silent no-op や fallback にしない。

実装:

- `NativeWindowHostLoopLinuxWindowEventSourceFd` を追加し、fd `0` を有効、負値だけ invalid とする checked constructor を用意する。
- `NativeWindowHostLoopLinuxSelectorTimerFdRawApi` に `register_window_event_source_fd_raw` と `unregister_window_event_source_fd_raw` を追加する。raw API 名は host-event fd と混同しないよう window-specific にする。
- `NativeWindowHostLoopLinuxSelectorTimerFdBackend` は `Option NativeWindowHostLoopLinuxWindowEventSourceFd` を保持し、duplicate registration は raw API 呼び出し前に拒否する。
- `try_close_handles_if_open` は registered window event source fd の unregister を先に試み、unregister failure では token を戻して owned selector / timerfd / host-event fd を閉じない。`close_handles_if_open` は compatibility cleanup として残すが、fallible cleanup の authority は `try_close_handles_if_open` とする。
- cfg Linux sys shim は external fd を `epoll_ctl EPOLL_CTL_ADD` / `EPOLL_CTL_DEL` で selector に出し入れするだけで、external fd を close しない。

検証:

- Rust unit tests で fd `0` valid / negative invalid、success path の register/unregister call、invalid / duplicate / raw failure before-state-preservation、unregister failure で owned handles を閉じないことを検査する。
- source policy で window-specific raw API、non-owning fd、duplicate before raw call、fallible cleanup helper、external fd close 禁止、support gate / runner / CLI / minifb / fallback / synthetic readiness 非導入を固定する。

非目標:

- actual X11 / Wayland fd acquisition、window event source readiness status classification、`run_linux_platform_wait_window_loop`、Linux CLI dispatch、Linux support gate の `Ok` 化、macOS actual sys shim、minifb wait replacement、sleep、busy loop、fallback、silent no-op、synthetic `HostEventReady`、timer fired evidence、scheduler-ready evidence は実装しない。

### Phase F5is: Native Linux window event source readiness status boundary

目的:

- F5ir で登録できるようになった external window event source fd の readiness を、Linux selector 層で timer / host-event `eventfd` と別 status として分類する。
- window event source readiness は event pump へ戻るための non-timer wake であり、timer fired evidence、scheduler completion、Linux runner readiness ではないことを型と test で固定する。
- host-event-only wait は host-event `eventfd` だけを success とし、window event source readiness を silent success にしない。

実装:

- `NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_WINDOW_EVENT_SOURCE_READY` と `NativeWindowHostLoopLinuxSelectorTimerFdWake::WindowEventSourceReady` を追加する。
- registered window event source fd がある場合だけ、`selector_wait_for_timer_host_or_window_event_raw selector timer host_event window_event_source` を呼ぶ。registered fd が無い場合は従来の `selector_wait_for_timer_or_event_raw` を使う。
- cfg Linux sys shim は `epoll_wait` の `event.u64` が window event source fd と一致した場合、fd を read / drain / close せず `WINDOW_EVENT_SOURCE_READY` を返す。
- `NativeWindowHostLoopInterruptibleDeadlineWaiter` 実装では、`WindowEventSourceReady` を `HostEventReady` と同じ non-timer wake として写し、event pump へ戻す。Linux selector enum では source の区別を保持する。

検証:

- Rust unit tests で status mapping、host-event-only strictness、registered window fd path の new raw method 呼び出し、run-loop adapter が window fd readiness を event pump wake として扱うことを検査する。
- source policy で new status / enum / raw method、registered path only、external fd read / drain / close 禁止、support gate / runner / CLI / minifb / fallback / synthetic timer evidence 非導入を固定する。

非目標:

- actual X11 / Wayland fd acquisition、X11 / Wayland event parsing、`run_linux_platform_wait_window_loop`、Linux CLI dispatch、Linux support gate の `Ok` 化、macOS actual sys shim、minifb wait replacement、sleep、busy loop、fallback、silent no-op、timer fired evidence の偽装、scheduler-ready evidence は実装しない。

### Phase F5it: Native Linux window event source fd config-to-registration boundary

目的:

- F5ir / F5is で追加した fd registration / readiness classification を、低レベル Linux `from_config_with_apis` builder から実際に使えるようにする。
- `ExternallyWakeableEventSource` という分類値だけで owner / host を構築できる穴を閉じ、actual fd を config に明示する。
- fd が config にある場合、generic support gate の missing reason を `LinuxWindowEventSourceFdMissing` ではなく event decoding / runner dispatch missing として truthfully fail-closed にする。

実装:

- `NativeWindowRunLoopPlatformWaitBackendConfig` に optional `linux_window_event_source_raw_fd` を追加する。
- `new_with_linux_window_event_source_raw_fd selection raw_fd` は capability を `ExternallyWakeableEventSource` にし、raw fd を non-owning value として保持する。
- `MissingLinuxWindowEventSourceRawFd` config error と `LinuxWindowEventSourceEventParsingMissing` integration-missing reason を追加する。
- `native_window_host_loop_linux_platform_wait_run_loop_host_from_config_with_apis` は capability validation 後、raw backend construction 前に raw fd を要求する。
- backend construction 後、owner construction 前に `register_window_event_source_fd_from_raw` を呼ぶ。registration failure は host、config、backend、typed registration error を保持して返す。
- generic support gate は Linux fd-present config でも `Ok` を返さず、event parsing / runner dispatch missing として fail-closed にする。

検証:

- Rust unit tests で raw fd `0` が config に保持されること、fd なし externally-wakeable config が raw API 呼び出し前に失敗すること、fd-present support gate が event parsing missing で fail-closed になることを検査する。
- Rust unit tests で `from_config_with_apis` が fd を register してから owner を作り、window readiness を event pump wake へ写すこと、raw registration failure が backend を保持することを検査する。
- source policy で CLI raw-fd option、runner dispatch、support-gate `Ok`、minifb promotion、fd read / drain / close、synthetic readiness / timer evidence 非導入を固定する。

非目標:

- actual X11 / Wayland fd acquisition、X11 / Wayland event parsing、`run_linux_platform_wait_window_loop`、Linux CLI dispatch、Linux support gate の `Ok` 化、macOS actual sys shim、minifb wait replacement、sleep、busy loop、fallback、silent no-op、timer fired evidence の偽装、scheduler-ready evidence は実装しない。

### Phase F5iu: Native Linux window event source provider-owned platform config boundary

目的:

- F5it の raw fd config を、fd owner を落とさない provider-owned boundary へ進める。
- actual X11 / Wayland backend や toolkit が持つ event source fd は non-owning raw value として config へ写せるが、その fd の owner / event decoder provider は prepared value が保持し続ける。
- selection と provider だけから full `NativeWindowRunLoopConfig` を作らない。demo、counter、scale、target FPS、run policy は caller が既存 constructor へ明示的に渡す。

実装:

- `NativeWindowLinuxWindowEventSourceKind` と `NativeWindowLinuxWindowEventSourceDescriptor` を追加する。kind は `X11Connection`、`WaylandDisplay`、`ToolkitExternal` を分け、descriptor は kind と raw fd だけを持つ。
- `NativeWindowLinuxWindowEventSourceProvider` trait を追加し、provider が `window_event_source_descriptor` を 1 回返す境界にする。
- `NativeWindowLinuxWindowEventSourcePreparedPlatformWaitConfig Provider` を追加し、`NativeWindowRunLoopPlatformWaitBackendConfig`、descriptor、provider owner を同時に保持する。prepared value は provider に対して Clone / Copy を実装しない。
- `native_window_linux_window_event_source_prepare_platform_wait_backend_config selection provider` を追加する。Linux + `LinuxSelectorTimerFd` selection を検査してから provider を 1 回だけ呼び、descriptor fd を既存 typed fd validation で検査し、`NativeWindowRunLoopPlatformWaitBackendConfig::new_with_linux_window_event_source_raw_fd` だけを作って prepared value に入れる。
- error は provider owner を保持し、selection validation failure、provider failure、descriptor fd validation failure を enum で分ける。

検証:

- Rust unit tests で fd `0` が valid descriptor として prepared platform config に残ること、provider owner が success/error で回収できること、provider failure と negative fd が型付き error になることを検査する。
- Rust unit tests で Windows selection などの invalid selection が provider call 前に拒否されることを検査する。
- Rust unit tests で caller が prepared platform config を明示的に `NativeWindowRunLoopConfig::new_with_platform_wait_backend_config` へ渡した場合でも、Linux support gate は `LinuxWindowEventSourceEventParsingMissing` のまま fail-closed になることを検査する。
- source policy で F5iu helper が full `NativeWindowRunLoopConfig` を作らないこと、CLI raw-fd option、runner dispatch、minifb fallback、fd read / drain / close、synthetic readiness、fallback / silent no-op を導入しないことを固定する。

非目標:

- actual X11 / Wayland fd acquisition、X11 / Wayland event parsing、provider から event object を drain する処理、`run_linux_platform_wait_window_loop`、Linux CLI dispatch、Linux support gate の `Ok` 化、macOS actual sys shim、minifb wait replacement、sleep、busy loop、fallback、silent no-op、timer fired evidence の偽装、scheduler-ready evidence は実装しない。

### Phase F5iv: Native Linux window event source provider-owned run-loop handoff boundary

目的:

- F5iu の provider-owned platform-wait config を、provider owner を落とさず full `NativeWindowRunLoopConfig` と Linux owner-retaining run-loop host へ渡せる境界にする。
- selection + provider だけから full config を作らない方針を維持し、demo、counter、scale、target FPS、run policy は caller の明示引数として受ける。
- provider owner は descriptor fd の所有元または event decoder owner として prepared run-loop value / host wrapper が保持し続け、raw fd value だけを config / backend に残して provider を drop しない。

実装:

- `NativeWindowLinuxWindowEventSourcePreparedRunLoopConfig Provider` を追加し、`NativeWindowRunLoopConfig`、descriptor、provider owner を同時に保持する。constructor は private にし、F5iu prepared platform config からだけ作る。config-only escape path は作らず、`into_config`、consuming `config self`、provider を返さない split は禁止する。許可するのは borrowed accessor と、config / descriptor / provider を同時に返す `into_parts` だけである。
- `native_window_linux_window_event_source_prepare_run_loop_config` を追加する。F5iu prepared platform config と caller 明示の demo / counter / scale / target FPS / run policy を受け、`NativeWindowRunLoopConfig::new_with_platform_wait_backend_config` で full config を組み立て、provider owner と descriptor を保持する。
- `NativeWindowLinuxWindowEventSourceRunLoopHost Host BackendApi ProducerApi Provider` を追加し、既存の Linux owner-retaining run-loop host、descriptor、provider owner を同時に保持する。host-only escape path は作らず、`into_host`、consuming `host self`、provider を返さない split は禁止する。許可するのは borrowed accessor と、host / descriptor / provider を同時に返す `into_parts` だけである。
- `native_window_host_loop_linux_platform_wait_run_loop_host_from_prepared_window_event_source_with_apis` を追加する。入力は visual `host`、prepared run-loop config、`backend_api`、`producer_api` の 4 つとする。prepared run-loop config を consume し、既存 `native_window_host_loop_linux_platform_wait_run_loop_host_from_config_with_apis` を 1 回だけ呼び、success では host wrapper と provider owner を保持し、failure では provider owner、descriptor、lower owner-bearing error を返す。lower error に含まれる host / config / backend recovery は潰さない。

検証:

- Rust unit tests で explicit demo / counter / scale / target FPS / run policy が full config に反映され、provider owner と descriptor が prepared run-loop config に残ることを検査する。
- Rust unit tests で host build success 後も provider owner が host wrapper に残り、window event source fd が existing Linux selector backend へ register されることを検査する。
- Rust unit tests で backend build / registration / owner build failure が provider owner と descriptor と lower error を保持して返ることを検査する。
- source policy で F5iv helper が selection + provider だけから full config を作らないこと、prepared config / host wrapper に public constructor bypass や config-only / host-only escape path を作らないこと、support gate `Ok` 化、runner / CLI dispatch、minifb fallback、fd read / drain / close、event decoding、synthetic readiness を導入しないことを固定する。

非目標:

- actual X11 / Wayland fd acquisition、X11 / Wayland event parsing、provider event drain / read、`run_linux_platform_wait_window_loop`、Linux CLI dispatch、Linux support gate の `Ok` 化、macOS actual sys shim、minifb wait replacement、sleep、busy loop、fallback、silent no-op、timer fired evidence の偽装、scheduler-ready evidence は実装しない。

### Phase F5iw: Native Linux window event source provider-owned event pump boundary

目的:

- F5iv で保持した provider owner を、actual X11 / Wayland binding ではなく抽象 event pump provider として `NativeWindowEventPumpSnapshot` 供給境界へ接続する。
- fd readiness は selector / wait 層の wake evidence に留め、window event の drain / decode authority は provider owner 側に置く。
- support gate `Ok` 化、runner / CLI dispatch、minifb fallback、fd read / drain / close、X11 / Wayland concrete parsing を導入せず、provider-owned event pump contract だけを型で固定する。

実装:

- `NativeWindowLinuxWindowEventSourceEventPumpProvider` trait を追加し、descriptor と `NativeWindowEventPumpInput` から `NativeWindowEventPumpSnapshot` を返す。error は provider-local enum / error type とし、string fallback や silent no-op にしない。
- `NativeWindowLinuxWindowEventSourceEventPumpRunLoopHost Host BackendApi ProducerApi Provider` を追加し、F5iv host wrapper、descriptor、provider owner を保持したまま event pump provider を `NativeWindowRunLoopHost::poll_event_snapshot` に接続する。
- event pump wrapper は `set_window_title`、`pump_events_only`、`present_frame`、`wait_after_budget_exhausted` を lower host へ委譲する。`poll_event_snapshot` だけ provider owner を使い、lower host event pump と provider event pump を同時 authority にしない。
- provider poll failure は `NativeWindowLinuxWindowEventSourceEventPumpRunLoopHostError::ProviderPollFailed { descriptor, error }` のような wrapper error に写す。`poll_event_snapshot &mut self` は provider owner を value として返せないため、failure 後も wrapper が lower host と provider owner を保持し続け、error は descriptor と provider-local error だけを持つ。
- conversion helper は F5iv host wrapper を consume する infallible helper とする。validation や backend construction は F5iv までに済んでいるため、ここで fallible owner recovery contract は作らない。
- event pump wrapper は host-only / provider-only escape path を作らず、borrowed accessor と owner を同時に返す `into_parts` だけを許す。
- source policy で F5iw wrapper の `poll_event_snapshot` が provider を呼び、`self.host.poll_event_snapshot` を呼ばないこと、`set_window_title` / `pump_events_only` / `present_frame` / `wait_after_budget_exhausted` が lower host に委譲されることを固定する。あわせて support gate `Ok` 化、runner / CLI dispatch、minifb fallback、fd read / drain / close、actual X11 / Wayland API、synthetic readiness、timer-fired evidence を禁止する。

検証:

- Rust unit tests で provider event pump が descriptor と input を受け取り、snapshot を返すことを検査する。
- Rust unit tests で provider event pump failure が typed error になり、silent no-op や lower host event pump fallback にならず、failure 後も wrapper が provider owner を保持していることを検査する。
- Rust unit tests で title / pump-only / present / wait は lower F5iv host へ委譲され、poll だけ provider authority になることを検査する。
- source policy で event pump wrapper が runner dispatch / CLI / fd read-drain-close / support gate `Ok` 化へ進んでいないことを固定する。

非目標:

- actual X11 / Wayland fd acquisition、X11 / Wayland concrete event parsing、provider 内部での fd read / drain / close、`run_linux_platform_wait_window_loop`、Linux CLI dispatch、Linux support gate の `Ok` 化、macOS actual sys shim、minifb wait replacement、sleep、busy loop、fallback、silent no-op、timer fired evidence の偽装、scheduler-ready evidence は実装しない。

### Phase F5ix: Native Linux window event source normalized observation boundary

目的:

- F5iw の provider-owned event pump を、future X11 / Wayland / toolkit event decoder が共有できる normalized observation から `NativeWindowEventPumpSnapshot` を作る境界へ分割する。
- descriptor と `NativeWindowEventPumpInput` と observation を同時に扱い、previous size / mouse state に基づく transition と resize 判定は標準 helper 側で行う。
- provider ごとに pointer finite check、close-state priority、surface-state derivation、size_changed 計算を重複実装しない。

実装:

- `NativeWindowLinuxWindowEventSourceObservation` を追加し、os close、exit shortcut、current size、mouse down、optional raw pointer を保持する。
- `NativeWindowLinuxWindowEventSourceObservationSnapshotError` を追加し、snapshot 生成 failure を `SnapshotConstructionFailed { descriptor, error }` のように descriptor 付き typed error へ写す。
- `native_window_linux_window_event_source_snapshot_from_observation` を追加し、descriptor、`NativeWindowEventPumpInput`、observation から `NativeWindowEventPumpSnapshot` を返す。実装は既存 `build_native_window_event_pump_snapshot_from_raw` を通し、pointer finite check、close-state priority、surface-state derivation を共通化する。
- `NativeWindowLinuxWindowEventSourceObservationProvider` trait と、その provider を F5iw の `NativeWindowLinuxWindowEventSourceEventPumpProvider` として使う adapter を追加する。adapter は provider owner を保持し、provider observation failure と snapshot failure を別 variant で返す。
- adapter は host / runner / CLI / wait backend を持たず、observation provider owner だけを event pump provider へ変換する。support gate `Ok` 化や `run_linux_platform_wait_window_loop` への接続は行わない。
- source policy で snapshot helper が descriptor + input + observation を要求し、既存 raw snapshot helperを通すこと、adapter が provider observation を呼んでから snapshot helper を呼ぶこと、fd read / drain / close、actual X11 / Wayland API、runner / CLI dispatch、support gate `Ok` 化、fallback、silent no-op を導入しないことを固定する。

検証:

- Rust unit tests で normalized observation から resize / pointer transition / close-state priority / surface-state が既存 event pump snapshot と一致することを検査する。
- Rust unit tests で non-finite pointer が descriptor 付き typed error になり、silent no-op や unavailable pointer へ変換されないことを検査する。
- Rust unit tests で observation provider adapter が provider owner を保持し、observation failure と snapshot failure を分け、F5iw event pump provider として使えることを検査する。
- source policy で F5ix が actual fd acquisition / read / drain / close / runner dispatch / CLI dispatch / support gate `Ok` 化へ進んでいないことを固定する。

非目標:

- actual X11 / Wayland fd acquisition、X11 / Wayland concrete event parsing、provider 内部での fd read / drain / close、`run_linux_platform_wait_window_loop`、Linux CLI dispatch、Linux support gate の `Ok` 化、macOS actual sys shim、minifb wait replacement、sleep、busy loop、fallback、silent no-op、timer fired evidence の偽装、scheduler-ready evidence は実装しない。

### Phase F5iy: Native Linux window event source observation run-loop adapter boundary

目的:

- F5iv の provider-owned run-loop host を、F5ix の observation provider adapter へ infallible に接続する。
- future X11 / Wayland / toolkit decoder が `NativeWindowLinuxWindowEventSourceProvider` と `NativeWindowLinuxWindowEventSourceObservationProvider` を実装すれば、snapshot authority として provider-owned run-loop に入れる境界を作る。
- descriptor provider owner と observation provider owner を別 owner に分けず、F5iv で保持した同じ provider owner を adapter で包んで event pump provider にする。

実装:

- `native_window_linux_window_event_source_enable_observation_provider_event_pump` を追加する。
- helper は `NativeWindowLinuxWindowEventSourceRunLoopHost Host BackendApi ProducerApi Provider` を consume し、`host.into_parts` で lower host、descriptor、provider owner を同時に取り出す。
- provider owner を `native_window_linux_window_event_source_observation_event_pump_provider provider` で包み、`NativeWindowLinuxWindowEventSourceEventPumpRunLoopHost` へ渡す。
- helper は infallible とし、validation、backend construction、support gate、runner / CLI dispatch、fd read / drain / close、actual X11 / Wayland API、minifb fallback は行わない。
- source policy で helper が `into_parts`、observation adapter、event pump run-loop host constructor だけを使うこと、support gate `Ok` 化、runner / CLI dispatch、fd IO、fallback、silent no-op を導入しないことを固定する。

検証:

- Rust unit tests で observation provider を持つ F5iv host wrapper が helper 経由で event pump run-loop host になり、provider observation から snapshot が得られることを検査する。
- Rust unit tests で observation failure と snapshot construction failure が既存 F5ix adapter error として伝播し、helper 自体が owner-dropping error path を持たないことを検査する。
- Rust unit tests で lower host の title / pump-only / present / wait delegation が維持されることを検査する。
- source policy で F5iy helper が actual fd acquisition / read / drain / close / runner dispatch / CLI dispatch / support gate `Ok` 化へ進んでいないことを固定する。

非目標:

- actual X11 / Wayland fd acquisition、X11 / Wayland concrete event parsing、provider 内部での fd read / drain / close、`run_linux_platform_wait_window_loop`、Linux CLI dispatch、Linux support gate の `Ok` 化、macOS actual sys shim、minifb wait replacement、sleep、busy loop、fallback、silent no-op、timer fired evidence の偽装、scheduler-ready evidence は実装しない。

### Phase F5iz: Native Linux window event source fd acquisition boundary

目的:

- actual X11 / Wayland event decoding へ進む前に、Linux window event source fd acquisition を typed sys boundary と provider owner boundary として固定する。
- acquisition を runner readiness と偽装せず、local Unix domain socket fd を取得して provider owner が保持するところまでに限定する。
- environment / display form / path length / socket / connect / close failure を enum error と `Result` で表し、fallback、silent no-op、panic にしない。

実装:

- `NativeWindowLinuxWindowEventSourceFdAcquisitionKind`、environment variable enum、typed acquisition error を追加する。
- `NativeWindowLinuxWindowEventSourceFdAcquisitionRawApi` を追加し、environment read、Unix stream socket create、connect、close、last error code を trait-injected contract にする。
- cfg Linux の `NativeWindowLinuxWindowEventSourceFdAcquisitionSysApi` は `std::env::var`、`libc::socket AF_UNIX SOCK_STREAM SOCK_CLOEXEC`、`libc::connect`、`libc::close` だけを行う薄い wrapper とする。
- Wayland は `XDG_RUNTIME_DIR` + relative `WAYLAND_DISPLAY` だけを受け、absolute path、slash-containing name、path length overflow を typed error にする。
- X11 は `:N`、`:N.screen`、`unix/:N`、`unix/:N.screen` だけを受け、host / tcp display form、invalid display number、path length overflow を typed error にする。
- acquired fd owner は `Copy` / `Clone` を持たず、private `Option<i32>` state を保持する。明示 close は typed `Result`、`Drop` は best-effort cleanup とする。
- owned fd provider は open fd の descriptor だけを返し、明示 close 後は stale descriptor ではなく typed `Closed` error を返す。

検証:

- Rust unit tests で trait-injected Wayland success、missing env、unsupported absolute display form、path too long、connect failure cleanup、X11 success、host display form rejection、close failure state を検査する。
- source policy で trait-injected tests、cfg wrapper の薄さ、support gate `Ok` 非導入、runner / CLI dispatch 非導入、fd read / drain / protocol parsing 非導入、minifb fallback / synthetic readiness / timer evidence 非導入を固定する。

非目標:

- F5iz acquired fd を full runner-safe として文書化しない。selector registration 後に provider-owned fd が backend unregister より先に close されないことを保証する final owner bundle / drop-order contract は後続に分ける。
- `run_linux_platform_wait_window_loop`、Linux CLI dispatch、Linux support gate `Ok` 化、actual X11 / Wayland protocol handshake / event parsing、fd read / drain、minifb wait replacement、sleep、busy loop、fallback、silent no-op、synthetic readiness、timer evidence、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。

subagent review:

- Beauvoir the 2nd の初回 plan review は `PLAN_CHANGES`。fd owner が descriptor raw fd を公開した後の drop order、explicit close 後の stale descriptor、environment-dependent unit tests、display form contract が blocker として指摘された。
- revised plan では F5iz を acquisition/provider-only に絞り、final runner safety は後続 owner bundle/drop-order contract へ分離した。owned fd は non-Copy / non-Clone、private state、explicit close typed result、Drop cleanup-only とし、trait-injected API を primary tested boundary にする方針で `PLAN_APPROVED` を得た。

### Phase F5ja: Native Linux owned fd run-loop drop-order boundary

目的:

- F5iz の owned fd provider を runner-safe path へ渡す final owner bundle / drop-order contract を追加する。
- selector backend が registered window event source fd を unregister する前に provider-owned fd が close される path を閉じる。
- acquired fd runner-safe path は F5ja final wrapper helper only とし、generic F5iv provider host の裸 path は externally-managed provider 用に残す。

実装:

- `NativeWindowLinuxWindowEventSourceOwnedFdRunLoopHost Host BackendApi ProducerApi Api` を追加し、`NativeWindowLinuxWindowEventSourceRunLoopHost Host BackendApi ProducerApi (NativeWindowLinuxWindowEventSourceOwnedFdProvider Api)` を private field として保持する。
- final wrapper は public `into_parts`、provider accessor、provider mutable accessor、owned fd mutable accessor、owned fd close escape、host-only / provider-only split を出さない。公開は descriptor inspection と `NativeWindowRunLoopHost` delegation に限定する。
- `NativeWindowLinuxWindowEventSourceOwnedFdRunLoopHostBuildError` を追加する。provider-bearing variants は lower error / descriptor / config を provider より先に宣言し、error drop 時にも registered selector backend が provider-owned fd より先に drop される field order にする。
- helper は owned fd、selection、explicit demo / counter / scale / target FPS / run policy、host、backend API、producer API を受け、owned fd provider -> F5iu prepared platform config -> F5iv prepared run-loop config -> generic F5iv host build を通す。
- generic F5iv error はそのまま保持せず、`HostBuildFailed { provider, descriptor, error }` を即 destructure して F5ja の `HostBuildFailed { error, descriptor, provider }` へ詰め替える。

検証:

- Rust unit tests で final wrapper success drop 時に shared order log が `unregister-window-fd`、`close-owned-fd` の順になることを検査する。
- Rust unit tests で window fd registration 後の producer creation failure error を drop しても `unregister-window-fd`、`close-owned-fd` の順になることを検査する。
- source policy で F5ja final wrapper slice に public `into_parts`、`host_mut`、`provider` / `provider_mut`、`owned_fd_mut`、`into_owned_fd`、`.close()` escape がないことを固定する。
- source policy で F5ja helper が support gate `Ok` 化、runner / CLI dispatch、fd read / drain / event parsing、minifb fallback、synthetic readiness、silent no-op へ進んでいないことを固定する。

非目標:

- actual X11 / Wayland protocol handshake / event parsing、fd read / drain、`run_linux_platform_wait_window_loop`、Linux CLI dispatch、Linux support gate `Ok` 化、macOS actual sys shim、minifb wait replacement、sleep、busy loop、fallback、silent no-op、synthetic readiness、timer evidence、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は実装しない。

subagent review:

- Beauvoir the 2nd の初回 plan review は `PLAN_CHANGES`。success path だけでなく failure error drop order を field order で固定すること、generic F5iv host の `into_parts` bypass との関係を文書化すること、final wrapper の borrowed accessor を narrow にすること、success / post-registration failure の drop order test を入れることが要求された。
- revised plan では lower error を provider より先に置く F5ja 専用 error、generic F5iv error の即時詰め替え、final wrapper の provider mutable access 不提供、source policy の final wrapper slice scoping、success / failure shared order log tests を追加し、`PLAN_APPROVED` を得た。

## Checkpoint Commit Rule

各 phase は小さく commit する。

1. docs only
2. `core/gui` types + tests
3. `features/gui` + terminal capability bridge
4. `alloc/gui` app model + tests
5. `alloc/gui` layout/widget/accessibility slice + tests
6. TUI backend replacement checkpoint

各 commit 前に、その checkpoint に対応する focused test、`node nodesrc/issues.js check --dir issues`、`git diff --check` を通す。
