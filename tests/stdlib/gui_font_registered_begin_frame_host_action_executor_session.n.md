# GUI font registered BeginFrame host action executor session

このfixtureはactual registered F5nzj authorityを分断せず、F5nh executor session request pendingへ接続するF5nzk契約を検証する。

neplg2:test[stdio, normalize_newlines]

```neplg2
---
stdout: "test_report name=\"gui_font_registered_begin_frame_host_action_executor_session\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"BeginFrame enters executor session request pending\" expected=\"255\" actual=\"255\" message=\"\"\n"
---
#entry main
#indent 4
#target std
#import "std/test" as * with tests
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_executor_session_test" as * with tests
fn main %impure fn void i32 \void:
    let result %i32 gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_action_executor_session_test_success_contract unit
    let report %TestReport test_report_new "gui_font_registered_begin_frame_host_action_executor_session"
    let report1 %TestReport test_report_push report (assert_eq_i32 "BeginFrame enters executor session request pending" 255 result)
    test_report_exit_code test_report_print_stdout report1
```
