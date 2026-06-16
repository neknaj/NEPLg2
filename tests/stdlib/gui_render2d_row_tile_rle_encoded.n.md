# GUI render2d row tile RLE encoded doctests

このファイルは、F5cl の RGBA8888 row tile RLE sealed encoded owner の public import surface を固定する。
runtime で重い row surface pipeline を再実行せず、詳細契約は `nodesrc/test_web_gui_font_rendering_contract.js` の source policy で固定する。

source policy coverage labels:

- render2d_row_tile_rle_encoded_facade_ok
- render2d_row_tile_rle_encoded_seal_owner_ok
- render2d_row_tile_rle_encoded_counts_before_cursor_status_ok
- render2d_row_tile_rle_encoded_writer_not_complete_ok
- render2d_row_tile_rle_encoded_owner_recovery_ok
- render2d_row_tile_rle_encoded_no_reader_no_platform_no_fallback

## module import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_tile_rle_encoded\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"RLE encoded import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/row_tile_rle_encoded" as *
#import "std/test" as test

// render2d_row_tile_rle_encoded_facade_ok
// render2d_row_tile_rle_encoded_seal_owner_ok
// render2d_row_tile_rle_encoded_counts_before_cursor_status_ok
// render2d_row_tile_rle_encoded_writer_not_complete_ok
// render2d_row_tile_rle_encoded_owner_recovery_ok
// render2d_row_tile_rle_encoded_no_reader_no_platform_no_fallback

fn main %impure fn void i32 \void:
    let actual %i32 0
    let report:
        test::test_report_new "gui_render2d_row_tile_rle_encoded"
        |> test::test_report_push test::assert_eq_i32 "RLE encoded import" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
