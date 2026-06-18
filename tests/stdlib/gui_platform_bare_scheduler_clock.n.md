# GUI platform bare scheduler clock doctests

このファイルは、F5et の Bare scheduler clock one-tick helper の public import surface を固定する。

F5et bare scheduler clock は long-running scheduler backend ではない。bare host が明示提供する clock sample を F5eo `backend_clock_start` / `backend_clock_advance` に渡すだけの one-tick helper である。新しい scheduler policy は作らず、policy は F5eo `BackendClockPolicy` そのものを受ける。既定 doctest host は bare clock unsupported を返すため、start と tick の sample failure が `GuiError::Unsupported` として現れることを確認する。timer、sleep、queue、while loop、present、minifb、Canvas、video memory、fallback、silent no-op は実装しない。source policy は `nodesrc/test_web_gui_font_rendering_contract.js` が検査する。

source policy labels:

- platform_bare_scheduler_clock_facade_ok
- platform_bare_scheduler_clock_one_tick_helper_ok
- platform_bare_scheduler_clock_f5eo_policy_ok
- platform_bare_scheduler_clock_start_sample_error_recovers_policy_ok
- platform_bare_scheduler_clock_tick_sample_error_recovers_policy_state_ok
- platform_bare_scheduler_clock_start_delegates_f5eo_ok
- platform_bare_scheduler_clock_tick_delegates_f5eo_ok
- platform_bare_scheduler_clock_no_loop_queue_timer_present_fallback

## unsupported start and tick

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_scheduler_clock\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"bare scheduler clock start unsupported\" expected=\"-1\" actual=\"-1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"bare scheduler clock tick unsupported\" expected=\"-2\" actual=\"-2\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/gui/error" as *
#import "core/result" as *
#import "platforms/gui/bare/scheduler_clock" as *
#import "std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock" as *
#import "std/test" as test

// platform_bare_scheduler_clock_facade_ok
// platform_bare_scheduler_clock_one_tick_helper_ok
// platform_bare_scheduler_clock_f5eo_policy_ok
// platform_bare_scheduler_clock_start_sample_error_recovers_policy_ok
// platform_bare_scheduler_clock_tick_sample_error_recovers_policy_state_ok
// platform_bare_scheduler_clock_start_delegates_f5eo_ok
// platform_bare_scheduler_clock_tick_delegates_f5eo_ok
// platform_bare_scheduler_clock_no_loop_queue_timer_present_fallback

fn bare_start_actual %impure fn void i32 \void:
    match gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock_policy 16:
        Result::Err kind:
            -10
        Result::Ok policy:
            match gui_bare_scheduler_clock_start policy:
                Result::Ok state:
                    1
                Result::Err error:
                    match error:
                        GuiBareSchedulerClockError::StartSampleFailed failure:
                            let gui_error %GuiError gui_bare_scheduler_clock_start_sample_failed_error &failure
                            if gui_error_is_unsupported gui_error:
                                then -1
                                else -2
                        GuiBareSchedulerClockError::StartBackendClockFailed failure:
                            -3
                        GuiBareSchedulerClockError::TickSampleFailed failure:
                            -4
                        GuiBareSchedulerClockError::TickBackendClockFailed failure:
                            -5

fn bare_tick_actual %impure fn void i32 \void:
    match gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock_sample 0:
        Result::Err kind:
            -10
        Result::Ok sample:
            match gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock_start sample:
                Result::Err lower:
                    -20
                Result::Ok state:
                    match gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock_policy 16:
                        Result::Err kind:
                            -30
                        Result::Ok policy:
                            match gui_bare_scheduler_clock_tick policy state:
                                Result::Ok advance:
                                    2
                                Result::Err error:
                                    match error:
                                        GuiBareSchedulerClockError::StartSampleFailed failure:
                                            -3
                                        GuiBareSchedulerClockError::StartBackendClockFailed failure:
                                            -4
                                        GuiBareSchedulerClockError::TickSampleFailed failure:
                                            let gui_error %GuiError gui_bare_scheduler_clock_tick_sample_failed_error &failure
                                            if gui_error_is_unsupported gui_error:
                                                then -2
                                                else -5
                                        GuiBareSchedulerClockError::TickBackendClockFailed failure:
                                            -6

fn main %impure fn void i32 \void:
    let start_actual %i32 bare_start_actual
    let tick_actual %i32 bare_tick_actual
    let report:
        test::test_report_new "gui_platform_bare_scheduler_clock"
        |> test::test_report_push test::assert_eq_i32 "bare scheduler clock start unsupported" -1 start_actual
        |> test::test_report_push test::assert_eq_i32 "bare scheduler clock tick unsupported" -2 tick_actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
