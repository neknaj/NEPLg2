# GUI font registered simple glyph indexed stroke packed mask software drain start

production F5nxv sealed prepared command ownerとgeneric RGBA8888 software surface ownerを同時に消費し、pixelを変更せずcell index 0のF5nxw drain ownerを開始する境界を検査する。

## validated owners become a zero-index software drain cursor

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_normal\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"prepared command and surface remain paired at cell zero\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_test_normal_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_normal"
    let checked %TestReport test_report_push report assert "prepared command and surface remain paired at cell zero" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```

## rejected bounds preserve both owners for retry

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_recovery\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"surface rejection preserves prepared authority and surface ownership\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_test_recovery_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_recovery"
    let checked %TestReport test_report_push report assert "surface rejection preserves prepared authority and surface ownership" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```
