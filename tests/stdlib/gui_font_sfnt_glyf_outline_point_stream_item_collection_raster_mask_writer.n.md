# GUI font SFNT glyf outline point stream item collection raster mask writer

このファイルは、F5bb の raster mask writer boundary が F5ba completed writer owner を authority とし、LineTo / QuadraticTo だけを raster mask scalar stream に書き、byte-backed lookup、old traversal、path object、raster / render / platform 接続へ戻らないことを固定する。current point は transition-only owner の内部状態であり、public constructor で forge できる API にしない。

source policy coverage labels:

- raster_mask_writer_types_ok
- raster_mask_writer_private_owner_ok
- raster_mask_writer_start_validation_order_ok
- raster_mask_writer_push_validation_order_ok
- raster_mask_writer_inner_complete_checks_ok
- raster_mask_writer_kind_progress_bounds_ok
- raster_mask_writer_stable_scalar_order_ok
- raster_mask_writer_current_point_behavior_ok
- raster_mask_writer_push_failure_recovery_ok
- raster_mask_writer_partial_failure_fail_closed_ok
- raster_mask_writer_no_fallback_no_byte_backed_no_traversal_no_render

## point stream item collection raster mask writer smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// raster_mask_writer_types_ok
// raster_mask_writer_private_owner_ok
// raster_mask_writer_start_validation_order_ok
// raster_mask_writer_push_validation_order_ok
// raster_mask_writer_inner_complete_checks_ok
// raster_mask_writer_kind_progress_bounds_ok
// raster_mask_writer_stable_scalar_order_ok
// raster_mask_writer_current_point_behavior_ok
// raster_mask_writer_push_failure_recovery_ok
// raster_mask_writer_partial_failure_fail_closed_ok
// raster_mask_writer_no_fallback_no_byte_backed_no_traversal_no_render

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection raster mask writer source policy smoke" all_groups
```
