# GUI font registered BeginFrame host action completion

このfixtureはcaller-supplied outcomeだけをF5nhへ渡し、success/error双方でregistered continuationを保持するF5nzl契約を検証する。

neplg2:test[stdio, normalize_newlines]

```neplg2
---
stdout: "test_report name=\"gui_font_registered_begin_frame_host_action_completion\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"success rejoins continuation\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"unsupported rejoins continuation\" expected=\"60\" actual=\"60\" message=\"\"\n"
---
#entry main
#indent 4
#target std
#import "std/test" as * with tests
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_completion_test" as * with tests
fn main %impure fn void i32 \void:
    let success %i32 gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_action_completion_test_success_contract unit
    let unsupported %i32 gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_action_completion_test_unsupported_contract unit
    let report %TestReport test_report_new "gui_font_registered_begin_frame_host_action_completion"
    let report1 %TestReport test_report_push report (assert_eq_i32 "success rejoins continuation" 3 success)
    let report2 %TestReport test_report_push report1 (assert_eq_i32 "unsupported rejoins continuation" 60 unsupported)
    test_report_exit_code test_report_print_stdout report2
```
