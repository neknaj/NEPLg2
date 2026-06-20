# GUI font SFNT glyf outline point stream item collection render fill alpha mask boundary

このファイルは、F5bg の render fill alpha mask boundary が F5bf completed packed alpha mask owner を authority とし、fill paint と blend を保持した owner handoff だけを行うことを固定する。full glyph paint、stroke、shadow、platform 接続、command emission へは進まない。

source policy coverage labels:

- render_fill_alpha_mask_config_ok
- render_fill_alpha_mask_shape_alpha_revalidation_ok
- render_fill_alpha_mask_fill_paint_blend_preserved_ok
- render_fill_alpha_mask_owner_handoff_ok
- render_fill_alpha_mask_recovery_ok
- render_fill_alpha_mask_free_ok
- render_fill_alpha_mask_no_platform_no_command

## point stream item collection render fill alpha mask boundary smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_fill_alpha_mask_config_ok
// render_fill_alpha_mask_shape_alpha_revalidation_ok
// render_fill_alpha_mask_fill_paint_blend_preserved_ok
// render_fill_alpha_mask_owner_handoff_ok
// render_fill_alpha_mask_recovery_ok
// render_fill_alpha_mask_free_ok
// render_fill_alpha_mask_no_platform_no_command

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render fill alpha mask boundary source policy smoke" all_groups
```
