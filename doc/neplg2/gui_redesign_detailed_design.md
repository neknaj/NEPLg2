# NEPLg2 GUI bitmap surface detailed design

作成日: 2026-06-13

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

## 目的

この文書は `doc/neplg2/gui_redesign_spec.md` の詳細設計である。主に pixel buffer、video memory surface、Web bitmap presenter、offscreen / headless backend、virtual event source の内部 contract を固定する。

## 型と責務

標準 API は platform 名を public type に入れない。

```text
GuiSurfaceId
GuiSurfaceConfig
GuiSurfaceState
PixelBufferDescriptor
VideoMemoryDescriptor
FrameEpoch
DirtyRegion
GuiEventSource
VirtualGuiHost
OffscreenGuiHost
HeadlessGuiHost
```

Web の `SharedArrayBuffer`、Canvas `ImageData`、native の window handle、bare の display driver は backend implementation detail である。

## Pixel format

初期実装の pixel format は `Rgba8888` に限定する。

```text
byte offset:
    0 red
    1 green
    2 blue
    3 alpha
```

Alpha は straight alpha とする。Rasterizer が source-over alpha blending を行う場合、pixel buffer には合成後の straight alpha を書く。

Unsupported format は `GuiError::Unsupported` で返す。Format 変換を暗黙に行わない。

## Pixel buffer

Pixel buffer は次の invariant を持つ。

```text
width > 0
height > 0
stride_bytes >= width * 4
stride_bytes % 4 == 0
pixels.length >= stride_bytes * height
```

Out-of-bounds drawing は backend contract で決める。Web software rasterizer は clipping により範囲外 pixel を破棄する。Invalid geometry は clipping ではなく `GuiError::InvalidGeometry` として拒否する。

最低性能目標は 1920 x 1080 の `Rgba8888` pixel buffer を 60 fps で present することである。したがって、hot path は same-size frame ごとの巨大配列再確保を避け、resize generation が変わった時だけ surface slot / bitmap storage を作り直す。Web の legacy command frame renderer も、正式 video memory surface へ移行するまでの checkpoint として canvas ごとの bitmap buffer と `ImageData` を再利用する。これは fallback ではなく、同一 presentation contract 内の allocation policy である。

## Effect and runtime command bridge

`GuiSurfacePresentCommand` は `std/gui` の host surface ABI data contract であり、application はこれを直接 platform へ送らない。`alloc/gui/app` は std 型を import せず、core 型と検査前の request data だけを effect として保持する。`std/gui/runtime` が host capability と checked id constructor を使って `GuiSurfacePresentCommand` を作り、runtime command へ変換する。

```text
alloc/gui/app:
    PresentSurfaceEffect:
        surface i32
        frame i32
        width i32
        height i32
        stride_bytes i32
        format ColorFormat
        dirty DirtyRegion

    GuiEffect:
        None
        RequestRedraw
        SetTitle
        PresentSurface PresentSurfaceEffect

std/gui/runtime:
    GuiRuntimeCommand:
        Noop
        RequestRedraw
        SetTitle
        PresentSurface GuiSurfacePresentCommand
```

Runtime gate:

```text
SurfaceKind::WindowPixel      allow PresentSurface
SurfaceKind::OffscreenPixel   allow PresentSurface
SurfaceKind::DevicePixel      allow PresentSurface
SurfaceKind::TextGrid         reject GuiError::Unsupported
SurfaceKind::Headless         reject GuiError::Unsupported
```

`TextGrid` を reject する理由は、pixel frame と cell frame は同じ surface presentation ではないためである。TUI backend は `TextCellRun` / terminal frame contract を使い、pixel buffer を暗黙に text grid へ変換しない。

`Headless` を reject する理由は、headless が surface なし backend であり、presentation を伴わない app state transition と effect interpretation を検査する target だからである。`PresentSurface` が出る app を headless で実行した場合は `GuiError::Unsupported` を返し、test はその unsupported を `match` で検査する。

`OffscreenPixel` は visible window を持たないが pixel buffer surface を持つため allow する。Screenshot / snapshot test は runtime command を offscreen host が受け、owned pixel buffer へ反映する。

Command validation:

- `PresentSurfaceEffect` は `alloc/gui` に置くため、`SurfaceId`、`FrameId`、`GuiPixelBufferDescriptor`、`GuiSurfacePresentCommand` を持たない。
- `PresentSurfaceEffect` の `surface` と `frame` は platform handle ではなく、runtime が checked constructor に渡す request id である。0 以下や不正値は `GuiError::InvalidCommand` になる。
- `PresentSurfaceEffect` の `width`、`height`、`stride_bytes`、`format` は runtime が `gui_pixel_buffer_descriptor` で検査する。不正 geometry は `GuiError::InvalidGeometry`、未対応 format は `GuiError::Unsupported` になる。
- runtime は payload の中身を platform handle に戻さない。
- unsupported surface kind は `GuiError::Unsupported`、malformed effect payload を作れる経路が後で増えた場合は `GuiError::InvalidCommand` とする。
- batch capacity overflow は既存 `GuiEffectBatch` / `GuiRuntimeCommandBatch` と同じ `GuiError::ResourceExhausted` を返す。

Implementation note:

- Phase 4.1 の NEPL stdlib 実装では `GuiEffectBatch` の capacity 2 を維持する。これは現在の bounded data contract の継続であり、hidden fallback ではない。
- 将来 `Vec GuiEffect` へ置換するときも、`PresentSurfaceEffect` は `alloc/gui` の request data として維持し、checked `GuiSurfacePresentCommand` の生成責務は `std/gui/runtime` に残す。

## Offscreen snapshot and virtual event contract

Offscreen は visible window のない pixel surface backend である。`Headless` と同一視しない。

```text
SurfaceKind::OffscreenPixel:
    accepts GuiRuntimeCommand::PresentSurface
    can produce GuiOffscreenSnapshot
    can be used by screenshot / golden image tests

SurfaceKind::Headless:
    rejects PresentSurface with GuiError::Unsupported
    rejects screenshot with GuiError::Unsupported
    can run update / event replay tests without presentation
```

`GuiOffscreenSnapshot` は platform handle や pixel owner を持たない std layer value である。

```text
GuiOffscreenSnapshot:
    surface SurfaceId
    frame FrameId
    width i32
    height i32
    stride_bytes i32
    format ColorFormat
    dirty DirtyRegion
    pixel_hash i32
```

`pixel_hash` は backend presenter が実 pixel bytes から計算して std contract へ渡す。`std/gui/offscreen` は pixel memory を読まない。Web では video memory surface、native では framebuffer presenter、bare では device / offscreen adapter が hash authority である。これにより std layer は deterministic snapshot comparison の data boundary を持つが、DOM、Canvas、OS handle、raw pointer へ依存しない。

Validation:

- `GuiRuntimeCommand::PresentSurface` だけが snapshot source になる。
- `GuiRuntimeCommand::Noop`、`RequestRedraw`、`SetTitle` は screenshot source ではないため `GuiError::Unsupported` を返す。
- `SurfaceKind::OffscreenPixel` 以外の host で snapshot capture を要求した場合は `GuiError::Unsupported` を返す。visible window screenshot は将来別 command として定義する。
- `pixel_hash` は backend-supplied value として保持する。0 を special sentinel にしない。

Virtual event source は正規化済み `GuiEvent` を保持する test helper である。

```text
GuiVirtualClock:
    now_ms i32
    tick i32

GuiVirtualEventScript:
    first Option GuiEvent
    second Option GuiEvent
    count i32
    cursor i32

GuiVirtualEventPoll:
    script GuiVirtualEventScript
    event Option GuiEvent
```

Initial implementation は `GuiEffectBatch` と同じく bounded capacity 2 とする。内部 slot は `Option GuiEvent` であり、empty script は `Option::None` を 2 つ保持する。push は空 slot に `Option::Some event` を入れる。capacity overflow は `GuiError::ResourceExhausted` を返す。この bounded script は test helper contract であり、platform event queue の最終設計ではない。後続で `Vec GuiEvent` に置換しても、poll が `Option GuiEvent` を返し、overflow を typed error として扱う契約は維持する。

Sentinel は使わない。

- `GuiEvent::None` は追加しない。
- empty poll は `Option::None` で表す。
- raw string、DOM event object、OS event handle は virtual event script に入れない。

Virtual clock:

- `gui_virtual_clock_result now_ms` は 0 以上の initial clock を作る。negative time は `GuiError::InvalidCommand` として拒否する。
- `gui_virtual_clock_advance clock delta_ms` は negative delta を `GuiError::InvalidCommand` として拒否する。
- `now_ms + delta_ms` または `tick + 1` が i32 positive range を超える場合は wrap せず `GuiError::InvalidCommand` とする。
- advance は OS clock を読まない。caller が渡した delta だけで deterministic に進む。
- timer event は `GuiEvent::Timer` として script に入れる。virtual timer scheduler は `TimerRequest` から同じ event shape を生成する。

Virtual timer scheduler:

```text
GuiVirtualTimerState:
    request Option TimerRequest
    elapsed_ms i32
    tick i32

GuiVirtualTimerAdvance:
    state GuiVirtualTimerState
    event Option GuiEvent
```

`GuiVirtualTimerState` は deterministic timer の現在状態である。`request` が `Option::None` の場合、`elapsed_ms` と `tick` は 0 でなければならない。`request` が `Option::Some TimerRequest` の場合、window id と timer id は 1 以上、interval は 1 以上、elapsed と tick は 0 以上でなければならない。public struct constructor で壊れた state を作れるため、schedule と advance は毎回この invariant を再検査する。

`gui_virtual_timer_schedule state request` は incoming state と request を検査する。`interval_ms == 0` は clear request として active timer を消す。`interval_ms > 0` は active request を保持し、elapsed と tick を 0 へ戻す。invalid state、invalid ids、negative interval は `GuiError::InvalidCommand` である。

`gui_virtual_timer_advance state delta_ms` は real clock を読まない。negative delta、elapsed overflow、tick overflow、malformed state は `GuiError::InvalidCommand` である。発火しない場合は `Option::None`、発火する場合は `Option::Some GuiEvent::Timer` を返す。

Repeating timer は 1 回の advance で最大 1 event だけを返す。catch-up で extra elapsed があっても捨てず、`sub next_elapsed interval_ms` を remainder として state に保持する。remainder がまだ interval 以上なら、caller は `advance state 0` により queue を使わず 1 event ずつ drain できる。One-shot timer は 1 event を返すときに state を `None` へ戻し、残り elapsed を保持しない。これは Web one-shot timer が enqueue 前に active entry を clear する挙動と対応する。

Virtual timer scheduler は std layer の deterministic test contract であり、DOM、Canvas、minifb、OS timer、browser timer、stdout protocol、event queue、video memory、presentation fallback を持たない。

Virtual timer turn bridge:

```text
GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerPending:
    pending GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnTimerPending
    timer_state GuiVirtualTimerState

GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerAdvance:
    Pending GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerPending
    Ready GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnSchedulerDecision
```

F5dz の std layer row tile RLE present host span operation presenter executor session turn virtual timer bridge は、F5dw の target-neutral timer pending と F5dy の deterministic virtual timer state を結びつける。`gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_schedule pending timer_state` は F5dw pending から borrowed `TimerRequest` を読み、`gui_virtual_timer_schedule` を 1 回だけ呼ぶ。schedule failure は original pending、original virtual timer state、lower `GuiError` を保持する。

`gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_advance pending delta_ms` は `gui_virtual_timer_advance` を 1 回だけ呼ぶ。event がなければ next pending を返す。`GuiEvent::Timer` が出た場合だけ F5dw `turn_timer_complete` を 1 回だけ呼び、成功時は scheduler decision を返す。unexpected event は F5dw pending、advance-after virtual timer state、event を保持する owner-bearing error にする。timer complete failure は F5dw complete error と advance-after virtual timer state を保持する。ここでは real scheduler loop、actual timer backend、queue、DOM、Canvas、minifb、video memory、presentation fallback、silent no-op、loop drain を持たない。

Virtual scheduler state boundary:

```text
GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState:
    Turn TurnPayload
    WaitingTimer VirtualTimerPending
    Execute ExecutePayload
    Completed CompletedPayload

TurnPayload:
    timer_state GuiVirtualTimerState
    turn_state TurnState

ExecutePayload:
    timer_state GuiVirtualTimerState
    pending TurnDriverPending

CompletedPayload:
    timer_state GuiVirtualTimerState
```

F5ea の std layer row tile RLE present host span operation presenter executor session turn virtual scheduler state boundary は、F5dv scheduler decision、F5dw timer request、F5dz virtual timer bridge を deterministic state として接続する。`GuiVirtualTimerState` は policy ではなく dynamic state なので、`Turn`、`Execute`、`Completed` の payload または F5dz `WaitingTimer` pending に保持する。`ContinueNow` は reusable decision に戻すと no-progress state になり得るため、次に driver poll できる `Turn` phase として保持する。

Decision boundary は F5dw `turn_timer_interpret_decision` を 1 回だけ呼ぶ。`ScheduleTimer` だけが F5dz schedule を呼び、success は `WaitingTimer` になる。Timer advance boundary は F5dz `virtual_timer_advance` を 1 回だけ呼ぶ。`Ready` decision が返った場合、F5dw request は one-shot で F5dy / F5dz は completion 前に virtual timer を clear しているため、F5ea は `gui_virtual_timer_empty` を明示的な next dynamic state として decision boundary へ戻す。F5ea は loop drain、timeslice budget、actual backend timer、event queue、platform API、DOM、Canvas、minifb、video memory、fallback、silent no-op を持たない。

F5eb の std layer row tile RLE present host span operation presenter executor session turn virtual scheduler single step boundary は、F5ea state を 1 回だけ進める境界である。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerStepResult` は `Advanced`、`BlockedWaitingTimer`、`BlockedExecute`、`Completed` を持つ。Turn path は F5du driver poll、F5dv scheduler decide、F5ea timer decide の順序を固定し、それぞれを 1 回だけ呼ぶ。

`WaitingTimer`、`Execute`、`Completed` は F5eb 内で queue、backend、executor、platform API へ進まない。`WaitingTimer` は `BlockedWaitingTimer`、`Execute` は `BlockedExecute` として返し、real scheduler loop / timeslice policy / headless app-loop integration が次の authority として処理する。poll failure と scheduler decision failure は current `GuiVirtualTimerState` を失わず、timer decision failure は F5ea lower owner-bearing error を保持する。F5eb は loop drain、timeslice budget、event queue、platform API、DOM、Canvas、minifb、video memory、fallback、silent no-op を持たない。

F5ec の std layer row tile RLE present host span operation presenter executor session turn virtual scheduler bounded drain boundary は、F5eb step を `max_advance_count` で bounded に消費する境界である。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainPolicy` は F5eb step policy と `max_advance_count` だけを保持し、dynamic timer state、backend handle、queue owner を持たない。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainResult` は `BudgetExhausted`、`BlockedWaitingTimer`、`BlockedExecute`、`Completed` を持つ。

`max_advance_count` は construction と drain entry の両方で 0 以上に検査する。0 は step を呼ばない `BudgetExhausted` であり、test / headless runtime が no-progress を明示的に扱うための terminal である。`Advanced` だけが budget を 1 消費し、`BlockedWaitingTimer`、`BlockedExecute`、`Completed` は budget を消費せずに外側 authority へ返る。`StepFailed` は F5eb lower error だけを保持し、original state を重複保持しない。F5ec は timer advance、executor completion、real scheduler loop、platform API、DOM、Canvas、minifb、video memory、fallback、silent no-op を持たない。

F5ed の std layer row tile RLE present host span operation presenter executor session turn virtual scheduler transition boundary は、F5ec drain terminal を後続 loop が扱う action boundary へ写す。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransition` は `YieldSlice`、`AwaitTimer`、`ExecuteHostAction`、`Done` を持つ。`YieldSlice` は `BudgetExhausted` の state、`AwaitTimer` は `BlockedWaitingTimer` の pending timer、`ExecuteHostAction` は `BlockedExecute` の execute authority、`Done` は `Completed` の completed payload を保持する。

F5ed は F5ec payload struct を transition payload として保持しない。各 branch は F5ec accessor で `remaining_count` を先に読み、owner-bearing payload から state / pending / execute / completed を取り出して transition-owned payload に詰め替える。`remaining_count` は正規化、減算、再計算をしない。F5ed は F5ec drain 再実行、F5eb step、timer advance、executor completion、real scheduler loop、queue、platform API、DOM、Canvas、minifb、video memory、fallback、silent no-op を持たない。

F5ee の std layer row tile RLE present host span operation presenter executor session turn virtual scheduler slice boundary は、F5ec bounded drain と F5ed transition を 1 work slice の public boundary として接続する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSliceResult` は `YieldSlice`、`AwaitTimer`、`ExecuteHostAction`、`Done` を持つ。Policy は F5ec drain policy と `yield_delay_ms` だけを保持し、dynamic timer state、backend handle、queue owner を保持しない。

F5ee は policy construction と slice entry の両方で `yield_delay_ms >= 0` を検査する。public slice entry は F5ec drain を 1 回だけ呼び、成功時だけ F5ed transition mapping を 1 回だけ呼ぶ。F5ec / F5ed payload struct は slice payload として保持せず、state / pending / execute / completed と `remaining_count` を slice-owned payload に詰め替える。`YieldSlice` は state、`remaining_count`、`yield_delay_ms` を保持する。Drain failure は lower F5ec error だけを保持し、original scheduler state を重複保持しない。F5ee は F5eb step 直接呼び出し、timer advance、executor completion、real scheduler loop、queue、platform API、DOM、Canvas、minifb、video memory、fallback、silent no-op を持たない。

F5ef の std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop boundary は、F5ee `virtual_scheduler_slice` の結果を real scheduler loop / headless app-loop が扱う loop-owned result に変換する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopResult` は `Yield`、`AwaitTimer`、`ExecuteHostAction`、`Done` を持つ。Policy は F5ee slice policy だけを保持し、dynamic timer state、backend handle、queue owner を保持しない。

F5ef public step は F5ee `virtual_scheduler_slice` を 1 回だけ呼ぶ。F5ee payload struct は loop payload として保持せず、state / pending / execute / completed と `remaining_count` を loop-owned payload に詰め替える。`Yield` は state、`remaining_count`、`yield_delay_ms` を保持する。Failure は lower-only slice error として lower F5ee slice error だけを保持する。F5ef は F5ec drain、F5ed transition、F5eb step、F5ea state helper を直接呼ばず、timer advance、executor completion、actual while loop、queue drain、platform API、DOM、Canvas、minifb、video memory、fallback、silent no-op を持たない。

F5eg の std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop action boundary は、F5ef loop result を outer real scheduler loop / headless app-loop authority が消費する action value に変換する。`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopAction` は `YieldToClock`、`AwaitTimerAdvance`、`ExecuteHostAction`、`Complete` を持つ。`loop_action_from_result` は F5ef result の `Yield`、`AwaitTimer`、`ExecuteHostAction`、`Done` を explicit match で action へ写す total mapping である。F5eg は F5ef `loop_step` を呼ばず、F5ef payload struct を action payload として保持しない。Payload は state / pending / execute / completed authority、`remaining_count`、`yield_delay_ms` を action-owned value として保持する。F5eg は timer advance、executor completion、real scheduler loop、queue drain、native / bare / headless real backend、platform API、DOM、Canvas、minifb、video memory、fallback、silent no-op を持たない。

F5eh の std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop timer advance boundary は、F5eg `AwaitTimerAdvance` payload を consumed authority として受け、F5ea `virtual_scheduler_advance_timer` を 1 回だけ呼ぶ。`loop_timer_advance` は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionAwaitTimerAdvance`、`TurnTimerPolicy`、`delta_ms` だけを入力にし、general `LoopAction` や F5eg `loop_action_from_result` は扱わない。`remaining_count` は pending owner を消費する前に読み、成功時は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopTimerAdvanceCompleted` として次 state と original `remaining_count` を返す。失敗時は lower F5ea `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerAdvanceError` と original `remaining_count` を保持する。F5eh は executor completion、yield-to-clock handling、real scheduler loop、queue drain、native / bare / headless real backend、platform API、DOM、Canvas、minifb、video memory、fallback、silent no-op を持たない。

F5ei の std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop executor complete boundary は、F5eg `ExecuteHostAction` payload を consumed authority として受け、caller supplied `Result unit GuiError` を F5du `turn_driver_complete` へ 1 回だけ戻す。F5ei の `loop_executor_complete` は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorCompletePolicy`、`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionExecuteHostAction`、caller supplied outcome だけを入力にする。policy は scheduler policy と timer policy だけを保持し、timer state、queue、backend handle、host handle は保持しない。`remaining_count`、`execute`、`timer_state`、`pending` の順に取り出し、pending owner を消費した後、F5du `turn_driver_complete`、F5dv `scheduler_decide`、F5ea `virtual_scheduler_decide` をそれぞれ 1 回だけ呼ぶ。成功時は `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorCompleteCompleted` として次 state と original `remaining_count` を返す。失敗時は lower F5du / F5dv / F5ea error と original `remaining_count` を保持し、F5du / F5dv 由来の失敗では `category` と `timer_state` も保持する。F5ei は outcome 合成、yield-to-clock handling、complete handling、real scheduler loop、queue drain、native / bare / headless real backend、platform API、DOM、Canvas、minifb、video memory、fallback、silent no-op を持たない。

F5ej の std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop yield complete boundary は、F5eg `YieldToClock` / `Complete` payload を typed action authority として受ける。F5ej の `loop_yield_complete_yield_advance` は later actual real scheduler loop が clock delta を適用するときに呼ぶ deterministic clock-delta authority であり、`YieldToClock` payload と caller supplied `delta_ms` だけを入力にする。`remaining_count` と `yield_delay_ms` を state owner consumption 前に読み、`delta_ms < 0` を `DeltaInvalid`、`yield_delay_ms < 0` を `YieldDelayInvalid` として owner-bearing error にする。pending branch では `0 <= delta_ms < yield_delay_ms` が成り立つ場合に限って `sub yield_delay_ms delta_ms` を実行し、same state / same `remaining_count` / reduced delay の `YieldPending` を `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYieldCompleteYieldAdvanceResult` として返す。ready branch では state owner と original `remaining_count` を `YieldReady` として返す。`loop_yield_complete_complete_ack` は `Complete` payload の `remaining_count` を completed owner consumption 前に読み、terminal completed payload に変換する。F5ej は timer advance、executor completion、scheduler decision、actual real scheduler loop、headless app-loop integration、native / bare real backend、queue drain、platform API、DOM、Canvas、minifb、video memory、fallback、silent no-op を持たない。

Pixel hash:

- `pixel_hash` は signed opaque `i32` として全 bit pattern を有効値にする。
- 0 や -1 を sentinel にしない。
- hash algorithm と collision policy は backend presenter contract で定義し、std layer は hash value を保持するだけにする。

## Video memory header

Web backend の video memory surface は `SharedArrayBuffer` を 1 つ使い、header と 2 個以上の pixel plane を同じ buffer 内に置く。単一 pixel plane は writer と presenter が同じ memory を同時に触る危険があるため、正式 contract では禁止する。

Header は `Int32Array` view で扱う。

```text
index 0  magic
index 1  version
index 2  width
index 3  height
index 4  stride_bytes
index 5  format
index 6  resize_generation
index 7  slot_count
index 8  latest_published_epoch
index 9  latest_presented_epoch
index 10 surface_state
index 11 error_code
index 12 header_int32_length
index 13 pixel_plane_byte_offset
index 14 pixel_plane_byte_length
index 15 reserved
```

Slot header は global header の後ろに slot ごとに並べる。

```text
slot index 0  slot_state
slot index 1  slot_epoch
slot index 2  dirty_kind
slot index 3  dirty_x
slot index 4  dirty_y
slot index 5  dirty_width
slot index 6  dirty_height
slot index 7  reserved
```

Pixel plane は slot header 群の後ろに slot 順で置き、各 slot を `Uint8ClampedArray` view として読む。`pixel_plane_byte_length` は 1 slot 分の byte length であり、`stride_bytes * height` 以上である。

`magic` と `version` は incompatible buffer を fail-closed に拒否するために使う。Unknown magic / version は `GuiError::InvalidCommand` 相当の Web typed error にする。

## Web Canvas video memory presenter

Web の正式 presenter は video memory surface の published slot を `ImageData` として `putImageData` へ渡す。Visible Canvas は presentation device であり、GUI content の drawing authority ではない。

Presenter が使ってよい Canvas API は次に限定する。

```text
new ImageData
putImageData
```

`fillRect`、`stroke`、`drawImage`、CSS transform、DOM element による widget 表現は正式 video memory presenter の hot path に入れない。図形、文字、UI chrome は pixel buffer へ rasterize 済みの byte 列として渡される。

Initial Web presenter は tightly packed `Rgba8888` だけを受ける。

```text
stride_bytes == width * 4
pixel_byte_length == width * height * 4
```

Padded stride は暗黙に row copy しない。`ImageData` と互換でない stride は typed `UnsupportedStride` 相当の error として拒否する。Acquired slot の stride が拒否された場合は、slot を discard して writer を詰まらせず、presented epoch は進めない。後続で tiled presenter や row-copy presenter を追加する場合も、allocation / copy policy を明示した別 path とする。

Dirty region は fail-closed に検査する。

```text
x >= 0
y >= 0
width >= 0
height >= 0
x + width <= surface.width
y + height <= surface.height
```

範囲外 dirty region は clamp しない。`InvalidDirtyRegion` として返し、slot は `Reading` から解放するが、presented epoch は進めない。

Zero-size dirty region は valid な no-op presentation とする。

```text
width == 0 または height == 0:
    putImageData は呼ばない
    slot は release する
    presented epoch は進める
```

Canvas presentation が失敗した場合は typed `PresentFailed` として返す。JavaScript exception の message は branch authority にしない。失敗した frame は表示済みではないため、slot は discard して writer を詰まらせないが、presented epoch は進めない。

成功時:

```text
Published -> Reading -> putImageData -> Free
presented_epoch = slot.epoch
```

reject / failure 時:

```text
Published -> Reading -> discard -> Free
presented_epoch は変更しない
```

FHD 60 fps の最低性能目標を満たすため、presenter は `SharedArrayBuffer` と slot index ごとに `ImageData` を cache する。同じ slot を再利用する frame では `ImageData` を再生成せず、同じ underlying byte view の内容更新だけを presentation に反映する。

## Web runtime video memory bridge

Web runtime bridge は `neplGuiHost.presentVideoMemory` を公開する。これは legacy stdout transport や command frame stream の代替 fallback ではなく、Web backend の正式 video memory surface を visible floating window へ提示するための typed runtime boundary である。

Input shape:

```text
VideoMemoryFrame:
    windowId positive integer
    title string
    buffer SharedArrayBuffer
```

`buffer` は `SharedArrayBuffer` だけを受ける。`ArrayBuffer`、typed array、numeric id、string handle、transferable object は `invalid-video-memory-frame` として拒否する。`ArrayBuffer` transfer path や stdout protocol へ自動的に戻る処理は持たない。

Runtime error kind:

```text
invalid-video-memory-frame
video-memory-open-failed
video-memory-present-failed
```

`openGuiVideoMemorySurface` の失敗は `video-memory-open-failed`、`presentNewestGuiVideoMemoryFrameToCanvas` の失敗は `video-memory-present-failed` へ写す。元の `GuiVideoMemoryError.kind` は `actual` に含め、JavaScript exception や platform string handle を branch authority にしない。

Floating window は `windowId` で既存 window を再利用する。同じ `windowId` に command frame と video memory frame が交互に来た場合、latest presentation kind が panel state を置き換える。Panel state は次の union とする。

```text
none
command-frame
video-memory
```

Panel は同じ `SharedArrayBuffer` identity の video memory surface を再利用し、buffer identity が変わった時だけ `openGuiVideoMemorySurface` を再実行する。これは FHD 60 fps の hot path で header validation object を作り直し続けないための policy である。

Surface size と drawable surface size が異なる場合、presenter は CSS scale や Canvas transform で引き伸ばさない。初期実装では pixel buffer を top-left に 1:1 で提示し、window resize event により application が新しい surface / frame を生成する。Hidden stretch、row-copy fallback、`drawImage` 拡大縮小は禁止する。

## Web video memory host import ABI

Web worker runtime は Web-only import module `nepl_gui_web` に video memory host import ABI を持つ。この ABI は `core/gui`、`alloc/gui`、`std/gui` の public contract ではなく、`platforms/gui/web` が Web backend に接続するための platform boundary である。

現在の scalar import は次である。

```text
video_memory_create_surface width height slot_count -> surface_id_or_negative_status
video_memory_acquire_write_slot surface_id -> frame_id_or_negative_status
video_memory_write_slot_bytes surface_id frame_id dst_offset src_ptr byte_len -> status
video_memory_write_rgba8888_row surface_id frame_id x y width src_ptr -> status
video_memory_fill_rect_rgba8888 surface_id frame_id x y width height r g b a -> status
video_memory_discard_write_slot surface_id frame_id -> status
video_memory_publish_slot surface_id frame_id dirty_kind x y width height -> status
video_memory_present_surface window_id title_ptr title_len surface_id -> status
video_memory_close_surface surface_id -> status
request_timer window_id timer_id interval_ms repeating -> status
```

`surface_id` と `frame_id` は worker-local opaque positive integer である。NEPL/Wasm code は `SharedArrayBuffer`、DOM handle、Canvas handle、JS object handle、ArrayBuffer transfer object、string handle を受け取らない。`SharedArrayBuffer remains a Web backend detail` であり、Worker は `video-memory-surface.ts` の ownership API だけで surface / slot を扱う。

`write_slot_bytes` は low-level byte copy escape hatch である。application code が row payload を扱う場合は `write_rgba8888_row` を使い、app 側で `y * stride + x * 4` の byte offset を計算しない。`write_rgba8888_row` は `width > 0`、`x + width <= surface.width`、`0 <= y < surface.height`、source byte length が `width * 4` であることを Worker と surface helper の両方で検査する。成功時は pixel plane だけを更新し、dirty metadata、slot epoch、published epoch、presented epoch は更新しない。dirty region と epoch の authority は `publish_slot` である。

`examples/gui_video_memory_rows.nepl` はこの row payload 境界の focused source example である。row bytes は `ByteBuilder` / `ByteBuf` owner と borrowed `MemPtr u8` で渡し、stdout `rgba-row` や command frame fallback は使わない。通常 doctest の `run_test.js` default `nepl_gui_web` import は unsupported stub のまま残し、positive path は `nodesrc/test_web_gui_video_memory_fake_host_harness.js` が opt-in fake host import を注入して通常 path の NEPL/Wasm 実行として検査する。

Negative status は Web platform module 内で `Result` と `GuiError` へ写す。Raw sentinel は public wrapper から漏らさない。

```text
0  Ok
-1 Unsupported
-2 InvalidArgument
-3 ResourceExhausted
-4 NoWritableSlot
-5 BackendFailure
-6 StaleFrame
```

`request_timer` は formal event loop の timer 登録 request である。`window_id` は既に `present_surface` に成功して Shell が active window として保持している window だけを受ける。未提示 window への timer request は `InvalidArgument` とし、別 window 作成、stdout `NEPLG2_GUI_ANIMATE_MS`、polling loop へ fallback しない。`interval_ms == 0` は同じ window / timer id の timer clear request である。`repeating == 1` は repeating timer、`repeating == 0` は one-shot timer として受ける。Web host は repeating timer を `setInterval`、one-shot timer を `setTimeout` へ接続し、one-shot timer は `GuiEvent::Timer` を input queue へ入れる前に active timer entry を消す。timeslice budget、virtual scheduler と real scheduler の統合、native / bare / headless scheduler backend は後続 slice で定義する。

`discard_write_slot` は未公開 write frame の `Writing -> Free` 状態遷移だけを行う。描画途中の error や application 側の中断で publish しない frame は、surface close ではなくこの import で明示的に破棄する。成功時は dirty metadata を消し、published epoch と presented epoch は進めない。frame が存在しない、既に publish / discard 済み、resize generation が古い場合は typed negative status を返し、stdout protocol や別 surface へ fallback しない。

`publish_slot` は `Writing -> Published` の状態遷移だけを行う。Visible window への提示は `present_surface` が別に行い、Worker から main thread へ typed `gui_video_memory_present` message を送る。main thread の Shell は `presentGuiWebRuntimeVideoMemory` を呼び、その結果を ack 用 `SharedArrayBuffer` へ書いて `Atomics.notify` する。Worker import は ack を `Atomics.wait` してから status を返すため、message queued を success として扱わない。

`title_ptr` と `title_len` は Wasm linear memory 内の UTF-8 byte slice である。Worker は bounds と UTF-8 validity を検査し、不正な pointer、length、UTF-8 は `InvalidArgument` として返す。Platform boundary は JavaScript exception message や browser string handle を branch authority にしない。

この host import ABI は stdout protocol や command frame stream への fallback を持たない。Worker import path は `presentGuiWebRuntimeVideoMemory` を直接呼ばず、main thread handler だけが runtime presenter を呼ぶ。これにより Web Worker と DOM / Canvas authority の境界を保ち、NEPL/Wasm import の戻り値は actual presenter result を反映する。

## Frame publish protocol

Slot state:

```text
Free
Writing
Published
Reading
Closed
```

Surface state:

```text
Ready
Closing
Closed
Unavailable
```

Protocol:

1. writer は slot header を走査し、`Atomics.compareExchange(slot_state, Free, Writing)` に成功した slot だけを取得する。
2. writer は取得した slot の pixel plane だけを更新する。`Published`、`Reading`、`Closed` の slot へ書いてはいけない。row payload は `write_rgba8888_row` で `GuiPoint + width + src` として渡し、byte offset arithmetic は Web platform module と host helper に閉じ込める。
3. writer が frame を破棄する場合は、dirty metadata を 0 に戻し、`Atomics.compareExchange(slot_state, Writing, Free)` に成功した時だけ frame id の ownership record を消す。published epoch と presented epoch は進めない。
4. writer が frame を公開する場合は slot dirty region を書く。
5. writer は slot epoch を新しい値へ `Atomics.store` する。
6. writer は `Atomics.store(slot_state, Published)` で publish し、`latest_published_epoch` を更新して `Atomics.notify` する。
7. presenter は `Atomics.compareExchange(slot_state, Published, Reading)` に成功した slot だけを読む。
8. presenter は `ImageData` を作り、visible canvas へ `putImageData` する。`putImageData` が完了するまで slot は `Reading` のまま保持する。
9. presenter は `latest_presented_epoch` を更新し、`Atomics.store(slot_state, Free)` で slot を writer へ返す。

Atomics ordering:

- slot acquisition は `Atomics.compareExchange` で行う。
- pixel plane 書き込み後、slot metadata と slot state は `Atomics.store` で publish する。
- presenter は slot state を `Atomics.load` / `compareExchange` で確認してから pixel plane を読む。
- presenter への通知は `Atomics.notify`、blocking wait が必要な writer / presenter は `Atomics.wait` を使う。
- writer は free slot がない場合、API contract に応じて wait するか `GuiVideoMemoryError::NoWritableSlot` を返す。別 transport を選んではいけない。

途中で surface が resize / close された場合、writer は old generation への publish を `GuiVideoMemoryError::StaleResizeGeneration` として失敗させる。Old surface へ silently publish しない。

Visible window presenter は published pixel buffer を 1:1 で表示する。Window が広がった場合も、presenter は古い pixel buffer を拡大縮小しない。Backend は drawable surface の logical pixel size を `WindowEvent::Resized` として発行し、application / layout engine が新しい size の pixel buffer を生成する。新しい frame が来るまでの余白は surface background であり、CSS transform、Canvas scale、fit-to-window viewport による content stretch は禁止する。

## Dirty region

Dirty region は初期実装では次を扱う。

```text
DirtyKind:
    Empty
    Rect
    Full
```

Multiple rect set は stdlib の `DirtyRegionSet` contract が安定した後に追加する。初期 Web presenter は `Rect` と `Full` を扱い、`Empty` は present しない。

Dirty rect の width / height が負の場合は invalid。Zero-size rect は valid だが present は行わない。

## Web rasterizer

`web/src/gui-preview` は次に分割する。

```text
commands.ts
host-bridge.ts
stdout-protocol.ts
bitmap-buffer.ts
bitmap-rasterizer.ts
bitmap-presenter.ts
video-memory-surface.ts
canvas-renderer.ts
```

`canvas-renderer.ts` は compatibility facade として残す場合でも、visible canvas direct primitive を呼ばない。責務は frame を bitmap buffer に rasterize し、presenter に渡すことだけである。

`canvas-renderer.ts` が legacy command frame を visible canvas に表示する場合でも、viewport は `left = 0`、`top = 0`、`scale = 1` の logical pixel mapping に固定する。Device pixel ratio は backing bitmap への rasterize scale としてだけ扱う。`padding`、centering、`availableWidth / frame.width` による fit scale を再導入してはいけない。

禁止:

```text
ctx.fillText
ctx.strokeText
ctx.fillRect for app content
ctx.stroke
ctx.drawImage for app content
DOM element creation for app content
```

許可:

```text
new ImageData
ctx.putImageData
canvas width / height synchronization
```

Canvas background clear も pixel buffer 側で行う。Visible canvas context は pixel presentation 以外に使わない。

## Text rasterization

初期 Web implementation は ASCII bitmap font を持つ。

Contract:

- ASCII printable range を deterministic bitmap glyph として描く。
- Unsupported scalar は replacement box glyph として描くのではなく、初期実装では `GuiError::Unsupported` として frame を publish しない。
- Text alignment は rasterizer が glyph width から開始 x を計算する。
- Font shaping、IME composition、complex script は後続の NEPL font rasterizer で扱う。

この設計により、visible canvas の `fillText` に依存しない。

## Offscreen backend

Offscreen backend は owned `PixelBuffer` を持つ。

```text
OffscreenGuiHost:
    create_surface
    render_frame
    capture_surface
    destroy_surface
```

`capture_surface` は current pixel buffer の snapshot を返す。Snapshot は pixel hash、width、height、format を持つ。

Offscreen backend は CI、screenshot、visual regression に使う。Visible window を作らない。

## Headless backend

Headless backend は surface を持たない。

許可:

- `init`
- `update`
- `view`
- layout without presentation
- effect interpretation
- virtual event replay

禁止:

- `present`
- `capture_surface`
- visible window creation

禁止 operation は `GuiError::Unsupported` で返す。

## Virtual event source

Virtual event source は platform event と同じ `GuiEvent` を生成する。

```text
VirtualEventScript:
    events
    clock
    cursor

VirtualClock:
    now_ms
    scheduled_timers
```

Contract:

- replay order は deterministic。
- timer は virtual clock / virtual timer の advance によってだけ発火する。
- pointer / keyboard / text input / resize / close request は platform event と同じ typed shape を使う。
- invalid event script は `GuiError::InvalidCommand` とする。

## Native backend

Native backend は OS event pump を ownership boundary とする。

```text
native event pump
    -> GuiEvent
GuiEvent
    -> app update
DrawCommand
    -> native software rasterizer
PixelBuffer
    -> native framebuffer presenter
```

Window handle、minifb、Win32、AppKit、Wayland、X11 は backend 内部に閉じる。Resize 中の pointer move や window state は coalesce してよいが、close request、button up、keyboard、text input、action event を上書きしてはいけない。

## Bare backend

Bare backend は `DevicePixel` surface を使う。

```text
fixed framebuffer
optional flush target
polling input
dirty region
```

Allocator は要求しない。Widget tree が allocator を必要とする場合、その app は bare no_alloc profile では `MissingCapability` になる。

## Error policy

Video memory typed errors:

```text
GuiVideoMemoryError:
    SharedBufferUnavailable
    InvalidSurfaceConfig
    InvalidBufferLength
    InvalidHeaderMagic
    UnsupportedHeaderVersion
    InvalidHeaderLayout
    InvalidSurfaceState
    InvalidSlotState
    NoWritableSlot
    NoPublishedSlot
    StaleResizeGeneration
    PresenterUnavailable
    WriterClosed
    WaitUnavailable
    UnsupportedPixelFormat
    UnsupportedCommand
```

`GuiVideoMemoryError` は Web implementation detail の typed error であり、stdlib 境界では対応する `GuiError::Unsupported`、`GuiError::InvalidCommand`、`GuiError::ResourceExhausted`、`GuiError::BackendFailure` へ lossless に近い形で map する。文字列 sentinel へ潰さない。

次を禁止する。

- unsupported operation の no-op 成功
- hidden fallback
- panic / unreachable による通常失敗処理
- string sentinel
- `null` / `undefined` の public boundary 露出

次を使う。

- `Option`
- `Result`
- `GuiError`
- backend-specific typed error union
- exhaustive `match`

## Static regression policy

実装後は source policy test で次を固定する。

- visible Web renderer に `fillText` がない。
- visible Web renderer に app content 用 `fillRect` がない。
- visible Web renderer に `putImageData` がある。
- Web command DTO に DOM / Canvas 型がない。
- `core/gui` / `alloc/gui` / `std/gui` に Web / native concrete type name がない。
- stdout GUI presentation を正式 path として参照しない。
## F5ek std layer row tile RLE present host span operation presenter executor session turn virtual scheduler real loop step boundary

F5ek は actual real scheduler loop / headless app-loop のために、F5eg `LoopAction` と explicit input の dispatch を std layer で固定する境界である。ここでは loop 本体、queue drain、sleep、backend executor、platform API は扱わない。

`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerRealLoopStepPolicy` は `scheduler_policy` と `timer_policy` だけを持つ。`LoopExecutorCompletePolicy` を保持すると timer policy が二重化するため、Execute branch は F5ei の `loop_executor_complete_with_policy_refs` を呼んで同じ timer policy authority を借用する。

`RealLoopStepInput` は `ClockDelta`、caller supplied `ExecutorOutcome`、explicit `CompleteAck` である。action/input の組み合わせは wildcard なしで match し、不一致は action owner と input owner を持つ mismatch error として返す。fallback、silent no-op、executor outcome の合成は禁止する。
