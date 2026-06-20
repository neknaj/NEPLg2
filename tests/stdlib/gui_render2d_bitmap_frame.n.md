# GUI render2d bitmap frame

このファイルは、F5bu の RGBA8888 bitmap frame owner が dirty surface owner を formal transport 前に検証済み metadata boundary へ変換し、platform transport や fallback に進まないことを固定する。

source policy coverage labels:

- render2d_bitmap_frame_facade_ok
- render2d_bitmap_frame_positive_id_config_ok
- render2d_bitmap_frame_prepare_metadata_ok
- render2d_bitmap_frame_invalid_frame_id_recovery_ok
- render2d_bitmap_frame_forged_stride_recovery_ok
- render2d_bitmap_frame_dirty_bounds_recovery_ok
- render2d_bitmap_frame_finish_surface_teardown_ok
- render2d_bitmap_frame_no_platform_no_fallback

## bitmap frame owner contract

[目的/もくてき]:
- dirty surface owner から frame owner へ[進/すす]む[前/まえ]に、frame id、surface shape、dirty bounds を[再検証/さいけんしょう]することを確認します。
- [失敗/しっぱい]時に owner を[失/うしな]わないことを確認します。
- `finish_surface` は全 validation 成功後の teardown API としてだけ[使/つか]うことを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_bitmap_frame_contract\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
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

// render2d_bitmap_frame_facade_ok
// render2d_bitmap_frame_positive_id_config_ok
// render2d_bitmap_frame_prepare_metadata_ok
// render2d_bitmap_frame_invalid_frame_id_recovery_ok
// render2d_bitmap_frame_forged_stride_recovery_ok
// render2d_bitmap_frame_dirty_bounds_recovery_ok
// render2d_bitmap_frame_finish_surface_teardown_ok
// render2d_bitmap_frame_no_platform_no_fallback

fn kind_is_frame_id_invalid %fn GuiRgba8888BitmapFramePrepareErrorKind bool \kind:
    match kind:
        GuiRgba8888BitmapFramePrepareErrorKind::FrameIdInvalid:
            true
        _:
            false

fn kind_is_surface_stride_mismatch %fn GuiRgba8888BitmapFramePrepareErrorKind bool \kind:
    match kind:
        GuiRgba8888BitmapFramePrepareErrorKind::SurfaceStrideMismatch:
            true
        _:
            false

fn kind_is_dirty_rect_out_of_bounds %fn GuiRgba8888BitmapFramePrepareErrorKind bool \kind:
    match kind:
        GuiRgba8888BitmapFramePrepareErrorKind::DirtyRectOutOfBounds:
            true
        _:
            false

fn category_is_invalid_command %fn Option GuiError bool \category:
    match category:
        Option::Some error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false
        Option::None:
            false

fn positive_config_case %fn void bool \void:
    let positive_ok %bool match gui_rgba8888_bitmap_frame_config_checked 1:
        Result::Err _:
            false
        Result::Ok config:
            eq gui_rgba8888_bitmap_frame_config_frame_id &config 1
    let invalid_ok %bool match gui_rgba8888_bitmap_frame_config_checked 0:
        Result::Err kind:
            kind_is_frame_id_invalid kind
        Result::Ok _:
            false
    and positive_ok invalid_ok

fn prepare_metadata_case %fn void bool \void:
    match gui_rgba8888_software_surface_create 3 2:
        Result::Err _:
            false
        Result::Ok surface:
            let owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let rect %GuiRect gui_rect_new 1 1 2 1
            match dirty_region_rect_checked rect:
                Result::Err _:
                    match gui_rgba8888_software_surface_dirty_owner_free owner0:
                        Result::Ok _:
                            false
                        Result::Err _:
                            false
                Result::Ok region:
                    match gui_rgba8888_software_surface_dirty_owner_push_region_checked owner0 region:
                        Result::Err error:
                            match gui_rgba8888_software_surface_dirty_push_error_free error:
                                Result::Ok _:
                                    false
                                Result::Err _:
                                    false
                        Result::Ok owner1:
                            match gui_rgba8888_bitmap_frame_config_checked 9:
                                Result::Err _:
                                    match gui_rgba8888_software_surface_dirty_owner_free owner1:
                                        Result::Ok _:
                                            false
                                        Result::Err _:
                                            false
                                Result::Ok config:
                                    match gui_rgba8888_bitmap_frame_prepare owner1 config:
                                        Result::Err error:
                                            match gui_rgba8888_bitmap_frame_prepare_error_free error:
                                                Result::Ok _:
                                                    false
                                                Result::Err _:
                                                    false
                                        Result::Ok frame:
                                            let id_ok %bool eq gui_rgba8888_bitmap_frame_frame_id &frame 9
                                            let shape_ok %bool and:
                                                eq gui_rgba8888_bitmap_frame_width &frame 3
                                                and:
                                                    eq gui_rgba8888_bitmap_frame_height &frame 2
                                                    and:
                                                        eq gui_rgba8888_bitmap_frame_stride_bytes &frame 12
                                                        eq gui_rgba8888_bitmap_frame_byte_len &frame 24
                                            let dirty %DirtyRegionSet gui_rgba8888_bitmap_frame_dirty &frame
                                            let dirty_ok %bool dirty_regions_is_one dirty
                                            match gui_rgba8888_bitmap_frame_free frame:
                                                Result::Err _:
                                                    false
                                                Result::Ok _:
                                                    and id_ok and shape_ok dirty_ok

fn invalid_id_recovery_case %fn void bool \void:
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            false
        Result::Ok surface:
            let owner %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let forged_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 0
            match gui_rgba8888_bitmap_frame_prepare owner forged_config:
                Result::Ok frame:
                    match gui_rgba8888_bitmap_frame_free frame:
                        Result::Ok _:
                            false
                        Result::Err _:
                            false
                Result::Err error:
                    let kind_ok %bool kind_is_frame_id_invalid gui_rgba8888_bitmap_frame_prepare_error_kind &error
                    let category_ok %bool category_is_invalid_command gui_rgba8888_bitmap_frame_prepare_error_category_value &error
                    let recovered %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_bitmap_frame_prepare_error_owner error
                    match gui_rgba8888_software_surface_dirty_owner_free recovered:
                        Result::Err _:
                            false
                        Result::Ok _:
                            and kind_ok category_ok

fn dirty_bounds_recovery_case %fn void bool \void:
    match gui_rgba8888_software_surface_create 2 2:
        Result::Err _:
            false
        Result::Ok surface:
            let owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let rect %GuiRect gui_rect_new 2 0 1 1
            match dirty_region_rect_checked rect:
                Result::Err _:
                    match gui_rgba8888_software_surface_dirty_owner_free owner0:
                        Result::Ok _:
                            false
                        Result::Err _:
                            false
                Result::Ok region:
                    match gui_rgba8888_software_surface_dirty_owner_push_region_checked owner0 region:
                        Result::Err error:
                            match gui_rgba8888_software_surface_dirty_push_error_free error:
                                Result::Ok _:
                                    false
                                Result::Err _:
                                    false
                        Result::Ok owner1:
                            let config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 6
                            match gui_rgba8888_bitmap_frame_prepare owner1 config:
                                Result::Ok frame:
                                    match gui_rgba8888_bitmap_frame_free frame:
                                        Result::Ok _:
                                            false
                                        Result::Err _:
                                            false
                                Result::Err error:
                                    let kind_ok %bool kind_is_dirty_rect_out_of_bounds gui_rgba8888_bitmap_frame_prepare_error_kind &error
                                    let recovered %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_bitmap_frame_prepare_error_owner error
                                    match gui_rgba8888_software_surface_dirty_owner_free recovered:
                                        Result::Err _:
                                            false
                                        Result::Ok _:
                                            kind_ok

fn finish_surface_case %fn void bool \void:
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            false
        Result::Ok surface:
            let owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked owner0 dirty_region_full:
                Result::Err error:
                    match gui_rgba8888_software_surface_dirty_push_error_free error:
                        Result::Ok _:
                            false
                        Result::Err _:
                            false
                Result::Ok owner1:
                    let config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 7
                    match gui_rgba8888_bitmap_frame_prepare owner1 config:
                        Result::Err error:
                            match gui_rgba8888_bitmap_frame_prepare_error_free error:
                                Result::Ok _:
                                    false
                                Result::Err _:
                                    false
                        Result::Ok frame:
                            let dirty %DirtyRegionSet gui_rgba8888_bitmap_frame_dirty &frame
                            let dirty_ok %bool dirty_regions_is_full dirty
                            let surface1 %GuiRgba8888SoftwareSurfaceOwner gui_rgba8888_bitmap_frame_finish_surface frame
                            match gui_rgba8888_software_surface_free surface1:
                                Result::Err _:
                                    false
                                Result::Ok _:
                                    dirty_ok

fn run_case %fn void i32 \void:
    let config_ok %bool positive_config_case
    let metadata_ok %bool prepare_metadata_case
    let id_ok %bool invalid_id_recovery_case
    let dirty_ok %bool dirty_bounds_recovery_case
    let finish_ok %bool finish_surface_case
    let first %bool and config_ok metadata_ok
    let second %bool and id_ok dirty_ok
    let third %bool finish_ok
    if and first and second third 0 1

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_bitmap_frame_contract"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## public owner token boundary

この compile-fail は、通常の application code が `GuiRgba8888SoftwareSurfaceOwner` の storage token を取り出して forged surface metadata を作れないことを固定する。実装本体は defense-in-depth として stride / byte_len mismatch を検査する。

neplg2:test[compile_fail]
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "core/field" as field
#import "core/result" as *

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 2 2:
        Result::Err _:
            0
        Result::Ok surface:
            let storage field::get surface "storage"
            1
```
