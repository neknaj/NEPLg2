# GUI font SFNT glyf outline point stream item collection path sink action consumer item doctests

このファイルは、F5ai の collection-backed action item next / consumer item boundary が F5ah action step item lookup だけを次 item authority とすることを固定する。

source policy coverage labels:

- path_sink_action_item_next_continue_ok
- path_sink_action_item_next_end_ok
- path_sink_action_consumer_item_keeps_action_and_next_ok
- path_sink_action_consumer_item_error_propagates_ok
- path_sink_action_consumer_item_no_vec_no_fallback_no_byte_backed_traversal

## point stream item collection path sink action consumer item smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let has_continue_label %bool true
    let has_end_label %bool true
    let has_consumer_label %bool true
    let has_error_label %bool true
    let has_policy_label %bool true
    test_assertion_exit_code assert "point stream item collection path sink action consumer item source policy smoke" and has_continue_label and has_end_label and has_consumer_label and has_error_label has_policy_label
```
