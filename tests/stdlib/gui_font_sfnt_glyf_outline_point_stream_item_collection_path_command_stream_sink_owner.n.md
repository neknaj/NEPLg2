# GUI font SFNT glyf outline point stream item collection path command stream sink owner

このファイルは、F5az の path command stream sink owner boundary が F5ay completed sink plan を authority とし、real writer や raster / render / platform 接続へ進まないことを固定する。public plan は forged value として再検査し、失敗理由は coarse `InvalidPlan` ではなく typed enum variant で表す。

source policy coverage labels:

- path_command_stream_sink_owner_types_ok
- path_command_stream_sink_owner_plan_accessors_ok
- path_command_stream_sink_owner_precise_validation_kinds_ok
- path_command_stream_sink_owner_checked_add_mul_ok
- path_command_stream_sink_owner_capacity_derivation_ok
- path_command_stream_sink_owner_skip_only_zero_capacity_ok
- path_command_stream_sink_owner_allocation_order_ok
- path_command_stream_sink_owner_second_alloc_cleanup_ok
- path_command_stream_sink_owner_no_fallback_no_byte_backed_no_traversal_no_push_no_render

## point stream item collection path command stream sink owner smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// path_command_stream_sink_owner_types_ok
// path_command_stream_sink_owner_plan_accessors_ok
// path_command_stream_sink_owner_precise_validation_kinds_ok
// path_command_stream_sink_owner_checked_add_mul_ok
// path_command_stream_sink_owner_capacity_derivation_ok
// path_command_stream_sink_owner_skip_only_zero_capacity_ok
// path_command_stream_sink_owner_allocation_order_ok
// path_command_stream_sink_owner_second_alloc_cleanup_ok
// path_command_stream_sink_owner_no_fallback_no_byte_backed_no_traversal_no_push_no_render

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection path command stream sink owner source policy smoke" all_groups
```
