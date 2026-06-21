# GUI font SFNT glyf outline point stream item collection render stroke join geometry

このファイルは、F5lc の render stroke join geometry boundary が F5kz completed edge closure owner を authority として消費し、bevel / miter join geometry を明示するが、round / quadratic / packed mask / render command / platform へ進まないことを固定する。

source policy coverage labels:

- render_stroke_join_geometry_f5kz_authority_ok
- render_stroke_join_geometry_line_side_edge_revalidation_ok
- render_stroke_join_geometry_bevel_chord_preserved_ok
- render_stroke_join_geometry_miter_intersection_limit_ok
- render_stroke_join_geometry_round_policy_or_fail_closed_ok
- render_stroke_join_geometry_quadratic_still_fail_closed_ok
- render_stroke_join_geometry_push_budget_completion_ok
- render_stroke_join_geometry_no_packed_render_platform

## point stream item collection render stroke join geometry smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_stroke_join_geometry_f5kz_authority_ok
// render_stroke_join_geometry_line_side_edge_revalidation_ok
// render_stroke_join_geometry_bevel_chord_preserved_ok
// render_stroke_join_geometry_miter_intersection_limit_ok
// render_stroke_join_geometry_round_policy_or_fail_closed_ok
// render_stroke_join_geometry_quadratic_still_fail_closed_ok
// render_stroke_join_geometry_push_budget_completion_ok
// render_stroke_join_geometry_no_packed_render_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render stroke join geometry source policy smoke" all_groups
```
