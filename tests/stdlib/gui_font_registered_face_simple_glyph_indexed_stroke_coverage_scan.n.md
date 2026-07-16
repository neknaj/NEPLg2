# GUI font registered simple glyph indexed stroke coverage scan

actual F5nxp factory を一回だけ呼ぶ固定 entry ごとに、F5nxr の normal、work-bound、coordinate guard を独立して検査する。

## normal scan recovers owners and completes exact cells

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_normal\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"registered coverage scan preserves recovery and exact values\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_test_normal_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_normal"
    let checked %TestReport test_report_push report assert "registered coverage scan preserves recovery and exact values" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```

## work bound rejects excessive per-cell work and recovers owner

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_work_bound\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"registered coverage scan rejects excessive work\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_test_work_bound_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_work_bound"
    let checked %TestReport test_report_push report assert "registered coverage scan rejects excessive work" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```

## coordinate guard fails closed and recovers owner

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_coordinate\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"registered coverage scan rejects unsafe coordinate\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_test_coordinate_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_coordinate"
    let checked %TestReport test_report_push report assert "registered coverage scan rejects unsafe coordinate" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```
