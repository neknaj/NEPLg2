# GUI font SFNT glyf outline point stream item collection render shadow request

このファイルは、F5li の render shadow request boundary が completed path command stream authority と single shadow metadata を使い、fill / stroke mask、platform、render command、compositor に進まず shadow request owner だけを作ることを固定する。

source policy coverage labels:

- render_shadow_request_config_ok
- render_shadow_request_common_writer_authority_ok
- render_shadow_request_single_shadow_ok
- render_shadow_request_reject_shadow_run_ok
- render_shadow_request_source_metadata_ok
- render_shadow_request_stroke_width_revalidation_ok
- render_shadow_request_source_over_ok
- render_shadow_request_recovery_free_ok
- render_shadow_request_no_mask_resource_platform_compositor

## point stream item collection render shadow request smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_shadow_request_config_ok
// render_shadow_request_common_writer_authority_ok
// render_shadow_request_single_shadow_ok
// render_shadow_request_reject_shadow_run_ok
// render_shadow_request_source_metadata_ok
// render_shadow_request_stroke_width_revalidation_ok
// render_shadow_request_source_over_ok
// render_shadow_request_recovery_free_ok
// render_shadow_request_no_mask_resource_platform_compositor

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render shadow request source policy smoke" all_groups
```
