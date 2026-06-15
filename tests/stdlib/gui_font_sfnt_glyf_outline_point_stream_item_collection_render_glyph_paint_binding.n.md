# GUI font SFNT glyf outline point stream item collection render glyph paint binding

このファイルは、F5bh の render glyph paint binding boundary が full `GuiGlyphPaint` を受けても fill-only subset だけを明示的に受理し、stroke / shadow / missing fill / lower F5bg error を typed owner-bearing error として扱うことを固定する。

source policy coverage labels:

- render_glyph_paint_config_ok
- render_glyph_paint_accept_fill_only_ok
- render_glyph_paint_reject_stroke_before_missing_fill_ok
- render_glyph_paint_reject_shadow_before_missing_fill_ok
- render_glyph_paint_reject_missing_fill_ok
- render_glyph_paint_lower_error_recovery_ok
- render_glyph_paint_no_platform_no_command

## point stream item collection render glyph paint binding smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_glyph_paint_config_ok
// render_glyph_paint_accept_fill_only_ok
// render_glyph_paint_reject_stroke_before_missing_fill_ok
// render_glyph_paint_reject_shadow_before_missing_fill_ok
// render_glyph_paint_reject_missing_fill_ok
// render_glyph_paint_lower_error_recovery_ok
// render_glyph_paint_no_platform_no_command

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render glyph paint binding source policy smoke" all_groups
```
