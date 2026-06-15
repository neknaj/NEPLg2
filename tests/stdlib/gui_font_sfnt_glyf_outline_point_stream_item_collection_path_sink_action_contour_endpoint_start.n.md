# GUI font SFNT glyf outline point stream item collection path sink action contour endpoint start doctests

このファイルは、F5ao の contour endpoint start boundary が F5an storage terminal を authority とし、`Allocated` branch だけを F5d cursor start へ進めることを固定する。capacity mismatch と cursor start failure は original owner を返し、`Rejected` / `StepBudgetExhausted` は cursor を開始せず typed terminal として通過する。

source policy coverage labels:

- path_sink_action_contour_endpoint_start_types_ok
- path_sink_action_contour_endpoint_start_borrows_storage_capacity_ok
- path_sink_action_contour_endpoint_start_capacity_mismatch_recovers_owner_ok
- path_sink_action_contour_endpoint_start_allocated_starts_cursor_ok
- path_sink_action_contour_endpoint_start_rejected_no_cursor_ok
- path_sink_action_contour_endpoint_start_step_budget_no_cursor_ok
- path_sink_action_contour_endpoint_start_no_fallback_no_byte_backed_no_push

## point stream item collection path sink action contour endpoint start smoke

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
    let has_borrowed_capacity_label %bool true
    let has_owner_recovery_label %bool true
    let has_allocated_cursor_label %bool true
    let has_rejected_label %bool true
    let has_step_budget_label %bool true
    let has_policy_label %bool true
    let first_group %bool and has_types_label has_borrowed_capacity_label
    let second_group %bool and has_owner_recovery_label has_allocated_cursor_label
    let third_group %bool and has_rejected_label has_step_budget_label
    let prefix_group %bool and first_group second_group
    let terminal_group %bool and third_group has_policy_label
    test_assertion_exit_code assert "point stream item collection path sink action contour endpoint start source policy smoke" and prefix_group terminal_group
```
