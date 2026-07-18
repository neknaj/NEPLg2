# GUI font registered resumed EndFrame

このfixtureはscheduled Run continuationから次のEndFrame commandだけを進める。

neplg2:test[stdio, normalize_newlines]

```neplg2
---
stdout: "test_report name=\"gui_font_registered_begin_frame_resumed_end_frame\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"resumed EndFrame authority\" expected=\"63\" actual=\"63\" message=\"\"\n"
---
#entry main
#indent 4
#target std
#import "std/test" as * with tests
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_end_frame_test" as * with tests
fn main %impure fn void i32 \void:
    let evidence %i32 gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_end_frame_test_contract unit
    let report %TestReport test_report_new "gui_font_registered_begin_frame_resumed_end_frame"
    test_report_exit_code test_report_print_stdout test_report_push report assert_eq_i32 "resumed EndFrame authority" 63 evidence
```
