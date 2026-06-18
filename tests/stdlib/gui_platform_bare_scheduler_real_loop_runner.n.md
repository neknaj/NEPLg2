# GUI platform bare scheduler real-loop runner doctests

このファイルは、F5gb の Bare scheduler bounded real-loop runner boundary の public import surface を固定する。

F5gb は F5el start から F5ga `gui_bare_scheduler_real_loop_step` を bounded に繰り返す bare runner である。policy は F5ga step policy と `max_step_count` だけを保持し、F5el driver policy は F5ga accessor から借用する。`max_step_count < 0` は typed `PolicyInvalid` として original owner を返し、F5ga step failure は `gui_bare_scheduler_real_loop_step_error_owner` から owner を回収する。queue、sleep、timer wait、direct host import、present loop、DOM、Canvas、minifb、video memory、fallback、silent no-op は実装しない。

executable labels:

- platform_bare_scheduler_real_loop_runner_facade_ok
- platform_bare_scheduler_real_loop_runner_import_ok

source policy only labels:

- platform_bare_scheduler_real_loop_runner_policy_shape_ok
- platform_bare_scheduler_real_loop_runner_start_authority_ok
- platform_bare_scheduler_real_loop_runner_budget_zero_start_ok
- platform_bare_scheduler_real_loop_runner_owner_recovery_ok
- platform_bare_scheduler_real_loop_runner_bounded_recursion_ok
- platform_bare_scheduler_real_loop_runner_no_raw_queue_fallback
- platform_bare_scheduler_real_loop_runner_no_clone_copy_owner_values

## import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_scheduler_real_loop_runner\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"bare scheduler real loop runner import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "platforms/gui/bare/scheduler_real_loop_runner" as *
#import "std/test" as test

// platform_bare_scheduler_real_loop_runner_facade_ok
// platform_bare_scheduler_real_loop_runner_import_ok
// platform_bare_scheduler_real_loop_runner_policy_shape_ok
// platform_bare_scheduler_real_loop_runner_start_authority_ok
// platform_bare_scheduler_real_loop_runner_budget_zero_start_ok
// platform_bare_scheduler_real_loop_runner_owner_recovery_ok
// platform_bare_scheduler_real_loop_runner_bounded_recursion_ok
// platform_bare_scheduler_real_loop_runner_no_raw_queue_fallback
// platform_bare_scheduler_real_loop_runner_no_clone_copy_owner_values

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_platform_bare_scheduler_real_loop_runner"
        |> test::test_report_push test::assert_eq_i32 "bare scheduler real loop runner import" 0 0
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
