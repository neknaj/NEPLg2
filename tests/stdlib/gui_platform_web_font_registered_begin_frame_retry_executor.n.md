# Web registered BeginFrame retry executor

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_web_font_registered_begin_frame_retry_executor\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"actual Web recovered-state abort\" expected=\"63\" actual=\"63\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std
#import "std/test" as * with tests
#import "platforms/gui/web/font_registered_begin_frame_retry_executor_test" as * with tests
fn main %impure fn void i32 \void:
    let evidence %i32 gui_font_web_registered_begin_frame_retry_executor_test_unsupported_contract unit
    let report %TestReport test_report_new "gui_platform_web_font_registered_begin_frame_retry_executor"
    let report1 %TestReport test_report_push report (assert_eq_i32 "actual Web recovered-state abort" 63 evidence)
    test_report_exit_code test_report_print_stdout report1
```
