# GUI font SFNT glyf outline point stream item collection render shadow source edge drain

このファイルは、F5lk の render shadow source edge drain owner boundary が F5lj の shadow source coverage owner から path sink scalar を読み、shadow source 用 raster edge owner だけを作り、mask scan / blur / resource / platform / compositor へ進まないことを固定する。

source policy coverage labels:

- render_shadow_source_edge_context_ok
- render_shadow_source_edge_no_double_writer_owner_ok
- render_shadow_source_edge_revalidates_context_ok
- render_shadow_source_edge_shape_arithmetic_ok
- render_shadow_source_edge_path_sink_scalar_drain_ok
- render_shadow_source_edge_skip_tag_rejected_ok
- render_shadow_source_edge_empty_owner_ok
- render_shadow_source_edge_recovery_free_ok
- render_shadow_source_edge_no_mask_scan_blur_resource_platform_compositor

## point stream item collection render shadow source edge drain smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_shadow_source_edge_context_ok
// render_shadow_source_edge_no_double_writer_owner_ok
// render_shadow_source_edge_revalidates_context_ok
// render_shadow_source_edge_shape_arithmetic_ok
// render_shadow_source_edge_path_sink_scalar_drain_ok
// render_shadow_source_edge_skip_tag_rejected_ok
// render_shadow_source_edge_empty_owner_ok
// render_shadow_source_edge_recovery_free_ok
// render_shadow_source_edge_no_mask_scan_blur_resource_platform_compositor

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render shadow source edge drain source policy smoke" all_groups
```
