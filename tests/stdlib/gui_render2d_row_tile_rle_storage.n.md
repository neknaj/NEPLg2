# GUI render2d row tile RLE storage doctests

このファイルは、F5cj の RGBA8888 row tile RLE encoded storage owner と、F5ck の exact run writer cursor の public import surface を固定する。
runtime で重い row surface pipeline を再実行せず、writer の詳細契約は `nodesrc/test_web_gui_font_rendering_contract.js` の source policy で固定する。

source policy coverage labels:

- render2d_row_tile_rle_storage_facade_ok
- render2d_row_tile_rle_storage_writer_plan_to_storage_ok
- render2d_row_tile_rle_storage_exact_byte_count_ok
- render2d_row_tile_rle_storage_prepare_error_owner_recovery_ok
- render2d_row_tile_rle_storage_allocation_only_no_write_no_platform_no_fallback
- render2d_row_tile_rle_write_cursor_start_ok
- render2d_row_tile_rle_write_cursor_step_three_runs_ok
- render2d_row_tile_rle_write_cursor_completion_ok
- render2d_row_tile_rle_write_cursor_no_reader_no_platform_no_fallback

## module import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_tile_rle_storage\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"RLE storage import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/row_tile_rle_storage" as *
#import "std/test" as test

// render2d_row_tile_rle_storage_facade_ok
// render2d_row_tile_rle_storage_writer_plan_to_storage_ok
// render2d_row_tile_rle_storage_exact_byte_count_ok
// render2d_row_tile_rle_storage_prepare_error_owner_recovery_ok
// render2d_row_tile_rle_storage_allocation_only_no_write_no_platform_no_fallback
// render2d_row_tile_rle_write_cursor_start_ok
// render2d_row_tile_rle_write_cursor_step_three_runs_ok
// render2d_row_tile_rle_write_cursor_completion_ok
// render2d_row_tile_rle_write_cursor_no_reader_no_platform_no_fallback

fn main %impure fn void i32 \void:
    let actual %i32 0
    let report:
        test::test_report_new "gui_render2d_row_tile_rle_storage"
        |> test::test_report_push test::assert_eq_i32 "RLE storage import" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
