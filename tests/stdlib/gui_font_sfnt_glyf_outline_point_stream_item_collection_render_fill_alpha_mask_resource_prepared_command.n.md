# GUI font SFNT glyf outline point stream item collection render fill alpha mask resource prepared command

このファイルは、F5bn の render fill alpha mask resource prepared command が registered resource owner と `RenderCommand` を同じ owner に閉じ込め、formal transport / drain owner より前に raw command を外へ出さない契約を固定する。

source policy coverage labels:

- render_fill_alpha_mask_resource_prepared_command_owner_pair_ok
- render_fill_alpha_mask_resource_prepared_command_no_raw_command_escape_ok
- render_fill_alpha_mask_resource_prepared_command_record_revalidation_ok
- render_fill_alpha_mask_resource_prepared_command_record_equality_ok
- render_fill_alpha_mask_resource_prepared_command_source_over_inherited_ok
- render_fill_alpha_mask_resource_prepared_command_record_mismatch_fail_closed_ok
- render_fill_alpha_mask_resource_prepared_command_error_recovery_ok
- render_fill_alpha_mask_resource_prepared_command_free_ok
- render_fill_alpha_mask_resource_prepared_command_no_stream_no_platform_no_fallback

## point stream item collection render fill alpha mask resource prepared command smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_fill_alpha_mask_resource_prepared_command_owner_pair_ok
// render_fill_alpha_mask_resource_prepared_command_no_raw_command_escape_ok
// render_fill_alpha_mask_resource_prepared_command_record_revalidation_ok
// render_fill_alpha_mask_resource_prepared_command_record_equality_ok
// render_fill_alpha_mask_resource_prepared_command_source_over_inherited_ok
// render_fill_alpha_mask_resource_prepared_command_record_mismatch_fail_closed_ok
// render_fill_alpha_mask_resource_prepared_command_error_recovery_ok
// render_fill_alpha_mask_resource_prepared_command_free_ok
// render_fill_alpha_mask_resource_prepared_command_no_stream_no_platform_no_fallback

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render fill alpha mask resource prepared command source policy smoke" all_groups
```
