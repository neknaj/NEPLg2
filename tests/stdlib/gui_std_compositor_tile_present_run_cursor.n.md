# GUI std compositor tile RLE present run cursor bridge doctests

このファイルは、F5mr の std layer compositor tile RLE present run cursor bridge が compositor present-frame owner から lower F5co run cursor start へ進み、command cursor / step / packet record / host / platform / fallback へ進まないことを固定する。

source policy labels:

- std_compositor_tile_rle_present_run_cursor_facade_ok
- std_compositor_tile_rle_present_run_cursor_start_error_kind_runtime_ok
- std_compositor_tile_rle_present_run_cursor_start_calls_lower_f5co_ok
- std_compositor_tile_rle_present_run_cursor_metadata_before_owner_move_ok
- std_compositor_tile_rle_present_run_cursor_owner_recovery_ok
- std_compositor_tile_rle_present_run_cursor_free_delegates_run_cursor_ok
- std_compositor_tile_rle_present_run_cursor_no_command_step_record_host_platform_fallback

## facade and wrapped kind smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present_run_cursor\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"run cursor start kind\" expected=\"37\" actual=\"37\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/compositor_tile_present_run_cursor" as *
#import "std/gui/tile_present_run_cursor" as *
#import "std/test" as test

// std_compositor_tile_rle_present_run_cursor_facade_ok
// std_compositor_tile_rle_present_run_cursor_start_error_kind_runtime_ok

fn start_error_kind_code %fn GuiRgba8888CompositorTileRlePresentRunCursorStartErrorKind i32 \kind:
    match kind:
        GuiRgba8888CompositorTileRlePresentRunCursorStartErrorKind::RunCursorStartFailed lower:
            match lower:
                GuiRgba8888RowTileRlePresentRunCursorStartErrorKind::EncodedByteCountMismatch:
                    37
                _:
                    0
        GuiRgba8888CompositorTileRlePresentRunCursorStartErrorKind::PresentFrameRewrapFailed _:
            41

fn main %impure fn void i32 \void:
    let lower %GuiRgba8888RowTileRlePresentRunCursorStartErrorKind GuiRgba8888RowTileRlePresentRunCursorStartErrorKind::EncodedByteCountMismatch
    let kind %GuiRgba8888CompositorTileRlePresentRunCursorStartErrorKind GuiRgba8888CompositorTileRlePresentRunCursorStartErrorKind::RunCursorStartFailed lower
    let report:
        test::test_report_new "gui_std_compositor_tile_present_run_cursor"
        |> test::test_report_push test::assert_eq_i32 "run cursor start kind" 37 start_error_kind_code kind
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## source-policy fixtures

neplg2:test[skip]
```neplg2
#indent 4

#import "std/gui/compositor_tile_present_run_cursor" as *

// std_compositor_tile_rle_present_run_cursor_start_calls_lower_f5co_ok
// std_compositor_tile_rle_present_run_cursor_metadata_before_owner_move_ok
// std_compositor_tile_rle_present_run_cursor_owner_recovery_ok
// std_compositor_tile_rle_present_run_cursor_free_delegates_run_cursor_ok
// std_compositor_tile_rle_present_run_cursor_no_command_step_record_host_platform_fallback
```
