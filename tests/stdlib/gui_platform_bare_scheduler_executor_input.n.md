# GUI platform bare scheduler executor input doctests

このファイルは、F5ev の Bare scheduler executor outcome input helper の public import surface を固定する。

F5ev bare scheduler executor input は backend-facing input boundary であり、not long-running scheduler backend ではない。F5eg `ExecuteHostAction` typed payload と caller supplied `Result unit GuiError` outcome を受け、F5ek `RealLoopStepInput::ExecutorOutcome` に total packaging して original action と一緒に保持する。general `LoopAction`、`YieldToClock`、`AwaitTimerAdvance`、`Complete` は対象外で、`ClockDelta` や `CompleteAck` は合成しない。helper は does not return Result であり、unsupported path は型で除外される。F5ei executor complete、F5ek real loop step、action sink / driver、support validation、timer、sleep、queue、while loop、present、minifb、Canvas、DOM、video memory、fallback、silent no-op は実装しない。source policy は `nodesrc/test_web_gui_font_rendering_contract.js` が検査する。

source policy labels:

- platform_bare_scheduler_executor_input_facade_ok
- platform_bare_scheduler_executor_input_backend_boundary_ok
- platform_bare_scheduler_executor_input_execute_only_ok
- platform_bare_scheduler_executor_input_total_packaging_ok
- platform_bare_scheduler_executor_input_preserves_outcome_ok
- platform_bare_scheduler_executor_input_no_executor_complete_backend_queue_fallback

## import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_scheduler_executor_input\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"bare scheduler executor input import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "platforms/gui/bare/scheduler_executor_input" as *
#import "std/test" as test

// platform_bare_scheduler_executor_input_facade_ok
// platform_bare_scheduler_executor_input_backend_boundary_ok
// platform_bare_scheduler_executor_input_execute_only_ok
// platform_bare_scheduler_executor_input_total_packaging_ok
// platform_bare_scheduler_executor_input_preserves_outcome_ok
// platform_bare_scheduler_executor_input_no_executor_complete_backend_queue_fallback

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_platform_bare_scheduler_executor_input"
        |> test::test_report_push test::assert_eq_i32 "bare scheduler executor input import" 0 0
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
