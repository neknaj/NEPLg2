# GUI std row tile RLE present scheduled span operation doctests

このファイルは、F5dh の std layer RGBA8888 row tile RLE present scheduled span operation boundary の public import surface を固定する。

behavior order と禁止依存は `nodesrc/test_web_gui_font_rendering_contract.js` が source policy として検査する。ここでは heavy action scenario を再構築せず、doctest timeout を避けるため import smoke だけを実行する。

source policy labels:

- std_row_tile_rle_present_scheduled_span_operation_facade_ok
- std_row_tile_rle_present_scheduled_span_operation_policy_result_ok
- std_row_tile_rle_present_scheduled_span_operation_uses_f5dg_authority_ok
- std_row_tile_rle_present_scheduled_span_operation_exact_yield_keeps_operation_ok
- std_row_tile_rle_present_scheduled_span_operation_resume_slice_keeps_cursor_ok
- std_row_tile_rle_present_scheduled_span_operation_over_budget_typed_error_ok
- std_row_tile_rle_present_scheduled_span_operation_no_f5ct_no_raw_no_platform_no_fallback

## import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_scheduled_span_operation\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"std tile present scheduled span operation import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/tile_present_scheduled_span_operation" as *
#import "std/test" as test

// std_row_tile_rle_present_scheduled_span_operation_facade_ok
// std_row_tile_rle_present_scheduled_span_operation_policy_result_ok
// std_row_tile_rle_present_scheduled_span_operation_uses_f5dg_authority_ok
// std_row_tile_rle_present_scheduled_span_operation_exact_yield_keeps_operation_ok
// std_row_tile_rle_present_scheduled_span_operation_resume_slice_keeps_cursor_ok
// std_row_tile_rle_present_scheduled_span_operation_over_budget_typed_error_ok
// std_row_tile_rle_present_scheduled_span_operation_no_f5ct_no_raw_no_platform_no_fallback

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_tile_present_scheduled_span_operation"
        |> test::test_report_push test::assert_eq_i32 "std tile present scheduled span operation import" 0 0
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
