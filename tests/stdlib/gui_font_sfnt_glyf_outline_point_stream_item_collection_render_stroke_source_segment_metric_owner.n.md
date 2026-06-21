# GUI font SFNT glyf outline point stream item collection render stroke source segment metric owner

このファイルは、F5ku の render stroke source segment metric owner boundary が F5ks fresh cursor を消費し、source segment metric Vec owner だけを作り、stroke offset geometry / stroke edge / coverage / render command / platform へ進まないことを固定する。

source policy coverage labels:

- render_stroke_source_segment_metric_owner_fresh_cursor_ok
- render_stroke_source_segment_metric_owner_exact_capacity_ok
- render_stroke_source_segment_metric_owner_line_push_ok
- render_stroke_source_segment_metric_owner_quadratic_push_ok
- render_stroke_source_segment_metric_owner_completion_counts_ok
- render_stroke_source_segment_metric_owner_push_recovery_ok
- render_stroke_source_segment_metric_owner_metric_error_context_ok
- render_stroke_source_segment_metric_owner_no_reread_edge_mask_command_platform

## point stream item collection render stroke source segment metric owner smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_stroke_source_segment_metric_owner_fresh_cursor_ok
// render_stroke_source_segment_metric_owner_exact_capacity_ok
// render_stroke_source_segment_metric_owner_line_push_ok
// render_stroke_source_segment_metric_owner_quadratic_push_ok
// render_stroke_source_segment_metric_owner_completion_counts_ok
// render_stroke_source_segment_metric_owner_push_recovery_ok
// render_stroke_source_segment_metric_owner_metric_error_context_ok
// render_stroke_source_segment_metric_owner_no_reread_edge_mask_command_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render stroke source segment metric owner source policy smoke" all_groups
```
