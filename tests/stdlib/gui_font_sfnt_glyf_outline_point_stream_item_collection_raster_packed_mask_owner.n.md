# GUI font SFNT glyf outline point stream item collection raster packed mask owner

このファイルは、F5bf の raster packed mask owner が F5be completed coverage mask owner を authority とし、raw coverage cell を normalized alpha cell へ変換することを固定する。zero-fill、render / platform 接続、fallback へ進まない。

source policy coverage labels:

- raster_packed_mask_config_start_ok
- raster_packed_mask_shape_raw_revalidation_ok
- raster_packed_mask_owner_invariant_ok
- raster_packed_mask_read_alpha_normalize_ok
- raster_packed_mask_push_recovery_ok
- raster_packed_mask_budget_completion_free_ok
- raster_packed_mask_no_fallback_no_render

## point stream item collection raster packed mask owner smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// raster_packed_mask_config_start_ok
// raster_packed_mask_shape_raw_revalidation_ok
// raster_packed_mask_owner_invariant_ok
// raster_packed_mask_read_alpha_normalize_ok
// raster_packed_mask_push_recovery_ok
// raster_packed_mask_budget_completion_free_ok
// raster_packed_mask_no_fallback_no_render

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection raster packed mask owner source policy smoke" all_groups
```
