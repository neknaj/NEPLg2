# GUI font registered BeginFrame host execution driver

このfixtureはactual registered F5nzi authorityを分断せず、F5ne driver prepareでmetadata-preserving Offscreen BeginFrame actionへ接続するF5nzj契約を検証する。

neplg2:test[stdio, normalize_newlines]

```neplg2
---
stdout: "test_report name=\"gui_font_registered_begin_frame_host_execution_driver\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"BeginFrame pending enters host execution driver\" expected=\"127\" actual=\"127\" message=\"\"\n"
---
#entry main
#indent 4
#target std
#import "std/test" as * with tests
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_execution_driver_test" as * with tests
fn main %impure fn void i32 \void:
    let result %i32 gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_execution_driver_test_success_contract unit
    let report %TestReport test_report_new "gui_font_registered_begin_frame_host_execution_driver"
    let report1 %TestReport test_report_push report (assert_eq_i32 "BeginFrame pending enters host execution driver" 127 result)
    test_report_exit_code test_report_print_stdout report1
```
