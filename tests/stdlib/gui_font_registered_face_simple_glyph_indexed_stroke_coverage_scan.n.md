# GUI font registered simple glyph indexed stroke coverage scan

actual F5nxp factory から F5nxq writer を開始し、F5nxr が production の line side edge と bevel connector を parity scan して exact cell を完成させることを検査する。

## bounded scan recovers owners and completes exact cells

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"registered coverage scan preserves recovery and exact values\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let ok %bool gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_test_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan"
    let checked %TestReport test_report_push report assert "registered coverage scan preserves recovery and exact values" ok
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```
