# GUI render2d row tile RLE packet doctests

このファイルは、F5cm の RGBA8888 row tile RLE packet owner の public import surface を固定する。

source policy labels:

- render2d_row_tile_rle_packet_facade_ok
- render2d_row_tile_rle_packet_prepare_owner_ok
- render2d_row_tile_rle_packet_descriptor_authority_ok
- render2d_row_tile_rle_packet_checked_geometry_ok
- render2d_row_tile_rle_packet_owner_recovery_ok
- render2d_row_tile_rle_packet_no_reader_no_platform_no_fallback

## module import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_tile_rle_packet\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"RLE packet import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "std/test" as test

// render2d_row_tile_rle_packet_facade_ok
// render2d_row_tile_rle_packet_prepare_owner_ok
// render2d_row_tile_rle_packet_descriptor_authority_ok
// render2d_row_tile_rle_packet_checked_geometry_ok
// render2d_row_tile_rle_packet_owner_recovery_ok
// render2d_row_tile_rle_packet_no_reader_no_platform_no_fallback

fn main %impure fn void i32 \void:
    let actual %i32 0
    let report:
        test::test_report_new "gui_render2d_row_tile_rle_packet"
        |> test::test_report_push test::assert_eq_i32 "RLE packet import" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
