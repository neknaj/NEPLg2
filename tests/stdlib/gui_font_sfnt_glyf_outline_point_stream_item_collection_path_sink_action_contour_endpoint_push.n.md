# GUI font SFNT glyf outline point stream item collection path sink action contour endpoint push doctests

このファイルは、F5ap の contour endpoint push boundary が F5ao start terminal を authority とし、`Started` branch だけを F5e typed endpoint push へ進めることを固定する。push failure では lower metadata を storage 回収前に読み、returned storage と保存済み summary / cursor / previous endpoint から start owner を復元する。`Rejected` / `StepBudgetExhausted` は endpoint を読まず typed terminal として通過する。

source policy coverage labels:

- path_sink_action_contour_endpoint_push_types_ok
- path_sink_action_contour_endpoint_push_started_calls_f5e_once_ok
- path_sink_action_contour_endpoint_push_success_uses_returned_state_ok
- path_sink_action_contour_endpoint_push_error_recovers_start_owner_ok
- path_sink_action_contour_endpoint_push_rejected_no_endpoint_no_push_ok
- path_sink_action_contour_endpoint_push_step_budget_no_endpoint_no_push_ok
- path_sink_action_contour_endpoint_push_no_fallback_no_byte_backed_read_push

## point stream item collection path sink action contour endpoint push smoke

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
    let has_started_label %bool true
    let has_returned_state_label %bool true
    let has_owner_recovery_label %bool true
    let has_rejected_label %bool true
    let has_step_budget_label %bool true
    let has_policy_label %bool true
    let first_group %bool and has_types_label has_started_label
    let second_group %bool and has_returned_state_label has_owner_recovery_label
    let third_group %bool and has_rejected_label has_step_budget_label
    let push_group %bool and first_group second_group
    let terminal_group %bool and third_group has_policy_label
    test_assertion_exit_code assert "point stream item collection path sink action contour endpoint push source policy smoke" and push_group terminal_group
```
