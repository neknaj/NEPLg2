# GUI std compositor tile RLE present host action executor session doctests

このファイルは、F5nh の std layer compositor tile RLE present host action executor session boundary が、F5ne one-shot driver pending を actual executor 向けの request / complete session に包むことを固定する。

source policy labels:

- std_compositor_tile_rle_present_host_action_executor_session_facade_ok
- std_compositor_tile_rle_present_host_action_executor_session_state_terminal_ok
- std_compositor_tile_rle_present_host_action_executor_session_pending_owner_expected_action_ok
- std_compositor_tile_rle_present_host_action_executor_session_outcome_only_complete_ok
- std_compositor_tile_rle_present_host_action_executor_session_attempt_driver_completion_ok
- std_compositor_tile_rle_present_host_action_executor_session_lower_recovery_authority_ok
- std_compositor_tile_rle_present_host_action_executor_session_no_scheduler_no_platform_no_fallback

## host action executor session behavior

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present_host_action_executor_session\" count=5 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"ready request exposes expected action\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"completed request is terminal\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"complete success returns dispatch completion\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"unsupported lower recovery category\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"executor failure outcome preserved\" expected=\"5\" actual=\"5\" message=\"\"\n"
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
#import "std/gui/compositor_tile_present_host_action_attempt_driver" as *
#import "std/gui/compositor_tile_present_host_action_executor_session" as *
#import "std/gui/compositor_tile_present_host_action_sink" as *
#import "std/gui/compositor_tile_present_host_action_sink_driver" as *
#import "std/gui/compositor_tile_present_host_command" as *
#import "std/gui/compositor_tile_present_host_execution" as *
#import "std/gui/compositor_tile_present_host_execution_driver" as *
#import "std/gui/compositor_tile_present_host_executor" as *
#import "std/gui/compositor_tile_present_schedule" as *
#import "std/gui/host" as *
#import "std/gui/tile_present" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_compositor_tile_rle_present_host_action_executor_session_facade_ok
// std_compositor_tile_rle_present_host_action_executor_session_state_terminal_ok
// std_compositor_tile_rle_present_host_action_executor_session_pending_owner_expected_action_ok
// std_compositor_tile_rle_present_host_action_executor_session_outcome_only_complete_ok
// std_compositor_tile_rle_present_host_action_executor_session_attempt_driver_completion_ok
// std_compositor_tile_rle_present_host_action_executor_session_lower_recovery_authority_ok
// std_compositor_tile_rle_present_host_action_executor_session_no_scheduler_no_platform_no_fallback

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

fn ready_request_exposes_expected_action_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    let driver %GuiRgba8888CompositorTileRlePresentHostExecutionDriverPending gui_rgba8888_compositor_tile_rle_present_host_execution_driver_prepare pending
    let expected %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_execution_driver_pending_action &driver
    let state %GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionState gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_start driver
    match gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_request state:
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Action session_pending:
            let actual %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_pending_expected_action &session_pending
            if gui_rgba8888_compositor_tile_rle_present_host_executor_action_same &expected &actual:
                then 1
                else -2
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Completed:
            -1

fn completed_request_is_terminal_code %fn void i32 \void:
    let state %GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionState GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionState::Completed
    match gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_request state:
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Completed:
            2
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Action _pending:
            -1

fn complete_success_returns_dispatch_completion_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    let driver %GuiRgba8888CompositorTileRlePresentHostExecutionDriverPending gui_rgba8888_compositor_tile_rle_present_host_execution_driver_prepare pending
    let state %GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionState gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_start driver
    match gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_request state:
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Action session_pending:
            let expected %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_pending_expected_action &session_pending
            match gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_complete GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Offscreen session_pending Result::Ok unit:
                Result::Ok completion:
                    let attempt_step %GuiRgba8888CompositorTileRlePresentHostActionAttemptDriverStep gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_completion_attempt_driver_step &completion
                    let attempt %GuiRgba8888CompositorTileRlePresentHostActionAttempt gui_rgba8888_compositor_tile_rle_present_host_action_attempt_driver_step_attempt &attempt_step
                    let attempted %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_action_attempt_action &attempt
                    if gui_rgba8888_compositor_tile_rle_present_host_executor_action_same &expected &attempted:
                        then:
                            match gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_completion_dispatch_loop_completion &completion:
                                GuiRgba8888CompositorTileRlePresentDispatchLoopCompletion::Continue next_state:
                                    if eq loop_state_command_count &next_state 1:
                                        then 3
                                        else -6
                                _:
                                    -5
                        else:
                            -4
                Result::Err _error:
                    -3
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Completed:
            -1

fn unsupported_lower_recovery_category_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    let driver %GuiRgba8888CompositorTileRlePresentHostExecutionDriverPending gui_rgba8888_compositor_tile_rle_present_host_execution_driver_prepare pending
    let state %GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionState gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_start driver
    match gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_request state:
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Action session_pending:
            let expected %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_pending_expected_action &session_pending
            match gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_complete GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Window session_pending Result::Ok unit:
                Result::Ok _completion:
                    -2
                Result::Err error:
                    match gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_complete_error_category_value &error:
                        Option::Some category:
                            match category:
                                GuiError::Unsupported:
                                    let lower %GuiRgba8888CompositorTileRlePresentHostActionAttemptDriverError gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_complete_error_lower error
                                    match lower:
                                        GuiRgba8888CompositorTileRlePresentHostActionAttemptDriverError::SinkDriverFailed failure:
                                            let sink_driver_error %GuiRgba8888CompositorTileRlePresentHostActionSinkDriverError gui_rgba8888_compositor_tile_rle_present_host_action_attempt_sink_driver_failed_lower failure
                                            match sink_driver_error:
                                                GuiRgba8888CompositorTileRlePresentHostActionSinkDriverError::SinkRejected rejection:
                                                    let recovered %GuiRgba8888CompositorTileRlePresentHostExecutionDriverPending gui_rgba8888_compositor_tile_rle_present_host_action_sink_driver_rejected_driver rejection
                                                    let recovered_action %GuiRgba8888CompositorTileRlePresentHostExecutionAction gui_rgba8888_compositor_tile_rle_present_host_execution_driver_pending_action &recovered
                                                    if gui_rgba8888_compositor_tile_rle_present_host_executor_action_same &expected &recovered_action:
                                                        then 4
                                                        else -9
                                                _:
                                                    -8
                                        _:
                                            -7
                                _:
                                    -6
                        Option::None:
                            -5
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Completed:
            -1

fn executor_failure_outcome_preserved_code %fn void i32 \void:
    let pending %GuiRgba8888CompositorTileRlePresentDispatchLoopPendingRequest sample_pending
    let driver %GuiRgba8888CompositorTileRlePresentHostExecutionDriverPending gui_rgba8888_compositor_tile_rle_present_host_execution_driver_prepare pending
    let state %GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionState gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_start driver
    match gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_request state:
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Action session_pending:
            match gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_complete GuiRgba8888CompositorTileRlePresentHostExecutorSupport::All session_pending Result::Err GuiError::BackendFailure:
                Result::Ok _completion:
                    -2
                Result::Err error:
                    let lower %GuiRgba8888CompositorTileRlePresentHostActionAttemptDriverError gui_rgba8888_compositor_tile_rle_present_host_action_executor_session_complete_error_lower error
                    match lower:
                        GuiRgba8888CompositorTileRlePresentHostActionAttemptDriverError::SinkDriverFailed failure:
                            let sink_driver_error %GuiRgba8888CompositorTileRlePresentHostActionSinkDriverError gui_rgba8888_compositor_tile_rle_present_host_action_attempt_sink_driver_failed_lower failure
                            match sink_driver_error:
                                GuiRgba8888CompositorTileRlePresentHostActionSinkDriverError::DriverCompletionFailed completion_failure:
                                    let sink_step %GuiRgba8888CompositorTileRlePresentHostActionSinkStep gui_rgba8888_compositor_tile_rle_present_host_action_sink_driver_completion_failed_sink_step &completion_failure
                                    match gui_rgba8888_compositor_tile_rle_present_host_action_sink_step_outcome &sink_step:
                                        Result::Err sink_error:
                                            match sink_error:
                                                GuiError::BackendFailure:
                                                    5
                                                _:
                                                    -8
                                        Result::Ok _unit:
                                            -7
                                _:
                                    -6
                        _:
                            -5
        GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionRequestResult::Completed:
            -1

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_compositor_tile_present_host_action_executor_session"
        |> test::test_report_push test::assert_eq_i32 "ready request exposes expected action" 1 ready_request_exposes_expected_action_code
        |> test::test_report_push test::assert_eq_i32 "completed request is terminal" 2 completed_request_is_terminal_code
        |> test::test_report_push test::assert_eq_i32 "complete success returns dispatch completion" 3 complete_success_returns_dispatch_completion_code
        |> test::test_report_push test::assert_eq_i32 "unsupported lower recovery category" 4 unsupported_lower_recovery_category_code
        |> test::test_report_push test::assert_eq_i32 "executor failure outcome preserved" 5 executor_failure_outcome_preserved_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
