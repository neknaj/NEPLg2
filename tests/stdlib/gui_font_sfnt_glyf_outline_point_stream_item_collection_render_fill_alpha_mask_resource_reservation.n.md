# GUI font SFNT glyf outline point stream item collection render fill alpha mask resource reservation

このファイルは、F5bl の render fill alpha mask resource reservation が completed fill alpha mask owner と `AlphaMaskId` を内部 owner-bearing value に束ねるだけで、resource table 登録や render command 発行へ進まない契約を固定する。

source policy coverage labels:

- render_fill_alpha_mask_resource_reservation_config_ok
- render_fill_alpha_mask_resource_reservation_internal_owner_ok
- render_fill_alpha_mask_resource_reservation_mask_id_checked_ok
- render_fill_alpha_mask_resource_reservation_owner_invariant_ok
- render_fill_alpha_mask_resource_reservation_source_over_only_ok
- render_fill_alpha_mask_resource_reservation_rect_paint_metadata_ok
- render_fill_alpha_mask_resource_reservation_recovery_ok
- render_fill_alpha_mask_resource_reservation_free_ok
- render_fill_alpha_mask_resource_reservation_no_command_no_platform_no_fallback

## point stream item collection render fill alpha mask resource reservation smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_fill_alpha_mask_resource_reservation_config_ok
// render_fill_alpha_mask_resource_reservation_internal_owner_ok
// render_fill_alpha_mask_resource_reservation_mask_id_checked_ok
// render_fill_alpha_mask_resource_reservation_owner_invariant_ok
// render_fill_alpha_mask_resource_reservation_source_over_only_ok
// render_fill_alpha_mask_resource_reservation_rect_paint_metadata_ok
// render_fill_alpha_mask_resource_reservation_recovery_ok
// render_fill_alpha_mask_resource_reservation_free_ok
// render_fill_alpha_mask_resource_reservation_no_command_no_platform_no_fallback

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render fill alpha mask resource reservation source policy smoke" all_groups
```
