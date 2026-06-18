# GUI platform Web clock doctests

このファイルは、F5ep の Web formal monotonic clock source backend boundary の public import surface を固定する。

F5ep は Web host import `nepl_gui_web.monotonic_clock_ms` から i32 millisecond sample を受け、negative sentinel を `GuiError` に写してから F5eo backend clock sample constructor を通す。`performance.now` が使えない場合は `Unsupported`、非有限値や `i32::MAX` 超過は `BackendFailure` であり、`Date.now`、timer API、stdout protocol、polling loop、queue、fallback、silent no-op、wrap、clamp は使わない。source policy は `nodesrc/test_web_gui_font_rendering_contract.js` が検査する。ここでは doctest runtime の test host import が返す sample を使って public call surface を実行する。

source policy labels:

- platform_web_clock_facade_ok
- platform_web_clock_import_ok
- platform_web_clock_sample_constructor_bridge_ok
- platform_web_clock_negative_sentinel_result_ok
- platform_web_clock_no_date_timer_queue_fallback
- worker_web_clock_performance_now_i32_guard_ok

## import and host sample smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_web_clock\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"web backend clock sample\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/gui/error" as *
#import "core/result" as *
#import "platforms/gui/web/clock" as *
#import "std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock" as *
#import "std/test" as test

// platform_web_clock_facade_ok
// platform_web_clock_import_ok
// platform_web_clock_sample_constructor_bridge_ok
// platform_web_clock_negative_sentinel_result_ok
// platform_web_clock_no_date_timer_queue_fallback
// worker_web_clock_performance_now_i32_guard_ok

fn main %impure fn void i32 \void:
    let actual %i32:
        match gui_web_backend_clock_sample:
            Result::Ok sample:
                gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock_sample_monotonic_ms &sample
            Result::Err error:
                if gui_error_is_unsupported error:
                    then 1
                    else 2
    let report:
        test::test_report_new "gui_platform_web_clock"
        |> test::test_report_push test::assert_eq_i32 "web backend clock sample" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
