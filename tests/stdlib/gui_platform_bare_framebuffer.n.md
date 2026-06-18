# GUI platform bare framebuffer adapter doctests

このファイルは、F5fl の Bare display framebuffer adapter が pure validation と host-executor wrapper を分け、Begin / RunSpan / End の state machine を `Result` と enum で固定することを確認する。

source policy labels:

- platform_bare_framebuffer_facade_ok
- platform_bare_framebuffer_state_machine_ok
- platform_bare_framebuffer_valid_sequence_ok
- platform_bare_framebuffer_run_without_begin_ok
- platform_bare_framebuffer_target_mismatch_ok
- platform_bare_framebuffer_gap_overlap_ok
- platform_bare_framebuffer_incomplete_end_ok
- platform_bare_framebuffer_active_invariant_ok
- platform_bare_framebuffer_host_wrapper_no_new_extern
- platform_bare_framebuffer_no_loop_queue_fallback

## framebuffer adapter contract

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_framebuffer\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "platforms/gui/bare/framebuffer" as *
#import "std/gui/tile_present" as *
#import "std/gui/tile_present_host_span_operation" as *
#import "std/gui/tile_present_run_span" as *
#import "std/gui/window" as *
#import "std/test" as test

// platform_bare_framebuffer_facade_ok
// platform_bare_framebuffer_state_machine_ok
// platform_bare_framebuffer_valid_sequence_ok
// platform_bare_framebuffer_run_without_begin_ok
// platform_bare_framebuffer_target_mismatch_ok
// platform_bare_framebuffer_gap_overlap_ok
// platform_bare_framebuffer_descriptor_contract_ok
// platform_bare_framebuffer_bounds_and_count_ok
// platform_bare_framebuffer_incomplete_end_ok
// platform_bare_framebuffer_active_invariant_ok
// platform_bare_framebuffer_host_execution_failed_ok
// platform_bare_framebuffer_host_wrapper_no_new_extern
// platform_bare_framebuffer_no_loop_queue_fallback

fn test_color %fn void Rgba8888 \void:
    let r %u8 cast 10
    let g %u8 cast 20
    let b %u8 cast 30
    let a %u8 cast 255
    rgba8888_new r g b a

fn sample_packet_descriptor %fn i32 fn i32 GuiRgba8888RowTileRlePacketDescriptor \frame_raw\total_run_count:
    let encoded_byte_count %i32 mul total_run_count 12
    GuiRgba8888RowTileRlePacketDescriptor frame_raw 0 0 0 2 0 2 3 2 12 2 1 6 total_run_count encoded_byte_count

fn descriptor_for %fn SurfaceId fn FrameId fn i32 GuiRgba8888RowTileRlePresentDescriptor \surface\frame\total_run_count:
    let frame_raw %i32 frame_id_raw &frame
    let packet %GuiRgba8888RowTileRlePacketDescriptor sample_packet_descriptor frame_raw total_run_count
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet

fn descriptor_for_packet %fn SurfaceId fn FrameId fn GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePresentDescriptor \surface\frame\packet:
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet

fn span_at %fn i32 fn i32 fn i32 GuiRgba8888RowTileRlePresentRunRowSpan \x\y\width:
    GuiRgba8888RowTileRlePresentRunRowSpan x y width test_color

fn kind_is_run_without_begin %fn GuiBareFramebufferErrorKind bool \kind:
    match kind:
        GuiBareFramebufferErrorKind::RunWithoutBegin:
            true
        _:
            false

fn kind_is_target_mismatch %fn GuiBareFramebufferErrorKind bool \kind:
    match kind:
        GuiBareFramebufferErrorKind::TargetMismatch:
            true
        _:
            false

fn kind_is_run_gap %fn GuiBareFramebufferErrorKind bool \kind:
    match kind:
        GuiBareFramebufferErrorKind::RunGap:
            true
        _:
            false

fn kind_is_incomplete_end %fn GuiBareFramebufferErrorKind bool \kind:
    match kind:
        GuiBareFramebufferErrorKind::IncompleteEnd:
            true
        _:
            false

fn kind_is_run_overlap %fn GuiBareFramebufferErrorKind bool \kind:
    match kind:
        GuiBareFramebufferErrorKind::RunOverlap:
            true
        _:
            false

fn kind_is_run_out_of_bounds %fn GuiBareFramebufferErrorKind bool \kind:
    match kind:
        GuiBareFramebufferErrorKind::RunOutOfBounds:
            true
        _:
            false

fn kind_is_active_seen_run_count_invalid %fn GuiBareFramebufferErrorKind bool \kind:
    match kind:
        GuiBareFramebufferErrorKind::ActiveSeenRunCountInvalid:
            true
        _:
            false

fn kind_is_pixel_count_exceeded %fn GuiBareFramebufferErrorKind bool \kind:
    match kind:
        GuiBareFramebufferErrorKind::PixelCountExceeded:
            true
        _:
            false

fn kind_is_descriptor_frame_mismatch %fn GuiBareFramebufferErrorKind bool \kind:
    match kind:
        GuiBareFramebufferErrorKind::DescriptorInvalid lower:
            match lower:
                GuiRgba8888RowTileRlePresentFramePrepareErrorKind::FrameIdMismatch:
                    true
                _:
                    false
        _:
            false

fn kind_is_descriptor_row_extent_out_of_bounds %fn GuiBareFramebufferErrorKind bool \kind:
    match kind:
        GuiBareFramebufferErrorKind::DescriptorInvalid lower:
            match lower:
                GuiRgba8888RowTileRlePresentFramePrepareErrorKind::RowExtentOutOfBounds:
                    true
                _:
                    false
        _:
            false

fn kind_is_descriptor_pixel_count_mismatch %fn GuiBareFramebufferErrorKind bool \kind:
    match kind:
        GuiBareFramebufferErrorKind::DescriptorInvalid lower:
            match lower:
                GuiRgba8888RowTileRlePresentFramePrepareErrorKind::PixelCountMismatch:
                    true
                _:
                    false
        _:
            false

fn kind_is_host_execution_unsupported %fn GuiBareFramebufferErrorKind bool \kind:
    match kind:
        GuiBareFramebufferErrorKind::HostExecutionFailed lower:
            match lower:
                GuiError::Unsupported:
                    true
                _:
                    false
        _:
            false

fn valid_sequence_case %fn SurfaceId fn FrameId bool \surface\frame:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for surface frame 2
            let state0 %GuiBareFramebufferState gui_bare_framebuffer_state_initial config
            let begin %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceBegin descriptor
            match gui_bare_framebuffer_validate_operation state0 begin:
                Result::Err _:
                    false
                Result::Ok applied0:
                    let state1 %GuiBareFramebufferState gui_bare_framebuffer_step_applied_state &applied0
                    let run0 %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceRunSpan span_at 0 0 3
                    match gui_bare_framebuffer_validate_operation state1 run0:
                        Result::Err _:
                            false
                        Result::Ok applied1:
                            let state2 %GuiBareFramebufferState gui_bare_framebuffer_step_applied_state &applied1
                            let run1 %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceRunSpan span_at 0 1 3
                            match gui_bare_framebuffer_validate_operation state2 run1:
                                Result::Err _:
                                    false
                                Result::Ok applied2:
                                    let state3 %GuiBareFramebufferState gui_bare_framebuffer_step_applied_state &applied2
                                    let end %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceEnd descriptor
                                    match gui_bare_framebuffer_validate_operation state3 end:
                                        Result::Err _:
                                            false
                                        Result::Ok applied3:
                                            let final_state %GuiBareFramebufferState gui_bare_framebuffer_step_applied_state &applied3
                                            match gui_bare_framebuffer_state_phase &final_state:
                                                GuiBareFramebufferPhase::Active _:
                                                    false
                                                GuiBareFramebufferPhase::Idle:
                                                    match gui_bare_framebuffer_state_last_completed_frame &final_state:
                                                        Option::None:
                                                            false
                                                        Option::Some completed:
                                                            eq frame_id_raw &completed frame_id_raw &frame

fn run_without_begin_case %fn SurfaceId fn FrameId bool \surface\frame:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let state %GuiBareFramebufferState gui_bare_framebuffer_state_initial config
            let run0 %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceRunSpan span_at 0 0 3
            match gui_bare_framebuffer_validate_operation state run0:
                Result::Ok _:
                    false
                Result::Err error:
                    kind_is_run_without_begin gui_bare_framebuffer_step_error_kind &error

fn target_mismatch_case %fn SurfaceId fn FrameId fn WindowId bool \surface\frame\window:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for surface frame 2
            let state0 %GuiBareFramebufferState gui_bare_framebuffer_state_initial config
            let begin %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceBegin descriptor
            match gui_bare_framebuffer_validate_operation state0 begin:
                Result::Err _:
                    false
                Result::Ok applied0:
                    let state1 %GuiBareFramebufferState gui_bare_framebuffer_step_applied_state &applied0
                    let window_payload %GuiRgba8888RowTileRlePresentHostSpanOperationWindowRunSpan GuiRgba8888RowTileRlePresentHostSpanOperationWindowRunSpan window span_at 0 0 3
                    let run_window %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::WindowRunSpan window_payload
                    match gui_bare_framebuffer_validate_operation state1 run_window:
                        Result::Ok _:
                            false
                        Result::Err error:
                            kind_is_target_mismatch gui_bare_framebuffer_step_error_kind &error

fn gap_case %fn SurfaceId fn FrameId bool \surface\frame:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for surface frame 2
            let state0 %GuiBareFramebufferState gui_bare_framebuffer_state_initial config
            let begin %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceBegin descriptor
            match gui_bare_framebuffer_validate_operation state0 begin:
                Result::Err _:
                    false
                Result::Ok applied0:
                    let state1 %GuiBareFramebufferState gui_bare_framebuffer_step_applied_state &applied0
                    let run_gap %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceRunSpan span_at 1 0 2
                    match gui_bare_framebuffer_validate_operation state1 run_gap:
                        Result::Ok _:
                            false
                        Result::Err error:
                            kind_is_run_gap gui_bare_framebuffer_step_error_kind &error

fn descriptor_frame_mismatch_case %fn SurfaceId fn FrameId bool \surface\frame:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let frame_raw %i32 frame_id_raw &frame
            let bad_packet %GuiRgba8888RowTileRlePacketDescriptor sample_packet_descriptor add frame_raw 1 2
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for_packet surface frame bad_packet
            let state %GuiBareFramebufferState gui_bare_framebuffer_state_initial config
            let begin %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceBegin descriptor
            match gui_bare_framebuffer_validate_operation state begin:
                Result::Ok _:
                    false
                Result::Err error:
                    kind_is_descriptor_frame_mismatch gui_bare_framebuffer_step_error_kind &error

fn descriptor_row_extent_case %fn SurfaceId fn FrameId bool \surface\frame:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let frame_raw %i32 frame_id_raw &frame
            let bad_packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor frame_raw 0 0 0 2 0 3 3 2 12 2 1 9 3 36
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for_packet surface frame bad_packet
            let state %GuiBareFramebufferState gui_bare_framebuffer_state_initial config
            let begin %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceBegin descriptor
            match gui_bare_framebuffer_validate_operation state begin:
                Result::Ok _:
                    false
                Result::Err error:
                    kind_is_descriptor_row_extent_out_of_bounds gui_bare_framebuffer_step_error_kind &error

fn descriptor_pixel_count_case %fn SurfaceId fn FrameId bool \surface\frame:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let frame_raw %i32 frame_id_raw &frame
            let bad_packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor frame_raw 0 0 0 2 0 2 3 2 12 2 1 5 2 24
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for_packet surface frame bad_packet
            let state %GuiBareFramebufferState gui_bare_framebuffer_state_initial config
            let begin %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceBegin descriptor
            match gui_bare_framebuffer_validate_operation state begin:
                Result::Ok _:
                    false
                Result::Err error:
                    kind_is_descriptor_pixel_count_mismatch gui_bare_framebuffer_step_error_kind &error

fn overlap_case %fn SurfaceId fn FrameId bool \surface\frame:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for surface frame 3
            let state0 %GuiBareFramebufferState gui_bare_framebuffer_state_initial config
            let begin %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceBegin descriptor
            match gui_bare_framebuffer_validate_operation state0 begin:
                Result::Err _:
                    false
                Result::Ok applied0:
                    let state1 %GuiBareFramebufferState gui_bare_framebuffer_step_applied_state &applied0
                    let run0 %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceRunSpan span_at 0 0 2
                    match gui_bare_framebuffer_validate_operation state1 run0:
                        Result::Err _:
                            false
                        Result::Ok applied1:
                            let state2 %GuiBareFramebufferState gui_bare_framebuffer_step_applied_state &applied1
                            let overlap_run %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceRunSpan span_at 1 0 1
                            match gui_bare_framebuffer_validate_operation state2 overlap_run:
                                Result::Ok _:
                                    false
                                Result::Err error:
                                    kind_is_run_overlap gui_bare_framebuffer_step_error_kind &error

fn run_out_of_bounds_case %fn SurfaceId fn FrameId bool \surface\frame:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for surface frame 2
            let state0 %GuiBareFramebufferState gui_bare_framebuffer_state_initial config
            let begin %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceBegin descriptor
            match gui_bare_framebuffer_validate_operation state0 begin:
                Result::Err _:
                    false
                Result::Ok applied0:
                    let state1 %GuiBareFramebufferState gui_bare_framebuffer_step_applied_state &applied0
                    let out_run %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceRunSpan span_at 2 0 2
                    match gui_bare_framebuffer_validate_operation state1 out_run:
                        Result::Ok _:
                            false
                        Result::Err error:
                            kind_is_run_out_of_bounds gui_bare_framebuffer_step_error_kind &error

fn incomplete_end_case %fn SurfaceId fn FrameId bool \surface\frame:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for surface frame 2
            let state0 %GuiBareFramebufferState gui_bare_framebuffer_state_initial config
            let begin %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceBegin descriptor
            match gui_bare_framebuffer_validate_operation state0 begin:
                Result::Err _:
                    false
                Result::Ok applied0:
                    let state1 %GuiBareFramebufferState gui_bare_framebuffer_step_applied_state &applied0
                    let run0 %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceRunSpan span_at 0 0 3
                    match gui_bare_framebuffer_validate_operation state1 run0:
                        Result::Err _:
                            false
                        Result::Ok applied1:
                            let state2 %GuiBareFramebufferState gui_bare_framebuffer_step_applied_state &applied1
                            let end %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceEnd descriptor
                            match gui_bare_framebuffer_validate_operation state2 end:
                                Result::Ok _:
                                    false
                                Result::Err error:
                                    kind_is_incomplete_end gui_bare_framebuffer_step_error_kind &error

fn forged_active_descriptor_case %fn SurfaceId fn FrameId bool \surface\frame:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let frame_raw %i32 frame_id_raw &frame
            let bad_packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor frame_raw 0 0 0 2 0 3 3 2 12 2 1 9 3 36
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for_packet surface frame bad_packet
            let active %GuiBareFramebufferActiveSequence GuiBareFramebufferActiveSequence GuiRgba8888RowTileRlePresentHostSpanOperationTarget::Device descriptor 0 0
            let state %GuiBareFramebufferState GuiBareFramebufferState config GuiBareFramebufferPhase::Active active Option::None
            let run0 %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceRunSpan span_at 0 0 3
            match gui_bare_framebuffer_validate_operation state run0:
                Result::Ok _:
                    false
                Result::Err error:
                    kind_is_descriptor_row_extent_out_of_bounds gui_bare_framebuffer_step_error_kind &error

fn forged_active_negative_seen_case %fn SurfaceId fn FrameId bool \surface\frame:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for surface frame 2
            let active %GuiBareFramebufferActiveSequence GuiBareFramebufferActiveSequence GuiRgba8888RowTileRlePresentHostSpanOperationTarget::Device descriptor -1 0
            let state %GuiBareFramebufferState GuiBareFramebufferState config GuiBareFramebufferPhase::Active active Option::None
            let run0 %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceRunSpan span_at 0 0 3
            match gui_bare_framebuffer_validate_operation state run0:
                Result::Ok _:
                    false
                Result::Err error:
                    kind_is_active_seen_run_count_invalid gui_bare_framebuffer_step_error_kind &error

fn forged_active_pixel_exceeded_case %fn SurfaceId fn FrameId bool \surface\frame:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for surface frame 2
            let active %GuiBareFramebufferActiveSequence GuiBareFramebufferActiveSequence GuiRgba8888RowTileRlePresentHostSpanOperationTarget::Device descriptor 1 7
            let state %GuiBareFramebufferState GuiBareFramebufferState config GuiBareFramebufferPhase::Active active Option::None
            let end %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceEnd descriptor
            match gui_bare_framebuffer_validate_operation state end:
                Result::Ok _:
                    false
                Result::Err error:
                    kind_is_pixel_count_exceeded gui_bare_framebuffer_step_error_kind &error

fn host_execution_failed_case %impure fn SurfaceId impure fn FrameId bool \surface\frame:
    match gui_bare_framebuffer_config_checked surface 3 2:
        Result::Err _:
            false
        Result::Ok config:
            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for surface frame 2
            let state0 %GuiBareFramebufferState gui_bare_framebuffer_state_initial config
            let begin %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceBegin descriptor
            match gui_bare_framebuffer_execute_operation state0 begin:
                Result::Ok _:
                    false
                Result::Err error:
                    let state_kept %GuiBareFramebufferState gui_bare_framebuffer_step_error_state &error
                    let phase_kept %bool match gui_bare_framebuffer_state_phase &state_kept:
                        GuiBareFramebufferPhase::Active active:
                            false
                        GuiBareFramebufferPhase::Idle:
                            true
                    let operation_kept %bool match gui_bare_framebuffer_step_error_operation &error:
                        GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceBegin error_descriptor:
                            let error_frame %FrameId gui_rgba8888_row_tile_rle_present_descriptor_frame &error_descriptor
                            eq frame_id_raw &error_frame frame_id_raw &frame
                        _:
                            false
                    let kind_ok %bool kind_is_host_execution_unsupported gui_bare_framebuffer_step_error_kind &error
                    and kind_ok and phase_kept operation_kept

fn run_case %impure fn void i32 \void:
    match surface_id_result 77:
        Result::Err _:
            10
        Result::Ok surface:
            match frame_id_result 1:
                Result::Err _:
                    11
                Result::Ok frame:
                    match window_id_result 2:
                        Result::Err _:
                            12
                        Result::Ok window:
                            let valid_ok %bool valid_sequence_case surface frame
                            let run_without_begin_ok %bool run_without_begin_case surface frame
                            let target_mismatch_ok %bool target_mismatch_case surface frame window
                            let gap_ok %bool gap_case surface frame
                            let descriptor_frame_ok %bool descriptor_frame_mismatch_case surface frame
                            let descriptor_row_ok %bool descriptor_row_extent_case surface frame
                            let descriptor_pixel_ok %bool descriptor_pixel_count_case surface frame
                            let overlap_ok %bool overlap_case surface frame
                            let bounds_ok %bool run_out_of_bounds_case surface frame
                            let incomplete_ok %bool incomplete_end_case surface frame
                            let forged_descriptor_ok %bool forged_active_descriptor_case surface frame
                            let forged_seen_negative_ok %bool forged_active_negative_seen_case surface frame
                            let forged_seen_exceeded_ok %bool forged_active_pixel_exceeded_case surface frame
                            let host_failure_ok %bool host_execution_failed_case surface frame
                            let ordering_pair_ok %bool and valid_ok run_without_begin_ok
                            let target_pair_ok %bool and target_mismatch_ok gap_ok
                            let ordering_ok %bool and ordering_pair_ok target_pair_ok
                            let descriptor_pair_ok %bool and descriptor_frame_ok descriptor_row_ok
                            let descriptor_ok %bool and descriptor_pair_ok descriptor_pixel_ok
                            let run_bounds_ok %bool and overlap_ok bounds_ok
                            let forged_seen_ok %bool and forged_seen_negative_ok forged_seen_exceeded_ok
                            let active_invariant_ok %bool and forged_descriptor_ok forged_seen_ok
                            let failure_ok %bool and incomplete_ok host_failure_ok
                            let adapter_ok %bool and ordering_ok descriptor_ok
                            let terminal_pair_ok %bool and run_bounds_ok failure_ok
                            let terminal_ok %bool and terminal_pair_ok active_invariant_ok
                            let all_ok %bool and adapter_ok terminal_ok
                            if all_ok 0 1

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_platform_bare_framebuffer"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
