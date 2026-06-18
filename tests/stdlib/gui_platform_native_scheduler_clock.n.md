# GUI platform native scheduler clock doctests

このファイルは、F5et の Native scheduler clock one-tick helper の public import surface を固定する。

F5et native scheduler clock は long-running scheduler backend ではない。native clock source から取得した sample を F5eo `backend_clock_start` / `backend_clock_advance` に渡すだけの one-tick helper である。新しい scheduler policy は作らず、policy は F5eo `BackendClockPolicy` そのものを受ける。`ClockDelta` を直接合成せず、F5eo `BackendClockAdvance` を返す。timer、sleep、queue、while loop、present、minifb、Canvas、video memory、fallback、silent no-op は実装しない。source policy は `nodesrc/test_web_gui_font_rendering_contract.js` が検査する。

source policy labels:

- platform_native_scheduler_clock_facade_ok
- platform_native_scheduler_clock_one_tick_helper_ok
- platform_native_scheduler_clock_f5eo_policy_ok
- platform_native_scheduler_clock_start_sample_error_recovers_policy_ok
- platform_native_scheduler_clock_tick_sample_error_recovers_policy_state_ok
- platform_native_scheduler_clock_start_delegates_f5eo_ok
- platform_native_scheduler_clock_tick_delegates_f5eo_ok
- platform_native_scheduler_clock_no_loop_queue_timer_present_fallback

## start then zero delta tick

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_native_scheduler_clock\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"native scheduler clock zero delta\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "platforms/gui/native/scheduler_clock" as *
#import "std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock" as *
#import "std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step" as *
#import "std/test" as test

// platform_native_scheduler_clock_facade_ok
// platform_native_scheduler_clock_one_tick_helper_ok
// platform_native_scheduler_clock_f5eo_policy_ok
// platform_native_scheduler_clock_start_sample_error_recovers_policy_ok
// platform_native_scheduler_clock_tick_sample_error_recovers_policy_state_ok
// platform_native_scheduler_clock_start_delegates_f5eo_ok
// platform_native_scheduler_clock_tick_delegates_f5eo_ok
// platform_native_scheduler_clock_no_loop_queue_timer_present_fallback

fn main %impure fn void i32 \void:
    let actual %i32:
        match gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock_policy 16:
            Result::Err kind:
                -10
            Result::Ok start_policy:
                match gui_native_scheduler_clock_start start_policy:
                    Result::Err error:
                        -20
                    Result::Ok state:
                        match gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock_policy 16:
                            Result::Err kind:
                                -30
                            Result::Ok tick_policy:
                                match gui_native_scheduler_clock_tick tick_policy state:
                                    Result::Err error:
                                        -40
                                    Result::Ok advance:
                                        let input %GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerRealLoopStepInput gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock_advance_input advance
                                        match input:
                                            GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerRealLoopStepInput::ClockDelta delta:
                                                delta
                                            GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerRealLoopStepInput::ExecutorOutcome outcome:
                                                -50
                                            GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerRealLoopStepInput::CompleteAck:
                                                -60
    let report:
        test::test_report_new "gui_platform_native_scheduler_clock"
        |> test::test_report_push test::assert_eq_i32 "native scheduler clock zero delta" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
