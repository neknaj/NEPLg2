# GUI font registered BeginFrame host action Yield resume

このfixtureはactual F5nzl Yieldをscheduler-visible ownerへ分類し、正式F5nc helperによるresume前後のslice counterを検証する。

neplg2:test[stdio, normalize_newlines]

```neplg2
---
stdout: "test_report name=\"gui_font_registered_begin_frame_host_action_yield_resume\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"yield resume authority\" expected=\"63\" actual=\"63\" message=\"\"\n"
---
#entry main
#indent 4
#target std
#import "std/test" as * with tests
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_yield_resume_test" as * with tests
fn main %impure fn void i32 \void:
    let evidence %i32 gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_action_yield_resume_test_contract unit
    let report %TestReport test_report_new "gui_font_registered_begin_frame_host_action_yield_resume"
    test_report_exit_code test_report_print_stdout test_report_push report assert_eq_i32 "yield resume authority" 63 evidence
```
