# GUI font SFNT glyf outline point stream item collection raster coverage scan converter

このファイルは、F5be の raster coverage scan converter が F5bd coverage mask writer owner を authority とし、line / quadratic typed raster edge から cell coverage を計算して `push_cell` boundary へ接続することを固定する。zero-fill、packed mask、render / platform 接続、fallback へ進まない。

source policy coverage labels:

- raster_coverage_scan_start_validation_ok
- raster_coverage_scan_shape_revalidation_ok
- raster_coverage_scan_cell_index_bounds_ok
- raster_coverage_scan_line_crossing_ok
- raster_coverage_scan_quadratic_segment_policy_ok
- raster_coverage_scan_cell_sampling_ok
- raster_coverage_scan_push_budget_completion_ok
- raster_coverage_scan_no_fallback_no_render

## point stream item collection raster coverage scan converter smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// raster_coverage_scan_start_validation_ok
// raster_coverage_scan_shape_revalidation_ok
// raster_coverage_scan_cell_index_bounds_ok
// raster_coverage_scan_line_crossing_ok
// raster_coverage_scan_quadratic_segment_policy_ok
// raster_coverage_scan_cell_sampling_ok
// raster_coverage_scan_push_budget_completion_ok
// raster_coverage_scan_no_fallback_no_render

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection raster coverage scan converter source policy smoke" all_groups
```
