# GUI platform native clock doctests

このファイルは、F5er の Native formal monotonic clock source backend boundary の public import surface を固定する。

F5er は native host import `nepl_gui_native.monotonic_clock_ms` から i32 millisecond sample を受け、negative sentinel を `GuiError` に写してから F5eo backend clock sample constructor を通す。actual native host source は Rust `nepl-gui-native` 側の `Instant` helper が担当し、doctest runtime stub は import surface のためだけに 0 を返す。timer、sleep、queue、window loop、rendering API、stdout protocol、fallback、silent no-op、wrap、clamp は使わない。source policy は `nodesrc/test_web_gui_font_rendering_contract.js` と `nodesrc/test_native_gui_platform_behavior.js` が検査する。

source policy labels:

- platform_native_clock_facade_ok
- platform_native_clock_import_ok
- platform_native_clock_sample_constructor_bridge_ok
- platform_native_clock_negative_sentinel_result_ok
- platform_native_clock_no_timer_queue_fallback
- native_runner_clock_instant_i32_guard_ok

## import and host sample smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_native_clock\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"native backend clock sample\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/gui/error" as *
#import "core/result" as *
#import "platforms/gui/native/clock" as *
#import "std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock" as *
#import "std/test" as test

// platform_native_clock_facade_ok
// platform_native_clock_import_ok
// platform_native_clock_sample_constructor_bridge_ok
// platform_native_clock_negative_sentinel_result_ok
// platform_native_clock_no_timer_queue_fallback
// native_runner_clock_instant_i32_guard_ok

fn main %impure fn void i32 \void:
    let actual %i32:
        match gui_native_backend_clock_sample:
            Result::Ok sample:
                gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock_sample_monotonic_ms &sample
            Result::Err error:
                if gui_error_is_unsupported error:
                    then 1
                    else 2
    let report:
        test::test_report_new "gui_platform_native_clock"
        |> test::test_report_push test::assert_eq_i32 "native backend clock sample" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
