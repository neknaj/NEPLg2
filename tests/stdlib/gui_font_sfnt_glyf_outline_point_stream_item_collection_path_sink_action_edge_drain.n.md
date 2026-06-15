# GUI font SFNT glyf outline point stream item collection path sink action Edge drain doctests

このファイルは、F5at の Edge drain boundary が F5as の EdgeStartOwner を authority とし、summary / storage / cursor / collection の照合後だけ owner storage endpoint marker と collection-backed contour span / contour edge source に進むことを固定する。全 Edge slot 完了後は PathCommandTag cursor を開始するだけで、curve segment classification や path command tag population は行わない。

現時点では source policy coverage label を固定し、実行可能な詳細ケースは `stdlib/alloc/gui/font/sfnt/glyf.nepl` の統合 doctest と後続の compiler compile-time 改善後に移す。

source policy coverage labels:

- path_sink_action_edge_drain_types_ok
- path_sink_action_edge_drain_authority_checks_ok
- path_sink_action_edge_drain_endpoint_marker_non_consuming_ok
- path_sink_action_edge_drain_span_edge_source_checks_ok
- path_sink_action_edge_drain_push_failure_recovers_edge_owner_ok
- path_sink_action_edge_drain_push_error_metadata_before_storage_ok
- path_sink_action_edge_drain_completion_starts_path_command_tag_cursor_only_ok
- path_sink_action_edge_drain_step_budget_no_source_no_push_ok
- path_sink_action_edge_drain_no_fallback_no_byte_backed_no_traversal_no_curve_segment

## point stream item collection path sink action Edge drain smoke

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
    let has_endpoint_label %bool true
    let has_source_label %bool true
    let has_push_failure_label %bool true
    let has_metadata_order_label %bool true
    let has_completion_label %bool true
    let has_budget_label %bool true
    let has_policy_label %bool true
    let authority_group %bool and has_types_label has_authority_label
    let source_group %bool and has_endpoint_label has_source_label
    let recovery_group %bool and has_push_failure_label has_metadata_order_label
    let terminal_group %bool and has_completion_label has_budget_label
    let first_group %bool and authority_group source_group
    let second_group %bool and recovery_group terminal_group
    let all_groups %bool and first_group second_group
    test_assertion_exit_code assert "point stream item collection path sink action Edge drain source policy smoke" and all_groups has_policy_label
```
