# GUI platform bare scheduler real-loop step doctests

このファイルは、F5ga の Bare scheduler real-loop action step boundary の public import surface を固定する。

F5ga は F5el `NeedInput` を bare clock / display presenter input に 1 step だけ接続する bare-only boundary である。F5ek success 後は F5el `real_loop_driver_after_step` まで 1 回だけ進め、次の `NeedInput` または terminal `Completed` を返す。long-running loop、queue、timer wait、direct host import、present loop、DOM、Canvas、minifb、video memory、raw display state、fallback、silent no-op は実装しない。

executable labels:

- platform_bare_scheduler_real_loop_step_facade_ok
- platform_bare_scheduler_real_loop_step_import_ok

source policy only labels:

- platform_bare_scheduler_real_loop_step_policy_fields_ok
- platform_bare_scheduler_real_loop_step_need_input_owner_authority_ok
- platform_bare_scheduler_real_loop_step_clock_helpers_once_ok
- platform_bare_scheduler_real_loop_step_presenter_input_once_ok
- platform_bare_scheduler_real_loop_step_executor_ready_path_ok
- platform_bare_scheduler_real_loop_step_missing_category_direct_ok
- platform_bare_scheduler_real_loop_step_owner_recovery_ok
- platform_bare_scheduler_real_loop_step_complete_ack_only_ok
- platform_bare_scheduler_real_loop_step_f5ek_f5el_dispatch_ok
- platform_bare_scheduler_real_loop_step_no_raw_queue_fallback
- platform_bare_scheduler_real_loop_step_no_clone_copy_owner_values

## import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_scheduler_real_loop_step\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"bare scheduler real loop step import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "platforms/gui/bare/scheduler_real_loop_step" as *
#import "std/test" as test

// platform_bare_scheduler_real_loop_step_facade_ok
// platform_bare_scheduler_real_loop_step_import_ok
// platform_bare_scheduler_real_loop_step_policy_fields_ok
// platform_bare_scheduler_real_loop_step_need_input_owner_authority_ok
// platform_bare_scheduler_real_loop_step_clock_helpers_once_ok
// platform_bare_scheduler_real_loop_step_presenter_input_once_ok
// platform_bare_scheduler_real_loop_step_executor_ready_path_ok
// platform_bare_scheduler_real_loop_step_missing_category_direct_ok
// platform_bare_scheduler_real_loop_step_owner_recovery_ok
// platform_bare_scheduler_real_loop_step_complete_ack_only_ok
// platform_bare_scheduler_real_loop_step_f5ek_f5el_dispatch_ok
// platform_bare_scheduler_real_loop_step_no_raw_queue_fallback
// platform_bare_scheduler_real_loop_step_no_clone_copy_owner_values

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_platform_bare_scheduler_real_loop_step"
        |> test::test_report_push test::assert_eq_i32 "bare scheduler real loop step import" 0 0
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
