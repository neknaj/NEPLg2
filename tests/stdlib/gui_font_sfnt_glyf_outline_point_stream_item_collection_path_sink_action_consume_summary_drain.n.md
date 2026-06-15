# GUI font SFNT glyf outline point stream item collection path sink action consume summary drain doctests

このファイルは、F5al の collection-backed consume summary drain boundary が F5ak start summary と F5aj consume-once だけを traversal authority とし、byte-backed lookup や renderer/platform へ戻らないことを固定する。

source policy coverage labels:

- path_sink_action_consume_summary_advance_once_continue_ok
- path_sink_action_consume_summary_advance_once_terminal_ok
- path_sink_action_consume_summary_advance_once_uses_f5aj_ok
- path_sink_action_consume_summary_drain_budget_zero_negative_ok
- path_sink_action_consume_summary_drain_budget_continue_recurses_ok
- path_sink_action_start_consume_summary_drain_uses_f5ak_ok
- path_sink_action_consume_summary_drain_no_vec_no_fallback_no_byte_backed_traversal

## point stream item collection path sink action consume summary drain smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let has_advance_label %bool true
    let has_terminal_label %bool true
    let has_f5aj_label %bool true
    let has_budget_label %bool true
    let has_recurse_label %bool true
    let has_f5ak_label %bool true
    let has_policy_label %bool true
    let first_group %bool and has_advance_label has_terminal_label
    let second_group %bool and has_f5aj_label has_budget_label
    let third_group %bool and has_recurse_label has_f5ak_label
    let left_group %bool and first_group second_group
    let right_group %bool and third_group has_policy_label
    test_assertion_exit_code assert "point stream item collection path sink action consume summary drain source policy smoke" and left_group right_group
```
