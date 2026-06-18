# GUI platform native presenter input doctests

このファイルは、F5fg の Native presenter operation identity input boundary の public import surface を固定する。

F5fg native presenter input は presenter-facing input boundary であり、F5ev scheduler step input boundary の別名ではない。typed `ExecuteHostAction` から borrowed accessor で pending span operation identity を先に取り出し、`WindowBegin` / `WindowRunSpan` / `WindowEnd` / `OffscreenBegin` / `OffscreenRunSpan` / `OffscreenEnd` / `DeviceBegin` / `DeviceRunSpan` / `DeviceEnd` を保つ typed value と、F5ev が作る scheduler completion ready payload を同じ ready value に保持する。general `LoopAction`、backend execution、raw status mapping、scheduler step、timer、queue、window loop、minifb、Canvas、DOM、video memory、fallback、silent no-op は実装しない。source policy は `nodesrc/test_web_gui_font_rendering_contract.js` が検査する。

source policy labels:

- platform_native_presenter_input_facade_ok
- platform_native_presenter_input_presenter_boundary_ok
- platform_native_presenter_input_execute_only_ok
- platform_native_presenter_input_operation_identity_ok
- platform_native_presenter_input_reuses_f5ev_ok
- platform_native_presenter_input_no_backend_queue_fallback

## import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_native_presenter_input\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"native presenter input import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "platforms/gui/native/presenter_input" as *
#import "std/test" as test

// platform_native_presenter_input_facade_ok
// platform_native_presenter_input_presenter_boundary_ok
// platform_native_presenter_input_execute_only_ok
// platform_native_presenter_input_operation_identity_ok
// platform_native_presenter_input_reuses_f5ev_ok
// platform_native_presenter_input_no_backend_queue_fallback

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_platform_native_presenter_input"
        |> test::test_report_push test::assert_eq_i32 "native presenter input import" 0 0
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
