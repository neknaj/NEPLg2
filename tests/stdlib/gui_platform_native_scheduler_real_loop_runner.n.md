# GUI platform native scheduler real-loop runner doctests

このファイルは、F5gb の Native scheduler bounded real-loop runner boundary の public import surface を固定する。

F5gb は F5el start から F5fz `gui_native_scheduler_real_loop_step` を bounded に繰り返す native runner である。policy は F5fz step policy と `max_step_count` だけを保持し、F5el driver policy は F5fz accessor から借用する。`max_step_count < 0` は typed `PolicyInvalid`、`max_step_count == 0` は start 後の `NeedInput` を `BudgetExhausted` とする。queue、sleep、timer wait、DOM、Canvas、minifb、video memory、fallback、silent no-op は実装しない。

executable labels:

- platform_native_scheduler_real_loop_runner_facade_ok
- platform_native_scheduler_real_loop_runner_import_ok

source policy only labels:

- platform_native_scheduler_real_loop_runner_policy_shape_ok
- platform_native_scheduler_real_loop_runner_start_authority_ok
- platform_native_scheduler_real_loop_runner_budget_zero_start_ok
- platform_native_scheduler_real_loop_runner_step_clock_state_error_ok
- platform_native_scheduler_real_loop_runner_bounded_recursion_ok
- platform_native_scheduler_real_loop_runner_no_queue_fallback
- platform_native_scheduler_real_loop_runner_no_clone_copy_runner_values

## import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_native_scheduler_real_loop_runner\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"native scheduler real loop runner import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "platforms/gui/native/scheduler_real_loop_runner" as *
#import "std/test" as test

// platform_native_scheduler_real_loop_runner_facade_ok
// platform_native_scheduler_real_loop_runner_import_ok
// platform_native_scheduler_real_loop_runner_policy_shape_ok
// platform_native_scheduler_real_loop_runner_start_authority_ok
// platform_native_scheduler_real_loop_runner_budget_zero_start_ok
// platform_native_scheduler_real_loop_runner_step_clock_state_error_ok
// platform_native_scheduler_real_loop_runner_bounded_recursion_ok
// platform_native_scheduler_real_loop_runner_no_queue_fallback
// platform_native_scheduler_real_loop_runner_no_clone_copy_runner_values

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_platform_native_scheduler_real_loop_runner"
        |> test::test_report_push test::assert_eq_i32 "native scheduler real loop runner import" 0 0
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
