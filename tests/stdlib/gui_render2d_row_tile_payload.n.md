# GUI render2d row tile payload

このファイルは、F5cb の RGBA8888 row tile payload view が既存 copied row byte storage 上の tile-scoped byte view として働き、追加 buffer / RLE / host present / fallback へ進まないことを固定する。

source policy coverage labels:

- render2d_row_tile_payload_facade_ok
- render2d_row_tile_payload_prepare_descriptor_revalidated_ok
- render2d_row_tile_payload_view_over_existing_storage_ok
- render2d_row_tile_payload_tile_relative_read_ok
- render2d_row_tile_payload_bounds_error_typed_ok
- render2d_row_tile_payload_owner_recovery_ok
- render2d_row_tile_payload_no_raw_storage_escape
- render2d_row_tile_payload_no_platform_no_fallback

## row tile payload reads tile-relative bytes from existing copied storage

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_tile_payload_view\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
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
#import "alloc/gui/render2d/row_tile_payload" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/dirty_region" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as test

// render2d_row_tile_payload_facade_ok
// render2d_row_tile_payload_prepare_descriptor_revalidated_ok
// render2d_row_tile_payload_view_over_existing_storage_ok
// render2d_row_tile_payload_tile_relative_read_ok
// render2d_row_tile_payload_bounds_error_typed_ok
// render2d_row_tile_payload_owner_recovery_ok
// render2d_row_tile_payload_no_raw_storage_escape
// render2d_row_tile_payload_no_platform_no_fallback

fn fail_surface_write %fn GuiRgba8888SoftwareSurfaceWriteError fn i32 i32 \error\code:
    match gui_rgba8888_software_surface_write_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

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

fn fail_payload %fn GuiRgba8888RowTilePayloadPrepareError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_payload_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn finish_payload_code %fn GuiRgba8888RowTilePayloadOwner fn i32 i32 \owner\code:
    match gui_rgba8888_row_tile_payload_free owner:
        Result::Err _:
            code
        Result::Ok _:
            code

fn payload_byte_or_neg %fn &GuiRgba8888RowTilePayloadOwner fn i32 i32 \owner\index:
    match gui_rgba8888_row_tile_payload_byte_at owner index:
        Result::Err _:
            -1
        Result::Ok value:
            value

fn payload_ok %fn &GuiRgba8888RowTilePayloadOwner bool \owner:
    let descriptor %GuiRgba8888RowTileDescriptor gui_rgba8888_row_tile_payload_descriptor owner
    and:
        eq gui_rgba8888_row_tile_descriptor_row_start &descriptor 2
        and:
            eq gui_rgba8888_row_tile_descriptor_row_count &descriptor 1
            and:
                eq gui_rgba8888_row_tile_descriptor_byte_offset &descriptor 16
                and:
                    eq gui_rgba8888_row_tile_payload_byte_count owner 8
                    and:
                        eq payload_byte_or_neg owner 0 31
                        and:
                            eq payload_byte_or_neg owner 1 32
                            and:
                                eq payload_byte_or_neg owner 2 33
                                and:
                                    eq payload_byte_or_neg owner 3 34
                                    and:
                                        eq payload_byte_or_neg owner 4 41
                                        and:
                                            eq payload_byte_or_neg owner 7 44
                                            and:
                                                eq payload_byte_or_neg owner -1 -1
                                                eq payload_byte_or_neg owner 8 -1

fn run_payload %fn GuiRgba8888RowTilePlanOwner i32 \tile_owner:
    match gui_rgba8888_row_tile_payload_prepare tile_owner 1:
        Result::Err error:
            fail_payload error 12
        Result::Ok payload:
            if payload_ok &payload:
                then finish_payload_code payload 0
                else finish_payload_code payload 13

fn run_tile_plan %fn GuiRgba8888RowByteStorageOwner i32 \storage_owner:
    let config %GuiRgba8888RowTilePlanConfig GuiRgba8888RowTilePlanConfig 2
    match gui_rgba8888_row_tile_plan_prepare storage_owner config:
        Result::Err error:
            fail_tile_plan error 11
        Result::Ok tile_owner:
            run_payload tile_owner

fn run_storage %fn GuiRgba8888RowBatchRangeOwner i32 \range_owner:
    match gui_rgba8888_row_byte_storage_prepare range_owner:
        Result::Err error:
            fail_storage error 10
        Result::Ok storage_owner:
            run_tile_plan storage_owner

fn run_pipeline %fn GuiRgba8888SoftwareSurfaceOwner i32 \surface:
    let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
    match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
        Result::Err error:
            fail_dirty_push error 4
        Result::Ok dirty_owner:
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 701
            match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                Result::Err error:
                    fail_frame error 5
                Result::Ok frame:
                    let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 3
                    match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                        Result::Err error:
                            fail_plan error 6
                        Result::Ok plan:
                            match gui_rgba8888_row_batch_cursor_start plan:
                                Result::Err error:
                                    fail_cursor_start error 7
                                Result::Ok cursor:
                                    match gui_rgba8888_row_batch_cursor_next_batch cursor:
                                        Result::Err error:
                                            fail_cursor_step error 8
                                        Result::Ok batch:
                                            match gui_rgba8888_row_batch_range_prepare batch:
                                                Result::Err error:
                                                    fail_range error 9
                                                Result::Ok range_owner:
                                                    run_storage range_owner

fn write_second_row_values %fn GuiRgba8888SoftwareSurfaceOwner i32 \surface0:
    let r0 %u8 cast 31
    let g0 %u8 cast 32
    let b0 %u8 cast 33
    let a0 %u8 cast 34
    let color0 %Rgba8888 rgba8888_new r0 g0 b0 a0
    match gui_rgba8888_software_surface_write_pixel surface0 0 2 color0:
        Result::Err error:
            fail_surface_write error 2
        Result::Ok surface1:
            let r1 %u8 cast 41
            let g1 %u8 cast 42
            let b1 %u8 cast 43
            let a1 %u8 cast 44
            let color1 %Rgba8888 rgba8888_new r1 g1 b1 a1
            match gui_rgba8888_software_surface_write_pixel surface1 1 2 color1:
                Result::Err error:
                    fail_surface_write error 3
                Result::Ok surface2:
                    run_pipeline surface2

fn run_case %fn void i32 \void:
    match gui_rgba8888_software_surface_create 2 3:
        Result::Err _:
            1
        Result::Ok surface:
            write_second_row_values surface

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_row_tile_payload_view"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## forged row tile payload owners are not public application surface

通常の application code は row tile payload owner を直 constructor で作れない。tile payload view が必要な場合は `gui_rgba8888_row_tile_payload_prepare` で descriptor authority と bounds 検査を通す。

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
#import "alloc/gui/render2d/row_tile_payload" as *
#import "core/gui/dirty_region" as *
#import "core/result" as *

// render2d_row_tile_payload_prepare_descriptor_revalidated_ok
// render2d_row_tile_payload_no_raw_storage_escape

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            0
        Result::Ok surface:
            let dirty_owner %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 702
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
                                                            let config %GuiRgba8888RowTilePlanConfig GuiRgba8888RowTilePlanConfig 1
                                                            match gui_rgba8888_row_tile_plan_prepare storage_owner config:
                                                                Result::Err _:
                                                                    0
                                                                Result::Ok tile_owner:
                                                                    let descriptor %GuiRgba8888RowTileDescriptor GuiRgba8888RowTileDescriptor 0 0 1 0 4
                                                                    let payload %GuiRgba8888RowTilePayloadOwner GuiRgba8888RowTilePayloadOwner tile_owner descriptor
                                                                    match gui_rgba8888_row_tile_payload_free payload:
                                                                        Result::Err _:
                                                                            0
                                                                        Result::Ok _:
                                                                            0
```
