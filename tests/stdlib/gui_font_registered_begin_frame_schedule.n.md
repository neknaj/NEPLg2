# GUI font registered BeginFrame deterministic schedule

このfixtureはactual registered stroke pipelineからF5nze authorityを作り、BeginFrame recordを再投入せずF5mw deterministic scheduleへ引き継ぐF5nzf契約を検証する。

neplg2:test[stdio, normalize_newlines]

```neplg2
---
stdout: "test_report name=\"gui_font_registered_begin_frame_schedule\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"BeginFrame drain enters deterministic schedule\" expected=\"255\" actual=\"255\" message=\"\"\n"
---
#entry main
#indent 4
#target std
#import "std/test" as * with tests
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_schedule_test" as * with tests
fn main %impure fn void i32 \void:
    let result %i32 gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_begin_frame_schedule_bridge_test_success_contract unit
    let report %TestReport test_report_new "gui_font_registered_begin_frame_schedule"
    let report1 %TestReport test_report_push report (assert_eq_i32 "BeginFrame drain enters deterministic schedule" 255 result)
    test_report_exit_code test_report_print_stdout report1
```
