# GUI platform native scheduler clock input doctests

このファイルは、F5eu の Native scheduler clock action input helper の public import surface を固定する。

F5eu native scheduler clock input は action input helper only であり、not long-running scheduler backend ではない。F5eg `YieldToClock` / `AwaitTimerAdvance` typed payload と F5et native `gui_native_scheduler_clock_tick` を接続し、F5eo 由来の `BackendClockState` と F5ek `RealLoopStepInput` を success payload に保持する。`ExecuteHostAction` と `Complete` は対象外で、`ExecutorOutcome` や `CompleteAck` は合成しない。timer、sleep、queue、while loop、present、minifb、Canvas、DOM、video memory、fallback、silent no-op は実装しない。source policy は `nodesrc/test_web_gui_font_rendering_contract.js` が検査する。

source policy labels:

- platform_native_scheduler_clock_input_facade_ok
- platform_native_scheduler_clock_input_action_input_helper_ok
- platform_native_scheduler_clock_input_yield_timer_only_ok
- platform_native_scheduler_clock_input_success_payload_state_input_ok
- platform_native_scheduler_clock_input_error_recovers_action_state_lower_ok
- platform_native_scheduler_clock_input_tick_once_ok
- platform_native_scheduler_clock_input_no_executor_complete_backend_queue_fallback

## import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_native_scheduler_clock_input\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"native scheduler clock input import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "platforms/gui/native/scheduler_clock_input" as *
#import "std/test" as test

// platform_native_scheduler_clock_input_facade_ok
// platform_native_scheduler_clock_input_action_input_helper_ok
// platform_native_scheduler_clock_input_yield_timer_only_ok
// platform_native_scheduler_clock_input_success_payload_state_input_ok
// platform_native_scheduler_clock_input_error_recovers_action_state_lower_ok
// platform_native_scheduler_clock_input_tick_once_ok
// platform_native_scheduler_clock_input_no_executor_complete_backend_queue_fallback

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_platform_native_scheduler_clock_input"
        |> test::test_report_push test::assert_eq_i32 "native scheduler clock input import" 0 0
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
