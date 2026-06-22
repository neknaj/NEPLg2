# GUI std compositor tile RLE present host report loop bridge doctests

このファイルは、F5nd の std layer compositor tile RLE present host report loop bridge が、executor report を validation 後に dispatch loop completion へ戻すことを固定する。

source policy labels:

- std_compositor_tile_rle_present_host_report_loop_bridge_facade_ok
- std_compositor_tile_rle_present_host_report_loop_bridge_f5nc_f5mz_f5na_f5nb_ok
- std_compositor_tile_rle_present_host_report_loop_bridge_validation_before_completion_ok
- std_compositor_tile_rle_present_host_report_loop_bridge_failed_report_completion_error_ok
- std_compositor_tile_rle_present_host_report_loop_bridge_state_preserved_ok
- std_compositor_tile_rle_present_host_report_loop_bridge_no_f5my_f5mw_f5mx_f5mv_no_lower_no_platform_no_fallback

## host report loop bridge smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present_host_report_loop_bridge\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"success report completes loop\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"failed report becomes loop error\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"unsupported support stops before completion\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"wrong report action stops before completion\" expected=\"4\" actual=\"4\" message=\"\"\n"
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
#import "std/gui/compositor_tile_present_host_execution" as *
#import "std/gui/compositor_tile_present_host_execution_report" as *
#import "std/gui/compositor_tile_present_host_executor" as *
#import "std/gui/compositor_tile_present_host_report_loop_bridge" as *
#import "std/gui/compositor_tile_present_schedule" as *
#import "std/gui/host" as *
#import "std/gui/tile_present" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_compositor_tile_rle_present_host_report_loop_bridge_facade_ok
// std_compositor_tile_rle_present_host_report_loop_bridge_f5nc_f5mz_f5na_f5nb_ok
// std_compositor_tile_rle_present_host_report_loop_bridge_validation_before_completion_ok
// std_compositor_tile_rle_present_host_report_loop_bridge_failed_report_completion_error_ok
// std_compositor_tile_rle_present_host_report_loop_bridge_state_preserved_ok
// std_compositor_tile_rle_present_host_report_loop_bridge_no_f5my_f5mw_f5mx_f5mv_no_lower_no_platform_no_fallback

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

fn report_for_pending %fn &GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest fn Result unit GuiError GuiRgba8888CompositorTileRlePresentHostExecutionReport \pending\outcome:
    let request gui_rgba8888_compositor_tile_rle_present_dispatch_loop_pending_request pending
    gui_rgba8888_compositor_tile_rle_present_host_execution_report_for_request &request outcome

fn wrong_end_report %fn void GuiRgba8888CompositorTileRlePresentHostExecutionReport \void:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_compositor_descriptor
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction GuiRgba8888CompositorTileRlePresentHostExecutionAction::DeviceEnd descriptor
    gui_rgba8888_compositor_tile_rle_present_host_execution_report action Result::Ok unit

fn success_report_completes_loop_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    let report %GuiRgba8888CompositorTileRlePresentHostExecutionReport report_for_pending &pending Result::Ok unit
    match gui_rgba8888_compositor_tile_rle_present_host_report_loop_bridge_complete GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Offscreen pending report:
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

fn failed_report_becomes_loop_error_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    let report %GuiRgba8888CompositorTileRlePresentHostExecutionReport report_for_pending &pending Result::Err GuiError::BackendFailure
    match gui_rgba8888_compositor_tile_rle_present_host_report_loop_bridge_complete GuiRgba8888CompositorTileRlePresentHostExecutorSupport::All pending report:
        Result::Ok _completion:
            -1
        Result::Err error:
            match gui_rgba8888_compositor_tile_rle_present_host_report_loop_bridge_error_kind &error:
                GuiRgba8888CompositorTileRlePresentHostReportLoopBridgeErrorKind::LoopCompletionFailed lower:
                    match gui_rgba8888_compositor_tile_rle_present_dispatch_loop_error_kind &lower:
                        GuiRgba8888CompositorTileRlePresentDispatchLoopErrorKind::HostImportExecutionFailed host_error:
                            match host_error:
                                GuiError::BackendFailure:
                                    let state %GuiRgba8888CompositorTileRlePresentDispatchLoopState gui_rgba8888_compositor_tile_rle_present_host_report_loop_bridge_error_state &error
                                    if eq loop_state_command_count &state 0:
                                        then 2
                                        else -5
                                _:
                                    -4
                        _:
                            -3
                _:
                    -2

fn unsupported_support_stops_before_completion_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    let report %GuiRgba8888CompositorTileRlePresentHostExecutionReport report_for_pending &pending Result::Ok unit
    match gui_rgba8888_compositor_tile_rle_present_host_report_loop_bridge_complete GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Window pending report:
        Result::Ok _completion:
            -1
        Result::Err error:
            match gui_rgba8888_compositor_tile_rle_present_host_report_loop_bridge_error_kind &error:
                GuiRgba8888CompositorTileRlePresentHostReportLoopBridgeErrorKind::ExecutorValidationFailed lower:
                    match gui_rgba8888_compositor_tile_rle_present_host_executor_error_kind &lower:
                        GuiRgba8888CompositorTileRlePresentHostExecutorErrorKind::UnsupportedAction:
                            let state %GuiRgba8888CompositorTileRlePresentDispatchLoopState gui_rgba8888_compositor_tile_rle_present_host_report_loop_bridge_error_state &error
                            if eq loop_state_command_count &state 0:
                                then 3
                                else -4
                        _:
                            -3
                _:
                    -2

fn wrong_report_action_stops_before_completion_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    let report %GuiRgba8888CompositorTileRlePresentHostExecutionReport wrong_end_report
    match gui_rgba8888_compositor_tile_rle_present_host_report_loop_bridge_complete GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Offscreen pending report:
        Result::Ok _completion:
            -1
        Result::Err error:
            match gui_rgba8888_compositor_tile_rle_present_host_report_loop_bridge_error_kind &error:
                GuiRgba8888CompositorTileRlePresentHostReportLoopBridgeErrorKind::ExecutorValidationFailed lower:
                    match gui_rgba8888_compositor_tile_rle_present_host_executor_error_kind &lower:
                        GuiRgba8888CompositorTileRlePresentHostExecutorErrorKind::ReportActionMismatch:
                            let state %GuiRgba8888CompositorTileRlePresentDispatchLoopState gui_rgba8888_compositor_tile_rle_present_host_report_loop_bridge_error_state &error
                            if eq loop_state_command_count &state 0:
                                then 4
                                else -4
                        _:
                            -3
                _:
                    -2

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_compositor_tile_present_host_report_loop_bridge"
        |> test::test_report_push test::assert_eq_i32 "success report completes loop" 1 success_report_completes_loop_code
        |> test::test_report_push test::assert_eq_i32 "failed report becomes loop error" 2 failed_report_becomes_loop_error_code
        |> test::test_report_push test::assert_eq_i32 "unsupported support stops before completion" 3 unsupported_support_stops_before_completion_code
        |> test::test_report_push test::assert_eq_i32 "wrong report action stops before completion" 4 wrong_report_action_stops_before_completion_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
