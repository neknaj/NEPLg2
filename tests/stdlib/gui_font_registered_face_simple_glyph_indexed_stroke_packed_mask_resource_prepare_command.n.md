# GUI font registered simple glyph indexed stroke packed mask prepared command

production F5nxp geometryからF5nxq coverage scan、F5nxs packed mask、F5nxt reservation、F5nxu registrationを順に完成し、registered resourceと`AlphaMaskRect` commandを分離不能に保持するF5nxv sealed ownerを検査する。

## registered resource becomes a sealed prepared command owner

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_prepare_command_normal\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"registered authority and command remain sealed together\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_prepare_command_test_normal_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_prepare_command_normal"
    let checked %TestReport test_report_push report assert "registered authority and command remain sealed together" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```

## rejected metadata preserves the registered resource owner

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_prepare_command_recovery\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"record and storage failures preserve registered authority\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let result %bool gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_prepare_command_test_recovery_contract unit
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_prepare_command_recovery"
    let checked %TestReport test_report_push report assert "record and storage failures preserve registered authority" result
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```
