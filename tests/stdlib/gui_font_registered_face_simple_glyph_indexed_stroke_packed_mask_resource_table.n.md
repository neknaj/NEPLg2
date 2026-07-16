# GUI font registered simple glyph indexed stroke packed mask resource table

production F5nxp geometryからF5nxq coverage scan、F5nxs packed mask、F5nxt reservationを順に完成し、metadata-only resource tableの登録とpaired recoveryを検査する。

## resource table registers metadata without splitting packed authority

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_table_normal\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"table registers and looks up metadata while resource retains authority\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_table_test_normal_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_table_normal"
    let checked %TestReport test_report_push report assert "table registers and looks up metadata while resource retains authority" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```

## duplicate push and invalid metadata preserve table and reservation

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_table_recovery\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"duplicate push and validation failures preserve paired recovery\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_table_test_recovery_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_table_recovery"
    let checked %TestReport test_report_push report assert "duplicate push and validation failures preserve paired recovery" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```
