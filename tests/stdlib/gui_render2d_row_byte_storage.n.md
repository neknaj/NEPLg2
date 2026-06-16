# GUI render2d row byte storage

このファイルは、F5bz の RGBA8888 row byte storage が row batch range owner を再検証し、host present / tile / RLE / fallback へ進まず、exact-size の copied byte storage owner を作ることを固定する。

source policy coverage labels:

- render2d_row_byte_storage_facade_ok
- render2d_row_byte_storage_authority_revalidated_ok
- render2d_row_byte_storage_exact_copy_ok
- render2d_row_byte_storage_checked_byte_reader_ok
- render2d_row_byte_storage_scratch_cleanup_error_typed_ok
- render2d_row_byte_storage_no_raw_source_escape
- render2d_row_byte_storage_no_platform_no_fallback

## first row byte storage copies RGBA bytes

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_byte_storage_copy\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
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
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/dirty_region" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as test

// render2d_row_byte_storage_facade_ok
// render2d_row_byte_storage_authority_revalidated_ok
// render2d_row_byte_storage_exact_copy_ok
// render2d_row_byte_storage_checked_byte_reader_ok
// render2d_row_byte_storage_no_platform_no_fallback

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

fn checked_byte %fn &GuiRgba8888RowByteStorageOwner fn i32 fn i32 bool \owner\index\expected:
    match gui_rgba8888_row_byte_storage_byte_at owner index:
        Result::Err _:
            false
        Result::Ok actual:
            eq actual expected

fn copied_bytes_ok %fn &GuiRgba8888RowByteStorageOwner bool \owner:
    and:
        eq gui_rgba8888_row_byte_storage_byte_count owner 8
        and:
            checked_byte owner 0 10
            and:
                checked_byte owner 1 20
                and:
                    checked_byte owner 2 30
                    and:
                        checked_byte owner 3 40
                        and:
                            checked_byte owner 4 50
                            and:
                                checked_byte owner 5 60
                                and:
                                    checked_byte owner 6 70
                                    checked_byte owner 7 80

fn finish_storage_code %fn GuiRgba8888RowByteStorageOwner fn i32 i32 \owner\code:
    match gui_rgba8888_row_byte_storage_finish_cursor owner:
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

fn run_storage %fn GuiRgba8888RowBatchRangeOwner i32 \range_owner:
    match gui_rgba8888_row_byte_storage_prepare range_owner:
        Result::Err error:
            fail_storage error 12
        Result::Ok storage_owner:
            let bytes_ok %bool copied_bytes_ok &storage_owner
            if bytes_ok:
                then finish_storage_code storage_owner 0
                else finish_storage_code storage_owner 13

fn run_pipeline %fn GuiRgba8888SoftwareSurfaceOwner i32 \surface:
    let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
    match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
        Result::Err error:
            fail_dirty_push error 4
        Result::Ok dirty_owner:
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 501
            match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                Result::Err error:
                    fail_frame error 5
                Result::Ok frame:
                    let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 1
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

fn run_case %fn void i32 \void:
    match gui_rgba8888_software_surface_create 2 1:
        Result::Err _:
            1
        Result::Ok surface0:
            let r0 %u8 cast 10
            let g0 %u8 cast 20
            let b0 %u8 cast 30
            let a0 %u8 cast 40
            let first_color %Rgba8888 rgba8888_new r0 g0 b0 a0
            match gui_rgba8888_software_surface_write_pixel surface0 0 0 first_color:
                Result::Err error:
                    fail_surface_write error 2
                Result::Ok surface1:
                    let r1 %u8 cast 50
                    let g1 %u8 cast 60
                    let b1 %u8 cast 70
                    let a1 %u8 cast 80
                    let second_color %Rgba8888 rgba8888_new r1 g1 b1 a1
                    match gui_rgba8888_software_surface_write_pixel surface1 1 0 second_color:
                        Result::Err error:
                            fail_surface_write error 3
                        Result::Ok surface2:
                            run_pipeline surface2

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_row_byte_storage_copy"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## forged row byte storage owners are not public application surface

通常の application code は row byte storage owner を直 constructor で作れない。bytes が必要な場合は `gui_rgba8888_row_byte_storage_prepare` で authority と copy を通す。

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
#import "core/mem" as *
#import "core/result" as *

// render2d_row_byte_storage_no_raw_source_escape
// render2d_row_byte_storage_scratch_cleanup_error_typed_ok

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            0
        Result::Ok surface:
            let dirty_owner %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 502
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
                                    match alloc_region_bytes<u8> 4:
                                        Result::Err _:
                                            match gui_rgba8888_row_batch_cursor_free cursor:
                                                Result::Err _:
                                                    0
                                                Result::Ok _:
                                                    0
                                        Result::Ok storage:
                                            let range %GuiRgba8888RowBatchRange GuiRgba8888RowBatchRange 502 0 0 1 1 1 4 4 0 4
                                            let owner %GuiRgba8888RowByteStorageOwner GuiRgba8888RowByteStorageOwner cursor range storage
                                            match gui_rgba8888_row_byte_storage_free owner:
                                                Result::Err _:
                                                    0
                                                Result::Ok _:
                                                    0
```
