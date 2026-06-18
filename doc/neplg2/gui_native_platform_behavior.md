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

## Current implementation

`nepl-gui-native` は正式な `std/gui::GuiHost` ではなく、native smoke backend である。

現在の checkpoint では次を実装している。

- `WindowOptions.resize = true` により OS window manager の resize を許可する。
- `ScaleMode::UpperLeft` と dark background を使い、resize 後は current drawable surface と同じ size の RGB0 buffer を presenter state へ再 present する。
- `Window::set_target_fps 60` により event pump loop の busy spin を避ける。
- `poll_minifb_window_event_pump` が `Window::get_size`、close state、left button transition、pointer sample を snapshot に正規化し、window title に current surface size を反映する。
- counter hit test は current window size、framebuffer size、letterbox offset を使って scene coordinate へ変換する。
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
