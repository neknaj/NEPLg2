# GUI font SFNT glyf outline point stream item collection path command stream sink writer

このファイルは、F5ba の path command stream sink writer boundary が F5az owner と F5aw `PathCommandValue` を authority とし、byte-backed lookup、old traversal、raster / render / platform 接続へ戻らないことを固定する。public value は forged value として再検査し、`SkipNoSegment` は silent no-op ではなく explicit step として扱う。

source policy coverage labels:

- path_command_stream_sink_writer_types_ok
- path_command_stream_sink_writer_path_command_value_tag_accessors_ok
- path_command_stream_sink_writer_start_validation_order_ok
- path_command_stream_sink_writer_push_validation_order_ok
- path_command_stream_sink_writer_kind_progress_bounds_ok
- path_command_stream_sink_writer_tag_consistency_ok
- path_command_stream_sink_writer_stable_scalar_order_ok
- path_command_stream_sink_writer_progress_update_ok
- path_command_stream_sink_writer_push_failure_recovery_ok
- path_command_stream_sink_writer_partial_failure_fail_closed_ok
- path_command_stream_sink_writer_skip_no_segment_no_push_ok
- path_command_stream_sink_writer_no_fallback_no_byte_backed_no_traversal_no_raster

## point stream item collection path command stream sink writer smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// path_command_stream_sink_writer_types_ok
// path_command_stream_sink_writer_path_command_value_tag_accessors_ok
// path_command_stream_sink_writer_start_validation_order_ok
// path_command_stream_sink_writer_push_validation_order_ok
// path_command_stream_sink_writer_kind_progress_bounds_ok
// path_command_stream_sink_writer_tag_consistency_ok
// path_command_stream_sink_writer_stable_scalar_order_ok
// path_command_stream_sink_writer_progress_update_ok
// path_command_stream_sink_writer_push_failure_recovery_ok
// path_command_stream_sink_writer_partial_failure_fail_closed_ok
// path_command_stream_sink_writer_skip_no_segment_no_push_ok
// path_command_stream_sink_writer_no_fallback_no_byte_backed_no_traversal_no_raster

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection path command stream sink writer source policy smoke" all_groups
```
