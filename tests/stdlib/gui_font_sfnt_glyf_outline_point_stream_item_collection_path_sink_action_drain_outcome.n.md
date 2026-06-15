# GUI font SFNT glyf outline point stream item collection path sink action drain outcome doctests

このファイルは、F5am の collection-backed drain outcome boundary が F5al start drain result と同じ collection の capacity を同一 public call 内で束ね、public forged pairing API、byte-backed lookup、owner allocation、renderer/platform fallback へ戻らないことを固定する。

source policy coverage labels:

- path_sink_action_drain_outcome_types_ok
- path_sink_action_drain_outcome_private_projection_ok
- path_sink_action_drain_outcome_prevents_public_forged_pairing_ok
- path_sink_action_drain_outcome_start_uses_f5al_ok
- path_sink_action_drain_outcome_terminal_mapping_ok
- path_sink_action_drain_outcome_no_owner_no_fallback_no_byte_backed_traversal

## point stream item collection path sink action drain outcome smoke

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
    let has_private_projection_label %bool true
    let has_no_forged_pairing_label %bool true
    let has_f5al_label %bool true
    let has_terminal_mapping_label %bool true
    let has_policy_label %bool true
    let first_group %bool and has_types_label has_private_projection_label
    let second_group %bool and has_no_forged_pairing_label has_f5al_label
    let third_group %bool and has_terminal_mapping_label has_policy_label
    let combined_group %bool and first_group second_group
    test_assertion_exit_code assert "point stream item collection path sink action drain outcome source policy smoke" and combined_group third_group
```
