# GUI font registered simple glyph indexed stroke packed mask resource reservation

production F5nxp geometryからF5nxq coverage scanとF5nxs packed maskを順に完成し、未登録mask resource reservationの値とowner recoveryを検査する。

## reservation retains the production packed mask authority

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_reservation_normal\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"reservation retains id rect paint and packed owner\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_reservation_test_normal_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_reservation_normal"
    let checked %TestReport test_report_push report assert "reservation retains id rect paint and packed owner" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```

## reservation rejection preserves recoverable ownership

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_reservation_recovery\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"invalid id blend and storage preserve owned recovery\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_reservation_test_recovery_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_reservation_recovery"
    let checked %TestReport test_report_push report assert "invalid id blend and storage preserve owned recovery" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```
