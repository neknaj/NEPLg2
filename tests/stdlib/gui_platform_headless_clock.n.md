# GUI platform headless clock doctests

このファイルは、F5eq の Headless scripted monotonic clock source backend boundary の public import surface を固定する。

F5eq は deterministic headless / offscreen test 用に fixed slot script から monotonic clock sample を返す。constructor と poll は F5eo backend clock sample constructor を通して sample を検査する。`count` は 0 から 3、`cursor` は 0 から `count`、slot は count に一致する Some / None shape である必要がある。`cursor == count` は `Option::None` を返し、sample を合成しない。wall clock、timer、queue、platform API、fallback、silent no-op は使わない。source policy は `nodesrc/test_web_gui_font_rendering_contract.js` が検査する。

source policy labels:

- platform_headless_clock_facade_ok
- platform_headless_clock_fixed_script_shape_ok
- platform_headless_clock_constructor_sample_validation_ok
- platform_headless_clock_poll_shape_validation_ok
- platform_headless_clock_poll_end_none_ok
- platform_headless_clock_no_timer_queue_fallback

## fixed script poll smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_headless_clock\" count=8 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"first sample\" expected=\"12\" actual=\"12\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"cursor after sample\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"end after sample\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"empty script no sample\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"negative sample rejected\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"forged current sample rejected\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"forged consumed sample rejected\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"forged end sample rejected\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/option" as *
#import "core/result" as *
#import "core/math" as *
#import "platforms/gui/headless/clock" as *
#import "std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock" as *
#import "std/test" as test

// platform_headless_clock_facade_ok
// platform_headless_clock_fixed_script_shape_ok
// platform_headless_clock_constructor_sample_validation_ok
// platform_headless_clock_poll_shape_validation_ok
// platform_headless_clock_poll_end_none_ok
// platform_headless_clock_no_timer_queue_fallback

fn first_sample_ms %fn void i32 \void:
    match gui_headless_backend_clock_script_one 12:
        Result::Err kind:
            -1
        Result::Ok script:
            match gui_headless_backend_clock_poll script:
                Result::Err error:
                    -2
                Result::Ok poll:
                    match gui_headless_backend_clock_poll_sample &poll:
                        Option::None:
                            -3
                        Option::Some sample:
                            gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock_sample_monotonic_ms &sample

fn cursor_after_sample %fn void i32 \void:
    match gui_headless_backend_clock_script_one 12:
        Result::Err kind:
            -1
        Result::Ok script:
            match gui_headless_backend_clock_poll script:
                Result::Err error:
                    -2
                Result::Ok poll:
                    let next_script %GuiHeadlessBackendClockScript gui_headless_backend_clock_poll_script poll
                    gui_headless_backend_clock_script_cursor &next_script

fn end_after_sample %fn void i32 \void:
    match gui_headless_backend_clock_script_one 12:
        Result::Err kind:
            -1
        Result::Ok script:
            match gui_headless_backend_clock_poll script:
                Result::Err error:
                    -2
                Result::Ok poll:
                    let next_script %GuiHeadlessBackendClockScript gui_headless_backend_clock_poll_script poll
                    match gui_headless_backend_clock_poll next_script:
                        Result::Err error:
                            -3
                        Result::Ok second_poll:
                            match gui_headless_backend_clock_poll_sample &second_poll:
                                Option::None:
                                    0
                                Option::Some sample:
                                    1

fn empty_script_no_sample %fn void i32 \void:
    let script %GuiHeadlessBackendClockScript gui_headless_backend_clock_script_empty
    match gui_headless_backend_clock_poll script:
        Result::Err error:
            -1
        Result::Ok poll:
            match gui_headless_backend_clock_poll_sample &poll:
                Option::None:
                    0
                Option::Some sample:
                    1

fn negative_sample_rejected %fn void i32 \void:
    let negative_sample %i32 sub 0 1
    match gui_headless_backend_clock_script_one negative_sample:
        Result::Ok script:
            1
        Result::Err kind:
            match kind:
                GuiHeadlessBackendClockScriptErrorKind::CountInvalid:
                    2
                GuiHeadlessBackendClockScriptErrorKind::CursorInvalid:
                    3
                GuiHeadlessBackendClockScriptErrorKind::SlotShapeInvalid:
                    4
                GuiHeadlessBackendClockScriptErrorKind::SampleInvalid:
                    0

fn forged_script_error_code %fn GuiHeadlessBackendClockScript i32 \script:
    match gui_headless_backend_clock_poll script:
        Result::Ok poll:
            1
        Result::Err error:
            match error:
                GuiHeadlessBackendClockError::ScriptInvalid payload:
                    match gui_headless_backend_clock_script_invalid_kind &payload:
                        GuiHeadlessBackendClockScriptErrorKind::CountInvalid:
                            2
                        GuiHeadlessBackendClockScriptErrorKind::CursorInvalid:
                            3
                        GuiHeadlessBackendClockScriptErrorKind::SlotShapeInvalid:
                            4
                        GuiHeadlessBackendClockScriptErrorKind::SampleInvalid:
                            0

fn forged_current_sample_rejected %fn void i32 \void:
    let negative_sample %i32 sub 0 1
    let forged_sample %GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerBackendClockSample GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerBackendClockSample negative_sample
    let first_slot %Option GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerBackendClockSample some forged_sample
    let script %GuiHeadlessBackendClockScript GuiHeadlessBackendClockScript first_slot none none 1 0
    forged_script_error_code script

fn forged_consumed_sample_rejected %fn void i32 \void:
    let negative_sample %i32 sub 0 1
    let forged_sample %GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerBackendClockSample GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerBackendClockSample negative_sample
    let valid_sample %GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerBackendClockSample GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerBackendClockSample 20
    let first_slot %Option GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerBackendClockSample some forged_sample
    let second_slot %Option GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerBackendClockSample some valid_sample
    let script %GuiHeadlessBackendClockScript GuiHeadlessBackendClockScript first_slot second_slot none 2 1
    forged_script_error_code script

fn forged_end_sample_rejected %fn void i32 \void:
    let negative_sample %i32 sub 0 1
    let forged_sample %GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerBackendClockSample GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerBackendClockSample negative_sample
    let first_slot %Option GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerBackendClockSample some forged_sample
    let script %GuiHeadlessBackendClockScript GuiHeadlessBackendClockScript first_slot none none 1 1
    forged_script_error_code script

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_platform_headless_clock"
        |> test::test_report_push test::assert_eq_i32 "first sample" 12 first_sample_ms
        |> test::test_report_push test::assert_eq_i32 "cursor after sample" 1 cursor_after_sample
        |> test::test_report_push test::assert_eq_i32 "end after sample" 0 end_after_sample
        |> test::test_report_push test::assert_eq_i32 "empty script no sample" 0 empty_script_no_sample
        |> test::test_report_push test::assert_eq_i32 "negative sample rejected" 0 negative_sample_rejected
        |> test::test_report_push test::assert_eq_i32 "forged current sample rejected" 0 forged_current_sample_rejected
        |> test::test_report_push test::assert_eq_i32 "forged consumed sample rejected" 0 forged_consumed_sample_rejected
        |> test::test_report_push test::assert_eq_i32 "forged end sample rejected" 0 forged_end_sample_rejected
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
