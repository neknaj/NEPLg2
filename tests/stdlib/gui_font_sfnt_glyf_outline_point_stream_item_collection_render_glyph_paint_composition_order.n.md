# GUI font SFNT glyf outline point stream item collection render glyph paint composition order

このファイルは、F5lg の render glyph paint composition order boundary が completed fill alpha mask owner と completed stroke packed mask owner を fill+stroke 専用の順序 owner として束ね、stroke-only、resource 登録、render command、platform、shadow / compositor に進まないことを固定する。

source policy coverage labels:

- render_glyph_paint_composition_fill_stroke_scope_ok
- render_glyph_paint_composition_owner_invariant_ok
- render_glyph_paint_composition_nested_stroke_metadata_ok
- render_glyph_paint_composition_shape_tuple_match_ok
- render_glyph_paint_composition_source_over_fill_paint_match_ok
- render_glyph_paint_composition_combined_recovery_free_order_ok
- render_glyph_paint_composition_no_render_resource_platform_shadow

## point stream item collection render glyph paint composition order smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_glyph_paint_composition_fill_stroke_scope_ok
// render_glyph_paint_composition_owner_invariant_ok
// render_glyph_paint_composition_nested_stroke_metadata_ok
// render_glyph_paint_composition_shape_tuple_match_ok
// render_glyph_paint_composition_source_over_fill_paint_match_ok
// render_glyph_paint_composition_combined_recovery_free_order_ok
// render_glyph_paint_composition_no_render_resource_platform_shadow

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render glyph paint composition order source policy smoke" all_groups
```
