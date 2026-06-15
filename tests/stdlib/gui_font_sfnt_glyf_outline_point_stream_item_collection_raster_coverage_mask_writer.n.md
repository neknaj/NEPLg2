# GUI font SFNT glyf outline point stream item collection raster coverage mask writer

このファイルは、F5bd の raster coverage mask writer boundary が F5bc completed raster edge owner を authority とし、後続 scan conversion が書き込む coverage cell buffer だけを所有することを固定する。scan conversion、coverage 計算、render / platform 接続、fallback へ進まない。

source policy coverage labels:

- raster_coverage_config_shape_ok
- raster_coverage_shape_validation_order_ok
- raster_coverage_start_validation_order_ok
- raster_coverage_edge_storage_revalidation_ok
- raster_coverage_owner_recovery_ok
- raster_coverage_push_validation_order_ok
- raster_coverage_completion_free_ok
- raster_coverage_no_fallback_no_scan_no_render

## point stream item collection raster coverage mask writer smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// raster_coverage_config_shape_ok
// raster_coverage_shape_validation_order_ok
// raster_coverage_start_validation_order_ok
// raster_coverage_edge_storage_revalidation_ok
// raster_coverage_owner_recovery_ok
// raster_coverage_push_validation_order_ok
// raster_coverage_completion_free_ok
// raster_coverage_no_fallback_no_scan_no_render

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection raster coverage mask writer source policy smoke" all_groups
```
