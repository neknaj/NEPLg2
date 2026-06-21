# GUI font SFNT glyf outline point stream item collection render stroke offset geometry

このファイルは、F5kx の render stroke offset geometry boundary が F5kw source contour owner を authority として消費し、actual stroke offset geometry owner を作り、stroke edge / coverage / render command / platform へ進まないことを固定する。

source policy coverage labels:

- render_stroke_offset_geometry_source_owner_authority_ok
- render_stroke_offset_geometry_exact_capacity_ok
- render_stroke_offset_geometry_line_offsets_ok
- render_stroke_offset_geometry_quadratic_tangent_policy_ok
- render_stroke_offset_geometry_style_guard_ok
- render_stroke_offset_geometry_push_recovery_ok
- render_stroke_offset_geometry_no_scalar_edge_mask_command_platform

## point stream item collection render stroke offset geometry smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_stroke_offset_geometry_source_owner_authority_ok
// render_stroke_offset_geometry_exact_capacity_ok
// render_stroke_offset_geometry_line_offsets_ok
// render_stroke_offset_geometry_quadratic_tangent_policy_ok
// render_stroke_offset_geometry_style_guard_ok
// render_stroke_offset_geometry_push_recovery_ok
// render_stroke_offset_geometry_no_scalar_edge_mask_command_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render stroke offset geometry source policy smoke" all_groups
```
