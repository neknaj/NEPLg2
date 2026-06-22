# GUI std compositor tile RLE present host action platform bridge doctests

このファイルは、F5ni の std layer compositor tile RLE present host action platform bridge boundary が、F5nh session pending から expected action を読み、platform outcome だけを F5nh outcome-only complete へ戻すことを固定する。

source policy labels:

- std_compositor_tile_rle_present_host_action_platform_bridge_facade_ok
- std_compositor_tile_rle_present_host_action_platform_bridge_pending_action_ok
- std_compositor_tile_rle_present_host_action_platform_bridge_target_record_kind_ok
- std_compositor_tile_rle_present_host_action_platform_bridge_descriptor_projection_ok
- std_compositor_tile_rle_present_host_action_platform_bridge_outcome_only_complete_ok
- std_compositor_tile_rle_present_host_action_platform_bridge_no_platform_no_lower_fallback

## host action platform bridge behavior

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present_host_action_platform_bridge\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"pending action is expected action\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"target and record kind project\" expected=\"109\" actual=\"109\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"descriptor projection preserves metadata\" expected=\"20\" actual=\"20\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"complete outcome returns dispatch completion\" expected=\"4\" actual=\"4\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/compositor_frame_entry" as *
#import "alloc/gui/render2d/row_tile_rle" as *
#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "core/cast" as *
#import "core/gui" as *
#import "core/gui/color" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/gui/compositor_tile_present" as *
#import "std/gui/compositor_tile_present_dispatch" as *
#import "std/gui/compositor_tile_present_dispatch_loop" as *
#import "std/gui/compositor_tile_present_host_action_executor_session" as *
#import "std/gui/compositor_tile_present_host_action_platform_bridge" as *
#import "std/gui/compositor_tile_present_host_command" as *
#import "std/gui/compositor_tile_present_host_execution" as *
#import "std/gui/compositor_tile_present_host_execution_driver" as *
#import "std/gui/compositor_tile_present_host_executor" as *
#import "std/gui/compositor_tile_present_host_import" as *
#import "std/gui/compositor_tile_present_schedule" as *
#import "std/gui/host" as *
#import "std/gui/tile_present" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_compositor_tile_rle_present_host_action_platform_bridge_facade_ok
// std_compositor_tile_rle_present_host_action_platform_bridge_pending_action_ok
// std_compositor_tile_rle_present_host_action_platform_bridge_target_record_kind_ok
// std_compositor_tile_rle_present_host_action_platform_bridge_descriptor_projection_ok
// std_compositor_tile_rle_present_host_action_platform_bridge_outcome_only_complete_ok
// std_compositor_tile_rle_present_host_action_platform_bridge_no_platform_no_lower_fallback

fn sample_present_descriptor %fn void GuiRgba8888RowTileRlePresentDescriptor \void:
    let surface %SurfaceId unwrap_ok surface_id_result 3
    let frame %FrameId unwrap_ok frame_id_result 4
    let packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor 4 0 0 0 1 0 1 3 1 12 1 1 3 1 12
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet

fn sample_compositor_descriptor %fn void GuiRgba8888CompositorTileRlePresentFrameDescriptor \void:
    let present %GuiRgba8888RowTileRlePresentDescriptor sample_present_descriptor
    let metadata %GuiRgba8888CompositorFrameEntryMetadata GuiRgba8888CompositorFrameEntryMetadata 4 20 30 0 1 1 4
    GuiRgba8888CompositorTileRlePresentFrameDescriptor present metadata

fn sample_run_record %fn void GuiRgba8888CompositorTileRlePresentHostCommandRunRecord \void:
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor sample_compositor_descriptor
    let r %u8 cast 11
    let g %u8 cast 12
    let b %u8 cast 13
    let a %u8 cast 255
    let color %Rgba8888 rgba8888_new r g b a
    let run %GuiRgba8888RowTileRleRun GuiRgba8888RowTileRleRun 0 2 color
    gui_rgba8888_compositor_tile_rle_present_host_command_run_record descriptor run

fn sample_window_run_action %fn void GuiRgba8888CompositorTileRlePresentHostExecutionAction \void:
    let window %WindowId unwrap_ok window_id_result 7
    let run_record %GuiRgba8888CompositorTileRlePresentHostCommandRunRecord sample_run_record
    let record %GuiRgba8888CompositorTileRlePresentHostCommandRecord GuiRgba8888CompositorTileRlePresentHostCommandRecord::RunRecord run_record
    let target %GuiRgba8888CompositorTileRlePresentHostImportTarget GuiRgba8888CompositorTileRlePresentHostImportTarget::Window window
    let request %GuiRgba8888CompositorTileRlePresentHostImportRequest GuiRgba8888CompositorTileRlePresentHostImportRequest target record
    gui_rgba8888_compositor_tile_rle_present_host_execution_action &request

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

fn record_kind_code %fn GuiRgba8888CompositorTileRlePresentHostActionPlatformRecordKind i32 \kind:
    match kind:
        GuiRgba8888CompositorTileRlePresentHostActionPlatformRecordKind::Begin:
            1
        GuiRgba8888CompositorTileRlePresentHostActionPlatformRecordKind::Run:
            2
        GuiRgba8888CompositorTileRlePresentHostActionPlatformRecordKind::End:
            3

fn pending_action_is_expected_action_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    let driver %GuiRgba8888CompositorTileRlePresentHostExecutionDriverPending gui_rgba8888_compositor_tile_rle_present_host_execution_driver_prepare pending
    let expected %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_execution_driver_pending_action &driver
    let state %GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionState gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_start driver
    match gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_request state:
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Action session_pending:
            let actual %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_action_platform_bridge_pending_action &session_pending
            if gui_rgba8888_compositor_tile_rle_present_host_executor_action_same &expected &actual:
                then 1
                else -2
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Completed:
            -1

fn target_and_record_kind_project_code %fn void i32 \void:
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction sample_window_run_action
    let target %GuiRgba8888CompositorTileRlePresentHostActionPlatformTarget gui_rgba8888_compositor_tile_rle_present_host_action_platform_bridge_action_target &action
    let target_for_window %GuiRgba8888CompositorTileRlePresentHostActionPlatformTarget gui_rgba8888_compositor_tile_rle_present_host_action_platform_bridge_action_target &action
    let kind %GuiRgba8888CompositorTileRlePresentHostActionPlatformRecordKind gui_rgba8888_compositor_tile_rle_present_host_action_platform_bridge_action_record_kind &action
    let target_kind %i32 gui_rgba8888_compositor_tile_rle_present_host_action_platform_bridge_target_kind target
    let window_raw %i32 gui_rgba8888_compositor_tile_rle_present_host_action_platform_bridge_target_window_raw &target_for_window
    let record_code %i32 record_kind_code kind
    add add mul target_kind 100 window_raw record_code

fn descriptor_projection_preserves_metadata_code %fn void i32 \void:
    let action %GuiRgba8888CompositorTileRlePresentHostExecutionAction sample_window_run_action
    let descriptor %GuiRgba8888CompositorTileRlePresentFrameDescriptor gui_rgba8888_compositor_tile_rle_present_host_action_platform_bridge_action_descriptor &action
    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_rle_present_frame_descriptor_metadata &descriptor
    gui_rgba8888_compositor_frame_entry_metadata_width &metadata

fn complete_outcome_returns_dispatch_completion_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    let driver %GuiRgba8888CompositorTileRlePresentHostExecutionDriverPending gui_rgba8888_compositor_tile_rle_present_host_execution_driver_prepare pending
    let state %GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionState gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_start driver
    match gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_request state:
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Action session_pending:
            match gui_rgba8888_compositor_tile_rle_present_host_action_platform_bridge_complete_outcome GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Offscreen session_pending Result::Ok unit:
                Result::Ok completion:
                    match gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_completion_dispatch_loop_completion &completion:
                        GuiRgba8888CompositorTileRlePresentDispatchLoopCompletion::Continue next_state:
                            if eq loop_state_command_count &next_state 1:
                                then 4
                                else -4
                        _:
                            -3
                Result::Err _error:
                    -2
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Completed:
            -1

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_compositor_tile_present_host_action_platform_bridge"
        |> test::test_report_push test::assert_eq_i32 "pending action is expected action" 1 pending_action_is_expected_action_code
        |> test::test_report_push test::assert_eq_i32 "target and record kind project" 109 target_and_record_kind_project_code
        |> test::test_report_push test::assert_eq_i32 "descriptor projection preserves metadata" 20 descriptor_projection_preserves_metadata_code
        |> test::test_report_push test::assert_eq_i32 "complete outcome returns dispatch completion" 4 complete_outcome_returns_dispatch_completion_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
