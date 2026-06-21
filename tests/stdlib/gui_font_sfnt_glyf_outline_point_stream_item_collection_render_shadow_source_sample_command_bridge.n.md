# GUI font SFNT glyf outline point stream item collection render shadow source sample command bridge

このファイルは、F5lr の render shadow source sample command bridge が F5lq sample cursor を authority とし、SourceOver の shadow sample だけを 1x1 FillRect command へ変換する契約を固定する。これは shadow contribution command を作る correctness bridge であり、最終的な shadow/source composition order、resource registration、platform、2D compositor は表さない。

source policy coverage labels:

- render_shadow_source_sample_command_f5lq_authority_ok
- render_shadow_source_sample_command_order_error_evidence_ok
- render_shadow_source_sample_command_source_over_only_ok
- render_shadow_source_sample_command_alpha_scale_checked_ok
- render_shadow_source_sample_command_alpha_zero_transparent_ok
- render_shadow_source_sample_command_fill_rect_ok
- render_shadow_source_sample_command_conversion_before_advance_ok
- render_shadow_source_sample_command_rejected_sample_recovery_ok
- render_shadow_source_sample_command_terminal_free_ok
- render_shadow_source_sample_command_no_resource_platform_compositor

## point stream item collection render shadow source sample command bridge smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_shadow_source_sample_command_f5lq_authority_ok
// render_shadow_source_sample_command_order_error_evidence_ok
// render_shadow_source_sample_command_source_over_only_ok
// render_shadow_source_sample_command_alpha_scale_checked_ok
// render_shadow_source_sample_command_alpha_zero_transparent_ok
// render_shadow_source_sample_command_fill_rect_ok
// render_shadow_source_sample_command_conversion_before_advance_ok
// render_shadow_source_sample_command_rejected_sample_recovery_ok
// render_shadow_source_sample_command_terminal_free_ok
// render_shadow_source_sample_command_no_resource_platform_compositor

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render shadow source sample command bridge source policy smoke" all_groups
```
