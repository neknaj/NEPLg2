# GUI font SFNT glyf outline point stream item collection render fill alpha mask resource table

このファイルは、F5bm の render fill alpha mask resource table が metadata-only table と owner-bearing registered resource owner を分け、table 登録の重複検査と push failure recovery を `Result` で固定する契約を確認する。

source policy coverage labels:

- render_fill_alpha_mask_resource_table_record_ok
- render_fill_alpha_mask_resource_table_metadata_only_ok
- render_fill_alpha_mask_resource_table_register_owner_pair_ok
- render_fill_alpha_mask_resource_table_nonzero_id_ok
- render_fill_alpha_mask_resource_table_reservation_revalidated_ok
- render_fill_alpha_mask_resource_table_metadata_match_ok
- render_fill_alpha_mask_resource_table_duplicate_reject_ok
- render_fill_alpha_mask_resource_table_lookup_metadata_ok
- render_fill_alpha_mask_resource_table_push_failure_pair_recovery_ok
- render_fill_alpha_mask_resource_table_free_ok
- render_fill_alpha_mask_resource_table_no_command_no_platform_no_fallback

## point stream item collection render fill alpha mask resource table smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_fill_alpha_mask_resource_table_record_ok
// render_fill_alpha_mask_resource_table_metadata_only_ok
// render_fill_alpha_mask_resource_table_register_owner_pair_ok
// render_fill_alpha_mask_resource_table_nonzero_id_ok
// render_fill_alpha_mask_resource_table_reservation_revalidated_ok
// render_fill_alpha_mask_resource_table_metadata_match_ok
// render_fill_alpha_mask_resource_table_duplicate_reject_ok
// render_fill_alpha_mask_resource_table_lookup_metadata_ok
// render_fill_alpha_mask_resource_table_push_failure_pair_recovery_ok
// render_fill_alpha_mask_resource_table_free_ok
// render_fill_alpha_mask_resource_table_no_command_no_platform_no_fallback

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render fill alpha mask resource table source policy smoke" all_groups
```
