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
