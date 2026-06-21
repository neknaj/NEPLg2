# GUI font SFNT glyf outline point stream item collection render stroke coverage scan converter

このファイルは、F5lb の render stroke coverage scan converter が F5la writer owner を authority とし、line side edge と bevel connector chord だけを scan し、non-bevel join / quadratic side edge を fail-closed にし、packed mask / render command / platform へ進まないことを固定する。

source policy coverage labels:

- render_stroke_coverage_scan_f5la_writer_authority_ok
- render_stroke_coverage_scan_closure_revalidation_ok
- render_stroke_coverage_scan_cell_bounds_ok
- render_stroke_coverage_scan_line_side_edge_ok
- render_stroke_coverage_scan_bevel_connector_chord_ok
- render_stroke_coverage_scan_non_bevel_fail_closed_ok
- render_stroke_coverage_scan_quadratic_fail_closed_ok
- render_stroke_coverage_scan_push_budget_completion_ok
- render_stroke_coverage_scan_no_fill_scan_packed_render_platform

## point stream item collection render stroke coverage scan converter smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_stroke_coverage_scan_f5la_writer_authority_ok
// render_stroke_coverage_scan_closure_revalidation_ok
// render_stroke_coverage_scan_cell_bounds_ok
// render_stroke_coverage_scan_line_side_edge_ok
// render_stroke_coverage_scan_bevel_connector_chord_ok
// render_stroke_coverage_scan_non_bevel_fail_closed_ok
// render_stroke_coverage_scan_quadratic_fail_closed_ok
// render_stroke_coverage_scan_push_budget_completion_ok
// render_stroke_coverage_scan_no_fill_scan_packed_render_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render stroke coverage scan converter source policy smoke" all_groups
```
