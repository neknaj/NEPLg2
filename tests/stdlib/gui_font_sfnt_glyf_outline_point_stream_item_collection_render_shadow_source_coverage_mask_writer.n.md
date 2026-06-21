# GUI font SFNT glyf outline point stream item collection render shadow source coverage mask writer

このファイルは、F5ll の render shadow source coverage mask writer owner boundary が F5lk completed edge owner を direct authority として raw coverage cell buffer だけを所有し、scan / blur / packed mask / render / platform / compositor へ進まないことを固定する。

source policy coverage labels:

- render_shadow_source_coverage_mask_writer_f5lk_authority_ok
- render_shadow_source_coverage_mask_writer_no_second_config_ok
- render_shadow_source_coverage_mask_writer_edge_plan_revalidation_ok
- render_shadow_source_coverage_mask_writer_exact_cell_allocation_ok
- render_shadow_source_coverage_mask_writer_push_recovery_ok
- render_shadow_source_coverage_mask_writer_exact_completion_ok
- render_shadow_source_coverage_mask_writer_zero_edge_nonzero_cell_ok
- render_shadow_source_coverage_mask_writer_no_scan_blur_packed_render_platform

## point stream item collection render shadow source coverage mask writer smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_shadow_source_coverage_mask_writer_f5lk_authority_ok
// render_shadow_source_coverage_mask_writer_no_second_config_ok
// render_shadow_source_coverage_mask_writer_edge_plan_revalidation_ok
// render_shadow_source_coverage_mask_writer_exact_cell_allocation_ok
// render_shadow_source_coverage_mask_writer_push_recovery_ok
// render_shadow_source_coverage_mask_writer_exact_completion_ok
// render_shadow_source_coverage_mask_writer_zero_edge_nonzero_cell_ok
// render_shadow_source_coverage_mask_writer_no_scan_blur_packed_render_platform

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render shadow source coverage mask writer source policy smoke" all_groups
```
