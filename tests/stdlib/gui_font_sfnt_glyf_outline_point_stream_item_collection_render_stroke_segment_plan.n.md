# GUI font SFNT glyf outline point stream item collection render stroke segment plan

このファイルは、F5kr の render stroke segment plan boundary が F5kq request owner authority を使い、stroke geometry / fill alpha mask / raster edge / platform / render command に進まず count-only stroke segment plan owner だけを作ることを固定する。

source policy coverage labels:

- render_stroke_segment_plan_request_authority_ok
- render_stroke_segment_plan_count_only_owner_ok
- render_stroke_segment_plan_revalidates_writer_invariants_ok
- render_stroke_segment_plan_reject_invalid_width_ok
- render_stroke_segment_plan_checked_draw_count_ok
- render_stroke_segment_plan_reject_no_drawable_segments_ok
- render_stroke_segment_plan_owner_recovery_ok
- render_stroke_segment_plan_no_geometry_no_mask_no_command_no_platform

## point stream item collection render stroke segment plan smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_stroke_segment_plan_request_authority_ok
// render_stroke_segment_plan_count_only_owner_ok
// render_stroke_segment_plan_revalidates_writer_invariants_ok
// render_stroke_segment_plan_reject_invalid_width_ok
// render_stroke_segment_plan_checked_draw_count_ok
// render_stroke_segment_plan_reject_no_drawable_segments_ok
// render_stroke_segment_plan_owner_recovery_ok
// render_stroke_segment_plan_no_geometry_no_mask_no_command_no_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render stroke segment plan source policy smoke" all_groups
```
