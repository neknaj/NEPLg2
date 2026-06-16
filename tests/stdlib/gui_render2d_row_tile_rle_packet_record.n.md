# GUI render2d row tile RLE packet record doctests

このファイルは、F5co の RGBA8888 row tile RLE packet typed record reader boundary の public import surface を固定する。

source policy labels:

- render2d_row_tile_rle_packet_record_facade_ok
- render2d_row_tile_rle_packet_record_typed_reader_ok
- render2d_row_tile_rle_packet_record_checked_counts_ok
- render2d_row_tile_rle_packet_record_checked_run_extent_ok
- render2d_row_tile_rle_packet_record_raw_read_quarantined_ok
- render2d_row_tile_rle_packet_record_no_platform_no_fallback

## module import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_tile_rle_packet_record\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"RLE packet record import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/row_tile_rle_packet_record" as *
#import "std/test" as test

// render2d_row_tile_rle_packet_record_facade_ok
// render2d_row_tile_rle_packet_record_typed_reader_ok
// render2d_row_tile_rle_packet_record_checked_counts_ok
// render2d_row_tile_rle_packet_record_checked_run_extent_ok
// render2d_row_tile_rle_packet_record_raw_read_quarantined_ok
// render2d_row_tile_rle_packet_record_no_platform_no_fallback

fn main %impure fn void i32 \void:
    let actual %i32 0
    let report:
        test::test_report_new "gui_render2d_row_tile_rle_packet_record"
        |> test::test_report_push test::assert_eq_i32 "RLE packet record import" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
