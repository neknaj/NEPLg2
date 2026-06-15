# GUI font SFNT glyf outline point stream item collection path sink action PathCommandTag drain doctests

このファイルは、F5au の PathCommandTag drain boundary が F5at の PathCommandTagStartOwner を authority とし、summary / storage / cursor / collection の照合後だけ owner storage Edge scalar と collection-backed path sink event kind source に進むことを固定する。全 PathCommandTag slot 完了後は PathCommandTagCompleteOwner を返すだけで、path command stream construction や raster / render には進まない。

現時点では source policy coverage label を固定し、実行可能な詳細ケースは `stdlib/alloc/gui/font/sfnt/glyf.nepl` の統合 doctest と後続の compiler compile-time 改善後に移す。

source policy coverage labels:

- path_sink_action_path_command_tag_drain_types_ok
- path_sink_action_path_command_tag_drain_authority_checks_ok
- path_sink_action_path_command_tag_drain_partial_restart_ok
- path_sink_action_path_command_tag_drain_logical_index_mapping_ok
- path_sink_action_path_command_tag_drain_edge_owner_non_consuming_ok
- path_sink_action_path_command_tag_drain_span_event_source_checks_ok
- path_sink_action_path_command_tag_drain_push_failure_recovers_path_command_tag_owner_ok
- path_sink_action_path_command_tag_drain_push_error_metadata_before_storage_ok
- path_sink_action_path_command_tag_drain_completion_returns_complete_owner_only_ok
- path_sink_action_path_command_tag_drain_step_budget_no_source_no_push_ok
- path_sink_action_path_command_tag_drain_no_fallback_no_byte_backed_no_traversal_no_raster

## point stream item collection path sink action PathCommandTag drain smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let has_types_label %bool true
    let has_authority_label %bool true
    let has_restart_label %bool true
    let has_index_label %bool true
    let has_edge_owner_label %bool true
    let has_source_label %bool true
    let has_push_failure_label %bool true
    let has_metadata_order_label %bool true
    let has_completion_label %bool true
    let has_budget_label %bool true
    let has_policy_label %bool true
    let authority_group %bool and has_types_label and has_authority_label has_restart_label
    let source_group %bool and has_index_label and has_edge_owner_label has_source_label
    let recovery_group %bool and has_push_failure_label has_metadata_order_label
    let terminal_group %bool and has_completion_label has_budget_label
    let first_group %bool and authority_group source_group
    let second_group %bool and recovery_group terminal_group
    let all_groups %bool and first_group second_group
    test_assertion_exit_code assert "point stream item collection path sink action PathCommandTag drain source policy smoke" and all_groups has_policy_label
```
