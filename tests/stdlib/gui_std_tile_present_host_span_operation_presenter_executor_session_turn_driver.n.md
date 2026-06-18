# GUI std row tile RLE present host span operation presenter executor session turn driver doctests

このファイルは、F5du の std layer RGBA8888 row tile RLE present host span operation presenter executor session turn driver boundary の public import surface を固定する。

behavior order と禁止依存は `nodesrc/test_web_gui_font_rendering_contract.js` が source policy として検査する。ここでは actual scheduler や executor scenario を再構築せず、doctest timeout を避けるため import smoke だけを実行する。

source policy labels:

- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_facade_ok
- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_pending_owner_ok
- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_borrowed_operation_ok
- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_outcome_only_complete_ok
- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_poll_complete_order_ok
- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_lower_recovery_authority_ok
- std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_no_scheduler_no_platform_no_fallback

## import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_host_span_operation_presenter_executor_session_turn_driver\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"std tile present host span operation presenter executor session turn driver import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/tile_present_host_span_operation_presenter_executor_session_turn_driver" as *
#import "std/test" as test

// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_facade_ok
// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_pending_owner_ok
// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_borrowed_operation_ok
// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_outcome_only_complete_ok
// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_poll_complete_order_ok
// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_lower_recovery_authority_ok
// std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_no_scheduler_no_platform_no_fallback

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_tile_present_host_span_operation_presenter_executor_session_turn_driver"
        |> test::test_report_push test::assert_eq_i32 "std tile present host span operation presenter executor session turn driver import" 0 0
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
