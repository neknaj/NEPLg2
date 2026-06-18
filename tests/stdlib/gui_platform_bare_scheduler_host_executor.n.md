# GUI platform bare scheduler host executor doctests

このファイルは、F5ex の Bare scheduler host action executor backend bridge の public import surface を固定する。

F5ex bare scheduler host executor は typed `ExecuteHostAction` だけを受ける backend bridge であり、not long-running scheduler backend ではない。borrowed accessor で pending span operation を読み、embedding host import の begin / run / end のいずれかを 1 回だけ呼び、status を `Result unit GuiError` に変換する。その outcome を F5ev input helper に渡し、F5ew one-step bridge で F5ek に戻す。embedding host が executor ABI を提供しない場合は `Unsupported` を `Result` として返す。general `LoopAction`、action sink / driver、queue、while loop、timer wait、present loop、minifb、Canvas、DOM、video memory、fallback、silent no-op は実装しない。source policy は `nodesrc/test_web_gui_font_rendering_contract.js` が検査する。

source policy labels:

- platform_bare_scheduler_host_executor_facade_ok
- platform_bare_scheduler_host_executor_backend_boundary_ok
- platform_bare_scheduler_host_executor_host_import_status_ok
- platform_bare_scheduler_host_executor_borrowed_operation_ok
- platform_bare_scheduler_host_executor_reuses_f5ev_f5ew_ok
- platform_bare_scheduler_host_executor_no_loop_queue_fallback

## import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_scheduler_host_executor\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"bare scheduler host executor import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "platforms/gui/bare/scheduler_host_executor" as *
#import "std/test" as test

// platform_bare_scheduler_host_executor_facade_ok
// platform_bare_scheduler_host_executor_backend_boundary_ok
// platform_bare_scheduler_host_executor_host_import_status_ok
// platform_bare_scheduler_host_executor_borrowed_operation_ok
// platform_bare_scheduler_host_executor_reuses_f5ev_f5ew_ok
// platform_bare_scheduler_host_executor_no_loop_queue_fallback

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_platform_bare_scheduler_host_executor"
        |> test::test_report_push test::assert_eq_i32 "bare scheduler host executor import" 0 0
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
