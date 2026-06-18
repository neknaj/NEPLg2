# GUI platform bare display driver outcome ledger doctests

このファイルは、F5fo の Bare display driver outcome ledger boundary が F5fn の checked memory action と caller supplied driver outcome を照合し、成功時だけ driver state を進める設計であることを確認する。

executable labels:

- platform_bare_display_driver_facade_ok
- platform_bare_display_driver_span_outcome_evidence_ok

source policy only labels:

- platform_bare_display_driver_source_policy_canonical_memory_reapply_ok
- platform_bare_display_driver_source_policy_forged_memory_step_rejected_ok
- platform_bare_display_driver_source_policy_outcome_match_before_advance_ok
- platform_bare_display_driver_source_policy_driver_rejected_result_ok
- platform_bare_display_driver_no_host_import_fallback

## display driver span outcome helper

`gui_bare_display_driver_span_write_outcome_from_plan` は F5fn の checked byte plan から driver outcome evidence を作る。full memory apply sequence は現 compiler では長くなりやすいため、canonical memory reapply、forged memory step rejection、outcome match before advance は `nodesrc/test_web_gui_font_rendering_contract.js` の source-policy で固定する。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_display_driver\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/cast" as *
#import "core/gui/color" as *
#import "core/math" as *
#import "core/result" as *
#import "platforms/gui/bare/display_driver" as *
#import "platforms/gui/bare/display_memory" as *
#import "platforms/gui/bare/display_storage" as *
#import "platforms/gui/bare/framebuffer" as *
#import "std/gui/tile_present_host_span_operation" as *
#import "std/gui/tile_present_run_span" as *
#import "std/gui/window" as *
#import "std/test" as test

// platform_bare_display_driver_facade_ok
// platform_bare_display_driver_span_outcome_evidence_ok
// platform_bare_display_driver_source_policy_canonical_memory_reapply_ok
// platform_bare_display_driver_source_policy_forged_memory_step_rejected_ok
// platform_bare_display_driver_source_policy_outcome_match_before_advance_ok
// platform_bare_display_driver_source_policy_driver_rejected_result_ok
// platform_bare_display_driver_no_host_import_fallback

fn test_color %fn void Rgba8888 \void:
    let r %u8 cast 10
    let g %u8 cast 20
    let b %u8 cast 30
    let a %u8 cast 255
    rgba8888_new r g b a

fn span_at %fn i32 fn i32 fn i32 GuiRgba8888RowTileRlePresentRunRowSpan \x\y\width:
    GuiRgba8888RowTileRlePresentRunRowSpan x y width test_color

fn run_case %impure fn void i32 \void:
    match surface_id_result 77:
        Result::Err surface_error:
            10
        Result::Ok surface:
            match gui_bare_framebuffer_config_checked surface 4 4:
                Result::Err config_error:
                    11
                Result::Ok config:
                    let target %GuiRgba8888RowTileRlePresentHostSpanOperationTarget GuiRgba8888RowTileRlePresentHostSpanOperationTarget::Device
                    let span %GuiRgba8888RowTileRlePresentRunRowSpan span_at 1 2 3
                    let effect %GuiBareDisplayStorageSpanWriteEffect GuiBareDisplayStorageSpanWriteEffect target span 0 0 3
                    match gui_bare_display_memory_span_write_plan_checked config effect:
                        Result::Err memory_error:
                            12
                        Result::Ok plan:
                            match gui_bare_display_driver_span_write_outcome_from_plan plan:
                                GuiBareDisplayDriverOutcome::SpanWriteAccepted accepted:
                                    let accepted_target %GuiRgba8888RowTileRlePresentHostSpanOperationTarget gui_bare_display_driver_span_write_accepted_target &accepted
                                    let accepted_span %GuiRgba8888RowTileRlePresentRunRowSpan gui_bare_display_driver_span_write_accepted_span &accepted
                                    let accepted_color %Rgba8888 gui_bare_display_driver_span_write_accepted_color &accepted
                                    let expected_r %u8 cast 10
                                    let expected_g %u8 cast 20
                                    let expected_b %u8 cast 30
                                    let expected_a %u8 cast 255
                                    let expected_color %Rgba8888 rgba8888_new expected_r expected_g expected_b expected_a
                                    if not gui_bare_display_memory_target_equal target accepted_target:
                                        then 13
                                        else:
                                            if not gui_bare_display_memory_span_equal &span &accepted_span:
                                                then 14
                                                else:
                                                    if ne gui_bare_display_driver_span_write_accepted_run_index &accepted 0:
                                                        then 15
                                                        else:
                                                            if ne gui_bare_display_driver_span_write_accepted_pixel_start &accepted 0:
                                                                then 16
                                                                else:
                                                                    if ne gui_bare_display_driver_span_write_accepted_pixel_end &accepted 3:
                                                                        then 17
                                                                        else:
                                                                            if ne gui_bare_display_driver_span_write_accepted_row_byte_start &accepted 32:
                                                                                then 18
                                                                                else:
                                                                                    if ne gui_bare_display_driver_span_write_accepted_x_byte_offset &accepted 4:
                                                                                        then 19
                                                                                        else:
                                                                                            if ne gui_bare_display_driver_span_write_accepted_byte_start &accepted 36:
                                                                                                then 20
                                                                                                else:
                                                                                                    if ne gui_bare_display_driver_span_write_accepted_byte_len &accepted 12:
                                                                                                        then 21
                                                                                                        else:
                                                                                                            if ne gui_bare_display_driver_span_write_accepted_byte_end &accepted 48:
                                                                                                                then 22
                                                                                                                else:
                                                                                                                    if ne gui_bare_display_driver_span_write_accepted_surface_byte_count &accepted 64:
                                                                                                                        then 23
                                                                                                                        else:
                                                                                                                            if not gui_bare_display_memory_color_equal expected_color accepted_color:
                                                                                                                                then 24
                                                                                                                                else 0
                                GuiBareDisplayDriverOutcome::BeginAccepted begin:
                                    25
                                GuiBareDisplayDriverOutcome::FramePresentAccepted present:
                                    26
                                GuiBareDisplayDriverOutcome::DriverRejected lower:
                                    27

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_platform_bare_display_driver"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
