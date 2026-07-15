# GUI font registered join geometry policy matrix

このファイルは、production metric projection から生成した offset geometry と side edge projection を使い、registered join geometry の policy matrix と左右 payload を検査する。

検査対象は bevel、miter、parallel clip、miter-limit clip、round、quadratic bevel、Left/Right payload である。

## registered policy matrix preserves geometry payload

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_join_geometry_policy_matrix\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"registered join geometry policy matrix\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let report %TestReport test_report_new "gui_font_registered_join_geometry_policy_matrix"
    let checked %TestReport test_report_push report assert "registered join geometry policy matrix" gui_sfnt_simple_glyph_render_stroke_join_geometry_test_policy_matrix unit
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```
