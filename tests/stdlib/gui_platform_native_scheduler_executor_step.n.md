# GUI platform native scheduler executor step doctests

このファイルは、F5ew の Native scheduler executor one-step bridge の public import surface を固定する。

F5ew native scheduler executor step は backend-facing one-step bridge であり、not long-running scheduler backend ではない。F5ev `GuiNativeSchedulerExecutorInputReady` と borrowed F5ek real loop step policy を受け、ready payload の original `ExecuteHostAction` と packaged `RealLoopStepInput::ExecutorOutcome` を使って F5ek `real_loop_step` を 1 回だけ呼ぶ。general `LoopAction`、support validation、action sink / driver、queue、while loop、clock / timer helper、present、minifb、Canvas、DOM、video memory、fallback、silent no-op は実装しない。source policy は `nodesrc/test_web_gui_font_rendering_contract.js` が検査する。

source policy labels:

- platform_native_scheduler_executor_step_facade_ok
- platform_native_scheduler_executor_step_backend_boundary_ok
- platform_native_scheduler_executor_step_ready_payload_ok
- platform_native_scheduler_executor_step_calls_real_loop_step_once_ok
- platform_native_scheduler_executor_step_returns_lower_result_ok
- platform_native_scheduler_executor_step_no_backend_queue_timer_fallback

## import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_native_scheduler_executor_step\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"native scheduler executor step import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "platforms/gui/native/scheduler_executor_step" as *
#import "std/test" as test

// platform_native_scheduler_executor_step_facade_ok
// platform_native_scheduler_executor_step_backend_boundary_ok
// platform_native_scheduler_executor_step_ready_payload_ok
// platform_native_scheduler_executor_step_calls_real_loop_step_once_ok
// platform_native_scheduler_executor_step_returns_lower_result_ok
// platform_native_scheduler_executor_step_no_backend_queue_timer_fallback

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_platform_native_scheduler_executor_step"
        |> test::test_report_push test::assert_eq_i32 "native scheduler executor step import" 0 0
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
