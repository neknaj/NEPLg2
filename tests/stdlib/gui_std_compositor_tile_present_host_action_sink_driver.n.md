# GUI std compositor tile RLE present host action sink driver doctests

このファイルは、F5nf の std layer compositor tile RLE present host action sink driver boundary が、F5nf sink preflight と F5ne one-shot driver completion を接続し、rejection では driver pending を caller へ戻すことを固定する。

source policy labels:

- std_compositor_tile_rle_present_host_action_sink_driver_facade_ok
- std_compositor_tile_rle_present_host_action_sink_driver_owner_recovery_ok
- std_compositor_tile_rle_present_host_action_sink_driver_sink_before_completion_ok
- std_compositor_tile_rle_present_host_action_sink_driver_no_manufactured_outcome_ok
- std_compositor_tile_rle_present_host_action_sink_driver_no_direct_bridge_no_platform_no_fallback

## host action sink driver smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present_host_action_sink_driver\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"success completes loop\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"rejection preserves driver\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"host failure wraps driver completion\" expected=\"3\" actual=\"3\" message=\"\"\n"
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
#import "std/gui/compositor_tile_present_host_action_sink" as *
#import "std/gui/compositor_tile_present_host_action_sink_driver" as *
#import "std/gui/compositor_tile_present_host_command" as *
#import "std/gui/compositor_tile_present_host_execution" as *
#import "std/gui/compositor_tile_present_host_execution_driver" as *
#import "std/gui/compositor_tile_present_host_executor" as *
#import "std/gui/compositor_tile_present_host_report_loop_bridge" as *
#import "std/gui/compositor_tile_present_schedule" as *
#import "std/gui/host" as *
#import "std/gui/tile_present" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_compositor_tile_rle_present_host_action_sink_driver_facade_ok
// std_compositor_tile_rle_present_host_action_sink_driver_owner_recovery_ok
// std_compositor_tile_rle_present_host_action_sink_driver_sink_before_completion_ok
// std_compositor_tile_rle_present_host_action_sink_driver_no_manufactured_outcome_ok
// std_compositor_tile_rle_present_host_action_sink_driver_no_direct_bridge_no_platform_no_fallback

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

fn success_completes_loop_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    let driver %GuiRgba8888CompositorTileRlePresentHostExecutionDriverPending gui_rgba8888_compositor_tile_rle_present_host_execution_driver_prepare pending
    match gui_rgba8888_compositor_tile_rle_present_host_action_sink_driver_step GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Offscreen driver Result::Ok unit:
        Result::Ok step:
            let sink_step %GuiRgba8888CompositorTileRlePresentHostActionSinkStep gui_rgba8888_compositor_tile_rle_present_host_action_sink_driver_step_sink_step &step
            match gui_rgba8888_compositor_tile_rle_present_host_action_sink_step_outcome &sink_step:
                Result::Ok _unit:
                    let completion %GuiRgba8888CompositorTileRlePresentDispatchLoopCompletion gui_rgba8888_compositor_tile_rle_present_host_action_sink_driver_step_completion &step
                    match completion:
                        GuiRgba8888CompositorTileRlePresentDispatchLoopCompletion::Continue state:
                            if eq loop_state_command_count &state 1:
                                then 1
                                else -5
                        _:
                            -4
                Result::Err _error:
                    -3
        Result::Err _error:
            -1

fn rejection_preserves_driver_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    let driver %GuiRgba8888CompositorTileRlePresentHostExecutionDriverPending gui_rgba8888_compositor_tile_rle_present_host_execution_driver_prepare pending
    let expected %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_execution_driver_pending_action &driver
    match gui_rgba8888_compositor_tile_rle_present_host_action_sink_driver_step GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Window driver Result::Ok unit:
        Result::Ok _step:
            -1
        Result::Err error:
            match error:
                GuiRgba8888CompositorTileRlePresentHostActionSinkDriverError::SinkRejected rejection:
                    let sink_error %GuiRgba8888CompositorTileRlePresentHostActionSinkError gui_rgba8888_compositor_tile_rle_present_host_action_sink_driver_rejected_sink_error &rejection
                    match gui_rgba8888_compositor_tile_rle_present_host_action_sink_error_kind &sink_error:
                        GuiRgba8888CompositorTileRlePresentHostActionSinkErrorKind::UnsupportedAction _lower:
                            let recovered %GuiRgba8888CompositorTileRlePresentHostExecutionDriverPending gui_rgba8888_compositor_tile_rle_present_host_action_sink_driver_rejected_driver rejection
                            let recovered_action %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_execution_driver_pending_action &recovered
                            if gui_rgba8888_compositor_tile_rle_present_host_executor_action_same &expected &recovered_action:
                                then 2
                                else -4
                _:
                    -2

fn host_failure_wraps_driver_completion_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    let driver %GuiRgba8888CompositorTileRlePresentHostExecutionDriverPending gui_rgba8888_compositor_tile_rle_present_host_execution_driver_prepare pending
    match gui_rgba8888_compositor_tile_rle_present_host_action_sink_driver_step GuiRgba8888CompositorTileRlePresentHostExecutorSupport::All driver Result::Err GuiError::BackendFailure:
        Result::Ok _step:
            -1
        Result::Err error:
            match error:
                GuiRgba8888CompositorTileRlePresentHostActionSinkDriverError::DriverCompletionFailed failure:
                    let sink_step %GuiRgba8888CompositorTileRlePresentHostActionSinkStep gui_rgba8888_compositor_tile_rle_present_host_action_sink_driver_completion_failed_sink_step &failure
                    match gui_rgba8888_compositor_tile_rle_present_host_action_sink_step_outcome &sink_step:
                        Result::Err sink_error:
                            match sink_error:
                                GuiError::BackendFailure:
                                    let driver_error %GuiRgba8888CompositorTileRlePresentHostExecutionDriverError gui_rgba8888_compositor_tile_rle_present_host_action_sink_driver_completion_failed_driver_error &failure
                                    match gui_rgba8888_compositor_tile_rle_present_host_execution_driver_error_kind &driver_error:
                                        GuiRgba8888CompositorTileRlePresentHostExecutionDriverErrorKind::BridgeFailed lower:
                                            match gui_rgba8888_compositor_tile_rle_present_host_report_loop_bridge_error_kind &lower:
                                                GuiRgba8888CompositorTileRlePresentHostReportLoopBridgeErrorKind::LoopCompletionFailed loop_error:
                                                    match gui_rgba8888_compositor_tile_rle_present_dispatch_loop_error_kind &loop_error:
                                                        GuiRgba8888CompositorTileRlePresentDispatchLoopErrorKind::HostImportExecutionFailed host_error:
                                                            match host_error:
                                                                GuiError::BackendFailure:
                                                                    let state %GuiRgba8888CompositorTileRlePresentDispatchLoopState gui_rgba8888_compositor_tile_rle_present_host_execution_driver_error_state &driver_error
                                                                    if eq loop_state_command_count &state 0:
                                                                        then 3
                                                                        else -9
                                                                _:
                                                                    -8
                                                        _:
                                                            -7
                                                _:
                                                    -6
                                _:
                                    -5
                        Result::Ok _unit:
                            -4
                _:
                    -2

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_compositor_tile_present_host_action_sink_driver"
        |> test::test_report_push test::assert_eq_i32 "success completes loop" 1 success_completes_loop_code
        |> test::test_report_push test::assert_eq_i32 "rejection preserves driver" 2 rejection_preserves_driver_code
        |> test::test_report_push test::assert_eq_i32 "host failure wraps driver completion" 3 host_failure_wraps_driver_completion_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
