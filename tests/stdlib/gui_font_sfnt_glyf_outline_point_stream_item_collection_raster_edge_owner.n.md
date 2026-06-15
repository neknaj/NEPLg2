# GUI font SFNT glyf outline point stream item collection raster edge owner

このファイルは、F5bc の raster edge owner boundary が F5bb completed raster mask writer owner を authority とし、raster mask scalar stream を typed edge Vec へ変換することを固定する。scan conversion、render / platform 接続、fallback へ進まない。

source policy coverage labels:

- raster_edge_owner_types_ok
- raster_edge_owner_start_validation_order_ok
- raster_edge_owner_error_recovery_ok
- raster_edge_owner_scalar_read_contract_ok
- raster_edge_owner_record_parsing_ok
- raster_edge_owner_budget_progress_guard_ok
- raster_edge_owner_push_failure_recovery_ok
- raster_edge_owner_free_contract_ok
- raster_edge_owner_no_fallback_no_byte_backed_no_traversal_no_render

## point stream item collection raster edge owner smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// raster_edge_owner_types_ok
// raster_edge_owner_start_validation_order_ok
// raster_edge_owner_error_recovery_ok
// raster_edge_owner_scalar_read_contract_ok
// raster_edge_owner_record_parsing_ok
// raster_edge_owner_budget_progress_guard_ok
// raster_edge_owner_push_failure_recovery_ok
// raster_edge_owner_free_contract_ok
// raster_edge_owner_no_fallback_no_byte_backed_no_traversal_no_render

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection raster edge owner source policy smoke" all_groups
```
