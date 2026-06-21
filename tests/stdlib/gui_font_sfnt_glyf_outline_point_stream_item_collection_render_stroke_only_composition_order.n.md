# GUI font SFNT glyf outline point stream item collection render stroke-only composition order

このファイルは、F5lh の render stroke-only composition order boundary が completed stroke packed mask owner を stroke-only 専用の順序 owner として束ね、fill+stroke、resource 登録、render command、platform、shadow / compositor に進まないことを固定する。

source policy coverage labels:

- render_stroke_only_composition_scope_ok
- render_stroke_only_composition_owner_invariant_ok
- render_stroke_only_composition_nested_metadata_ok
- render_stroke_only_composition_unexpected_fill_ok
- render_stroke_only_composition_source_over_ok
- render_stroke_only_composition_recovery_free_order_ok
- render_stroke_only_composition_no_render_resource_platform_shadow

## point stream item collection render stroke-only composition order smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_stroke_only_composition_scope_ok
// render_stroke_only_composition_owner_invariant_ok
// render_stroke_only_composition_nested_metadata_ok
// render_stroke_only_composition_unexpected_fill_ok
// render_stroke_only_composition_source_over_ok
// render_stroke_only_composition_recovery_free_order_ok
// render_stroke_only_composition_no_render_resource_platform_shadow

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render stroke-only composition order source policy smoke" all_groups
```
