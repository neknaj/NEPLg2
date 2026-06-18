# GUI platform bare clock doctests

このファイルは、F5es の Bare formal monotonic clock source backend boundary の public import surface を固定する。

F5es は bare host import `nepl_gui_bare.monotonic_clock_ms` から i32 millisecond sample を受け、negative sentinel を `GuiError` に写してから F5eo backend clock sample constructor を通す。bare には universal wall clock がないため、doctest runtime stub は import surface のためだけに -1 を返し、`Unsupported` を明示的に確認する。timer、sleep、queue、window loop、rendering API、stdout protocol、fallback、silent no-op、wrap、clamp は使わない。source policy は `nodesrc/test_web_gui_font_rendering_contract.js` が検査する。

source policy labels:

- platform_bare_clock_facade_ok
- platform_bare_clock_import_ok
- platform_bare_clock_sample_constructor_bridge_ok
- platform_bare_clock_negative_sentinel_result_ok
- platform_bare_clock_no_timer_queue_fallback
- bare_runner_clock_unsupported_default_ok

## import and unsupported smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_clock\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"bare backend clock unsupported\" expected=\"-1\" actual=\"-1\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/gui/error" as *
#import "core/result" as *
#import "platforms/gui/bare/clock" as *
#import "std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock" as *
#import "std/test" as test

// platform_bare_clock_facade_ok
// platform_bare_clock_import_ok
// platform_bare_clock_sample_constructor_bridge_ok
// platform_bare_clock_negative_sentinel_result_ok
// platform_bare_clock_no_timer_queue_fallback
// bare_runner_clock_unsupported_default_ok

fn main %impure fn void i32 \void:
    let actual %i32:
        match gui_bare_backend_clock_sample:
            Result::Ok sample:
                gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock_sample_monotonic_ms &sample
            Result::Err error:
                if gui_error_is_unsupported error:
                    then -1
                    else -2
    let report:
        test::test_report_new "gui_platform_bare_clock"
        |> test::test_report_push test::assert_eq_i32 "bare backend clock unsupported" -1 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
