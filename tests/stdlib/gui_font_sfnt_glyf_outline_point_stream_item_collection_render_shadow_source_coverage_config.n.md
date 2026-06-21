# GUI font SFNT glyf outline point stream item collection render shadow source coverage config

このファイルは、F5lj の render shadow source coverage config boundary が F5li の shadow request owner と caller supplied coverage config を検査済み source shape へ束ね、mask / resource / platform / compositor へ進まないことを固定する。

source policy coverage labels:

- render_shadow_source_coverage_config_ok
- render_shadow_source_coverage_shape_validation_ok
- render_shadow_source_coverage_lower_error_mapping_ok
- render_shadow_source_coverage_source_metadata_revalidated_ok
- render_shadow_source_coverage_canonical_source_shape_ok
- render_shadow_source_coverage_source_placement_origin_checked_ok
- render_shadow_source_coverage_shadow_extent_stored_ok
- render_shadow_source_coverage_recovery_free_ok
- render_shadow_source_coverage_no_mask_resource_platform_compositor

## point stream item collection render shadow source coverage config smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_shadow_source_coverage_config_ok
// render_shadow_source_coverage_shape_validation_ok
// render_shadow_source_coverage_lower_error_mapping_ok
// render_shadow_source_coverage_source_metadata_revalidated_ok
// render_shadow_source_coverage_canonical_source_shape_ok
// render_shadow_source_coverage_source_placement_origin_checked_ok
// render_shadow_source_coverage_shadow_extent_stored_ok
// render_shadow_source_coverage_recovery_free_ok
// render_shadow_source_coverage_no_mask_resource_platform_compositor

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render shadow source coverage config source policy smoke" all_groups
```
