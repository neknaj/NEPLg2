# NEPLg2 native GUI platform behavior notes

作成日: 2026-06-02

## 目的

この文書は、macOS、Windows、Linux の native GUI が window lifecycle、resize、close request、event pump をどのように扱うかを整理し、NEPLg2 GUI backend の実装方針へ落とすための作業 notes である。

標準 API の public name に `NSWindow`、`HWND`、`xdg_toplevel`、`XWindow`、`minifb` を入れない。これらは `platforms/gui/native` または smoke backend の実装詳細として扱う。

## 調査要点

macOS AppKit:

- `NSApplication.run` が application event loop を開始する。CLI smoke runner でも、window を出した後は process が即時終了せず、backend の pump が継続して input と redraw を受ける必要がある。
- `NSWindowDelegate.windowShouldClose` は user が window close を試みたことを通知し、delegate が close を許可するかを `Bool` で返せる。ただし application quit では必ず呼ばれるとは限らない。
- resize、move、close は main thread 上の AppKit event と delegate notification として扱われるため、backend は「close request」と「実際に surface が消えた状態」を分けて model 化する。

Windows Win32:

- GUI thread には message queue があり、message loop が queue から message を取り出して window procedure へ dispatch する。
- mouse / keyboard input は基本的に FIFO で window に配送されるが、`WM_PAINT` などは統合される。
- `WM_CLOSE` は window / application が terminate すべきという request であり、application は確認 dialog などを挟んでから `DestroyWindow` できる。
- `WM_SIZE` は resize 後に送られ、minimized、maximized、restored の区別と client area size を含む。render surface はこの size 変更を受けて再確保または再配置する。
- resize 中に大量の size / pointer state が届く backend では、描画用 state を最新値へ寄せる。ただし close、keyboard、text input、button up、action event を上書きする coalescing はしない。

Linux Wayland:

- compositor が `xdg_toplevel.configure` で size / state を提案し、client は `ack_configure` 後に surface state を反映する。
- `xdg_toplevel.close` は user が close したいという request であり、client は ignore したり保存確認を出したりできる。
- compositor capability によって minimize などが無い場合があり、backend は unsupported operation を silent success とみなさず capability と `Result` で表す。
- Wayland は compositor 主導で window geometry が決まるため、application が要求した size を唯一の正としない。標準 API では backend から来た surface state を `WindowEventKind` と size に正規化する。

Linux X11:

- window manager は top-level window を reparent したり、client の希望と異なる size / position を割り当てたりできる。
- top-level window の deletion は `WM_DELETE_WINDOW` protocol の `ClientMessage` として受け、client は確認後に window を withdraw または destroy する。
- `ConfigureNotify` は real / synthetic event で coordinate の意味が異なる。backend は absolute window position に依存しすぎず、surface size を authoritative state として扱う。

minifb smoke backend:

- `Window::update_with_buffer` は 32-bit `0RGB` buffer を表示し、同時に window input / event pump のために毎 loop 呼ぶ必要がある。
- `Window::is_open` は user close button などで window が閉じられたかを application が確認するための状態である。
- `WindowOptions.resize` を有効にし、OS / window manager が与えた size に合わせて RGB0 buffer を再生成する。minifb 側は `ScaleMode::UpperLeft` に固定し、OS scaling ではなく application 側の redraw を authority とする。
- `Window::set_target_fps` で tight loop の CPU 消費を避ける。
- `get_unscaled_mouse_pos` と backend-local placement 計算を使い、letterbox 部分への pointer input を action hit test へ渡さない。
- minifb は native handle として Windows は `HWND`、macOS は `NSWindow`、X11 は `XWindow` を返せるが、この smoke backend では handle を標準 API へ公開しない。

## NEPLg2 backend contract

Native backend は次を守る。

- event pump は backend の責務であり、application logic に OS message loop を露出しない。
- close button はまず close request として扱う。現 `nepl-gui-native` smoke runner は unsaved state を持たないため、`Window::is_open` が false になったら process を正常終了する。
- resize は surface state change として扱う。zero size や unavailable surface は `Unavailable` として明示し、描画座標計算や hit test で除外する。
- min / max / restore は platform ごとに直接 event 名が異なるため、標準 API では `WindowEventKind`、`GuiCapabilities`、surface size に正規化する。
- high-frequency resize / pointer move は coalesce してよいが、close request、button up、keyboard、text input、action をまたいで古い event を上書きしてはいけない。
- backend-specific handle、DOM、Canvas、AppKit、Win32、Wayland、X11、minifb は `core/gui` / `alloc/gui` / `std/gui` の public type に入れない。

## Native window event pump boundary checkpoint

F5gd では `nepl-gui-native` の OS window observation を `NativeWindowEventPumpSnapshot` に集約する。snapshot は close state、observed window size、drawable surface state、size changed flag、left button transition、pointer sample を持つ。main loop は snapshot を `match` し、minifb の `Key`、`MouseButton`、`MouseMode`、`is_open`、`is_key_down`、`get_mouse_down`、`get_unscaled_mouse_pos` を直接読まない。

`NativeWindowSize` は observed size であり、zero dimension を許す。zero width / height は `NativeWindowPresenterSurfaceState::Unavailable` に写し、Drawable として扱わない。この状態では smoke runner は `window.update` で event pump を進めるだけで、blank frame や fallback frame を合成しない。positive drawable size へ戻った場合は `resize_surface` の後、same width and height の RGB0 buffer を再生成してから `update_with_buffer` する。

close state は `Open`、`OsCloseRequested`、`ExitShortcutRequested` を分ける。OS close button と Escape は現 smoke runner ではどちらも process を正常終了させるが、standard GUI contract では close request、keyboard shortcut、lifecycle event を後続で別々に扱うため、event pump 境界で潰さない。

pointer sample は `NativeWindowPointerSample::Unavailable` と `Available { x, y }` を分ける。pointer が取得できないことは通常の unavailable state である。非有限 coordinate は `NativeWindowEventPumpError::InvalidPointerSample` として返し、hit test から silently discard しない。

`poll_minifb_window_event_pump` は minifb adapter であり、`window.update` / `update_with_buffer` を呼ばない。presentation timing と buffer ownership は presenter state / backend loop の責務である。この checkpoint は event pump boundary のみであり、formal `std/gui` host import execution、scheduler loop、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。

## Native backend loop step checkpoint

F5ge では、F5gd の `NativeWindowEventPumpSnapshot` を受け取った後の native smoke backend state transition を `NativeWindowBackendLoop` に移す。loop は `GuiDemo`、counter value、current `GuiFrame`、presenter frame id、previous observed size、previous mouse state、`NativeWindowPresenterState` を所有するが、minifb window、OS handle、DOM、Canvas、video memory transport は所有しない。

`NativeWindowBackendLoop::new_for_scale` は initial frame render、scale validation、checked initial size、presenter state creation、initial present を一括して行う。`main.rs` は initial size を読んで minifb window を作るだけで、initial present buffer を直接作らない。`event_pump_input` は previous observed size と previous mouse state から F5gd input を返し、main 側に duplicate state を持たせない。

`step` は close、unavailable、drawable を enum outcome に分ける。close は no-progress であり、close request を受けても previous size / mouse、presenter frame、counter は進まない。unavailable は surface availability observation として presenter surface state、observed size、mouse state だけを更新し、last frame pixels / frame id / current frame / counter は維持する。blank frame や fallback frame は作らない。

positive resize は resize 先の RGB0 buffer を作り、present が成功した後だけ `resize_surface` と frame id / previous size を commit する。present failure では old surface state、old frame id、old frame pixels、previous size が残る。counter action は pointer unavailable、letterbox/outside、hit を enum で分け、hit の場合だけ counter overflow と frame id overflow を mutation 前に検査し、new frame present success の後だけ counter/current frame/frame id を進める。resize と counter hit が同じ snapshot に入った場合、resize redraw evidence と counter presentation evidence を両方 outcome に残す。

outcome は pixel borrow を持たない。minifb `update_with_buffer` に渡す final committed frame は `current_present_frame_for_window` からだけ借用する。この helper は current presenter surface と last frame size の一致を検査し、不一致は `FrameWindowMismatch` として返す。

## Native host action boundary checkpoint

F5gf では、F5ge の backend step outcome を native host execution action へ写す `NativeWindowHostAction` を追加する。`NativeWindowBackendLoop::step` は詳細な state transition evidence として残し、`step_host_action` が `CloseRequested`、`Unavailable`、`Drawable` をそれぞれ `Terminate`、`PumpEventsOnly`、`PresentFrame` へ変換する。

`Terminate` は `NativeWindowHostTerminalReason` を持ち、`OsCloseRequested` と `ExitShortcutRequested` を分ける。現 smoke runner ではどちらも process の正常終了になるが、future lifecycle / close request / shortcut policy では別の入力源として扱うため host action 境界で潰さない。

`PumpEventsOnly` は `NativeWindowSize` と `size_changed` を保持し、surface unavailable 中に `window.update` だけを実行するための action である。blank frame、zero fill frame、fallback frame は作らない。

`PresentFrame` は `NativeWindowBackendLoopPresentation`、`NativeWindowSize`、`size_changed` を保持するが、pixel borrow は持たない。minifb `update_with_buffer` 直前の frame borrow は `current_present_frame_for_window` に限定する。`main.rs` は `NativeWindowBackendLoopStepOutcome` を直接 match せず、`NativeWindowHostAction` を match して title update、`window.update`、`update_with_buffer`、loop termination だけを実行する。

`NativeWindowHostActionError` は contradictory close state を `UnsupportedCloseState` として扱い、backend step の overflow / rasterize / presenter failure は `StepFailed NativeWindowBackendLoopError` として original typed error を保持する。この checkpoint は host action selection boundary であり、formal scheduler loop、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization へは進まない。

## Native minifb window run-loop adapter checkpoint

F5gg では、F5gf の host action を実行する cfg-gated minifb window run-loop adapter として `run_minifb_window_loop` を追加する。F5ge / F5gf の説明で `main.rs` が minifb window creation、title update、`window.update`、`update_with_buffer` を持つと述べていた部分は、この checkpoint で supersede される。F5gg 以降、`main.rs` は CLI option を `NativeWindowRunLoopConfig` へ変換して runner を呼ぶだけで、minifb window lifecycle と host action execution は `run_minifb_window_loop` に閉じる。

`NativeWindowRunLoopConfig` は demo、counter value、scale を保持する。`NativeWindowRunLoopExit` は `NativeWindowHostTerminalReason` を保持し、OS close と Escape shortcut を正常終了の中でも区別する。`NativeWindowRunLoopError` は backend loop initialization、window creation、event pump、host action、presenter frame availability、`WindowPresentFailed` を enum で分ける。`WindowPresentFailed` は `update_with_buffer` failure だけを表し、error を返さない `window.update` の failure を捏造しない。minifb error text は platform detail として message に保持し、backend / event pump / host action の typed error は original error を保持する。

run-loop adapter は direct minifb input API を読まない。`Key`、`MouseButton`、`MouseMode`、`is_open`、`is_key_down`、`get_mouse_down`、`get_unscaled_mouse_pos` は引き続き `poll_minifb_window_event_pump` の内部に隔離される。run-loop adapter は `poll_minifb_window_event_pump` と `step_host_action` を呼び、`PumpEventsOnly` では `window.update` だけを行い、`PresentFrame` では `current_present_frame_for_window` から借用した exact-size RGB0 frame を `update_with_buffer` へ渡す。

`native_window_title` は title text construction を lib 側の deterministic helper とし、surface unavailable と drawable size を同じ規則で表す。`set_target_fps(60)` は smoke runner の busy spin 抑制であり、formal timer queue、OS wait strategy、scheduler policy ではない。この checkpoint は minifb adapter boundary であり、formal `std/gui` host import execution、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization、DOM / Canvas / video memory transport、fallback、silent no-op へは進まない。

## Native window host-loop core checkpoint

F5gh では、F5gg の minifb window run-loop から minifb 非依存の host-loop core を切り出す。`NativeWindowRunLoopHost` は host が実装する event snapshot polling、title update、pump-only update、present operation の境界であり、`run_native_window_host_loop` は `NativeWindowBackendLoop` と host を `&mut` で受けて 1 つの long loop を進める。

core loop は initial title を設定し、各 iteration で `poll_event_snapshot`、`step_host_action`、host action execution を行う。`Terminate` は `NativeWindowRunLoopExit` を返し、`PumpEventsOnly` は unavailable surface 中に host pump だけを呼び、`PresentFrame` は `current_present_frame_for_window` から exact-size frame を借用して host present へ渡す。title は initial と size changed 時だけ更新する。

`NativeWindowHostLoopError` は host event pump error、`NativeWindowHostActionError`、`NativeWindowBackendLoopError`、host present error を分ける。backend loop は `&mut` で渡されるため、error path でも caller は backend state を失わない。minifb smoke backend は private `MinifbNativeWindowRunLoopHost` で trait を実装し、direct minifb input API は引き続き `poll_minifb_window_event_pump` に閉じる。

この checkpoint は host-loop core boundary であり、formal scheduler queue、timer wait、OS wait strategy、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization、DOM / Canvas / video memory transport、fallback、silent no-op へは進まない。

## Native window host-loop turn checkpoint

F5gi では、F5gh の long loop body を `step_native_window_host_loop` へ分ける。`NativeWindowHostLoopTurn` は `Continue` と `Exit NativeWindowRunLoopExit` だけを返す typed turn result であり、queue、timer、wait state、present fallback を隠さない。

`step_native_window_host_loop` は initial title を設定しない。host event snapshot を 1 件読み、`NativeWindowBackendLoop::step_host_action` を 1 回だけ進め、`Terminate` は `Exit`、`PumpEventsOnly` は optional title update と host pump 後に `Continue`、`PresentFrame` は optional title update と exact-size frame present 後に `Continue` を返す。`run_native_window_host_loop` は initial title を 1 回だけ設定し、その後は one-turn function だけを loop で呼ぶ。

この checkpoint は future formal native OS scheduler / window backend loop が再利用する turn boundary であり、scheduler queue、timer wait、OS wait strategy、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization、fallback、silent no-op へは進まない。

## Native window host-loop bounded runner checkpoint

F5gj では、F5gi の one-turn function を bounded に反復する `run_native_window_host_loop_bounded` を追加する。`NativeWindowHostLoopRunnerState` は initial title 設定済みかどうかを保持し、複数 slice に分けて bounded run を呼んでも initial title を重複設定しない。

`initialize_native_window_host_loop` は `Initialized` と `AlreadyInitialized` を返す。二度目以降の初期化は unit を返す silent no-op ではなく、すでに初期化済みであることを typed evidence として返す。`run_native_window_host_loop_bounded` は `max_turn_count == 0` でも初期化だけは確認し、event poll を行わず `BudgetExhausted` を返す。

bounded runner は `step_native_window_host_loop` だけで turn を進める。`Continue` は completed turn count を増やし、`Exit` は exit turn を含めた count で `Exited` を返す。F5gj は future native scheduler の cooperative timeslice boundary であり、OS wait strategy、queue、timer wait、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization、fallback、silent no-op へは進まない。

## Native window frame pacing config checkpoint

F5gk では、native smoke window loop の frame pacing を固定値ではなく `NativeWindowTargetFps` と `NativeWindowRunLoopConfig.target_fps` で表す。target FPS は `1..=240` の typed config とし、`0` は `Zero`、上限超過は `TooHigh max` として返す。invalid value は clamp せず、`NativeWindowRunLoopError::TargetFpsInvalid value reason` または CLI parse error として表面化する。

minifb adapter は validation 済みの `target_fps.as_usize` だけを `Window::set_target_fps` に渡す。`set_target_fps 60` のような hidden constant や raw config value の直渡しは行わない。CLI の `--fps` は window mode 用の frame pacing 設定であり、headless mode の raster output には影響しない。

## Native window host-loop run policy checkpoint

F5gl では、native smoke window loop の long-running path を `NativeWindowHostLoopRunPolicy` へ接続する。`NativeWindowHostLoopTurnSlice` は `1..=4096` の bounded turn budget であり、default は `1` である。この値は OS wait / timer wait / FHD 60fps の保証ではなく、`run_native_window_host_loop_bounded` を何 turn ずつ反復するかを表す sanity bound である。

`run_native_window_host_loop_with_policy` は `NativeWindowHostLoopRunnerState` を保持したまま bounded runner を繰り返す。`BudgetExhausted` は同じ initialized state で次 slice へ進み、`Exited` は terminal reason を返す。`run_native_window_host_loop` は default policy を使う wrapper であり、minifb adapter は `NativeWindowRunLoopConfig.host_loop_policy` を渡す。`usize::MAX`、sleep、queue、timer、fallback、silent no-op は導入しない。

## Native window host-loop turn evidence checkpoint

F5gm では、`NativeWindowHostLoopTurn::Continue` が `NativeWindowHostLoopContinueEvidence` を保持する。`PumpedEventsOnly` は unavailable surface などで host pump だけを行った turn を表し、`PresentedFrame` は `NativeWindowBackendLoopPresentation` と observed window size を保持して successful present turn を表す。

`PresentedFrame` evidence は pixel borrow を持たない。host present が失敗した場合は evidence を返さず、`NativeWindowHostLoopError::HostPresentFailed` を返す。bounded runner と policy runner はまだ evidence を消費せず、turn count だけを進める。これは future wait decision 用の evidence boundary であり、OS wait strategy、queue / timer wait、sleep、fallback、silent no-op は実装しない。

## Native window host-loop wait decision checkpoint

F5gn では、`NativeWindowHostLoopContinueEvidence` を `NativeWindowHostLoopWaitDecision` へ分類する。`PumpedEventsOnly` は `WaitForHostEvent`、`PresentedFrame` は `WaitForFrameInterval` になる。これは future OS wait strategy の入力 class であり、実際の wait 実装ではない。

`run_native_window_host_loop_bounded` の `BudgetExhausted` は `completed_turns` と `last_wait_decision` を返す。turn budget が 0 の場合は `None`、継続 turn を処理した場合は最後の `Continue` evidence から得た decision を返す。F5gn の時点では decision の分類までを責務とし、実 wait dispatch は次 checkpoint の F5go で扱う。`WaitForFrameInterval` は frame-paced wait class evidence であり、timer registration、sleep、FHD 60fps guarantee ではない。

wait decision は pixel borrow、host handle、scheduler state を持たない。queue / timer wait、`Duration`、`std::thread::sleep`、fallback、silent no-op、DOM / Canvas / video memory transport はこの checkpoint の責務ではない。

## Native window host-loop wait dispatch checkpoint

F5go では、F5gn の `NativeWindowHostLoopWaitDecision` を policy runner が `NativeWindowRunLoopHost::wait_after_budget_exhausted` へ渡す。host trait は `WaitError` associated type を持ち、wait hook の失敗は `NativeWindowHostLoopError::HostWaitFailed` として event pump / present / host action failure と分離する。

wait hook は `NativeWindowHostLoopWaitOutcome` を返す。minifb smoke backend では `Window::set_target_fps` による `Window::update` / `update_with_buffer` 内部の pacing が wait authority であるため、F5go の hook は `HostEventPumpAlreadyPaced` または `FramePresentAlreadyPaced` の typed outcome を返すだけで追加の `window.update`、`update_with_buffer`、`std::thread::sleep`、`Duration`、queue、timer を実行しない。F5gx の `FrameIntervalTimerRegistered` は external scheduler 向けの timer registration evidence であり、minifb already-paced outcome とは分離する。これにより event pump の重複、二重 pacing、timer registration 成功の wait completion 偽装を避ける。

`run_native_window_host_loop_with_policy` は `BudgetExhausted last_wait_decision = Some decision` の場合だけ host wait hook を呼び、`None` は `WaitDecisionMissing` として fail closed にする。zero turn slice は validation により作れないが、bounded runner API 自体は zero budget を許すため、long-running policy runner 側で missing decision を silent no-op にしない。

## Native window host-loop scheduler slice checkpoint

F5gp では、F5go まで `run_native_window_host_loop_with_policy` の内部に閉じていた `bounded run -> wait dispatch -> repeat` の 1 cycle を、external scheduler が呼べる typed slice として切り出す。

`NativeWindowHostLoopSchedulerState` は `NativeWindowHostLoopRunnerState` を所有し、initial title が slice を跨いで二重設定されないようにする。`run_native_window_host_loop_scheduler_slice_with_policy` は policy の turn slice から bounded runner を 1 回だけ実行し、結果を `NativeWindowHostLoopSchedulerSliceResult` に写す。`Exited` は terminal exit と completed turn count を返し、`Waited` は completed turn count、wait decision、host wait outcome を返す。

F5gp の目的は hidden long loop authority を外へ出し、formal native OS scheduler / window backend loop が同じ slice contract を消費できるようにすることである。wait outcome を unit success に潰さず result enum に残すため、test / future scheduler は host-event wait と frame-interval wait のどちらが発生したかを検査できる。

F5gp は scheduler slice boundary までであり、actual OS wait strategy、queue / timer wait backend、real timer registration、`Duration`、`std::thread::sleep`、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization は実装しない。fallback、silent no-op、extra minifb update、DOM / Canvas / video memory transport も導入しない。

## Native window host-loop wait request plan checkpoint

F5gq では、F5gp の scheduler slice が持つ wait decision を、native backend が消費できる wait request plan へ変換する。

`NativeWindowHostLoopWaitRequest` は host event wait と frame interval wait を分ける。host event wait は event payload や queue owner を持たない。frame interval wait は `NativeWindowFrameIntervalRequest` を持ち、validated `NativeWindowTargetFps`、`nanos_per_frame`、`remainder_nanos_per_second` を保持する。`60fps` なら `16_666_666ns` と remainder `40ns/second`、`120fps` なら `8_333_333ns` と remainder `40ns/second` として表し、暗黙の clamp、sentinel、zero-fill fallback は使わない。

F5gq は decision から request plan を作る境界であり、F5gr 以降の `NativeWindowRunLoopHost::wait_after_budget_exhausted` は request から生成される instruction を受け取る。minifb backend では引き続き追加の `window.update`、`update_with_buffer`、`std::thread::sleep`、`Duration` を呼ばない。minifb の frame pacing authority は `Window::set_target_fps` と update path に残し、F5gq はその前段の request plan boundary に留める。

F5gq は actual OS wait strategy、queue / timer wait backend、real timer registration、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、DOM / Canvas / video memory transport も導入しない。

## Native window host-loop wait strategy instruction checkpoint

F5gr では、F5gq の wait request plan を host wait hook が消費する wait strategy instruction へ変換する。

`NativeWindowHostLoopWaitStrategyState` は scheduler slice 間で frame pacing target FPS と remainder accumulator を保持する。target FPS が同じ場合だけ previous remainder を使い、target FPS が変わった場合は accumulator を `0` に戻す。accumulator は常に `0 <= remainder < fps` の範囲に保ち、saturating、clamp、sentinel、zero-fill fallback は使わない。

`NativeWindowHostLoopWaitInstruction` は host event wait と frame interval wait を分ける。host event instruction は event payload、queue owner、poll result を持たない。frame interval instruction は `NativeWindowFrameIntervalRequest` と `wait_nanos` を持ち、`wait_nanos` は `nanos_per_frame` または `nanos_per_frame + 1` だけである。

`NativeWindowRunLoopHost::wait_after_budget_exhausted` は request ではなく instruction を受け取る。scheduler slice は wait hook 成功後だけ `NativeWindowHostLoopSchedulerState` の strategy state を進め、failure path では remainder を消費しない。minifb backend は instruction を match して existing pacing outcome を返すだけで、追加の `window.update`、`update_with_buffer`、`std::thread::sleep`、`Duration` 変換は行わない。

F5gr は actual OS wait strategy、queue / timer wait backend、real timer registration、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、DOM / Canvas / video memory transport も導入しない。

## Native window host-loop thread wait backend checkpoint

F5gs では、F5gr の wait instruction を native thread sleep backend へ渡す実行境界を追加する。ただし minifb smoke backend の wait hook へは接続しない。

minifb 0.28 の `Window::set_target_fps` は `update` / `update_with_buffer` の末尾で update rate を調整し、必要なら sleep する。このため minifb smoke backend の wait hook がさらに thread sleep を呼ぶと二重 pacing になる。F5gs の thread wait backend は formal native scheduler が frame interval instruction を実行するための別境界として扱う。

`NativeWindowHostLoopThreadSleeper` は injected sleeper interface である。test は scripted sleeper を使い、std native helper は `StdNativeWindowHostLoopThreadSleeper` として `std::thread::sleep(std::time::Duration::from_nanos(u64::from(wait_nanos)))` を呼ぶ。`Duration` と `std::thread::sleep` はこの helper に閉じ、scheduler slice、instruction planner、minifb wait hook へ漏らさない。

`execute_native_window_host_loop_thread_wait_with_sleeper` は frame interval instruction だけを実行し、`wait_nanos` が `nanos_per_frame` または `nanos_per_frame + 1` であることを再検査する。不一致は `FrameIntervalWaitNanosMismatch` として返し、sleeper を呼ばない。sleeper failure は `SleeperFailed` に保持する。

host event wait は `HostEventWaitUnsupported` を返す。OS event queue / selector / message pump backend が未実装の段階で host event wait を busy loop、thread sleep、silent no-op に変換しない。

F5gs は native thread wait backend boundary までであり、host event queue、timer registration、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、DOM / Canvas / video memory transport も導入しない。

## Native window host-loop timer registration backend checkpoint

F5gt では、F5gr の wait instruction を native timer registration backend へ渡す実行境界を追加する。ただし minifb smoke backend の wait hook へは接続しない。

`NativeWindowHostLoopTimerRegistrar` は injected registrar interface である。test は scripted registrar を使う。registrar は host boundary として raw `u32` timer id を返し、`execute_native_window_host_loop_timer_registration_with_registrar` が positive id を `NativeWindowHostLoopTimerRegistrationId` へ変換する。raw id `0` は `InvalidTimerRegistrationId` として拒否する。

`execute_native_window_host_loop_timer_registration_with_registrar` は frame interval instruction だけを実行し、`wait_nanos` が `nanos_per_frame` または `nanos_per_frame + 1` であることを再検査する。不一致は `FrameIntervalWaitNanosMismatch` として返し、registrar を呼ばない。registrar failure は `RegistrarFailed` に保持する。

host event wait は `HostEventTimerRegistrationUnsupported` を返す。OS event queue / selector / message pump backend が未実装の段階で host event wait を timer registration、thread sleep、busy loop、silent no-op に変換しない。

F5gt は native timer registration backend boundary までであり、host event queue、real OS timer backend、minifb wait hook への接続、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、DOM / Canvas / video memory transport も導入しない。

## Native window host-loop event queue wait backend checkpoint

F5gu では、F5gr の wait instruction のうち host event wait を event queue wait backend へ渡す実行境界を追加する。ただし real OS event queue / selector / message pump adapter へは接続しない。

`NativeWindowHostLoopEventQueueWaiter` は injected waiter interface である。test は scripted waiter を使う。`execute_native_window_host_loop_event_queue_wait_with_waiter` は `WaitForHostEvent` の場合だけ waiter を 1 回呼び、成功時に `NativeWindowHostLoopEventQueueWaitOutcome::HostEventReady` を返す。waiter failure は `WaiterFailed` に保持する。

frame interval wait は `FrameIntervalEventQueueWaitUnsupported` を返す。event queue wait backend が frame interval wait を timer registration、thread sleep、busy loop、silent no-op に変換することは禁止する。

F5gu は event queue wait backend boundary までであり、real OS event queue / selector / message pump adapter、real OS timer backend、minifb wait hook への接続、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、DOM / Canvas / video memory transport も導入しない。

## Native window host-loop event queue normalized status adapter checkpoint

F5gv では、F5gu の event queue wait backend に接続できる normalized status adapter 境界を追加する。ただし real OS event queue / selector / message pump adapter へはまだ接続しない。

`NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY` は OS API 固有の値ではなく、platform adapter が `nepl-gui-native` の境界へ返す internal normalized status である。`NativeWindowHostLoopEventQueueStatusAdapter` は `wait_for_host_event_raw_status` で normalized raw status を返す。

`wait_native_window_host_loop_event_queue_raw_status_with_adapter` は adapter を 1 回だけ呼び、ready status 以外を `InvalidRawStatus` として fail closed にする。adapter の失敗は `AdapterFailed` として元の error value を保持する。`NativeWindowHostLoopEventQueueStatusWaiter` はこの adapter を F5gu の `NativeWindowHostLoopEventQueueWaiter` へ接続する。

frame interval wait は F5gu executor が `FrameIntervalEventQueueWaitUnsupported` として止めるため、status adapter を呼ばない。event queue status adapter が frame interval wait を timer registration、thread sleep、busy loop、silent no-op に変換することは禁止する。

F5gv は normalized status adapter boundary までであり、real OS event queue / selector / message pump adapter、real OS timer backend、minifb wait hook への接続、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。fallback、silent no-op、DOM / Canvas / video memory transport も導入しない。

## Native window host-loop message pump adapter checkpoint

F5gw では、F5gv の normalized status adapter に接続する message pump adapter 境界を追加する。これは platform window backend が持つ message pump を実行し、成功時だけ `NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY` へ正規化する境界である。

`NativeWindowHostLoopMessagePumpAdapter` は `pump_host_messages` を 1 回だけ呼ぶ。`NativeWindowHostLoopMessagePumpStatusAdapter` は pump 成功を F5gv の ready status へ写し、pump failure は `PumpFailed` として保持する。

minifb smoke backend では `MinifbNativeWindowHostLoopMessagePumpAdapter` が `window.update` を実行する。`MinifbNativeWindowRunLoopHost::wait_after_budget_exhausted` は host event wait を direct update ではなく `wait_minifb_window_host_event_message_pump` へ渡し、F5gu / F5gv の event queue waiter 境界を通して `HostEventPumpAlreadyPaced` へ戻す。

F5gw は message pump adapter boundary までであり、real OS timer backend、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。message pump adapter が frame interval wait を timer registration、thread sleep、busy loop、silent no-op、fallback へ変換することは禁止する。

## Native window host-loop frame interval timer registration outcome checkpoint

F5gx では、F5gt の timer registration backend を `NativeWindowHostLoopWaitOutcome` へ接続する。ただし timer registration 成功は frame wait completion ではないため、`FramePresentAlreadyPaced` とは別の `FrameIntervalTimerRegistered` outcome として扱う。

`NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered` は `presentation`、`window_size`、`size_changed`、`wait_nanos`、`timer_registration_id` を持つ。`execute_native_window_host_loop_timer_registration_wait_with_registrar` は F5gt executor を呼び、successful timer registration だけを wait outcome evidence へ写す。

host event wait は F5gt の `HostEventTimerRegistrationUnsupported` のまま fail closed にする。timer registration backend は host event wait を event queue wait、message pump、thread sleep、busy loop、silent no-op へ変換しない。

minifb smoke backend の wait hook は F5gx helper へ接続しない。minifb は引き続き `Window::set_target_fps` / `update_with_buffer` の pacing authority を使い、frame interval wait では `FramePresentAlreadyPaced` を返す。F5gx は timer registration outcome boundary までであり、actual timer fire / wakeup、real OS timer backend connection、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。

## Native window host-loop timer fire/wakeup backend checkpoint

F5gy では、F5gx の `FrameIntervalTimerRegistered` outcome を、backend が観測した timer fire / wakeup evidence へ接続する。登録成功と fire 成功は別の event であるため、registered id と fired id が一致した場合だけ `FrameIntervalTimerFired` を返す。

`NativeWindowHostLoopTimerFireWaiter` は registered timer id を受け取り、backend-observed fired raw id を返す。`execute_native_window_host_loop_timer_fire_wait_with_waiter` は `FrameIntervalTimerRegistered` の場合だけ waiter を呼ぶ。fired raw id `0` は invalid、registered raw id と異なる fired raw id は mismatch として fail closed にする。

`HostEventPumpAlreadyPaced` と `FramePresentAlreadyPaced` は timer fire input ではないため unsupported として拒否し、waiter を呼ばない。minifb smoke backend の already-paced frame outcome を timer fire success として扱わない。

F5gy は timer fire / wakeup backend boundary までであり、OS 固有 timer API、selector wakeup ownership、minifb wait hook 接続、scheduler resume policy、FHD 60fps measurement harness、2D compositor drain、font / stroke / shadow rasterization へは進まない。

F5gz では、timer registration wait と timer fire wait を 1 つの wakeup executor として合成する。`NativeWindowHostLoopTimerWakeError` は registration 段階の failure を `RegistrationFailed`、fire 段階の failure を `FireFailed` として保持する。

`execute_native_window_host_loop_timer_wakeup_with_backend` は、先に `execute_native_window_host_loop_timer_registration_wait_with_registrar` を呼び、registration failure では waiter を呼ばない。registration が successful `FrameIntervalTimerRegistered` outcome を返した場合だけ `execute_native_window_host_loop_timer_fire_wait_with_waiter` を呼び、registered id と fired id の完全一致だけを `FrameIntervalTimerFired` として返す。

F5gz は scheduler resume policy の入力になる typed wakeup boundary であり、OS 固有 timer API、selector ownership、minifb wait hook 接続、thread sleep、busy loop、message pump、event queue substitution は実装しない。

F5ha では、scheduler long runner が `NativeWindowHostLoopSchedulerSliceResult::Waited` を無条件に次 loop へ進めることを禁止する。`HostEventPumpAlreadyPaced` と `FramePresentAlreadyPaced` は already-paced ready evidence として再開できるが、`FrameIntervalTimerRegistered` は timer fire 待ちの pending state である。

`NativeWindowHostLoopSchedulerResumeState` は `Ready` と `WaitingForFrameIntervalTimer` を持つ。`run_native_window_host_loop_with_policy_and_target_fps` は `Waited` outcome を resume gate に通し、`WaitingForFrameIntervalTimer` の場合は `NativeWindowHostLoopError::TimerFireResumeRequired` を返して、次の event poll や present へ進まない。

F5ha は resume gate までであり、real OS timer adapter、selector ownership、minifb timer path、thread sleep、busy loop、message pump、event queue substitution は実装しない。

F5hb では、native host-loop 用の std deadline timer adapter を追加する。`NativeWindowHostLoopDeadlineTimerAdapter` は injected clock / sleeper と active timer state を所有し、F5gt の timer registration と F5gy の timer fire wait を同じ adapter state で実行できる。

adapter は active timer overlap、missing active timer、timer id overflow、deadline overflow、clock failure、sleeper failure、mismatched fire id を enum error として保持する。registration 成功時だけ active timer を作り、fire wait 成功時だけ active timer を消費する。sleeper failure では active timer を消費しない。

F5hb は std deadline timer adapter boundary までであり、macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd、minifb wait hook 接続、`Window::set_target_fps` 置換は扱わない。minifb smoke backend は既存 pacing authority を維持し、frame interval wait では引き続き `FramePresentAlreadyPaced` を返す。

F5hc では、F5gz/F5hb で得た `FrameIntervalTimerFired` evidence を `NativeWindowHostLoopWaitOutcome` として運べるようにする。`FrameIntervalTimerRegistered` は引き続き pending state であり、scheduler resume gate では `WaitingForFrameIntervalTimer` として扱う。`FrameIntervalTimerFired` だけが `Ready(FrameIntervalTimerFired)` へ進める。

`execute_native_window_host_loop_timer_wakeup_wait_with_backend` と `execute_native_window_host_loop_deadline_timer_wakeup_wait_with_adapter` は、registration error と fire error の段階を保ったまま、成功時だけ fired timer evidence を wait outcome へ写す。これは selector / message loop timer ownership を実装する前の typed boundary であり、minifb wait hook、`Window::set_target_fps`、OS 固有 timer API へは接続しない。

F5hd では、host-loop wait instruction を host event queue wait backend と frame interval deadline timer backend のどちらへ渡すかを所有する `NativeWindowHostLoopWaitOwner` を追加する。owner は event queue waiter と deadline timer adapter を保持するが、backend 同士を直接依存させない。

`execute_native_window_host_loop_wait_with_owner` は `WaitForHostEvent` を event queue waiter だけへ渡し、成功を `HostEventPumpAlreadyPaced` として返す。`WaitForFrameInterval` は deadline timer wakeup wait helper だけへ渡し、成功を `FrameIntervalTimerFired` として返す。error は `EventQueueWaitFailed` と `FrameIntervalTimerWakeFailed` に分け、lower error 全体を保持する。

F5hd は wait owner composition boundary までであり、selector ownership、OS message loop timer、minifb wait hook の pacing 置換、`Window::set_target_fps` 置換は扱わない。host event path が timer clock / sleeper を呼ぶこと、frame interval path が event queue waiter を呼ぶこと、fallback / silent no-op / busy loop は禁止する。

F5he では、minifb smoke backend の frame pacing authority を `NativeWindowMinifbFramePacingAuthority` として明示する。minifb は引き続き `Window::set_target_fps` を使うが、`WaitForFrameInterval` が `FramePresentAlreadyPaced` を返す path は authority helper を通る。authority は validated `NativeWindowTargetFps` を保持し、instruction 側の target fps と `wait_nanos` が一致する場合だけ already-paced evidence を返す。不一致は enum error として fail closed にする。

F5he は minifb internal target-fps pacing authority boundary であり、F5hd wait owner や std deadline timer adapter を minifb wait hook へ接続しない。`set_target_fps 0` は minifb internal wait を無効化して host event wait path を tight loop にするため禁止する。future selector / message-loop timer backend が frame interval authority になる場合は、minifb internal pacing と同時に使わない。

F5hf では、frame interval wait authority を `NativeWindowFrameIntervalWaitAuthorityMode` に分ける。`MinifbInternalTargetFps` は minifb internal pacing、`HostOwnedDeadlineTimer` は future selector / message-loop timer path の deadline timer owner を表す。これは selector / message-loop timer ownership の実装ではなく、二重 authority を拒否する safety boundary である。

`combine_native_window_frame_interval_wait_authority_mode` は同じ minifb target fps 同士と host-owned deadline timer 同士だけを受け入れる。minifb と host-owned の混在、または target fps が異なる minifb mode 同士は `ConflictingFrameIntervalAuthorities` として fail closed にする。`validate_native_window_frame_interval_wait_authority_mode` は minifb mode の場合だけ instruction の target fps を検査し、host-owned mode では wait evidence を作らない。`NativeWindowMinifbFramePacingAuthority` は `FramePresentAlreadyPaced` を返す前にこの validation helper を通る。

F5hg では、F5hf の authority mode を `NativeWindowHostLoopWaitOwner` の frame interval branch に接続する。owner は `frame_interval_wait_authority_mode` として `HostOwnedDeadlineTimer` を返し、`execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode` は explicit requested authority mode を受け取る。

frame interval branch は `combine_native_window_frame_interval_wait_authority_mode` と `validate_native_window_frame_interval_wait_authority_mode` を通ってから deadline timer wakeup helper へ進む。minifb authority が混入した場合は `FrameIntervalAuthorityFailed` として返し、timer registration、clock read、sleeper call、active timer mutation は起こさない。host event wait は authority を参照しない。F5hg は real selector / message-loop timer backend ではなく、macOS run loop timer、Windows waitable timer / message wait、Linux selector / timerfd は後続で扱う。

## Current implementation

`nepl-gui-native` は正式な `std/gui::GuiHost` ではなく、native smoke backend である。

現在の checkpoint では次を実装している。

- `WindowOptions.resize = true` により OS window manager の resize を許可する。
- `ScaleMode::UpperLeft` と dark background を使い、resize 後は current drawable surface と同じ size の RGB0 buffer を presenter state へ再 present する。
- `NativeWindowTargetFps` で検査した target FPS を `Window::set_target_fps` に渡し、event pump loop の busy spin を避ける。
- `NativeWindowHostLoopRunPolicy` で検査した turn slice により、minifb host loop は bounded runner を明示的に反復する。
- `NativeWindowHostLoopContinueEvidence` により、pump-only turn と present-frame turn を区別する。
- `NativeWindowHostLoopWaitDecision` により、bounded runner の最後の continue turn を host-event wait class または frame-interval wait class として分類する。
- `NativeWindowHostLoopWaitRequest` により、wait decision と target FPS から backend wait request plan を作る。
- `NativeWindowHostLoopWaitInstruction` と `NativeWindowHostLoopWaitStrategyState` により、frame interval remainder を scheduler slice 間で配分し、host wait boundary へ渡す typed instruction を作る。
- `NativeWindowRunLoopHost::wait_after_budget_exhausted` により、policy runner が wait instruction を host wait boundary へ渡す。
- `NativeWindowHostLoopThreadSleeper` と `execute_native_window_host_loop_thread_wait_with_sleeper` により、formal native thread wait backend は frame interval instruction を sleep 実行境界へ渡せる。host event wait は queue 未実装として fail closed にする。
- `NativeWindowHostLoopTimerRegistrar` と `execute_native_window_host_loop_timer_registration_with_registrar` により、formal native timer registration backend は frame interval instruction を raw timer id registration 境界へ渡せる。host event wait は queue 未実装として fail closed にする。
- `NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered` と `execute_native_window_host_loop_timer_registration_wait_with_registrar` により、timer registration 成功を already-paced outcome ではなく timer registration evidence として host wait outcome へ渡せる。
- `NativeWindowHostLoopTimerFireWaiter` と `execute_native_window_host_loop_timer_fire_wait_with_waiter` により、registered timer id と backend-observed fired timer id を照合し、完全一致だけを `FrameIntervalTimerFired` evidence へ進められる。
- `NativeWindowHostLoopTimerWakeError` と `execute_native_window_host_loop_timer_wakeup_with_backend` により、timer registration と fire wait を順に実行し、どちらの段階で失敗したかを enum で保持できる。
- `NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired` と `native_window_host_loop_wait_outcome_from_timer_fire` により、timer fire completion を host wait outcome として scheduler へ渡せる。
- `NativeWindowHostLoopSchedulerResumeState` と `NativeWindowHostLoopError::TimerFireResumeRequired` により、timer registration outcome を scheduler resume completion として扱わず、timer fired outcome だけを ready として扱う。
- `NativeWindowHostLoopDeadlineTimerAdapter` により、std deadline timer の active state、id allocation、deadline arithmetic、clock / sleeper failure を typed error として扱える。ただし minifb wait hook へは接続していない。
- `NativeWindowHostLoopEventQueueWaiter` と `execute_native_window_host_loop_event_queue_wait_with_waiter` により、formal native event queue wait backend は host event instruction を queue wait 境界へ渡せる。frame interval wait は timer / sleep backend の責務として fail closed にする。
- `NativeWindowHostLoopEventQueueStatusAdapter` と `NativeWindowHostLoopEventQueueStatusWaiter` により、platform adapter が返す normalized raw status を検証して F5gu waiter 境界へ接続できる。
- `NativeWindowHostLoopMessagePumpAdapter` と `NativeWindowHostLoopMessagePumpStatusAdapter` により、platform message pump の実行成功を F5gv の normalized ready status へ写せる。
- `NativeWindowHostLoopWaitOwner` と `execute_native_window_host_loop_wait_with_owner` により、host event wait と frame interval timer wait を同じ owner で分岐できる。ただし minifb wait hook へはまだ接続していない。
- `NativeWindowHostLoopSchedulerState` と `run_native_window_host_loop_scheduler_slice_with_policy` により、bounded run と wait dispatch の 1 cycle を external scheduler が呼べる typed slice として公開する。
- `NativeWindowMinifbFramePacingAuthority` により、minifb smoke backend の frame interval wait は validated target FPS と wait nanos を検査してから `FramePresentAlreadyPaced` を返す。これは minifb internal `Window::set_target_fps` pacing が有効であることの evidence であり、wait hook が sleep や deadline timer wait を実行したという意味ではない。
- `NativeWindowFrameIntervalWaitAuthorityMode` と `validate_native_window_frame_interval_wait_authority_mode` により、minifb internal target-fps pacing と future host-owned deadline timer authority を同時に frame interval authority として扱わない。host-owned mode validation は compatibility check だけで、wait evidence は生成しない。
- `execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode` により、formal wait owner の frame interval branch は `HostOwnedDeadlineTimer` authority を deadline timer wakeup より前に検査する。minifb authority が渡された場合は timer state を変更せず `FrameIntervalAuthorityFailed` を返す。
- minifb smoke backend の host event wait hook は `MinifbNativeWindowHostLoopMessagePumpAdapter` の `window.update` だけを message pump adapter として呼ぶ。frame interval wait は additional sleep や timer registration を行わず、F5hd wait owner / F5gx helper / std deadline timer adapter へは接続していない。
- `poll_minifb_window_event_pump` が `Window::get_size`、close state、left button transition、pointer sample を snapshot に正規化する。
- `NativeWindowBackendLoop` が snapshot 後の state transition、resize redraw、counter hit test、frame id update、presenter surface commit を所有する。
- `step_host_action` が backend loop outcome を `NativeWindowHostAction` へ写し、host-side execution decision を typed enum にする。
- `step_native_window_host_loop` が host event snapshot 1 件、host action 1 件、pump / present / exit の 1 turn だけを実行する。
- `run_native_window_host_loop_bounded` が bounded turn count で `Exited` と `BudgetExhausted` を分け、budget exhaustion 時は最後の wait decision を保持する。
- `run_native_window_host_loop` が scheduler slice API を long loop として反復する。
- `MinifbNativeWindowRunLoopHost` が minifb window lifecycle、window title update、`window.update`、`update_with_buffer` を所有し、main.rs は runner 呼び出しだけを行う。
- `native_window_title` が drawable size と unavailable surface の title text を deterministic に構築する。
- counter hit test は backend loop 内で current window size、framebuffer size、letterbox offset を使って scene coordinate へ変換する。
- zero size または invalid size は `NativeSurfaceState::Unavailable` として扱い、hit test を行わない。
- close button または Escape により loop を抜け、terminal side の process が正常終了する。

## Native monotonic clock source checkpoint

F5er では Native formal monotonic clock source backend boundary として、`platforms/gui/native/clock` と `nepl-gui-native` の `Instant` helper を追加する。`nepl_gui_native.monotonic_clock_ms` は単一 `i32` return ABI であり、0 以上を monotonic millisecond sample、-1 を unsupported、その他の負値を `BackendFailure` として扱う。

Rust side は `Instant::elapsed().as_millis()` を `i32::MAX` 以下で検査し、範囲外を wrap、clamp、saturating cast で処理しない。範囲外は backend failure sentinel として返す。NEPL wrapper は negative sentinel を `GuiError` へ写した後だけ F5eo backend clock sample constructor へ渡す。

この checkpoint は clock source だけを扱う。window loop、present、scheduler backend、timer、queue、minifb rendering、stdout protocol、fallback、silent no-op は clock source として使わない。native formal scheduler backend、native formal present implementation、long-running backend loop は後続 slice で実装する。

## Native span operation host executor ABI checkpoint

F5ey では Native scheduler host executor import の Rust 側境界として、`nepl-gui-native` に span operation ABI validator と injected sink を追加する。この境界は `platforms/gui/native/scheduler_host_executor` が呼ぶ begin / run / end host import と同じ scalar payload を受け取るが、window loop、minifb rendering、video memory、DOM、Canvas、queue、timer、fallback、silent no-op は実装しない。

Rust side は status sentinel を Web video memory host ABI と揃え、0 を success、-1 を unsupported、-2 を invalid argument、-3 を resource exhausted、-4 を no writable slot、-5 を backend failure、-6 を stale frame として扱う。sink が未知の正値または未知の負値を返した場合は backend failure へ正規化する。

descriptor payload は sink 実行前に検査する。window target は positive window id を必須にし、offscreen / device target は window id 0 だけを受ける。surface id、frame id、packet frame id、width、height、row count、stride、tile rows、tile count、pixel count、run count、encoded byte count は positive でなければならない。`packet_frame_id == frame_id`、`stride_bytes == width * 4`、`pixel_count == width * row_count`、`encoded_byte_count == total_run_count * 12`、`tile_count == ceil(plan_row_count / tile_rows)`、`tile_index < tile_count` をすべて満たす必要がある。extent は checked arithmetic で計算し、row extent は plan row extent の内側かつ surface height 以下に収まる必要がある。

run span payload は current row 用の span なので height は 1 だけを受ける。x / y は non-negative、width は positive、RGBA channel は 0 から 255 の範囲でなければならない。invalid scalar input returns -2 before the sink is called ため、検証に失敗した operation は renderer や presenter へ渡らない。

## Native RGBA8888 framebuffer sink checkpoint

F5ez では F5ey の typed `NativeSpanOperation` を受ける offscreen framebuffer sink を `nepl-gui-native` に追加する。この sink は actual native presenter の前段であり、window loop、minifb rendering、video memory、DOM、Canvas、queue、timer、fallback、silent no-op には接続しない。

framebuffer の pixel は semantic `0xRRGGBBAA` の `u32` として保持する。これは native endian の byte view ではないため、将来の presenter は host surface が要求する `0RGB` や byte sequence へ明示変換する。`from_raw_parts` や transmute による byte view はこの checkpoint の contract に含めない。

`NativeRgba8888FrameBuffer` は checked constructor だけで作る。width と height は positive、`stride_bytes == width * 4`、`pixels.len == width * height` を checked arithmetic で満たす必要がある。内部 field は private であり、外部から壊れた stride や pixel length を注入できない。

operation sequence は Begin / RunSpan... / End の complete sequence として扱う。Begin は active sequence が無い場合だけ受け、`seen_run_count = 0` を保持する。RunSpan は active descriptor と target、`x >= 0`、`width > 0`、`height == 1`、row extent、x extent、remaining run count をすべて検査し、全検査が終わった後だけ pixel を書き、成功後だけ `seen_run_count` を増やす。End は descriptor equality と `seen_run_count == descriptor.total_run_count` を満たす場合だけ active sequence を閉じる。短い span 列、余分な span、mismatched end は `InvalidArgument` であり、silent partial frame として成功させない。Run failure と End failure は active sequence、seen count、pixels を壊さない。

## Native RGB0 present buffer conversion checkpoint

F5fa では completed `NativeRgba8888FrameBuffer` を native presenter 用の semantic `0x00RRGGBB` buffer へ変換する境界を追加する。この checkpoint は pixel contract conversion だけであり、window loop、minifb `update_with_buffer`、scheduler loop、timer、queue、video memory、DOM、Canvas、fallback、silent no-op へ進まない。

変換は `0xRRGGBBAA` から channel を shift / mask で取り出し、explicit background color に source-over alpha composition する。計算は channel ごとに `(source * alpha + background * (255 - alpha) + 127) / 255` を使う。alpha 255 は source RGB、alpha 0 は background RGB になる。background は caller が明示的に渡すため、hidden default background や fallback background は持たない。

`NativeRgb0PresentBuffer` は checked conversion だけで作る。source framebuffer に active sequence が残っている場合は present buffer へ変換せず、silent partial frame を拒否する。output pixel は semantic `0x00RRGGBB` の `u32` であり、native endian byte view は公開しない。将来の native presenter はこの buffer を host surface contract に合わせて明示的に渡す。

## Native presenter frame adapter checkpoint

F5fb では `NativeRgb0PresentBuffer` を native window presenter が受け取る immutable frame に借用変換する境界を追加する。この checkpoint は presenter-side contract adapter であり、scheduler loop、queue、timer、bare runtime host import、formal `std/gui` present host import、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization へ進まない。

adapter は width / height を `usize` へ checked conversion し、`width * height == pixels.len` を checked arithmetic で検査する。typed presenter frame は immutable `&[u32]` と checked `usize` dimensions だけを公開し、minifb 型や OS handle を public type に含めない。minifb の `update_with_buffer` 呼び出しは smoke runner の `main.rs` だけが行う。

smoke demo は正式 NEPL span path が native window loop へ接続されるまでの互換 source として、既存 demo rasterizer の `0x00RRGGBB` pixels を `NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo` へ渡す。この import は every pixel の high byte が 0 であることを検査し、`0xAARRGGBB` や native-endian byte sequence を silent masking しない。invalid pixel は `Result` error であり、fallback background や best-effort normalization は行わない。

## Native RGB0 presenter sink checkpoint

F5fc では formal span operation path から completed `NativeRgb0PresentBuffer` と typed presenter frame を得る native sink boundary を追加する。この checkpoint は Rust side sink と last completed frame state だけを扱い、scheduler loop、timer、queue、bare runtime host import、formal `std/gui` present host import、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization へ進まない。

`NativeRgb0PresenterSink` は `NativeRgba8888FrameBuffer`、explicit background color、last completed `NativeRgb0PresentBuffer`、last presented frame id を所有する。Begin / RunSpan は existing framebuffer sink と同じ validation / write contract を使う。End は descriptor equality と exact run count を検査し、RGB0 conversion succeeds の後だけ active sequence を閉じ、last completed buffer と frame id を更新する。conversion が失敗した場合は active sequence と previous completed frame を保持する。

existing `NativeRgba8888FrameBuffer` の End はこれまで通り sequence を閉じるだけであり、present buffer を作らない。`NativeRgb0PresenterSink::last_present_frame` は last completed buffer を immutable typed presenter frame として借用するだけで、mutable pixels、native endian byte view、OS handle は公開しない。

## Native window presenter state checkpoint

F5fd では completed RGB0 presenter frame を native window presenter state に保持する lib-only boundary を追加する。この checkpoint は minifb / OS window API に接続せず、scheduler loop、timer、queue、bare runtime host import、formal `std/gui` present host import、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization へ進まない。

`NativeWindowPresenterState` は `NativeWindowPresenterSurfaceState` と last presented frame id / dimensions / RGB0 pixels を private に所有する。`resize_surface` は positive size を `Drawable`、zero dimension を `NativeWindowPresenterSurfaceState::Unavailable` として記録する。resize does not stretch or crop last frame pixels; application / layout が新しい frame を present するまで previous frame を保持する。

`present_sink_frame` は `NativeRgb0PresenterSink` から completed typed frame と completed frame id の両方を要求する。frame missing、frame id missing、presenter frame validation failure、resource exhausted、dimension overflow は `NativeWindowPresenterError` の distinct variant で返す。replacement は temporary buffer の validation と reservation が成功した後だけ行うため、failure は previous frame、dimensions、frame id を保持する。

## Native window presenter smoke integration checkpoint

F5fe では native smoke runner の window loop が `NativeWindowPresenterState` を display / hit-test authority として使うようにする。この checkpoint は smoke window integration であり、formal `std/gui` host import、scheduler loop、timer、queue、bare runtime host import、FHD 60fps measurement、2D compositor drain、stroke / shadow rasterization へ進まない。

`NativeWindowPresenterState::present_frame` は positive frame id と checked typed frame を受け取り、`present_buffer` は `NativeRgb0PresentBuffer` を typed frame に変換した後で `present_frame` に委譲する。`present_sink_frame` も completed frame と frame id を取り出した後は同じ `present_frame` を使う。frame id が 0 以下の場合は `InvalidFrameId` で失敗し、`wrapping`、`saturating`、reset、silent reuse は行わない。

`nepl-gui-native` の window loop は初期 RGB0 frame を presenter state に present し、resize を `resize_surface` へ通知し、hit test と `Window::update_with_buffer` の両方で `last_present_frame_required` を読む。frame が無い場合は `FrameMissing` から error を返し、blank frame や fallback frame を合成しない。minifb / OS window API は引き続き `main.rs` のみに閉じ込める。

## Native window resize redraw checkpoint

F5ff では native smoke runner の positive resize を redraw requirement として扱う。window loop は `resize_surface` の後、`checked_add` で frame id を進め、`rasterize_frame_to_surface` で same width and height as the current drawable surface の RGB0 buffer を作り、`present_buffer` へ渡してから `Window::update_with_buffer` を呼ぶ。

minifb の `ScaleMode::UpperLeft` は OS / toolkit による stretch を避けるための smoke backend detail である。display される frame は `NativeWindowPresenterState::last_present_frame_required` から取得し、`update_with_buffer` の直前に stored frame size と current window size が一致することを検査する。一致しない場合は backend error として止める。

zero-size resize は `NativeWindowPresenterSurfaceState::Unavailable` として記録し、blank frame や fallback frame を合成しない。この state では `Window::update` だけで event pump を進め、positive drawable size が戻った時点で新しい exact-size frame を present する。

## Native presenter operation identity input checkpoint

F5fg では native presenter operation identity input boundary として、typed `ExecuteHostAction` から pending span operation identity を取り出す `platforms/gui/native/presenter_input` を追加する。この checkpoint は presenter-facing input boundary であり、not long-running scheduler backend である。F5ev is the scheduler step input boundary; F5fg is the native presenter-facing identity input boundary.

`gui_native_presenter_input` は action owner を F5ev ready payload へ移す前に borrowed accessor で pending operation を読む。operation identity は `WindowBegin`、`WindowRunSpan`、`WindowEnd`、`OffscreenBegin`、`OffscreenRunSpan`、`OffscreenEnd`、`DeviceBegin`、`DeviceRunSpan`、`DeviceEnd` を保つ typed value であり、string tag や raw integer へ潰さない。scheduler completion input は `gui_native_scheduler_executor_input` を再利用するため、F5fg は `RealLoopStepInput::ExecutorOutcome` を再実装しない。

この checkpoint は backend execution、raw status mapping、scheduler step、minifb / OS window loop、queue、timer、Canvas、DOM、video memory、fallback、silent no-op を持たない。formal native window presenter integration、bare runtime host import、long-running real backend loop は後続 slice で実装する。

## Native formal presenter session checkpoint

F5fh では formal span operation stream と native window presenter state を結ぶ Rust lib-only boundary として、`NativeWindowPresenterSession` を追加する。session は `NativeRgb0PresenterSink` と `NativeWindowPresenterState` を所有し、typed `NativeSpanOperation` を `execute_span_operation` で 1 件ずつ受ける。

Begin と RunSpan は sink の typed execution helper を通し、成功時は `NativeWindowPresenterSessionOutcome::NotPresented` を返す。End だけが completed RGB0 frame と positive frame id を要求し、`NativeWindowPresenterState::present_sink_frame` に成功した場合だけ `NativeWindowPresenterSessionOutcome::Presented` を返す。

sink 側の失敗は `NativeWindowPresenterSessionError::SinkFailed`、presenter state 側の失敗は `NativeWindowPresenterSessionError::PresenterFailed` として分けて返す。failed RunSpan、failed End、missing completed frame、missing / invalid frame id、presenter frame validation failure は previous presenter state を置き換えない。

この checkpoint は formal native presenter integration の lib boundary であり、minifb / OS window loop、actual scheduler backend、timer、queue、stdout protocol、Canvas、DOM、video memory、fallback、silent no-op を実装しない。resize は `resize_surface` で surface state だけを更新し、previous frame pixels を stretch / crop しない。application / layout は resize event 後に新しい pixel buffer を生成し、その End 成功時に session が presenter state を更新する。

## Native presenter session host helper checkpoint

F5fi では F5ey / F5ex の scalar host ABI validation path と F5fh の `NativeWindowPresenterSession` を接続する Rust lib-only boundary として、`execute_native_window_presenter_session_begin`、`execute_native_window_presenter_session_run`、`execute_native_window_presenter_session_end` を追加する。

これらの helper は既存の `validate_native_span_operation_descriptor` / `validate_native_span_operation_run_span` を使い、invalid scalar input を `NativeWindowPresenterSessionHostError::ValidationFailed NativeSpanOperationStatus` として session / sink / presenter state に到達する前に返す。validation 成功後だけ typed `NativeSpanOperation` を `NativeWindowPresenterSession::execute_span_operation` へ渡す。

Begin と RunSpan の成功は `NativeWindowPresenterSessionOutcome::NotPresented` のままであり、End 成功だけが `NativeWindowPresenterSessionOutcome::Presented` を返す。session 側の失敗は `NativeWindowPresenterSessionHostError::SessionFailed` に包まれ、lower error は `SinkFailed` と `PresenterFailed` の区別を保つ。

F5fi は long-running scheduler backend、queue、timer wait、minifb loop、bare runtime host import、formal NEPL `#extern` import 名の差し替え、Canvas、DOM、video memory host import、fallback、silent no-op へ進まない。raw `i32` status への投影は `NativeSpanOperationStatus::as_raw` と `NativeWindowPresenterSessionHostError::status` に閉じ、内部の contract は enum / `Result` として保持する。

## Native presenter session host import checkpoint

F5fj では native `platforms/gui/native/scheduler_host_executor` の formal NEPL host import ABI を、F5fi の Rust helper と同じ presenter session 境界へ寄せる。`#extern "nepl_gui_native" "window_presenter_session_begin"`、`window_presenter_session_run`、`window_presenter_session_end` は existing scalar ABI shape を保ったまま、native host 側の `execute_native_window_presenter_session_begin`、`execute_native_window_presenter_session_run`、`execute_native_window_presenter_session_end` へ対応する。

この境界は generic `execute_span_operation_begin` / `run` / `end` を native public import contract として出さない。default doctest runtime は `window_presenter_session_*` を `-1` にして explicit `Unsupported` を返すため、native session host が未提供の環境でも fallback や silent no-op にはならない。

F5fj は NEPL `#extern` 名と status mapping の formalization だけを扱う。bare runtime host import、native / bare long-running scheduler backend、queue、timer wait、minifb loop、Canvas、DOM、video memory host import、FHD 60fps measurement、2D compositor drain、font / stroke / shadow rasterization は後続 slice に分ける。

## 参考

- Apple Developer Documentation: `NSApplication.run` https://developer.apple.com/documentation/appkit/nsapplication/run
- Apple Developer Documentation: `NSWindowDelegate.windowShouldClose` https://developer.apple.com/documentation/appkit/nswindowdelegate/windowshouldclose%28_%3A%29
- Microsoft Learn: About Messages and Message Queues https://learn.microsoft.com/en-us/windows/win32/winmsg/about-messages-and-message-queues
- Microsoft Learn: `WM_CLOSE` https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-close
- Microsoft Learn: `WM_SIZE` https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-size
- Wayland `xdg-shell` protocol: `xdg_toplevel.configure` / `close` / `wm_capabilities` https://wayland.emersion.fr/protocol/xdg-shell.html
- X.Org ICCCM: Window deletion and `ConfigureNotify` https://www.x.org/releases/X11R7.7/doc/xorg-docs/icccm/icccm.html
- docs.rs minifb: `Window` https://docs.rs/minifb/latest/minifb/struct.Window.html
- docs.rs minifb: `WindowOptions` https://docs.rs/minifb/latest/minifb/struct.WindowOptions.html
- docs.rs minifb: `ScaleMode` https://docs.rs/minifb/latest/minifb/enum.ScaleMode.html
