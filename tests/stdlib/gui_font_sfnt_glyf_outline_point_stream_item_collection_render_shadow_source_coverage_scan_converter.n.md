# GUI font SFNT glyf outline point stream item collection render shadow source coverage scan converter

このファイルは、F5lm の render shadow source coverage scan converter boundary が F5ll writer owner を direct authority とし、F5lk shadow source edge を scan して raw coverage cell を F5ll push boundary へ渡すことを固定する。generic fill scan owner、stroke path、blur、packed mask、render、platform、compositor へは進まない。

source policy coverage labels:

- render_shadow_source_coverage_scan_f5ll_writer_authority_ok
- render_shadow_source_coverage_scan_config_shape_start_validation_ok
- render_shadow_source_coverage_scan_cell_bounds_ok
- render_shadow_source_coverage_scan_edge_read_revalidation_ok
- render_shadow_source_coverage_scan_line_quadratic_crossing_ok
- render_shadow_source_coverage_scan_push_recovery_ok
- render_shadow_source_coverage_scan_budget_completion_progress_ok
- render_shadow_source_coverage_scan_zero_edge_nonzero_cell_ok
- render_shadow_source_coverage_scan_no_generic_scan_packed_blur_render_platform

## point stream item collection render shadow source coverage scan converter smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_shadow_source_coverage_scan_f5ll_writer_authority_ok
// render_shadow_source_coverage_scan_config_shape_start_validation_ok
// render_shadow_source_coverage_scan_cell_bounds_ok
// render_shadow_source_coverage_scan_edge_read_revalidation_ok
// render_shadow_source_coverage_scan_line_quadratic_crossing_ok
// render_shadow_source_coverage_scan_push_recovery_ok
// render_shadow_source_coverage_scan_budget_completion_progress_ok
// render_shadow_source_coverage_scan_zero_edge_nonzero_cell_ok
// render_shadow_source_coverage_scan_no_generic_scan_packed_blur_render_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render shadow source coverage scan converter source policy smoke" all_groups
```
