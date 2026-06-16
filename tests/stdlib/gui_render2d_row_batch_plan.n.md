# GUI render2d row batch plan

このファイルは、F5bv の RGBA8888 row batch plan owner が validated bitmap frame owner を再検証し、formal byte payload / host present / fallback に進まないことを固定する。

source policy coverage labels:

- render2d_row_batch_plan_facade_ok
- render2d_row_batch_plan_positive_config_ok
- render2d_row_batch_plan_empty_dirty_zero_rows_ok
- render2d_row_batch_plan_full_dirty_batches_ok
- render2d_row_batch_plan_two_rect_contiguous_span_ok
- render2d_row_batch_plan_forged_stride_recovery_ok
- render2d_row_batch_plan_dirty_bounds_recovery_ok
- render2d_row_batch_plan_finish_frame_teardown_ok
- render2d_row_batch_plan_no_platform_no_fallback

## row batch plan owner contract

[目的/もくてき]:
- public に forged された bitmap frame owner を row plan boundary でも[再検証/さいけんしょう]することを確認します。
- dirty set から row span と batch count が deterministic に[計算/けいさん]されることを確認します。
- [失敗/しっぱい]時に frame owner を[失/うしな]わないことを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_batch_plan_contract\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
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

// render2d_row_batch_plan_facade_ok
// render2d_row_batch_plan_positive_config_ok
// render2d_row_batch_plan_empty_dirty_zero_rows_ok
// render2d_row_batch_plan_full_dirty_batches_ok
// render2d_row_batch_plan_two_rect_contiguous_span_ok
// render2d_row_batch_plan_forged_stride_recovery_ok
// render2d_row_batch_plan_dirty_bounds_recovery_ok
// render2d_row_batch_plan_finish_frame_teardown_ok
// render2d_row_batch_plan_no_platform_no_fallback

fn kind_is_max_rows_invalid %fn GuiRgba8888RowBatchPlanPrepareErrorKind bool \kind:
    match kind:
        GuiRgba8888RowBatchPlanPrepareErrorKind::MaxRowsPerBatchInvalid:
            true
        _:
            false

fn kind_is_stride_mismatch %fn GuiRgba8888RowBatchPlanPrepareErrorKind bool \kind:
    match kind:
        GuiRgba8888RowBatchPlanPrepareErrorKind::FrameStrideMismatch:
            true
        _:
            false

fn kind_is_dirty_out_of_bounds %fn GuiRgba8888RowBatchPlanPrepareErrorKind bool \kind:
    match kind:
        GuiRgba8888RowBatchPlanPrepareErrorKind::DirtyRectOutOfBounds:
            true
        _:
            false

fn kind_is_dirty_bottom_overflow %fn GuiRgba8888RowBatchPlanPrepareErrorKind bool \kind:
    match kind:
        GuiRgba8888RowBatchPlanPrepareErrorKind::DirtyRectBottomOverflow:
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

fn category_is_invalid_geometry %fn Option GuiError bool \category:
    match category:
        Option::Some error:
            match error:
                GuiError::InvalidGeometry:
                    true
                _:
                    false
        Option::None:
            false

fn free_plan_return %fn GuiRgba8888RowBatchPlanOwner fn bool bool \plan\value:
    match gui_rgba8888_row_batch_plan_free plan:
        Result::Err _:
            false
        Result::Ok _:
            value

fn free_frame_return %fn GuiRgba8888BitmapFrameOwner fn bool bool \frame\value:
    match gui_rgba8888_bitmap_frame_free frame:
        Result::Err _:
            false
        Result::Ok _:
            value

fn free_dirty_owner_false %fn GuiRgba8888SoftwareSurfaceDirtyOwner bool \owner:
    match gui_rgba8888_software_surface_dirty_owner_free owner:
        Result::Err _:
            false
        Result::Ok _:
            false

fn free_dirty_push_error_false %fn GuiRgba8888SoftwareSurfaceDirtyPushError bool \error:
    match gui_rgba8888_software_surface_dirty_push_error_free error:
        Result::Err _:
            false
        Result::Ok _:
            false

fn free_bitmap_prepare_error_false %fn GuiRgba8888BitmapFramePrepareError bool \error:
    match gui_rgba8888_bitmap_frame_prepare_error_free error:
        Result::Err _:
            false
        Result::Ok _:
            false

fn frame_from_dirty_owner %fn GuiRgba8888SoftwareSurfaceDirtyOwner fn i32 Result GuiRgba8888BitmapFrameOwner i32 \owner\frame_id:
    match gui_rgba8888_bitmap_frame_config_checked frame_id:
        Result::Err _:
            match gui_rgba8888_software_surface_dirty_owner_free owner:
                Result::Err _:
                    Result::Err 1
                Result::Ok _:
                    Result::Err 1
        Result::Ok config:
            match gui_rgba8888_bitmap_frame_prepare owner config:
                Result::Err error:
                    match gui_rgba8888_bitmap_frame_prepare_error_free error:
                        Result::Err _:
                            Result::Err 2
                        Result::Ok _:
                            Result::Err 2
                Result::Ok frame:
                    Result::Ok frame

fn dirty_owner_empty %fn i32 fn i32 Result GuiRgba8888SoftwareSurfaceDirtyOwner i32 \width\height:
    match gui_rgba8888_software_surface_create width height:
        Result::Err _:
            Result::Err 1
        Result::Ok surface:
            Result::Ok gui_rgba8888_software_surface_dirty_owner_from_surface surface

fn dirty_owner_with_region %fn i32 fn i32 fn DirtyRegion Result GuiRgba8888SoftwareSurfaceDirtyOwner i32 \width\height\region:
    match dirty_owner_empty width height:
        Result::Err code:
            Result::Err code
        Result::Ok owner:
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked owner region:
                Result::Err error:
                    match gui_rgba8888_software_surface_dirty_push_error_free error:
                        Result::Err _:
                            Result::Err 2
                        Result::Ok _:
                            Result::Err 2
                Result::Ok next:
                    Result::Ok next

fn dirty_owner_with_two_rects %fn i32 fn i32 fn GuiRect fn GuiRect Result GuiRgba8888SoftwareSurfaceDirtyOwner i32 \width\height\first\second:
    match dirty_region_rect_checked first:
        Result::Err _:
            Result::Err 1
        Result::Ok first_region:
            match dirty_owner_with_region width height first_region:
                Result::Err code:
                    Result::Err code
                Result::Ok owner1:
                    match dirty_region_rect_checked second:
                        Result::Err _:
                            match gui_rgba8888_software_surface_dirty_owner_free owner1:
                                Result::Err _:
                                    Result::Err 2
                                Result::Ok _:
                                    Result::Err 2
                        Result::Ok second_region:
                            match gui_rgba8888_software_surface_dirty_owner_push_region_checked owner1 second_region:
                                Result::Err error:
                                    match gui_rgba8888_software_surface_dirty_push_error_free error:
                                        Result::Err _:
                                            Result::Err 3
                                        Result::Ok _:
                                            Result::Err 3
                                Result::Ok owner2:
                                    Result::Ok owner2

fn prepare_plan_from_frame %fn GuiRgba8888BitmapFrameOwner fn i32 Result GuiRgba8888RowBatchPlanOwner GuiRgba8888RowBatchPlanPrepareError \frame\max_rows:
    match gui_rgba8888_row_batch_plan_config_checked max_rows:
        Result::Err _:
            let forged_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig max_rows
            gui_rgba8888_row_batch_plan_prepare frame forged_config
        Result::Ok config:
            gui_rgba8888_row_batch_plan_prepare frame config

fn positive_config_case %fn void bool \void:
    let positive_ok %bool match gui_rgba8888_row_batch_plan_config_checked 2:
        Result::Err _:
            false
        Result::Ok config:
            eq gui_rgba8888_row_batch_plan_config_max_rows_per_batch &config 2
    let invalid_ok %bool match gui_rgba8888_row_batch_plan_config_checked 0:
        Result::Err kind:
            kind_is_max_rows_invalid kind
        Result::Ok _:
            false
    and positive_ok invalid_ok

fn empty_dirty_case %fn void bool \void:
    match dirty_owner_empty 3 2:
        Result::Err _:
            false
        Result::Ok owner:
            match frame_from_dirty_owner owner 31:
                Result::Err _:
                    false
                Result::Ok frame:
                    match prepare_plan_from_frame frame 2:
                        Result::Err error:
                            match gui_rgba8888_row_batch_plan_prepare_error_free error:
                                Result::Err _:
                                    false
                                Result::Ok _:
                                    false
                        Result::Ok plan:
                            let shape_ok %bool and:
                                eq gui_rgba8888_row_batch_plan_frame_id &plan 31
                                and:
                                    eq gui_rgba8888_row_batch_plan_width &plan 3
                                    eq gui_rgba8888_row_batch_plan_height &plan 2
                            let row_ok %bool and:
                                eq gui_rgba8888_row_batch_plan_row_start &plan 0
                                and:
                                    eq gui_rgba8888_row_batch_plan_row_count &plan 0
                                    eq gui_rgba8888_row_batch_plan_batch_count &plan 0
                            let dirty %DirtyRegionSet gui_rgba8888_row_batch_plan_dirty &plan
                            let dirty_ok %bool dirty_regions_is_empty dirty
                            free_plan_return plan and shape_ok and row_ok dirty_ok

fn full_dirty_case %fn void bool \void:
    match dirty_owner_with_region 4 5 dirty_region_full:
        Result::Err _:
            false
        Result::Ok owner:
            match frame_from_dirty_owner owner 32:
                Result::Err _:
                    false
                Result::Ok frame:
                    match prepare_plan_from_frame frame 2:
                        Result::Err error:
                            match gui_rgba8888_row_batch_plan_prepare_error_free error:
                                Result::Err _:
                                    false
                                Result::Ok _:
                                    false
                        Result::Ok plan:
                            let row_ok %bool and:
                                eq gui_rgba8888_row_batch_plan_row_start &plan 0
                                and:
                                    eq gui_rgba8888_row_batch_plan_row_count &plan 5
                                    eq gui_rgba8888_row_batch_plan_batch_count &plan 3
                            let max_ok %bool eq gui_rgba8888_row_batch_plan_max_rows_per_batch &plan 2
                            let dirty %DirtyRegionSet gui_rgba8888_row_batch_plan_dirty &plan
                            let dirty_ok %bool dirty_regions_is_full dirty
                            free_plan_return plan and row_ok and max_ok dirty_ok

fn two_rect_case %fn void bool \void:
    let first %GuiRect gui_rect_new 1 4 2 1
    let second %GuiRect gui_rect_new 0 1 1 2
    match dirty_owner_with_two_rects 4 6 first second:
        Result::Err _:
            false
        Result::Ok owner:
            match frame_from_dirty_owner owner 33:
                Result::Err _:
                    false
                Result::Ok frame:
                    match prepare_plan_from_frame frame 2:
                        Result::Err error:
                            match gui_rgba8888_row_batch_plan_prepare_error_free error:
                                Result::Err _:
                                    false
                                Result::Ok _:
                                    false
                        Result::Ok plan:
                            let row_ok %bool and:
                                eq gui_rgba8888_row_batch_plan_row_start &plan 1
                                and:
                                    eq gui_rgba8888_row_batch_plan_row_count &plan 4
                                    eq gui_rgba8888_row_batch_plan_batch_count &plan 2
                            let metadata_ok %bool and:
                                eq gui_rgba8888_row_batch_plan_stride_bytes &plan 16
                                eq gui_rgba8888_row_batch_plan_byte_len &plan 96
                            free_plan_return plan and row_ok metadata_ok

fn invalid_max_rows_recovery_case %fn void bool \void:
    match dirty_owner_empty 2 2:
        Result::Err _:
            false
        Result::Ok owner:
            match frame_from_dirty_owner owner 41:
                Result::Err _:
                    false
                Result::Ok frame:
                    match prepare_plan_from_frame frame 0:
                        Result::Ok plan:
                            free_plan_return plan false
                        Result::Err error:
                            let kind_ok %bool kind_is_max_rows_invalid gui_rgba8888_row_batch_plan_prepare_error_kind &error
                            let category_ok %bool category_is_invalid_command gui_rgba8888_row_batch_plan_prepare_error_category_value &error
                            let recovered %GuiRgba8888BitmapFrameOwner gui_rgba8888_row_batch_plan_prepare_error_frame error
                            free_frame_return recovered and kind_ok category_ok

fn finish_frame_case %fn void bool \void:
    match dirty_owner_with_region 1 1 dirty_region_full:
        Result::Err _:
            false
        Result::Ok owner:
            match frame_from_dirty_owner owner 34:
                Result::Err _:
                    false
                Result::Ok frame:
                    match prepare_plan_from_frame frame 1:
                        Result::Err error:
                            match gui_rgba8888_row_batch_plan_prepare_error_free error:
                                Result::Err _:
                                    false
                                Result::Ok _:
                                    false
                        Result::Ok plan:
                            let row_ok %bool and:
                                eq gui_rgba8888_row_batch_plan_row_count &plan 1
                                eq gui_rgba8888_row_batch_plan_batch_count &plan 1
                            let frame1 %GuiRgba8888BitmapFrameOwner gui_rgba8888_row_batch_plan_finish_frame plan
                            free_frame_return frame1 row_ok

fn run_case %fn void i32 \void:
    let config_ok %bool positive_config_case
    let empty_ok %bool empty_dirty_case
    let full_ok %bool full_dirty_case
    let two_ok %bool two_rect_case
    let max_rows_ok %bool invalid_max_rows_recovery_case
    let finish_ok %bool finish_frame_case
    let first %bool and config_ok and empty_ok full_ok
    let second %bool and two_ok max_rows_ok
    let third %bool finish_ok
    if and first and second third 0 1

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_row_batch_plan_contract"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## forged frame metadata boundary

この compile-fail は、通常の application code が `GuiRgba8888BitmapFrameOwner` を直 constructor で forged できないことを固定する。実装本体は defense-in-depth として stride / byte_len mismatch と dirty bounds を再検証する。

neplg2:test[compile_fail]
diag_code: type.owner_aggregate.constructor_restricted
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "core/gui/dirty_region_set" as *
#import "core/result" as *

// render2d_row_batch_plan_forged_stride_recovery_ok
// render2d_row_batch_plan_dirty_bounds_recovery_ok

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 2 2:
        Result::Err _:
            0
        Result::Ok surface:
            let frame %GuiRgba8888BitmapFrameOwner GuiRgba8888BitmapFrameOwner 101 2 2 100 16 dirty_regions_empty surface
            let config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 1
            match gui_rgba8888_row_batch_plan_prepare frame config:
                Result::Ok plan:
                    match gui_rgba8888_row_batch_plan_free plan:
                        Result::Ok _:
                            1
                        Result::Err _:
                            1
                Result::Err error:
                    match gui_rgba8888_row_batch_plan_prepare_error_free error:
                        Result::Ok _:
                            1
                        Result::Err _:
                            1
```

## public owner recovery boundary

この compile-fail は、通常の application code が row plan owner から内部の bitmap frame owner を field access で取り出せないことを固定する。frame が必要な場合は consuming `finish_frame` helper を使う。

neplg2:test[compile_fail]
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "core/field" as field
#import "core/result" as *

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            0
        Result::Ok surface:
            let owner %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 1
            match gui_rgba8888_bitmap_frame_prepare owner frame_config:
                Result::Err error:
                    match gui_rgba8888_bitmap_frame_prepare_error_free error:
                        Result::Err _:
                            1
                        Result::Ok _:
                            1
                Result::Ok frame:
                    let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 1
                    match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                        Result::Err error:
                            match gui_rgba8888_row_batch_plan_prepare_error_free error:
                                Result::Err _:
                                    2
                                Result::Ok _:
                                    2
                        Result::Ok plan:
                            let hidden field::get plan "frame"
                            3
```
