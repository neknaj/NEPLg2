# GUI font SFNT glyf outline point stream item collection path command stream cursor doctests

このファイルは、F5aw の path command stream cursor / stream preparation boundary が F5av の `PathCommandValue` lookup を順序付きに読むだけの value-only 境界であることを固定する。dummy value、`Vec` への蓄積、path object materialization、raster / render / platform 接続へは進まない。

source policy coverage labels:

- path_command_stream_cursor_types_ok
- path_command_stream_cursor_authority_checks_ok
- path_command_stream_cursor_validation_ok
- path_command_stream_step_completed_no_value_no_lookup_ok
- path_command_stream_step_emits_via_f5av_once_ok
- path_command_stream_drain_terminal_variants_ok
- path_command_stream_drain_budget_no_step_no_lookup_ok
- path_command_stream_no_fallback_no_byte_backed_no_traversal_no_vec_no_raster

## point stream item collection path command stream cursor smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection path command stream cursor source policy smoke" all_groups
```
