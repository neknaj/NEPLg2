# GUI font SFNT glyf outline point stream item collection render stroke source contour authority

このファイルは、F5kw の render stroke source contour authority boundary が F5ku metric owner と path command value authority を照合し、stroke offset geometry / stroke edge / coverage / render command / platform へ進まないことを固定する。

source policy coverage labels:

- render_stroke_source_contour_authority_metric_provenance_ok
- render_stroke_source_contour_authority_exact_capacity_ok
- render_stroke_source_contour_authority_command_stream_value_ok
- render_stroke_source_contour_authority_line_quadratic_match_ok
- render_stroke_source_contour_authority_metric_source_coordinate_guard_ok
- render_stroke_source_contour_authority_contour_span_guard_ok
- render_stroke_source_contour_authority_skipped_command_counts_ok
- render_stroke_source_contour_authority_completion_counts_ok
- render_stroke_source_contour_authority_no_scalar_geometry_mask_command_platform

## point stream item collection render stroke source contour authority smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_stroke_source_contour_authority_metric_provenance_ok
// render_stroke_source_contour_authority_exact_capacity_ok
// render_stroke_source_contour_authority_command_stream_value_ok
// render_stroke_source_contour_authority_line_quadratic_match_ok
// render_stroke_source_contour_authority_metric_source_coordinate_guard_ok
// render_stroke_source_contour_authority_contour_span_guard_ok
// render_stroke_source_contour_authority_skipped_command_counts_ok
// render_stroke_source_contour_authority_completion_counts_ok
// render_stroke_source_contour_authority_no_scalar_geometry_mask_command_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render stroke source contour authority source policy smoke" all_groups
```
