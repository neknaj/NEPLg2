# GUI std row tile RLE present host span operation presenter executor session turn virtual timer doctests

このファイルは、F5dz の std layer RGBA8888 row tile RLE present host span operation presenter executor session turn virtual timer bridge の public import surface を固定する。

behavior order と禁止依存は `nodesrc/test_web_gui_font_rendering_contract.js` と `nodesrc/test_web_gui_offscreen_headless_contract.js` が source policy として検査する。ここでは actual timer backend や scheduler backend を再構築せず、doctest timeout を避けるため import smoke だけを実行する。

source policy labels:

- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_facade_ok
- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_schedule_owner_recovery_ok
- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_advance_owner_recovery_ok
- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_timer_complete_state_recovery_ok
- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_unexpected_event_error_ok
- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_exact_authority_calls_ok
- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_no_loop_no_backend_no_queue_no_fallback

## import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"std tile present host span operation presenter executor session turn virtual timer import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer" as *
#import "std/test" as test

// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_facade_ok
// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_schedule_owner_recovery_ok
// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_advance_owner_recovery_ok
// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_timer_complete_state_recovery_ok
// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_unexpected_event_error_ok
// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_exact_authority_calls_ok
// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_no_loop_no_backend_no_queue_no_fallback

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer"
        |> test::test_report_push test::assert_eq_i32 "std tile present host span operation presenter executor session turn virtual timer import" 0 0
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
