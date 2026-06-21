# GUI font SFNT glyf outline point stream item collection render shadow source sample cursor

このファイルは、F5lq の render shadow source sample cursor boundary が F5lp completed shadow source composition order owner を direct authority とし、shadow alpha mask を cell ごとの sample stream として読み出す契約を固定する。resource reservation、render command、pixel write、platform、2D compositor へは進まない。

source policy coverage labels:

- render_shadow_source_sample_cursor_f5lp_authority_ok
- render_shadow_source_sample_cursor_order_error_evidence_ok
- render_shadow_source_sample_cursor_shadow_shape_storage_ok
- render_shadow_source_sample_cursor_position_checked_ok
- render_shadow_source_sample_cursor_read_alpha_ok
- render_shadow_source_sample_cursor_order_metadata_ok
- render_shadow_source_sample_cursor_step_terminal_ok
- render_shadow_source_sample_cursor_recovery_free_ok
- render_shadow_source_sample_cursor_no_command_resource_platform

## point stream item collection render shadow source sample cursor smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_shadow_source_sample_cursor_f5lp_authority_ok
// render_shadow_source_sample_cursor_order_error_evidence_ok
// render_shadow_source_sample_cursor_shadow_shape_storage_ok
// render_shadow_source_sample_cursor_position_checked_ok
// render_shadow_source_sample_cursor_read_alpha_ok
// render_shadow_source_sample_cursor_order_metadata_ok
// render_shadow_source_sample_cursor_step_terminal_ok
// render_shadow_source_sample_cursor_recovery_free_ok
// render_shadow_source_sample_cursor_no_command_resource_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render shadow source sample cursor source policy smoke" all_groups
```
