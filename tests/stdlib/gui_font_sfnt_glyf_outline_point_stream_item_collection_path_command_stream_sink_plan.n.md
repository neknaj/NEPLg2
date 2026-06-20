# GUI font SFNT glyf outline point stream item collection path command stream sink plan

このファイルは、F5ay の path command stream sink plan boundary が F5ax completed drain terminal だけを authority とすることを固定する。budget exhausted partial terminal、command list、real sink、raster / render / platform 接続へは進まない。

source policy coverage labels:

- path_command_stream_sink_plan_types_ok
- path_command_stream_sink_plan_completed_terminal_authority_ok
- path_command_stream_sink_plan_budget_exhausted_rejected_ok
- path_command_stream_sink_plan_non_negative_count_guard_ok
- path_command_stream_sink_plan_checked_add_guard_ok
- path_command_stream_sink_plan_capacity_derivation_ok
- path_command_stream_sink_plan_count_invariants_ok
- path_command_stream_sink_plan_no_fallback_no_byte_backed_no_traversal_no_vec_no_raster

## point stream item collection path command stream sink plan smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// path_command_stream_sink_plan_types_ok
// path_command_stream_sink_plan_completed_terminal_authority_ok
// path_command_stream_sink_plan_budget_exhausted_rejected_ok
// path_command_stream_sink_plan_non_negative_count_guard_ok
// path_command_stream_sink_plan_checked_add_guard_ok
// path_command_stream_sink_plan_capacity_derivation_ok
// path_command_stream_sink_plan_count_invariants_ok
// path_command_stream_sink_plan_no_fallback_no_byte_backed_no_traversal_no_vec_no_raster

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection path command stream sink plan source policy smoke" all_groups
```
