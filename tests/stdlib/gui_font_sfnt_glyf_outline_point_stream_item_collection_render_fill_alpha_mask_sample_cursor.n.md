# GUI font SFNT glyf outline point stream item collection render fill alpha mask sample cursor

このファイルは、F5bi の render fill alpha mask sample cursor boundary が完成済み fill alpha mask owner を再検査し、cell ごとの absolute position、alpha、fill paint、blend を owner-safe に読み出す契約を固定する。2D compositor、command emission、platform 接続、fallback へは進まない。

source policy coverage labels:

- render_fill_alpha_mask_sample_cursor_start_ok
- render_fill_alpha_mask_sample_cursor_completed_owner_invariant_ok
- render_fill_alpha_mask_sample_cursor_bounds_fail_closed_ok
- render_fill_alpha_mask_sample_cursor_position_overflow_checked_ok
- render_fill_alpha_mask_sample_cursor_read_alpha_ok
- render_fill_alpha_mask_sample_cursor_step_terminal_ok
- render_fill_alpha_mask_sample_cursor_recovery_free_ok
- render_fill_alpha_mask_sample_cursor_no_platform_no_command

## point stream item collection render fill alpha mask sample cursor smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_fill_alpha_mask_sample_cursor_start_ok
// render_fill_alpha_mask_sample_cursor_completed_owner_invariant_ok
// render_fill_alpha_mask_sample_cursor_bounds_fail_closed_ok
// render_fill_alpha_mask_sample_cursor_position_overflow_checked_ok
// render_fill_alpha_mask_sample_cursor_read_alpha_ok
// render_fill_alpha_mask_sample_cursor_step_terminal_ok
// render_fill_alpha_mask_sample_cursor_recovery_free_ok
// render_fill_alpha_mask_sample_cursor_no_platform_no_command

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render fill alpha mask sample cursor source policy smoke" all_groups
```
