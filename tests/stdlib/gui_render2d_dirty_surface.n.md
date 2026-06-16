# GUI render2d dirty surface

このファイルは、F5bt の RGBA8888 software surface dirty owner が surface owner と dirty metadata を同じ所有境界で扱い、platform transport や fallback に進まないことを固定する。

source policy coverage labels:

- render2d_dirty_surface_facade_ok
- render2d_dirty_surface_clean_owner_empty_dirty_ok
- render2d_dirty_surface_push_rect_checked_ok
- render2d_dirty_surface_invalid_unchecked_rect_recovery_ok
- render2d_dirty_surface_full_region_escalates_ok
- render2d_dirty_surface_finish_surface_teardown_ok
- render2d_dirty_surface_no_split_accessor_no_platform_no_fallback

## dirty surface owner contract

[目的/もくてき]:
- surface と dirty set を同じ owner に[束/たば]ねることを確認します。
- dirty push [失敗/しっぱい]で owner を[失/うしな]わないことを確認します。
- `finish_surface` は dirty metadata を[読/よ]んだ[後/あと]の teardown API としてだけ[使/つか]うことを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_dirty_surface_owner_contract\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "core/gui/dirty_region" as *
#import "core/gui/dirty_region_set" as *
#import "core/gui/error" as *
#import "core/gui/geometry" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as test

// render2d_dirty_surface_facade_ok
// render2d_dirty_surface_clean_owner_empty_dirty_ok
// render2d_dirty_surface_push_rect_checked_ok
// render2d_dirty_surface_invalid_unchecked_rect_recovery_ok
// render2d_dirty_surface_full_region_escalates_ok
// render2d_dirty_surface_finish_surface_teardown_ok
// render2d_dirty_surface_no_split_accessor_no_platform_no_fallback

fn gui_error_is_invalid_geometry %fn GuiError bool \error:
    match error:
        GuiError::InvalidGeometry:
            true
        _:
            false

fn clean_owner_case %fn void bool \void:
    match gui_rgba8888_software_surface_create 3 2:
        Result::Err _:
            false
        Result::Ok surface:
            let owner %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let width_ok %bool eq gui_rgba8888_software_surface_dirty_owner_width &owner 3
            let height_ok %bool eq gui_rgba8888_software_surface_dirty_owner_height &owner 2
            let stride_ok %bool eq gui_rgba8888_software_surface_dirty_owner_stride_bytes &owner 12
            let byte_ok %bool eq gui_rgba8888_software_surface_dirty_owner_byte_len &owner 24
            let dirty %DirtyRegionSet gui_rgba8888_software_surface_dirty_owner_dirty &owner
            let metadata_ok %bool and and width_ok height_ok and stride_ok byte_ok
            let dirty_ok %bool dirty_regions_is_empty dirty
            match gui_rgba8888_software_surface_dirty_owner_free owner:
                Result::Err _:
                    false
                Result::Ok _:
                    and metadata_ok dirty_ok

fn push_rect_case %fn void bool \void:
    match gui_rgba8888_software_surface_create 4 4:
        Result::Err _:
            false
        Result::Ok surface:
            let owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let rect %GuiRect gui_rect_new 1 2 3 4
            match dirty_region_rect_checked rect:
                Result::Err _:
                    match gui_rgba8888_software_surface_dirty_owner_free owner0:
                        Result::Ok _:
                            false
                        Result::Err _:
                            false
                Result::Ok region:
                    match gui_rgba8888_software_surface_dirty_owner_push_region_checked owner0 region:
                        Result::Err e:
                            match gui_rgba8888_software_surface_dirty_push_error_free e:
                                Result::Ok _:
                                    false
                                Result::Err _:
                                    false
                        Result::Ok owner1:
                            let dirty %DirtyRegionSet gui_rgba8888_software_surface_dirty_owner_dirty &owner1
                            let dirty_ok %bool match dirty_regions_rect_at dirty 0:
                                Option::None:
                                    false
                                Option::Some stored:
                                    and:
                                        eq gui_rect_x &stored 1
                                        and:
                                            eq gui_rect_y &stored 2
                                            and:
                                                eq gui_rect_width &stored 3
                                                eq gui_rect_height &stored 4
                            match gui_rgba8888_software_surface_dirty_owner_free owner1:
                                Result::Err _:
                                    false
                                Result::Ok _:
                                    and dirty_regions_is_one dirty dirty_ok

fn invalid_recovery_case %fn void bool \void:
    match gui_rgba8888_software_surface_create 2 2:
        Result::Err _:
            false
        Result::Ok surface:
            let owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let negative_width %i32 sub 0 1
            let invalid_rect %GuiRect gui_rect_new 0 0 negative_width 1
            let invalid_region %DirtyRegion dirty_region_rect_unchecked invalid_rect
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked owner0 invalid_region:
                Result::Ok owner1:
                    match gui_rgba8888_software_surface_dirty_owner_free owner1:
                        Result::Ok _:
                            false
                        Result::Err _:
                            false
                Result::Err e:
                    let error_ok %bool gui_error_is_invalid_geometry gui_rgba8888_software_surface_dirty_push_error_error &e
                    let recovered %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_push_error_owner e
                    let dirty %DirtyRegionSet gui_rgba8888_software_surface_dirty_owner_dirty &recovered
                    let dirty_ok %bool dirty_regions_is_empty dirty
                    match gui_rgba8888_software_surface_dirty_owner_free recovered:
                        Result::Err _:
                            false
                        Result::Ok _:
                            and error_ok dirty_ok

fn full_and_finish_case %fn void bool \void:
    match gui_rgba8888_software_surface_create 2 2:
        Result::Err _:
            false
        Result::Ok surface:
            let owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked owner0 dirty_region_full:
                Result::Err e:
                    match gui_rgba8888_software_surface_dirty_push_error_free e:
                        Result::Ok _:
                            false
                        Result::Err _:
                            false
                Result::Ok owner1:
                    let dirty %DirtyRegionSet gui_rgba8888_software_surface_dirty_owner_dirty &owner1
                    let dirty_ok %bool dirty_regions_is_full dirty
                    let surface1 %GuiRgba8888SoftwareSurfaceOwner gui_rgba8888_software_surface_dirty_owner_finish_surface owner1
                    match gui_rgba8888_software_surface_free surface1:
                        Result::Err _:
                            false
                        Result::Ok _:
                            dirty_ok

fn run_case %fn void i32 \void:
    let clean_ok %bool clean_owner_case
    let rect_ok %bool push_rect_case
    let recovery_ok %bool invalid_recovery_case
    let full_ok %bool full_and_finish_case
    let first %bool and clean_ok rect_ok
    if and first and recovery_ok full_ok 0 1

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_dirty_surface_owner_contract"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
