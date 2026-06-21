# GUI font SFNT glyf outline point stream item collection render stroke request

このファイルは、F5kq の render stroke request boundary が completed path command stream authority を使い、fill alpha mask / raster edge / platform / render command に進まず stroke request owner だけを作ることを固定する。

source policy coverage labels:

- render_stroke_request_config_ok
- render_stroke_request_completed_path_authority_ok
- render_stroke_request_reject_missing_stroke_ok
- render_stroke_request_reject_invalid_width_ok
- render_stroke_request_reject_shadow_ok
- render_stroke_request_reject_unsupported_blend_ok
- render_stroke_request_preserve_fill_metadata_ok
- render_stroke_request_owner_recovery_ok
- render_stroke_request_no_fill_mask_no_render_command_no_platform

## point stream item collection render stroke request smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_stroke_request_config_ok
// render_stroke_request_completed_path_authority_ok
// render_stroke_request_reject_missing_stroke_ok
// render_stroke_request_reject_invalid_width_ok
// render_stroke_request_reject_shadow_ok
// render_stroke_request_reject_unsupported_blend_ok
// render_stroke_request_preserve_fill_metadata_ok
// render_stroke_request_owner_recovery_ok
// render_stroke_request_no_fill_mask_no_render_command_no_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render stroke request source policy smoke" all_groups
```
