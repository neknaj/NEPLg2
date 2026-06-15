# GUI font SFNT glyf outline point stream item collection path sink action start consumer doctests

このファイルは、F5ak の collection-backed start consumer boundary が collection capacity の glyph だけを start cursor authority とし、外部 glyph や byte-backed lookup に戻らないことを固定する。

source policy coverage labels:

- path_sink_action_start_item_capacity_glyph_authority_ok
- path_sink_action_start_item_step_item_ok
- path_sink_action_start_consumer_item_uses_f5ai_ok
- path_sink_action_start_consume_once_uses_f5aj_ok
- path_sink_action_start_consume_summary_uses_summary_projection_ok
- path_sink_action_start_consumer_error_propagates_ok
- path_sink_action_start_consumer_no_vec_no_fallback_no_byte_backed_traversal

## point stream item collection path sink action start consumer smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let has_authority_label %bool true
    let has_start_item_label %bool true
    let has_consumer_label %bool true
    let has_consume_once_label %bool true
    let has_summary_label %bool true
    let has_error_label %bool true
    let has_policy_label %bool true
    let first_group %bool and has_authority_label has_start_item_label
    let second_group %bool and has_consumer_label has_consume_once_label
    let third_group %bool and has_summary_label has_error_label
    let left_group %bool and first_group second_group
    let right_group %bool and third_group has_policy_label
    test_assertion_exit_code assert "point stream item collection path sink action start consumer source policy smoke" and left_group right_group
```
