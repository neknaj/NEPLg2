# GUI font SFNT glyf outline point stream item collection render shadow source resource table

このファイルは、F5lt の render shadow source resource table が F5ls reservation owner を消費して metadata-only table と owner-bearing registered resource owner を分け、table 登録の lower order evidence、重複検査、push failure recovery を `Result` で固定する契約を確認する。

source policy coverage labels:

- render_shadow_source_resource_table_record_ok
- render_shadow_source_resource_table_metadata_only_ok
- render_shadow_source_resource_table_register_owner_pair_ok
- render_shadow_source_resource_table_lower_order_evidence_ok
- render_shadow_source_resource_table_nonzero_id_ok
- render_shadow_source_resource_table_reservation_revalidated_ok
- render_shadow_source_resource_table_metadata_match_ok
- render_shadow_source_resource_table_duplicate_reject_ok
- render_shadow_source_resource_table_lookup_metadata_ok
- render_shadow_source_resource_table_push_failure_pair_recovery_ok
- render_shadow_source_resource_table_free_ok
- render_shadow_source_resource_table_no_command_platform_compositor

## point stream item collection render shadow source resource table smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_shadow_source_resource_table_record_ok
// render_shadow_source_resource_table_metadata_only_ok
// render_shadow_source_resource_table_register_owner_pair_ok
// render_shadow_source_resource_table_lower_order_evidence_ok
// render_shadow_source_resource_table_nonzero_id_ok
// render_shadow_source_resource_table_reservation_revalidated_ok
// render_shadow_source_resource_table_metadata_match_ok
// render_shadow_source_resource_table_duplicate_reject_ok
// render_shadow_source_resource_table_lookup_metadata_ok
// render_shadow_source_resource_table_push_failure_pair_recovery_ok
// render_shadow_source_resource_table_free_ok
// render_shadow_source_resource_table_no_command_platform_compositor

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render shadow source resource table source policy smoke" all_groups
```
