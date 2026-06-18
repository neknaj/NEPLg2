# GUI std row tile RLE present host-import scheduler-start doctests

このファイルは、F5gc の std layer RGBA8888 row tile RLE present host import scheduler start boundary の public import surface を固定する。

source policy labels:

- std_row_tile_rle_present_host_import_scheduler_start_facade_ok
- std_row_tile_rle_present_host_import_scheduler_start_request_to_action_ok
- std_row_tile_rle_present_host_import_scheduler_start_turn_start_order_ok
- std_row_tile_rle_present_host_import_scheduler_start_dynamic_timer_state_ok
- std_row_tile_rle_present_host_import_scheduler_start_empty_timer_explicit_ok
- std_row_tile_rle_present_host_import_scheduler_start_error_context_ok
- std_row_tile_rle_present_host_import_scheduler_start_no_step_loop_backend_queue_fallback

## module import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_host_import_scheduler_start\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"std tile present host import scheduler start\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/tile_present_host_import_scheduler_start" as *
#import "std/test" as test

// std_row_tile_rle_present_host_import_scheduler_start_facade_ok
// std_row_tile_rle_present_host_import_scheduler_start_request_to_action_ok
// std_row_tile_rle_present_host_import_scheduler_start_turn_start_order_ok
// std_row_tile_rle_present_host_import_scheduler_start_dynamic_timer_state_ok
// std_row_tile_rle_present_host_import_scheduler_start_empty_timer_explicit_ok
// std_row_tile_rle_present_host_import_scheduler_start_error_context_ok
// std_row_tile_rle_present_host_import_scheduler_start_no_step_loop_backend_queue_fallback

fn main %impure fn void i32 \void:
    let actual %i32 0
    let report:
        test::test_report_new "gui_std_tile_present_host_import_scheduler_start"
        |> test::test_report_push test::assert_eq_i32 "std tile present host import scheduler start" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
