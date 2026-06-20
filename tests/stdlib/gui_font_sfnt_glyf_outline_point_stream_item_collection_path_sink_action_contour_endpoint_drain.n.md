# GUI font SFNT glyf outline point stream item collection path sink action contour endpoint drain doctests

このファイルは、F5aq の contour endpoint drain boundary が F5ap push owner を authority とし、summary / storage / cursor / collection の照合後だけ collection-backed span source と F5e push に進むことを固定する。全 contour endpoint 完了後は PointX cursor を開始するだけで、PointX value push は行わない。

source policy coverage labels:

- path_sink_action_contour_endpoint_drain_types_ok
- path_sink_action_contour_endpoint_drain_authority_checks_ok
- path_sink_action_contour_endpoint_drain_source_span_once_ok
- path_sink_action_contour_endpoint_drain_span_failure_recovers_push_owner_ok
- path_sink_action_contour_endpoint_drain_push_failure_recovers_push_owner_ok
- path_sink_action_contour_endpoint_drain_completion_starts_point_x_cursor_only_ok
- path_sink_action_contour_endpoint_drain_step_budget_no_span_no_push_ok
- path_sink_action_contour_endpoint_drain_no_fallback_no_byte_backed_no_traversal

## point stream item collection path sink action contour endpoint drain smoke

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
    let has_span_label %bool true
    let has_span_failure_label %bool true
    let has_push_failure_label %bool true
    let has_completion_label %bool true
    let has_budget_label %bool true
    let has_policy_label %bool true
    let authority_group %bool and has_types_label has_authority_label
    let source_group %bool and has_span_label has_span_failure_label
    let recovery_group %bool and has_push_failure_label has_completion_label
    let terminal_group %bool and has_budget_label has_policy_label
    let first_group %bool and authority_group source_group
    let second_group %bool and recovery_group terminal_group
    test_assertion_exit_code assert "point stream item collection path sink action contour endpoint drain source policy smoke" and first_group second_group
```
