# GUI font registered BeginFrame resumed Run record

このfixtureはactual F5nzn Run stepをF5muへ一度だけprojectionし、reset済stateとcleanupを検証する。

neplg2:test[stdio, normalize_newlines]

```neplg2
---
stdout: "test_report name=\"gui_font_registered_begin_frame_resumed_run_record\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"resumed Run record authority\" expected=\"127\" actual=\"127\" message=\"\"\n"
---
#entry main
#indent 4
#target std
#import "std/test" as * with tests
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_run_record_test" as * with tests
fn main %impure fn void i32 \void:
    let evidence %i32 gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_run_record_test_contract unit
    let report %TestReport test_report_new "gui_font_registered_begin_frame_resumed_run_record"
    test_report_exit_code test_report_print_stdout test_report_push report assert_eq_i32 "resumed Run record authority" 127 evidence
```
