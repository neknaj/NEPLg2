# NEPLg2 bare GUI platform behavior notes

作成日: 2026-06-18

## 目的

この文書は bare GUI backend が持つ制約を整理し、NEPLg2 GUI の platform boundary へ落とすための notes である。bare は Web、native desktop、headless test runner と違い、標準化された OS clock、window manager、timer queue、filesystem、thread sleep を持つとは限らない。

## Bare backend contract

Bare backend は次を守る。

- core / alloc / std GUI substrate は universal wall clock を仮定しない。
- monotonic clock が必要な場合、embedding host が `nepl_gui_bare.monotonic_clock_ms` を明示的に提供する。
- host が clock source を提供しない場合は -1 sentinel を返し、NEPL wrapper は `GuiError::Unsupported` として扱う。
- -1 以外の負値は `GuiError::BackendFailure` として扱う。
- non-negative sample だけを F5eo `BackendClockSample` constructor へ渡す。
- Web `performance.now`、native `Instant`、wall clock、timer、sleep、queue、stdout protocol、rendering API、fallback、silent no-op は bare clock source として使わない。

## Current implementation

F5es では Bare formal monotonic clock source backend boundary として、`platforms/gui/bare/clock` を追加する。これは bare 環境の clock を stdlib が生成する実装ではなく、embedding host が明示提供する import ABI の contract である。

`nodesrc/run_test.js` の `nepl_gui_bare.monotonic_clock_ms` は doctest-only unsupported source であり、hidden fallback や hidden mock ではない。既定で -1 を返すことで、host が clock を提供しない場合に `Unsupported` が返ることを検査する。bare scheduler backend、bare timer backend、display present、long-running real backend loop は後続 slice で実装する。

F5et の bare scheduler clock は long-running scheduler backend ではなく、bare host の clock sample を F5eo `BackendClockPolicy` / `BackendClockState` へ 1 tick 分だけ接続する helper である。host が `nepl_gui_bare.monotonic_clock_ms` を提供しない場合、start / tick は fallback source を探さず `Unsupported` を保持する typed error を返す。tick sample failure は policy と state を保持し、caller が次の判断を失わないようにする。

F5fk では Bare display presenter session host import boundary として、`platforms/gui/bare/scheduler_host_executor` の formal NEPL host import ABI を `display_presenter_session_begin`、`display_presenter_session_run`、`display_presenter_session_end` に差し替える。bare は window manager を持つとは限らないため、native の `window_presenter_session_*` ではなく、device / offscreen / display surface へ接続される presenter session として命名する。

この境界は generic `execute_span_operation_begin` / `run` / `end` を bare public import contract として出さない。`nodesrc/run_test.js` の doctest-only default stub は `display_presenter_session_*` を `-1` にして explicit `Unsupported` を返す。これは hidden fallback や hidden mock ではなく、embedding host が display presenter session ABI を提供しない場合の fail-closed contract である。bare actual display driver、framebuffer adapter、polling input、native / bare long-running scheduler backend、timer queue、present loop、Web / native API、fallback、silent no-op は後続 slice へ分ける。

F5fl では Bare display framebuffer adapter boundary として、`platforms/gui/bare/framebuffer` を追加する。これは actual display driver ではなく、F5fk の existing bare scheduler host executor へ渡す前の pure validation state machine である。Begin / RunSpan / End の順序、target、surface、frame、shape、row-major progress、incomplete end を検査し、validation failure は state と operation を保持する typed error で返す。Begin descriptor と RunSpan / End 前の active descriptor は `std/gui/tile_present` の descriptor contract と同じく、frame id 一致、positive geometry / counts、plan row extent、tile row extent、stride、tile count / index、pixel count、encoded byte count を再検査する。active state の `seen_run_count` / `seen_pixel_count` は non-negative かつ descriptor count 以下でなければならず、public state が偽造されても host executor には進まない。wrapper は pure validation が成功した後だけ existing bare scheduler host executor を 1 回だけ呼び、host failure は `HostExecutionFailed` として original state と operation を保持する。これは not long-running scheduler backend であり、timer queue、present loop、actual display storage、fallback、silent no-op は実装しない。

F5fm では Bare display storage adapter boundary として、`platforms/gui/bare/display_storage` を追加する。これは actual display driver ではなく、F5fl の validation result を bare display storage が消費できる typed effect ledger へ変換する boundary である。`GuiBareFramebufferStepApplied` は public value なので、その supplied next state を信用しない。storage state が保持する canonical framebuffer state から operation を再検証し、expected next framebuffer state と supplied next framebuffer state が一致しない場合は `AppliedStateMismatch` として拒否する。storage phase は canonical framebuffer phase と accepted run / pixel count に一致しなければならず、public storage state が偽造された場合は `StoragePhaseMismatch`、`TargetMismatch`、`DescriptorMismatch`、`AcceptedRunCountMismatch`、`AcceptedPixelCountMismatch` の enum error を返す。成功時だけ `FrameBegin`、`SpanWrite`、`FramePresent` の typed effect を返し、last presented frame を更新する。raw memory、actual display driver、host import、long-running scheduler backend、timer queue、present loop、fallback、silent no-op は実装しない。

F5fn では Bare display memory write plan boundary として、`platforms/gui/bare/display_memory` を追加する。これは actual display driver ではなく、F5fm の storage ledger を actual bare display driver が消費できる checked byte write plan へ変換する boundary である。memory state は canonical storage state として `GuiBareDisplayStorageState` を保持し、public value である `GuiBareDisplayStorageStepApplied` / `GuiBareDisplayStorageEffect` をそのまま信用しない。supplied storage step の framebuffer step を canonical storage state から再適用し、expected state / effect と一致しない場合は `StorageStepStateMismatch` / `StorageStepEffectMismatch` で拒否する。

span write では `height * stride_bytes`、`y * stride_bytes`、`x * 4`、`width * 4`、`byte_start`、`byte_end` を checked arithmetic で計算し、negative geometry、overflow、`byte_end > surface_byte_count` は enum error と `Result` で fail-closed に返す。present は storage present effect と descriptor complete count の一致を complete-frame evidence として確認した場合だけ `FramePresent` action へ変換する。raw byte buffer ownership、actual display driver write、host import、long-running scheduler backend、timer queue、present loop、fallback、silent no-op は実装しない。

F5fo では Bare display driver outcome ledger boundary として、`platforms/gui/bare/display_driver` を追加する。これは actual hardware display driver ではなく、F5fn の checked byte write plan と caller supplied driver outcome を照合する pure ledger boundary である。driver state は canonical memory state として `GuiBareDisplayMemoryState` を保持し、public value である `GuiBareDisplayMemoryStepApplied` をそのまま信用しない。supplied memory step から storage step を取り出し、canonical memory state で `gui_bare_display_memory_apply` を再適用し、expected state / action と supplied state / action が一致する場合だけ driver outcome を照合する。

driver outcome は `BeginAccepted`、`SpanWriteAccepted`、`FramePresentAccepted`、`DriverRejected` の typed value として表す。`BeginAccepted` は target、descriptor、surface byte count を、`SpanWriteAccepted` は span、run index、pixel start / end、row byte start、x byte offset、byte start / len / end、surface byte count、color を、`FramePresentAccepted` は frame、run count、pixel count、surface byte count を checked action evidence と exact match する。`DriverRejected` は lower `GuiError` を保持して `Result` error として返し、fallback や silent no-op へは変換しない。raw byte buffer ownership、actual hardware write、host import、long-running scheduler backend、timer queue、present loop、Canvas、DOM、minifb、video memory host import は実装しない。

F5fp では Bare display driver host import boundary として、`platforms/gui/bare/display_driver_host_import` を追加する。これは F5fo の pure driver outcome ledger を embedding host の actual display driver import へ接続する境界であり、`display_driver_begin`、`display_driver_span_write`、`display_driver_frame_present` の 3 import を使う。F5fk の `display_presenter_session_*` は scheduler host executor 用の presenter session ABI であり、F5fp は F5fn/F5fo の checked byte plan / outcome ledger に近い raw display driver ABI として分ける。

host import の前には必ず F5fo ledger preflight を通す。preflight が stale / forged memory step、state mismatch、action mismatch、driver phase mismatch を検出した場合、host import は呼ばれない。preflight success の action variant だけを match し、wildcard success や silent no-op branch は持たない。status `0` だけが success outcome に変換され、`-1` は `DriverRejected GuiError::Unsupported`、既知の負値は typed `GuiError`、unknown negative と positive non-zero は `DriverRejected GuiError::BackendFailure` になる。

`display_driver_span_write` には、span の x / y / width / height に加え、F5fn で計算済みの run index、pixel start / end、row byte start、x byte offset、byte_start、byte_len、byte_end、surface byte count、RGBA8888 color を渡す。success は host accepted status と ledger match の事実であり、raw byte buffer を読み戻した証明ではない。`nodesrc/run_test.js` の default stub は `display_driver_*` を `-1` にし、CLI-only / doctest host では Unsupported を explicit `Result` として返す。raw memory ownership、polling input、long-running backend loop、timer queue、present loop、Canvas、DOM、minifb、video memory host import、fallback、silent no-op は後続 slice とする。

F5fq では Bare display driver byte echo verification boundary として、`platforms/gui/bare/display_driver_byte_echo` を追加する。これは F5fp accepted status の後に host が返す単一 byte echo を、F5fn/F5fo の checked byte range と RGBA8888 color evidence に照合する pure verification boundary である。public `GuiBareDisplayDriverStepApplied` は Copy value として偽造できるため、F5fq public entry は supplied driver step を受け取らず、`GuiBareDisplayDriverState`、`GuiBareDisplayMemoryStepApplied`、`GuiBareDisplayDriverOutcome`、`GuiBareDisplayDriverByteEcho` から内部で `gui_bare_display_driver_apply` を必ず呼ぶ。

F5fo ledger が stale / forged memory step や outcome mismatch を検出した場合、F5fq は `DriverStepInvalid %GuiBareDisplayDriverErrorKind` として lower category を保持する。ledger success 後も `SpanWrite` / `SpanWriteAccepted` 以外は `NonSpanWriteAction` / `NonSpanWriteOutcome` で拒否する。byte index は accepted span の `byte_start <= index < byte_end` を満たし、relative offset `0..3` は `Red` / `Green` / `Blue` / `Alpha` の typed channel enum に写される。echo value は 0..255 でなければならず、expected channel value と一致しない場合は `EchoValueMismatch` になる。F5fq は単一 byte echo verification までであり、raw display memory ownership、bulk byte readback、actual driver adapter、fallback、silent no-op は後続 slice とする。

F5fr では Bare raw display memory ownership boundary として、`platforms/gui/bare/display_memory_owner` を追加する。これは F5fq の single-byte verification を bare display backend が所有する raw RGBA8888 memory に反映する boundary である。`GuiBareDisplayMemoryOwner` は canonical `GuiBareDisplayDriverState`、surface byte count、private `RegionToken u8`、verified byte count、last verified byte evidence を保持する。`GuiBareDisplayDriverByteEchoVerified` は Copy value として偽造できるため、public write API の権威にはしない。

F5fr の `gui_bare_display_memory_owner_write_echo` は owner 内の `GuiBareDisplayDriverState` と caller supplied `GuiBareDisplayMemoryStepApplied` / `GuiBareDisplayDriverOutcome` / `GuiBareDisplayDriverByteEcho` から F5fq verification を再実行する。成功した場合だけ exact echoed byte を owner 内の `RegionToken u8` に store し、同じ byte を load して expected value と一致することを確認してから owner の driver state と verified byte count を進める。store 前や readback 前に state を進めることは禁止する。

F5fr は single-byte owner boundary であり、span 全体や frame 全体の readiness は主張しない。write failure は owner-bearing error として input owner を保持し、caller が recovery / free / retry policy を選ぶ。owner と owner-bearing write error は Clone / Copy を実装しない。actual hardware driver、host import、scheduler loop、timer queue、DOM、Canvas、minifb、video memory transport、zero-fill fallback、silent no-op、bulk byte readback は後続 slice とする。
