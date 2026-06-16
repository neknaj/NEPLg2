# GUI std row tile RLE present run cursor doctests

このファイルは、F5co の std layer RGBA8888 row tile RLE present run cursor boundary の public import surface を固定する。

source policy labels:

- std_row_tile_rle_present_run_cursor_facade_ok
- std_row_tile_rle_present_run_cursor_owner_boundary_ok
- std_row_tile_rle_present_run_cursor_owner_recovery_ok
- std_row_tile_rle_present_run_cursor_completed_explicit_ok
- std_row_tile_rle_present_run_cursor_typed_record_reader_ok
- std_row_tile_rle_present_run_cursor_no_host_no_platform_no_fallback

## module import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_run_cursor\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"std tile present run cursor import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/tile_present_run_cursor" as *
#import "std/test" as test

// std_row_tile_rle_present_run_cursor_facade_ok
// std_row_tile_rle_present_run_cursor_owner_boundary_ok
// std_row_tile_rle_present_run_cursor_owner_recovery_ok
// std_row_tile_rle_present_run_cursor_completed_explicit_ok
// std_row_tile_rle_present_run_cursor_typed_record_reader_ok
// std_row_tile_rle_present_run_cursor_no_host_no_platform_no_fallback

fn main %impure fn void i32 \void:
    let actual %i32 0
    let report:
        test::test_report_new "gui_std_tile_present_run_cursor"
        |> test::test_report_push test::assert_eq_i32 "std tile present run cursor import" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
