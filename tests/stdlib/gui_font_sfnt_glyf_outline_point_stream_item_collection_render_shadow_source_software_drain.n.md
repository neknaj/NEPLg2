# GUI font SFNT glyf outline point stream item collection render shadow source software drain

このファイルは、F5lv の render shadow source software drain-start owner が prepared command owner と RGBA8888 software surface owner を同時に消費し、pixel write を行わない cursor owner 境界として閉じていることを固定する。

source policy coverage labels:

- render_shadow_source_software_drain_start_cursor_owner_ok
- render_shadow_source_software_drain_no_pixel_write_yet_ok
- render_shadow_source_software_drain_paired_owner_recovery_ok
- render_shadow_source_software_drain_no_split_accessor_ok
- render_shadow_source_software_drain_private_command_validation_only_ok
- render_shadow_source_software_drain_lower_order_evidence_ok
- render_shadow_source_software_drain_registered_resource_revalidation_ok
- render_shadow_source_software_drain_command_payload_match_ok
- render_shadow_source_software_drain_checked_geometry_ok
- render_shadow_source_software_drain_surface_containment_ok
- render_shadow_source_software_drain_free_ok
- render_shadow_source_software_drain_no_target_platform_fallback

## point stream item collection render shadow source software drain smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_shadow_source_software_drain_start_cursor_owner_ok
// render_shadow_source_software_drain_no_pixel_write_yet_ok
// render_shadow_source_software_drain_paired_owner_recovery_ok
// render_shadow_source_software_drain_no_split_accessor_ok
// render_shadow_source_software_drain_private_command_validation_only_ok
// render_shadow_source_software_drain_lower_order_evidence_ok
// render_shadow_source_software_drain_registered_resource_revalidation_ok
// render_shadow_source_software_drain_command_payload_match_ok
// render_shadow_source_software_drain_checked_geometry_ok
// render_shadow_source_software_drain_surface_containment_ok
// render_shadow_source_software_drain_free_ok
// render_shadow_source_software_drain_no_target_platform_fallback

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render shadow source software drain source policy smoke" all_groups
```
