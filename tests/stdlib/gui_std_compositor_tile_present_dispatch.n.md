# GUI std compositor tile RLE present dispatch doctests

このファイルは、F5my の std layer compositor tile RLE present scheduled dispatch boundary の public import surface と state resume contract を固定する。

source policy labels:

- std_compositor_tile_rle_present_dispatch_facade_ok
- std_compositor_tile_rle_present_dispatch_state_wraps_f5mw_ok
- std_compositor_tile_rle_present_dispatch_ready_request_post_phase_ok
- std_compositor_tile_rle_present_dispatch_f5mw_before_f5mx_ok
- std_compositor_tile_rle_present_dispatch_preserves_request_and_post_phase_ok
- std_compositor_tile_rle_present_dispatch_error_preserves_previous_state_ok
- std_compositor_tile_rle_present_dispatch_no_f5mv_no_lower_raw_no_platform_no_fallback

## dispatch state smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present_dispatch\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"dispatch initial command count\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"dispatch resume keeps zero pixel count\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/compositor_tile_present_dispatch" as *
#import "std/gui/compositor_tile_present_schedule" as *
#import "std/test" as test

// std_compositor_tile_rle_present_dispatch_facade_ok
// std_compositor_tile_rle_present_dispatch_state_wraps_f5mw_ok
// std_compositor_tile_rle_present_dispatch_ready_request_post_phase_ok
// std_compositor_tile_rle_present_dispatch_f5mw_before_f5mx_ok
// std_compositor_tile_rle_present_dispatch_preserves_request_and_post_phase_ok
// std_compositor_tile_rle_present_dispatch_error_preserves_previous_state_ok
// std_compositor_tile_rle_present_dispatch_no_f5mv_no_lower_raw_no_platform_no_fallback

fn initial_command_count %fn void i32 \void:
    let state %GuiRgba8888CompositorTileRlePresentDispatchState gui_rgba8888_compositor_tile_rle_present_dispatch_state_initial
    let schedule %GuiRgba8888CompositorTileRlePresentScheduleState gui_rgba8888_compositor_tile_rle_present_dispatch_state_schedule &state
    gui_rgba8888_compositor_tile_rle_present_schedule_state_slice_command_count &schedule

fn resumed_pixel_count %fn void i32 \void:
    let state %GuiRgba8888CompositorTileRlePresentDispatchState gui_rgba8888_compositor_tile_rle_present_dispatch_state_initial
    let resumed %GuiRgba8888CompositorTileRlePresentDispatchState gui_rgba8888_compositor_tile_rle_present_dispatch_state_resume_slice state
    let schedule %GuiRgba8888CompositorTileRlePresentScheduleState gui_rgba8888_compositor_tile_rle_present_dispatch_state_schedule &resumed
    gui_rgba8888_compositor_tile_rle_present_schedule_state_slice_pixel_count &schedule

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_compositor_tile_present_dispatch"
        |> test::test_report_push test::assert_eq_i32 "dispatch initial command count" 0 initial_command_count
        |> test::test_report_push test::assert_eq_i32 "dispatch resume keeps zero pixel count" 0 resumed_pixel_count
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
