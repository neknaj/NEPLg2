# GUI std compositor tile RLE present dispatch loop doctests

このファイルは、F5nc の std layer compositor tile RLE present dispatch loop が、scheduled dispatch request を one-shot pending value として保持し、host outcome を一回だけ state に反映することを固定する。

source policy labels:

- std_compositor_tile_rle_present_dispatch_loop_facade_ok
- std_compositor_tile_rle_present_dispatch_loop_pending_owner_ok
- std_compositor_tile_rle_present_dispatch_loop_f5my_only_ok
- std_compositor_tile_rle_present_dispatch_loop_complete_request_outcome_ok
- std_compositor_tile_rle_present_dispatch_loop_failure_preserves_previous_state_ok
- std_compositor_tile_rle_present_dispatch_loop_no_lower_no_platform_no_fallback

## dispatch loop smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present_dispatch_loop\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"success outcome advances loop\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"failed outcome preserves previous state\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"resume keeps zero pixel count\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/compositor_frame_entry" as *
#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "core/gui" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/gui/compositor_tile_present" as *
#import "std/gui/compositor_tile_present_dispatch" as *
#import "std/gui/compositor_tile_present_dispatch_loop" as *
#import "std/gui/compositor_tile_present_host_command" as *
#import "std/gui/compositor_tile_present_schedule" as *
#import "std/gui/host" as *
#import "std/gui/tile_present" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_compositor_tile_rle_present_dispatch_loop_facade_ok
// std_compositor_tile_rle_present_dispatch_loop_pending_owner_ok
// std_compositor_tile_rle_present_dispatch_loop_f5my_only_ok
// std_compositor_tile_rle_present_dispatch_loop_complete_request_outcome_ok
// std_compositor_tile_rle_present_dispatch_loop_failure_preserves_previous_state_ok
// std_compositor_tile_rle_present_dispatch_loop_no_lower_no_platform_no_fallback

fn sample_present_descriptor %fn void GuiRgba8888RowTileRlePresentDescriptor \void:
    let surface %SurfaceId unwrap_ok surface_id_result 3
    let frame %FrameId unwrap_ok frame_id_result 4
    let packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor 4 0 0 0 1 0 1 3 1 12 1 1 3 1 12
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet

fn sample_compositor_descriptor %fn void GuiRgba8888CompositorTileRlePresentFrameDescriptor \void:
    let present %GuiRgba8888RowTileRlePresentDescriptor sample_present_descriptor
    let metadata %GuiRgba8888CompositorFrameEntryMetadata GuiRgba8888CompositorFrameEntryMetadata 4 20 30 0 1 1 4
    GuiRgba8888CompositorTileRlePresentFrameDescriptor present metadata

fn sample_record %fn void GuiRgba8888CompositorTileRlePresentHostCommandRecord \void:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_compositor_descriptor
    GuiRgba8888CompositorTileRlePresentHostCommandRecord::BeginFrame descriptor

fn sample_host %fn void GuiHost \void:
    let capabilities %GuiCapabilities gui_capabilities_offscreen_pixel 16 16
    gui_host_new capabilities Option::None

fn sample_policy %fn void GuiRgba8888CompositorTileRlePresentSchedulePolicy \void:
    unwrap_ok gui_rgba8888_compositor_tile_rle_present_schedule_policy 8 256

fn sample_pending %fn void GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest \void:
    let host %GuiHost sample_host
    let policy %GuiRgba8888CompositorTileRlePresentSchedulePolicy sample_policy
    let state %GuiRgba8888CompositorTileRlePresentDispatchLoopState gui_rgba8888_compositor_tile_rle_present_dispatch_loop_state_initial
    let record %GuiRgba8888CompositorTileRlePresentHostCommandRecord sample_record
    let step %GuiRgba8888CompositorTileRlePresentDispatchLoopStep unwrap_ok gui_rgba8888_compositor_tile_rle_present_dispatch_loop_step_record &host &policy state record
    gui_rgba8888_compositor_tile_rle_present_dispatch_loop_step_pending step

fn loop_state_command_count %fn &GuiRgba8888CompositorTileRlePresentDispatchLoopState i32 \state:
    let dispatch %GuiRgba8888CompositorTileRlePresentDispatchState gui_rgba8888_compositor_tile_rle_present_dispatch_loop_state_dispatch state
    let schedule %GuiRgba8888CompositorTileRlePresentScheduleState gui_rgba8888_compositor_tile_rle_present_dispatch_state_schedule &dispatch
    gui_rgba8888_compositor_tile_rle_present_schedule_state_slice_command_count &schedule

fn loop_state_pixel_count %fn &GuiRgba8888CompositorTileRlePresentDispatchLoopState i32 \state:
    let dispatch %GuiRgba8888CompositorTileRlePresentDispatchState gui_rgba8888_compositor_tile_rle_present_dispatch_loop_state_dispatch state
    let schedule %GuiRgba8888CompositorTileRlePresentScheduleState gui_rgba8888_compositor_tile_rle_present_dispatch_state_schedule &dispatch
    gui_rgba8888_compositor_tile_rle_present_schedule_state_slice_pixel_count &schedule

fn success_outcome_advances_loop_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    match gui_rgba8888_compositor_tile_rle_present_dispatch_loop_complete_request pending Result::Ok unit:
        Result::Ok completion:
            match completion:
                GuiRgba8888CompositorTileRlePresentDispatchLoopCompletion::Continue state:
                    if eq loop_state_command_count &state 1:
                        then 1
                        else -3
                _:
                    -2
        Result::Err _error:
            -1

fn failed_outcome_preserves_previous_state_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    match gui_rgba8888_compositor_tile_rle_present_dispatch_loop_complete_request pending Result::Err GuiError::BackendFailure:
        Result::Ok _completion:
            -1
        Result::Err error:
            match gui_rgba8888_compositor_tile_rle_present_dispatch_loop_error_kind &error:
                GuiRgba8888CompositorTileRlePresentDispatchLoopErrorKind::HostImportExecutionFailed host_error:
                    match host_error:
                        GuiError::BackendFailure:
                            let state %GuiRgba8888CompositorTileRlePresentDispatchLoopState gui_rgba8888_compositor_tile_rle_present_dispatch_loop_error_state &error
                            if eq loop_state_command_count &state 0:
                                then 2
                                else -4
                        _:
                            -3
                _:
                    -2

fn resume_keeps_zero_pixel_count_code %fn void i32 \void:
    let state %GuiRgba8888CompositorTileRlePresentDispatchLoopState gui_rgba8888_compositor_tile_rle_present_dispatch_loop_state_initial
    let resumed %GuiRgba8888CompositorTileRlePresentDispatchLoopState gui_rgba8888_compositor_tile_rle_present_dispatch_loop_state_resume_slice state
    loop_state_pixel_count &resumed

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_compositor_tile_present_dispatch_loop"
        |> test::test_report_push test::assert_eq_i32 "success outcome advances loop" 1 success_outcome_advances_loop_code
        |> test::test_report_push test::assert_eq_i32 "failed outcome preserves previous state" 2 failed_outcome_preserves_previous_state_code
        |> test::test_report_push test::assert_eq_i32 "resume keeps zero pixel count" 0 resume_keeps_zero_pixel_count_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
