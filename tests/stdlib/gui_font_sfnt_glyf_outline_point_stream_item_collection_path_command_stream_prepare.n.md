# GUI font SFNT glyf outline point stream item collection path command stream prepare

このファイルは、F5ax の path command stream prepare boundary が F5aw stream step を value-only summary に畳むだけの境界であることを固定する。command list、real sink、raster / render / platform 接続へは進まない。

source policy coverage labels:

- path_command_stream_prepare_types_ok
- path_command_stream_prepare_initial_summary_ok
- path_command_stream_prepare_path_command_value_accessor_ok
- path_command_stream_prepare_single_command_classification_ok
- path_command_stream_prepare_step_completed_no_dummy_ok
- path_command_stream_prepare_step_uses_f5aw_once_ok
- path_command_stream_prepare_drain_terminal_variants_ok
- path_command_stream_prepare_drain_budget_no_step_ok
- path_command_stream_prepare_no_fallback_no_byte_backed_no_traversal_no_vec_no_raster

## point stream item collection path command stream prepare smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// path_command_stream_prepare_types_ok
// path_command_stream_prepare_initial_summary_ok
// path_command_stream_prepare_path_command_value_accessor_ok
// path_command_stream_prepare_single_command_classification_ok
// path_command_stream_prepare_step_completed_no_dummy_ok
// path_command_stream_prepare_step_uses_f5aw_once_ok
// path_command_stream_prepare_drain_terminal_variants_ok
// path_command_stream_prepare_drain_budget_no_step_ok
// path_command_stream_prepare_no_fallback_no_byte_backed_no_traversal_no_vec_no_raster

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection path command stream prepare source policy smoke" all_groups
```
