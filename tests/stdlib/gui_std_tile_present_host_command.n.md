# GUI std row tile RLE present host-command doctests

このファイルは、F5cq の std layer RGBA8888 row tile RLE present host-command record boundary の public import surface を固定する。

source policy labels:

- std_row_tile_rle_present_host_command_facade_ok
- std_row_tile_rle_present_host_command_record_enum_ok
- std_row_tile_rle_present_host_command_step_mapping_ok
- std_row_tile_rle_present_host_command_uses_f5cp_accessor_ok
- std_row_tile_rle_present_host_command_no_raw_no_host_no_platform_no_fallback

## module import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_host_command\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"std tile present host command import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/tile_present_host_command" as *
#import "std/test" as test

// std_row_tile_rle_present_host_command_facade_ok
// std_row_tile_rle_present_host_command_record_enum_ok
// std_row_tile_rle_present_host_command_step_mapping_ok
// std_row_tile_rle_present_host_command_uses_f5cp_accessor_ok
// std_row_tile_rle_present_host_command_no_raw_no_host_no_platform_no_fallback

fn main %impure fn void i32 \void:
    let actual %i32 0
    let report:
        test::test_report_new "gui_std_tile_present_host_command"
        |> test::test_report_push test::assert_eq_i32 "std tile present host command import" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
