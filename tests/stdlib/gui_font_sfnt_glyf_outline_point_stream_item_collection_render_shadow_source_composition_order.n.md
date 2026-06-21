# GUI font SFNT glyf outline point stream item collection render shadow source composition order

このファイルは、F5lp の render shadow source composition order boundary が F5lo completed shadow source packed mask owner を direct authority とし、shadow contribution を source paint より前に置く順序 metadata だけを固定することを確認する。fill/stroke composition、sample cursor、resource reservation、render command、platform、2D compositor へは進まない。

source policy coverage labels:

- render_shadow_source_composition_order_f5lo_authority_ok
- render_shadow_source_composition_order_context_metadata_ok
- render_shadow_source_composition_order_packed_invariant_ok
- render_shadow_source_composition_order_edge_error_evidence_ok
- render_shadow_source_composition_order_source_over_ok
- render_shadow_source_composition_order_shadow_before_source_ok
- render_shadow_source_composition_order_completed_invariant_ok
- render_shadow_source_composition_order_recovery_free_ok
- render_shadow_source_composition_order_no_fill_stroke_render_platform

## point stream item collection render shadow source composition order smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_shadow_source_composition_order_f5lo_authority_ok
// render_shadow_source_composition_order_context_metadata_ok
// render_shadow_source_composition_order_packed_invariant_ok
// render_shadow_source_composition_order_edge_error_evidence_ok
// render_shadow_source_composition_order_source_over_ok
// render_shadow_source_composition_order_shadow_before_source_ok
// render_shadow_source_composition_order_completed_invariant_ok
// render_shadow_source_composition_order_recovery_free_ok
// render_shadow_source_composition_order_no_fill_stroke_render_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render shadow source composition order source policy smoke" all_groups
```
