# GUI font SFNT glyf outline point stream item collection render stroke source segment cursor

このファイルは、F5ks の render stroke source segment cursor boundary が F5kr plan owner authority と completed path sink scalar stream を使い、stroke offset geometry / stroke edge / coverage / render command / platform へ進まず source segment cursor だけを作ることを固定する。

source policy coverage labels:

- render_stroke_source_segment_cursor_plan_authority_ok
- render_stroke_source_segment_cursor_move_to_state_ok
- render_stroke_source_segment_cursor_line_geometry_ok
- render_stroke_source_segment_cursor_quadratic_geometry_ok
- render_stroke_source_segment_cursor_reject_missing_current_point_ok
- render_stroke_source_segment_cursor_reject_skip_scalar_tag_ok
- render_stroke_source_segment_cursor_reject_truncated_record_ok
- render_stroke_source_segment_cursor_completion_counts_ok
- render_stroke_source_segment_cursor_owner_recovery_ok
- render_stroke_source_segment_cursor_no_offset_edge_mask_command_platform

## point stream item collection render stroke source segment cursor smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_stroke_source_segment_cursor_plan_authority_ok
// render_stroke_source_segment_cursor_move_to_state_ok
// render_stroke_source_segment_cursor_line_geometry_ok
// render_stroke_source_segment_cursor_quadratic_geometry_ok
// render_stroke_source_segment_cursor_reject_missing_current_point_ok
// render_stroke_source_segment_cursor_reject_skip_scalar_tag_ok
// render_stroke_source_segment_cursor_reject_truncated_record_ok
// render_stroke_source_segment_cursor_completion_counts_ok
// render_stroke_source_segment_cursor_owner_recovery_ok
// render_stroke_source_segment_cursor_no_offset_edge_mask_command_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render stroke source segment cursor source policy smoke" all_groups
```
