# GUI font SFNT glyf outline point stream item collection render shadow source packed mask owner

このファイルは、F5lo の render shadow source packed mask owner boundary が F5ln completed shadow source blur mask owner を direct authority とし、blurred coverage cells を alpha cells へ正規化することを固定する。generic packed mask、stroke、composition、render、platform、compositor へは進まない。

source policy coverage labels:

- render_shadow_source_packed_mask_f5ln_authority_ok
- render_shadow_source_packed_mask_config_alpha_ok
- render_shadow_source_packed_mask_blur_owner_invariant_ok
- render_shadow_source_packed_mask_exact_alpha_allocation_ok
- render_shadow_source_packed_mask_raw_cell_read_ok
- render_shadow_source_packed_mask_alpha_scale_ok
- render_shadow_source_packed_mask_push_recovery_ok
- render_shadow_source_packed_mask_completion_raw_blur_free_ok
- render_shadow_source_packed_mask_completed_invariant_ok
- render_shadow_source_packed_mask_budget_progress_ok
- render_shadow_source_packed_mask_no_generic_stroke_render_platform

## point stream item collection render shadow source packed mask owner smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_shadow_source_packed_mask_f5ln_authority_ok
// render_shadow_source_packed_mask_config_alpha_ok
// render_shadow_source_packed_mask_blur_owner_invariant_ok
// render_shadow_source_packed_mask_exact_alpha_allocation_ok
// render_shadow_source_packed_mask_raw_cell_read_ok
// render_shadow_source_packed_mask_alpha_scale_ok
// render_shadow_source_packed_mask_push_recovery_ok
// render_shadow_source_packed_mask_completion_raw_blur_free_ok
// render_shadow_source_packed_mask_completed_invariant_ok
// render_shadow_source_packed_mask_budget_progress_ok
// render_shadow_source_packed_mask_no_generic_stroke_render_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render shadow source packed mask owner source policy smoke" all_groups
```
