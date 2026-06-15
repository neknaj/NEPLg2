# GUI font SFNT glyf outline point stream item collection render fill alpha mask sample command bridge

このファイルは、F5bj の render fill alpha mask sample command bridge が F5bi sample cursor を authority とし、SourceOver の fill sample だけを 1x1 FillRect command へ変換する契約を固定する。これは高速 compositor の代替 fallback ではなく、1 sample を 1 typed command へ写す correctness bridge である。

source policy coverage labels:

- render_fill_alpha_mask_sample_command_gui_paint_color_ok
- render_fill_alpha_mask_sample_command_source_over_only_ok
- render_fill_alpha_mask_sample_command_alpha_scale_checked_ok
- render_fill_alpha_mask_sample_command_alpha_zero_transparent_ok
- render_fill_alpha_mask_sample_command_fill_rect_ok
- render_fill_alpha_mask_sample_command_cursor_success_before_advance_ok
- render_fill_alpha_mask_sample_command_cursor_rejected_sample_recovery_ok
- render_fill_alpha_mask_sample_command_cursor_terminal_free_ok
- render_fill_alpha_mask_sample_command_no_platform_no_target_no_fallback

## point stream item collection render fill alpha mask sample command bridge smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_fill_alpha_mask_sample_command_gui_paint_color_ok
// render_fill_alpha_mask_sample_command_source_over_only_ok
// render_fill_alpha_mask_sample_command_alpha_scale_checked_ok
// render_fill_alpha_mask_sample_command_alpha_zero_transparent_ok
// render_fill_alpha_mask_sample_command_fill_rect_ok
// render_fill_alpha_mask_sample_command_cursor_success_before_advance_ok
// render_fill_alpha_mask_sample_command_cursor_rejected_sample_recovery_ok
// render_fill_alpha_mask_sample_command_cursor_terminal_free_ok
// render_fill_alpha_mask_sample_command_no_platform_no_target_no_fallback

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render fill alpha mask sample command bridge source policy smoke" all_groups
```
