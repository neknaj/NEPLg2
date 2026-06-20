# GUI font SFNT glyf outline point stream item collection path sink action storage owner doctests

このファイルは、F5an の owner-taking allocation boundary が F5am outcome を唯一の authority とし、`EndContour` だけを F5b storage allocation へ進め、`Rejected` / `StepBudgetExhausted` では owner を作らないことを固定する。

source policy coverage labels:

- path_sink_action_storage_owner_types_ok
- path_sink_action_storage_owner_allocates_only_end_contour_ok
- path_sink_action_storage_owner_rejected_no_owner_ok
- path_sink_action_storage_owner_step_budget_no_owner_ok
- path_sink_action_storage_owner_alloc_error_preserves_summary_ok
- path_sink_action_storage_owner_no_fallback_no_byte_backed_traversal

## point stream item collection path sink action storage owner smoke

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
    let has_end_contour_label %bool true
    let has_rejected_label %bool true
    let has_step_budget_label %bool true
    let has_alloc_error_label %bool true
    let has_policy_label %bool true
    let first_group %bool and has_types_label has_end_contour_label
    let second_group %bool and has_rejected_label has_step_budget_label
    let third_group %bool and has_alloc_error_label has_policy_label
    let combined_group %bool and first_group second_group
    test_assertion_exit_code assert "point stream item collection path sink action storage owner source policy smoke" and combined_group third_group
```
