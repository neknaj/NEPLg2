# GUI font SFNT glyf outline point stream item collection render stroke side edge owner

このファイルは、F5ky の render stroke side edge owner boundary が F5kx completed offset geometry owner を authority として消費し、左右 side edge record owner を作り、閉じた stroke outline / coverage / render command / platform へ進まないことを固定する。

source policy coverage labels:

- render_stroke_side_edge_owner_offset_geometry_authority_ok
- render_stroke_side_edge_owner_exact_capacity_double_geometry_ok
- render_stroke_side_edge_owner_left_right_direction_ok
- render_stroke_side_edge_owner_quadratic_endpoint_policy_ok
- render_stroke_side_edge_owner_one_step_one_push_recovery_ok
- render_stroke_side_edge_owner_not_closed_outline_ok
- render_stroke_side_edge_owner_no_scalar_mask_command_platform

## point stream item collection render stroke side edge owner smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_stroke_side_edge_owner_offset_geometry_authority_ok
// render_stroke_side_edge_owner_exact_capacity_double_geometry_ok
// render_stroke_side_edge_owner_left_right_direction_ok
// render_stroke_side_edge_owner_quadratic_endpoint_policy_ok
// render_stroke_side_edge_owner_one_step_one_push_recovery_ok
// render_stroke_side_edge_owner_not_closed_outline_ok
// render_stroke_side_edge_owner_no_scalar_mask_command_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render stroke side edge owner source policy smoke" all_groups
```
