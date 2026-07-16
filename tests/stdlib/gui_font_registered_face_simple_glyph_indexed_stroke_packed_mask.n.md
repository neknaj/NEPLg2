# GUI font registered simple glyph indexed stroke packed mask

production F5nxp authorityからF5nxq raw cellsを作り、F5nxsのrecovery、budget、alpha normalization、exact completionを検査する。

## packed mask accepts production F5nxr completion

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_normal\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"production scan completion packs exact alpha\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_test_normal_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_normal"
    let checked %TestReport test_report_push report assert "production scan completion packs exact alpha" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```

## packed mask uses floor normalization for production F5nxq raw cells

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_numeric\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"raw coverage floor normalizes to alpha\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_test_numeric_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_numeric"
    let checked %TestReport test_report_push report assert "raw coverage floor normalizes to alpha" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```

## packed mask errors preserve pre-step ownership

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_recovery\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"packed mask errors preserve recoverable owner\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_test_recovery_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_recovery"
    let checked %TestReport test_report_push report assert "packed mask errors preserve recoverable owner" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```
