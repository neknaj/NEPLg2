# GUI render2d row tile plan

このファイルは、F5ca の RGBA8888 row tile plan が copied row byte storage owner の authority を再検証し、byte payload / RLE / host present / fallback へ進まず、storage-relative byte descriptor を返すことを固定する。

source policy coverage labels:

- render2d_row_tile_plan_facade_ok
- render2d_row_tile_plan_positive_config_ok
- render2d_row_tile_plan_storage_authority_revalidated_ok
- render2d_row_tile_plan_checked_ceil_ok
- render2d_row_tile_plan_last_partial_tile_ok
- render2d_row_tile_plan_descriptor_offsets_ok
- render2d_row_tile_plan_owner_recovery_ok
- render2d_row_tile_plan_invariant_revalidated_ok
- render2d_row_tile_plan_no_raw_storage_escape
- render2d_row_tile_plan_no_platform_no_fallback

## row tile plan computes storage-relative descriptors

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_tile_plan_descriptors\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as render2d
#import "alloc/gui/render2d/software_surface" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/bitmap_frame" as *
#import "alloc/gui/render2d/row_batch_plan" as *
#import "alloc/gui/render2d/row_batch_cursor" as *
#import "alloc/gui/render2d/row_batch_range" as *
#import "alloc/gui/render2d/row_byte_storage" as *
#import "alloc/gui/render2d/row_tile_plan" as *
#import "core/gui/dirty_region" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as test

// render2d_row_tile_plan_facade_ok
// render2d_row_tile_plan_positive_config_ok
// render2d_row_tile_plan_storage_authority_revalidated_ok
// render2d_row_tile_plan_checked_ceil_ok
// render2d_row_tile_plan_last_partial_tile_ok
// render2d_row_tile_plan_descriptor_offsets_ok
// render2d_row_tile_plan_owner_recovery_ok
// render2d_row_tile_plan_invariant_revalidated_ok
// render2d_row_tile_plan_no_raw_storage_escape
// render2d_row_tile_plan_no_platform_no_fallback

fn fail_dirty_push %fn GuiRgba8888SoftwareSurfaceDirtyPushError fn i32 i32 \error\code:
    match gui_rgba8888_software_surface_dirty_push_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_frame %fn GuiRgba8888BitmapFramePrepareError fn i32 i32 \error\code:
    match gui_rgba8888_bitmap_frame_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_plan %fn GuiRgba8888RowBatchPlanPrepareError fn i32 i32 \error\code:
    match gui_rgba8888_row_batch_plan_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_cursor_start %fn GuiRgba8888RowBatchCursorStartError fn i32 i32 \error\code:
    match gui_rgba8888_row_batch_cursor_start_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_cursor_step %fn GuiRgba8888RowBatchCursorStepError fn i32 i32 \error\code:
    match gui_rgba8888_row_batch_cursor_step_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_range %fn GuiRgba8888RowBatchRangePrepareError fn i32 i32 \error\code:
    match gui_rgba8888_row_batch_range_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_storage %fn GuiRgba8888RowByteStoragePrepareError fn i32 i32 \error\code:
    match gui_rgba8888_row_byte_storage_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_tile_plan %fn GuiRgba8888RowTilePlanPrepareError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_plan_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn finish_tile_plan_code %fn GuiRgba8888RowTilePlanOwner fn i32 i32 \owner\code:
    let storage %GuiRgba8888RowByteStorageOwner gui_rgba8888_row_tile_plan_finish_byte_storage owner
    match gui_rgba8888_row_byte_storage_finish_cursor storage:
        Result::Err error:
            let cursor %GuiRgba8888RowBatchCursorOwner gui_rgba8888_row_byte_storage_finish_error_cursor error
            match gui_rgba8888_row_batch_cursor_free cursor:
                Result::Err _:
                    code
                Result::Ok _:
                    code
        Result::Ok cursor:
            match gui_rgba8888_row_batch_cursor_free cursor:
                Result::Err _:
                    code
                Result::Ok _:
                    code

fn descriptor_matches %fn &GuiRgba8888RowTilePlanOwner fn i32 fn i32 fn i32 fn i32 fn i32 bool \owner\index\row_start\row_count\byte_offset\byte_count:
    match gui_rgba8888_row_tile_plan_descriptor_at owner index:
        Result::Err _:
            false
        Result::Ok descriptor:
            and:
                eq gui_rgba8888_row_tile_descriptor_tile_index &descriptor index
                and:
                    eq gui_rgba8888_row_tile_descriptor_row_start &descriptor row_start
                    and:
                        eq gui_rgba8888_row_tile_descriptor_row_count &descriptor row_count
                        and:
                            eq gui_rgba8888_row_tile_descriptor_byte_offset &descriptor byte_offset
                            eq gui_rgba8888_row_tile_descriptor_byte_count &descriptor byte_count

fn tile_plan_ok %fn &GuiRgba8888RowTilePlanOwner bool \owner:
    let plan %GuiRgba8888RowTilePlan gui_rgba8888_row_tile_plan_plan owner
    and:
        eq gui_rgba8888_row_tile_plan_tile_rows &plan 2
        and:
            eq gui_rgba8888_row_tile_plan_tile_count &plan 3
            and:
                eq gui_rgba8888_row_tile_plan_byte_count &plan 40
                and:
                    descriptor_matches owner 0 0 2 0 16
                    and:
                        descriptor_matches owner 1 2 2 16 16
                        descriptor_matches owner 2 4 1 32 8

fn run_tile_plan %fn GuiRgba8888RowByteStorageOwner i32 \storage_owner:
    let config %GuiRgba8888RowTilePlanConfig GuiRgba8888RowTilePlanConfig 2
    match gui_rgba8888_row_tile_plan_prepare storage_owner config:
        Result::Err error:
            fail_tile_plan error 12
        Result::Ok tile_owner:
            if tile_plan_ok &tile_owner:
                then finish_tile_plan_code tile_owner 0
                else finish_tile_plan_code tile_owner 13

fn run_storage %fn GuiRgba8888RowBatchRangeOwner i32 \range_owner:
    match gui_rgba8888_row_byte_storage_prepare range_owner:
        Result::Err error:
            fail_storage error 11
        Result::Ok storage_owner:
            run_tile_plan storage_owner

fn run_pipeline %fn GuiRgba8888SoftwareSurfaceOwner i32 \surface:
    let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
    match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
        Result::Err error:
            fail_dirty_push error 2
        Result::Ok dirty_owner:
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 601
            match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                Result::Err error:
                    fail_frame error 3
                Result::Ok frame:
                    let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 5
                    match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                        Result::Err error:
                            fail_plan error 4
                        Result::Ok plan:
                            match gui_rgba8888_row_batch_cursor_start plan:
                                Result::Err error:
                                    fail_cursor_start error 5
                                Result::Ok cursor:
                                    match gui_rgba8888_row_batch_cursor_next_batch cursor:
                                        Result::Err error:
                                            fail_cursor_step error 6
                                        Result::Ok batch:
                                            match gui_rgba8888_row_batch_range_prepare batch:
                                                Result::Err error:
                                                    fail_range error 7
                                                Result::Ok range_owner:
                                                    run_storage range_owner

fn run_case %fn void i32 \void:
    match gui_rgba8888_software_surface_create 2 5:
        Result::Err _:
            1
        Result::Ok surface:
            run_pipeline surface

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_row_tile_plan_descriptors"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## forged row tile plan owners are not public application surface

通常の application code は row tile plan owner を直 constructor で作れない。tile descriptor が必要な場合は `gui_rgba8888_row_tile_plan_prepare` で byte storage authority と invariant 検査を通す。

neplg2:test[compile_fail]
diag_code: type.owner_aggregate.constructor_restricted
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/software_surface" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/bitmap_frame" as *
#import "alloc/gui/render2d/row_batch_plan" as *
#import "alloc/gui/render2d/row_batch_cursor" as *
#import "alloc/gui/render2d/row_batch_range" as *
#import "alloc/gui/render2d/row_byte_storage" as *
#import "alloc/gui/render2d/row_tile_plan" as *
#import "core/gui/dirty_region" as *
#import "core/result" as *

// render2d_row_tile_plan_invariant_revalidated_ok
// render2d_row_tile_plan_no_raw_storage_escape

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            0
        Result::Ok surface:
            let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
                Result::Err _:
                    0
                Result::Ok dirty_owner:
                    let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 602
                    match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                        Result::Err _:
                            0
                        Result::Ok frame:
                            let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 1
                            match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                                Result::Err _:
                                    0
                                Result::Ok plan:
                                    match gui_rgba8888_row_batch_cursor_start plan:
                                        Result::Err _:
                                            0
                                        Result::Ok cursor:
                                            match gui_rgba8888_row_batch_cursor_next_batch cursor:
                                                Result::Err _:
                                                    0
                                                Result::Ok batch:
                                                    match gui_rgba8888_row_batch_range_prepare batch:
                                                        Result::Err _:
                                                            0
                                                        Result::Ok range_owner:
                                                            match gui_rgba8888_row_byte_storage_prepare range_owner:
                                                                Result::Err _:
                                                                    0
                                                                Result::Ok storage_owner:
                                                                    let plan_meta %GuiRgba8888RowTilePlan GuiRgba8888RowTilePlan 602 0 0 1 1 1 4 4 1 1
                                                                    let tile_owner %GuiRgba8888RowTilePlanOwner GuiRgba8888RowTilePlanOwner storage_owner plan_meta
                                                                    match gui_rgba8888_row_tile_plan_free tile_owner:
                                                                        Result::Err _:
                                                                            0
                                                                        Result::Ok _:
                                                                            0
```
