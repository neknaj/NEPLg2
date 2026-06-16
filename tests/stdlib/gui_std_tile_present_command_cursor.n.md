# GUI std row tile RLE present command cursor doctests

このファイルは、F5cp の std layer RGBA8888 row tile RLE present command cursor boundary の public import surface を固定する。

source policy labels:

- std_row_tile_rle_present_command_cursor_facade_ok
- std_row_tile_rle_present_command_cursor_command_stream_ok
- std_row_tile_rle_present_command_cursor_owner_boundary_ok
- std_row_tile_rle_present_command_cursor_owner_recovery_ok
- std_row_tile_rle_present_command_cursor_one_output_step_ok
- std_row_tile_rle_present_command_cursor_uses_f5co_ok
- std_row_tile_rle_present_command_cursor_no_raw_no_host_no_platform_no_fallback

## module import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_command_cursor\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"std tile present command cursor import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/tile_present_command_cursor" as *
#import "std/test" as test

// std_row_tile_rle_present_command_cursor_facade_ok
// std_row_tile_rle_present_command_cursor_command_stream_ok
// std_row_tile_rle_present_command_cursor_owner_boundary_ok
// std_row_tile_rle_present_command_cursor_owner_recovery_ok
// std_row_tile_rle_present_command_cursor_one_output_step_ok
// std_row_tile_rle_present_command_cursor_uses_f5co_ok
// std_row_tile_rle_present_command_cursor_no_raw_no_host_no_platform_no_fallback

fn main %impure fn void i32 \void:
    let actual %i32 0
    let report:
        test::test_report_new "gui_std_tile_present_command_cursor"
        |> test::test_report_push test::assert_eq_i32 "std tile present command cursor import" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
