# GUI font SFNT glyf outline point stream item collection path command value doctests

このファイルは、F5av の path command value lookup boundary が F5au の PathCommandTagCompleteOwner を borrow authority とし、summary / storage / collection / requested index の照合後だけ PathCommandTag scalar と collection-backed source event を読むことを固定する。payload は source event から再導出し、stream construction や raster / render には進まない。

source policy coverage labels:

- path_command_value_types_ok
- path_command_value_authority_checks_ok
- path_command_value_path_command_tag_scalar_read_ok
- path_command_value_complete_owner_non_consuming_ok
- path_command_value_edge_owner_non_consuming_ok
- path_command_value_source_event_exactly_once_ok
- path_command_value_source_kind_from_event_ok
- path_command_value_tag_mismatch_error_ok
- path_command_value_skip_reason_rederived_from_source_event_ok
- path_command_value_no_fallback_no_byte_backed_no_stream_no_raster

## point stream item collection path command value smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection path command value source policy smoke" all_groups
```
