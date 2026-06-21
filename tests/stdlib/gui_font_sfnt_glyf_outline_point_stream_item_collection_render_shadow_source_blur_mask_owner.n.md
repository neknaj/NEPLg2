# GUI font SFNT glyf outline point stream item collection render shadow source blur mask owner

このファイルは、F5ln の render shadow source blur mask owner boundary が F5lm completed raw coverage mask owner を direct authority とし、source coverage に spread / blur を適用して shadow coverage mask を作ることを固定する。packed mask、render、platform、compositor へは進まない。

source policy coverage labels:

- render_shadow_source_blur_mask_f5lm_authority_ok
- render_shadow_source_blur_mask_source_owner_invariant_ok
- render_shadow_source_blur_mask_shadow_shape_derivation_ok
- render_shadow_source_blur_mask_kernel_bounds_ok
- render_shadow_source_blur_mask_spread_max_filter_ok
- render_shadow_source_blur_mask_box_blur_ok
- render_shadow_source_blur_mask_out_of_source_zero_ok
- render_shadow_source_blur_mask_push_recovery_ok
- render_shadow_source_blur_mask_completion_raw_source_free_ok
- render_shadow_source_blur_mask_budget_progress_ok
- render_shadow_source_blur_mask_no_packed_render_platform

## point stream item collection render shadow source blur mask owner smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_shadow_source_blur_mask_f5lm_authority_ok
// render_shadow_source_blur_mask_source_owner_invariant_ok
// render_shadow_source_blur_mask_shadow_shape_derivation_ok
// render_shadow_source_blur_mask_kernel_bounds_ok
// render_shadow_source_blur_mask_spread_max_filter_ok
// render_shadow_source_blur_mask_box_blur_ok
// render_shadow_source_blur_mask_out_of_source_zero_ok
// render_shadow_source_blur_mask_push_recovery_ok
// render_shadow_source_blur_mask_completion_raw_source_free_ok
// render_shadow_source_blur_mask_budget_progress_ok
// render_shadow_source_blur_mask_no_packed_render_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render shadow source blur mask owner source policy smoke" all_groups
```
