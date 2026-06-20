# GUI platform bare display memory write plan doctests

このファイルは、F5fn の Bare display memory write plan boundary が F5fm の storage effect ledger を actual driver 直前の byte range contract へ変換することを確認する。

executable labels:

- platform_bare_display_memory_facade_ok
- platform_bare_display_memory_byte_range_checked_ok

source policy only labels:

- platform_bare_display_memory_source_policy_canonical_storage_reapply_ok
- platform_bare_display_memory_source_policy_forged_storage_step_rejected_ok
- platform_bare_display_memory_source_policy_phase_mismatch_ok
- platform_bare_display_memory_source_policy_overflow_oob_ok
- platform_bare_display_memory_source_policy_present_complete_evidence_ok
- platform_bare_display_memory_no_host_import_fallback

## display memory byte range helper

`gui_bare_display_memory_span_write_plan_checked` は RGBA8888 row span から `y * stride_bytes + x * 4` と `width * 4` を checked arithmetic で計算し、surface 範囲外を enum error で拒否する。full storage apply sequence は現 compiler では長くなりやすいため、canonical storage reapply、forged storage step rejection、present complete evidence は `nodesrc/test_web_gui_font_rendering_contract.js` の source-policy で固定する。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_display_memory\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/cast" as *
#import "core/gui/color" as *
#import "core/math" as *
#import "core/result" as *
#import "platforms/gui/bare/display_memory" as *
#import "platforms/gui/bare/display_storage" as *
#import "platforms/gui/bare/framebuffer" as *
#import "std/gui/tile_present_host_span_operation" as *
#import "std/gui/tile_present_run_span" as *
#import "std/gui/window" as *
#import "std/test" as test

// platform_bare_display_memory_facade_ok
// platform_bare_display_memory_byte_range_checked_ok
// platform_bare_display_memory_source_policy_canonical_storage_reapply_ok
// platform_bare_display_memory_source_policy_forged_storage_step_rejected_ok
// platform_bare_display_memory_source_policy_phase_mismatch_ok
// platform_bare_display_memory_source_policy_overflow_oob_ok
// platform_bare_display_memory_source_policy_present_complete_evidence_ok
// platform_bare_display_memory_no_host_import_fallback

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
                            if ne gui_bare_display_memory_span_write_plan_byte_start &plan 36:
                                then 13
                                else:
                                    if ne gui_bare_display_memory_span_write_plan_byte_len &plan 12:
                                        then 14
                                        else:
                                            if ne gui_bare_display_memory_span_write_plan_byte_end &plan 48:
                                                then 15
                                                else:
                                                    if ne gui_bare_display_memory_span_write_plan_surface_byte_count &plan 64:
                                                        then 16
                                                        else 0

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_platform_bare_display_memory"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
