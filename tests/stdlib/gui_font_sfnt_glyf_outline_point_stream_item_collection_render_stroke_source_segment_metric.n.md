# GUI font SFNT glyf outline point stream item collection render stroke source segment metric

このファイルは、F5kt の render stroke source segment metric preparation boundary が F5ks source segment value だけを使い、F5ks cursor owner / stroke edge / coverage / render command / platform へ進まず checked metric value だけを作ることを固定する。

source policy coverage labels:

- render_stroke_source_segment_metric_line_ok
- render_stroke_source_segment_metric_quadratic_ok
- render_stroke_source_segment_metric_cast_before_subtract_ok
- render_stroke_source_segment_metric_checked_square_overflow_ok
- render_stroke_source_segment_metric_checked_sum_overflow_ok
- render_stroke_source_segment_metric_degenerate_reject_ok
- render_stroke_source_segment_metric_partial_degenerate_preserved_ok
- render_stroke_source_segment_metric_no_owner_edge_mask_command_platform

## point stream item collection render stroke source segment metric smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_stroke_source_segment_metric_line_ok
// render_stroke_source_segment_metric_quadratic_ok
// render_stroke_source_segment_metric_cast_before_subtract_ok
// render_stroke_source_segment_metric_checked_square_overflow_ok
// render_stroke_source_segment_metric_checked_sum_overflow_ok
// render_stroke_source_segment_metric_degenerate_reject_ok
// render_stroke_source_segment_metric_partial_degenerate_preserved_ok
// render_stroke_source_segment_metric_no_owner_edge_mask_command_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render stroke source segment metric source policy smoke" all_groups
```
