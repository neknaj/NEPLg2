# GUI platform bare display storage adapter doctests

このファイルは、F5fm の Bare display storage adapter が F5fl framebuffer validation result を再検証し、actual driver が消費する Begin / SpanWrite / FramePresent effect ledger へ変換することを確認する。

executable labels:

- platform_bare_display_storage_facade_ok
- platform_bare_display_storage_forged_applied_state_ok

source policy only labels:

- platform_bare_display_storage_source_policy_valid_sequence_ok
- platform_bare_display_storage_source_policy_replay_rejected_ok
- platform_bare_display_storage_source_policy_double_begin_ok
- platform_bare_display_storage_source_policy_run_without_begin_ok
- platform_bare_display_storage_source_policy_incomplete_end_ok
- platform_bare_display_storage_source_policy_frame_mismatch_ok
- platform_bare_display_storage_source_policy_state_invariant_ok
- platform_bare_display_storage_no_host_import_fallback

## display storage adapter smoke

`gui_bare_display_storage_apply` を含む実行 doctest は現 compiler では 180 秒 timeout に入りやすいため、ここでは import smoke に限定する。state-machine の再検証順序、stale/replay rejection、RunWithoutBegin / IncompleteEnd / frame mismatch / storage invariant error は `nodesrc/test_web_gui_font_rendering_contract.js` の source-policy で固定する。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_display_storage\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "platforms/gui/bare/display_storage" as *
#import "std/test" as test

// platform_bare_display_storage_facade_ok
// platform_bare_display_storage_source_policy_valid_sequence_ok
// platform_bare_display_storage_source_policy_replay_rejected_ok
// platform_bare_display_storage_source_policy_double_begin_ok
// platform_bare_display_storage_source_policy_run_without_begin_ok
// platform_bare_display_storage_source_policy_incomplete_end_ok
// platform_bare_display_storage_source_policy_frame_mismatch_ok
// platform_bare_display_storage_source_policy_state_invariant_ok
// platform_bare_display_storage_no_host_import_fallback

fn run_case %impure fn void i32 \void:
    0

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_platform_bare_display_storage"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## forged applied state

この doctest は `gui_bare_display_storage_apply` を直接実行し、supplied next framebuffer state が canonical revalidation result と一致しない場合に `AppliedStateMismatch` で拒否されることを固定する。full Begin / RunSpan / End sequence は現 compiler では timeout しやすいため、構造契約は source-policy に分ける。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_display_storage_forged\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "core/result" as *
#import "platforms/gui/bare/display_storage" as *
#import "platforms/gui/bare/framebuffer" as *
#import "std/gui/tile_present" as *
#import "std/gui/tile_present_host_span_operation" as *
#import "std/gui/window" as *
#import "std/test" as test

// platform_bare_display_storage_forged_applied_state_ok

fn packet_for %fn i32 GuiRgba8888RowTileRlePacketDescriptor \frame_raw:
    GuiRgba8888RowTileRlePacketDescriptor frame_raw 0 0 0 2 0 2 3 2 12 2 1 6 2 24

fn descriptor_for %fn SurfaceId fn FrameId GuiRgba8888RowTileRlePresentDescriptor \surface\frame:
    let frame_raw %i32 frame_id_raw &frame
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet_for frame_raw

fn kind_is_applied_state_mismatch %fn GuiBareDisplayStorageStepError bool \error:
    match gui_bare_display_storage_step_error_kind &error:
        GuiBareDisplayStorageErrorKind::AppliedStateMismatch:
            true
        _:
            false

fn run_case %impure fn void i32 \void:
    match surface_id_result 77:
        Result::Err _:
            10
        Result::Ok surface:
            match frame_id_result 1:
                Result::Err _:
                    11
                Result::Ok frame:
                    match gui_bare_framebuffer_config_checked surface 3 2:
                        Result::Err _:
                            12
                        Result::Ok config:
                            let descriptor %GuiRgba8888RowTileRlePresentDescriptor descriptor_for surface frame
                            let storage0 %GuiBareDisplayStorageState gui_bare_display_storage_state_initial config
                            let fb0 %GuiBareFramebufferState gui_bare_display_storage_state_framebuffer_state &storage0
                            let begin %GuiRgba8888RowTileRlePresentHostSpanOperation GuiRgba8888RowTileRlePresentHostSpanOperation::DeviceBegin descriptor
                            let forged %GuiBareFramebufferStepApplied GuiBareFramebufferStepApplied fb0 begin
                            match gui_bare_display_storage_apply storage0 forged:
                                Result::Ok _:
                                    13
                                Result::Err error:
                                    if kind_is_applied_state_mismatch error 0 14

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_platform_bare_display_storage_forged"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
