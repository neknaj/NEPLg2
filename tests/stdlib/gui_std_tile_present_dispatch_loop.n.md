# GUI std row tile RLE present dispatch loop doctests

このファイルは、F5cv の std layer RGBA8888 row tile RLE present dispatch loop outcome boundary の public import surface と state resume contract を固定する。

source policy labels:

- std_row_tile_rle_present_dispatch_loop_facade_ok
- std_row_tile_rle_present_dispatch_loop_state_wraps_f5cu_ok
- std_row_tile_rle_present_dispatch_loop_pending_one_shot_ok
- std_row_tile_rle_present_dispatch_loop_previous_next_state_ok
- std_row_tile_rle_present_dispatch_loop_complete_request_outcome_ok
- std_row_tile_rle_present_dispatch_loop_error_preserves_rollback_state_ok
- std_row_tile_rle_present_dispatch_loop_no_direct_lower_no_raw_no_platform_no_fallback

## dispatch loop state smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_dispatch_loop\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"dispatch loop initial command count\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"dispatch loop resume keeps zero pixel count\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/tile_present_dispatch" as *
#import "std/gui/tile_present_dispatch_loop" as *
#import "std/gui/tile_present_schedule" as *
#import "std/test" as test

// std_row_tile_rle_present_dispatch_loop_facade_ok
// std_row_tile_rle_present_dispatch_loop_state_wraps_f5cu_ok
// std_row_tile_rle_present_dispatch_loop_pending_one_shot_ok
// std_row_tile_rle_present_dispatch_loop_previous_next_state_ok
// std_row_tile_rle_present_dispatch_loop_complete_request_outcome_ok
// std_row_tile_rle_present_dispatch_loop_error_preserves_rollback_state_ok
// std_row_tile_rle_present_dispatch_loop_no_direct_lower_no_raw_no_platform_no_fallback

fn initial_command_count %fn void i32 \void:
    let state %GuiRgba8888RowTileRlePresentDispatchLoopState gui_rgba8888_row_tile_rle_present_dispatch_loop_state_initial
    let dispatch %GuiRgba8888RowTileRlePresentDispatchState gui_rgba8888_row_tile_rle_present_dispatch_loop_state_dispatch &state
    let schedule %GuiRgba8888RowTileRlePresentScheduleState gui_rgba8888_row_tile_rle_present_dispatch_state_schedule &dispatch
    gui_rgba8888_row_tile_rle_present_schedule_state_slice_command_count &schedule

fn resumed_pixel_count %fn void i32 \void:
    let state %GuiRgba8888RowTileRlePresentDispatchLoopState gui_rgba8888_row_tile_rle_present_dispatch_loop_state_initial
    let resumed %GuiRgba8888RowTileRlePresentDispatchLoopState gui_rgba8888_row_tile_rle_present_dispatch_loop_state_resume_slice state
    let dispatch %GuiRgba8888RowTileRlePresentDispatchState gui_rgba8888_row_tile_rle_present_dispatch_loop_state_dispatch &resumed
    let schedule %GuiRgba8888RowTileRlePresentScheduleState gui_rgba8888_row_tile_rle_present_dispatch_state_schedule &dispatch
    gui_rgba8888_row_tile_rle_present_schedule_state_slice_pixel_count &schedule

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_tile_present_dispatch_loop"
        |> test::test_report_push test::assert_eq_i32 "dispatch loop initial command count" 0 initial_command_count
        |> test::test_report_push test::assert_eq_i32 "dispatch loop resume keeps zero pixel count" 0 resumed_pixel_count
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
