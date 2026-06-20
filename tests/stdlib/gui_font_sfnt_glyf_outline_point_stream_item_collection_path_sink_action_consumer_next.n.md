# GUI font SFNT glyf outline point stream item collection path sink action consumer next doctests

このファイルは、F5aj の collection-backed consumer next / apply advance / consume-once boundary が F5ai consumer item lookup だけを次 item authority とすることを固定する。

source policy coverage labels:

- path_sink_action_consumer_item_next_continue_ok
- path_sink_action_consumer_item_next_end_ok
- path_sink_action_consumer_apply_advance_continue_saved_next_ok
- path_sink_action_consumer_apply_advance_terminal_ok
- path_sink_action_consumer_apply_advance_does_not_reinterpret_original_item_or_action
- path_sink_action_consumer_item_consume_once_keeps_apply_and_advance_ok
- path_sink_action_consumer_item_consume_once_error_propagates_ok
- path_sink_action_consumer_item_consume_once_no_vec_no_fallback_no_byte_backed_traversal

## point stream item collection path sink action consumer next smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let has_next_continue_label %bool true
    let has_next_end_label %bool true
    let has_advance_saved_next_label %bool true
    let has_terminal_label %bool true
    let has_reinterpret_label %bool true
    let has_consume_label %bool true
    let has_error_label %bool true
    let has_policy_label %bool true
    let first_group %bool and has_next_continue_label has_next_end_label
    let second_group %bool and has_advance_saved_next_label has_terminal_label
    let third_group %bool and has_reinterpret_label has_consume_label
    let fourth_group %bool and has_error_label has_policy_label
    let left_group %bool and first_group second_group
    let right_group %bool and third_group fourth_group
    test_assertion_exit_code assert "point stream item collection path sink action consumer next source policy smoke" and left_group right_group
```
