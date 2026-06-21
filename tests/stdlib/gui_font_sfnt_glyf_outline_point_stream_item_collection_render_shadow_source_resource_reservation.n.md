# GUI font SFNT glyf outline point stream item collection render shadow source resource reservation

このファイルは、F5ls の render shadow source resource reservation が F5lp completed shadow source composition order owner と `AlphaMaskId` を内部 owner-bearing value に束ねるだけで、resource table 登録や render command 発行へ進まない契約を固定する。

source policy coverage labels:

- render_shadow_source_resource_reservation_f5lp_authority_ok
- render_shadow_source_resource_reservation_config_owner_error_ok
- render_shadow_source_resource_reservation_mask_id_checked_ok
- render_shadow_source_resource_reservation_order_error_evidence_ok
- render_shadow_source_resource_reservation_shadow_storage_ok
- render_shadow_source_resource_reservation_source_over_only_ok
- render_shadow_source_resource_reservation_rect_checked_ok
- render_shadow_source_resource_reservation_paint_order_metadata_ok
- render_shadow_source_resource_reservation_recovery_free_ok
- render_shadow_source_resource_reservation_no_command_table_platform_compositor

## point stream item collection render shadow source resource reservation smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_shadow_source_resource_reservation_f5lp_authority_ok
// render_shadow_source_resource_reservation_config_owner_error_ok
// render_shadow_source_resource_reservation_mask_id_checked_ok
// render_shadow_source_resource_reservation_order_error_evidence_ok
// render_shadow_source_resource_reservation_shadow_storage_ok
// render_shadow_source_resource_reservation_source_over_only_ok
// render_shadow_source_resource_reservation_rect_checked_ok
// render_shadow_source_resource_reservation_paint_order_metadata_ok
// render_shadow_source_resource_reservation_recovery_free_ok
// render_shadow_source_resource_reservation_no_command_table_platform_compositor

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render shadow source resource reservation source policy smoke" all_groups
```
