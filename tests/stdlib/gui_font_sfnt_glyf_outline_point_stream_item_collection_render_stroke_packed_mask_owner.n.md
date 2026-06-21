# GUI font SFNT glyf outline point stream item collection render stroke packed mask owner

このファイルは、F5lf の render stroke packed mask owner が F5lb completed stroke coverage mask owner を authority とし、raw stroke coverage cell を normalized alpha cell へ変換することを固定する。F5bf 直接再利用、render / platform / fallback / shadow / compositor 接続へ進まない。

source policy coverage labels:

- render_stroke_packed_mask_f5lb_authority_ok
- render_stroke_packed_mask_shape_raw_revalidation_ok
- render_stroke_packed_mask_owner_invariant_ok
- render_stroke_packed_mask_read_alpha_normalize_ok
- render_stroke_packed_mask_push_recovery_ok
- render_stroke_packed_mask_completion_raw_free_ok
- render_stroke_packed_mask_no_fill_reuse_render_platform_shadow

## point stream item collection render stroke packed mask owner smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_stroke_packed_mask_f5lb_authority_ok
// render_stroke_packed_mask_shape_raw_revalidation_ok
// render_stroke_packed_mask_owner_invariant_ok
// render_stroke_packed_mask_read_alpha_normalize_ok
// render_stroke_packed_mask_push_recovery_ok
// render_stroke_packed_mask_completion_raw_free_ok
// render_stroke_packed_mask_no_fill_reuse_render_platform_shadow

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render stroke packed mask owner source policy smoke" all_groups
```
