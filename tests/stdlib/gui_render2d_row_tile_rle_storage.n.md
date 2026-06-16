# GUI render2d row tile RLE storage doctests

このファイルは、F5cj の RGBA8888 row tile RLE encoded storage owner が writer plan owner から exact byte storage を確保し、byte writer や host present へ進まないことを固定する。

source policy coverage labels:

- render2d_row_tile_rle_storage_facade_ok
- render2d_row_tile_rle_storage_writer_plan_to_storage_ok
- render2d_row_tile_rle_storage_exact_byte_count_ok
- render2d_row_tile_rle_storage_prepare_error_owner_recovery_ok
- render2d_row_tile_rle_storage_allocation_only_no_write_no_platform_no_fallback

## writer plan becomes encoded storage owner

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_tile_rle_storage\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"RLE storage byte count\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/dirty_region" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as test

// render2d_row_tile_rle_storage_facade_ok
// render2d_row_tile_rle_storage_writer_plan_to_storage_ok
// render2d_row_tile_rle_storage_exact_byte_count_ok
// render2d_row_tile_rle_storage_prepare_error_owner_recovery_ok
// render2d_row_tile_rle_storage_allocation_only_no_write_no_platform_no_fallback

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

fn fail_rle_start %fn GuiRgba8888RowTileRleStartError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_rle_start_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_count_start %fn GuiRgba8888RowTileRleCountError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_rle_count_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_count_step %fn GuiRgba8888RowTileRleCountStep fn i32 i32 \step\code:
    match gui_rgba8888_row_tile_rle_count_step_free step:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_completed_error %fn GuiRgba8888RowTileRleCountCompletedError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_rle_count_completed_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_seed_error %fn GuiRgba8888RowTileRleEncodeSeedError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_rle_encode_seed_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_encode_cursor_error %fn GuiRgba8888RowTileRleEncodeCursorError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_rle_encode_cursor_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_writer_plan_owner %fn GuiRgba8888RowTileRleWriterPlanOwner fn i32 i32 \owner\code:
    match gui_rgba8888_row_tile_rle_writer_plan_owner_free owner:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_writer_plan_error %fn GuiRgba8888RowTileRleWriterPlanError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_rle_writer_plan_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_rle_storage_owner %fn GuiRgba8888RowTileRleStorageOwner fn i32 i32 \owner\code:
    match gui_rgba8888_row_tile_rle_storage_owner_free owner:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_rle_storage_error %fn GuiRgba8888RowTileRleStoragePrepareError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_rle_storage_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn count_status_completed %fn GuiRgba8888RowTileRleCountStepStatus bool \status:
    match status:
        GuiRgba8888RowTileRleCountStepStatus::Completed:
            true
        _:
            false

fn storage_success %fn GuiRgba8888RowTileRleWriterPlanOwner i32 \plan:
    match gui_rgba8888_row_tile_rle_storage_prepare plan:
        Result::Err error:
            fail_rle_storage_error error 51
        Result::Ok owner:
            let total_ok %bool eq gui_rgba8888_row_tile_rle_storage_owner_total_run_count &owner 3
            let bytes_ok %bool eq gui_rgba8888_row_tile_rle_storage_owner_encoded_byte_count &owner 36
            let cursor_ok %bool eq gui_rgba8888_row_tile_rle_storage_owner_cursor_next_pixel_index &owner 0
            let pixel_count_ok %bool eq gui_rgba8888_row_tile_rle_storage_owner_cursor_pixel_count &owner 4
            let code %i32 if and total_ok and bytes_ok and cursor_ok pixel_count_ok 0 52
            fail_rle_storage_owner owner code

fn writer_plan_success %fn GuiRgba8888RowTileRleEncodeCursorOwner i32 \ready:
    match gui_rgba8888_row_tile_rle_writer_plan_prepare ready:
        Result::Err error:
            fail_writer_plan_error error 41
        Result::Ok plan:
            storage_success plan

fn encode_cursor_success %fn GuiRgba8888RowTileRleEncodeSeedOwner i32 \seed:
    match gui_rgba8888_row_tile_rle_encode_cursor_start seed:
        Result::Err error:
            fail_encode_cursor_error error 31
        Result::Ok ready:
            writer_plan_success ready

fn seed_success %fn GuiRgba8888RowTileRleCountOwner i32 \owner:
    match gui_rgba8888_row_tile_rle_count_step_budget owner 8:
        Result::Err error:
            fail_count_start error 21
        Result::Ok step:
            let status %GuiRgba8888RowTileRleCountStepStatus gui_rgba8888_row_tile_rle_count_step_status &step
            let owner1 %GuiRgba8888RowTileRleCountOwner gui_rgba8888_row_tile_rle_count_step_finish_owner step
            let status_ok %bool count_status_completed status
            match gui_rgba8888_row_tile_rle_count_completed_prepare owner1:
                Result::Err error:
                    fail_completed_error error 22
                Result::Ok completed:
                    match gui_rgba8888_row_tile_rle_encode_seed_prepare completed:
                        Result::Err error:
                            fail_seed_error error 23
                        Result::Ok seed:
                            if status_ok:
                                then encode_cursor_success seed
                                else:
                                    match gui_rgba8888_row_tile_rle_encode_seed_owner_free seed:
                                        Result::Err _:
                                            24
                                        Result::Ok _:
                                            24

fn run_count_mode %fn GuiRgba8888RowTileRleCursorOwner i32 \cursor:
    match gui_rgba8888_row_tile_rle_count_start cursor:
        Result::Err error:
            fail_count_start error 20
        Result::Ok owner:
            seed_success owner

fn run_payload %fn GuiRgba8888RowTilePlanOwner i32 \tile_owner:
    match gui_rgba8888_row_tile_payload_prepare tile_owner 0:
        Result::Err error:
            fail_payload error 14
        Result::Ok payload:
            match gui_rgba8888_row_tile_rle_cursor_start payload:
                Result::Err error:
                    fail_rle_start error 15
                Result::Ok cursor:
                    run_count_mode cursor

fn run_tile_plan %fn GuiRgba8888RowByteStorageOwner i32 \storage_owner:
    let config %GuiRgba8888RowTilePlanConfig GuiRgba8888RowTilePlanConfig 1
    match gui_rgba8888_row_tile_plan_prepare storage_owner config:
        Result::Err error:
            fail_tile_plan error 13
        Result::Ok tile_owner:
            run_payload tile_owner

fn run_storage %fn GuiRgba8888RowBatchRangeOwner i32 \range_owner:
    match gui_rgba8888_row_byte_storage_prepare range_owner:
        Result::Err error:
            fail_storage error 12
        Result::Ok storage_owner:
            run_tile_plan storage_owner

fn run_pipeline %fn GuiRgba8888SoftwareSurfaceOwner i32 \surface:
    let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
    match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
        Result::Err error:
            fail_dirty_push error 4
        Result::Ok dirty_owner:
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 717
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

fn write_four_pixels %fn GuiRgba8888SoftwareSurfaceOwner fn Rgba8888 fn Rgba8888 i32 \surface0\color_a\color_b:
    match gui_rgba8888_software_surface_write_pixel surface0 0 0 color_a:
        Result::Err error:
            fail_surface_write error 1
        Result::Ok surface1:
            match gui_rgba8888_software_surface_write_pixel surface1 1 0 color_a:
                Result::Err error:
                    fail_surface_write error 2
                Result::Ok surface2:
                    match gui_rgba8888_software_surface_write_pixel surface2 2 0 color_b:
                        Result::Err error:
                            fail_surface_write error 3
                        Result::Ok surface3:
                            match gui_rgba8888_software_surface_write_pixel surface3 3 0 color_a:
                                Result::Err error:
                                    fail_surface_write error 10
                                Result::Ok surface4:
                                    run_pipeline surface4

fn run_case %fn i32 i32 \mode:
    let a_r %u8 cast 11
    let a_g %u8 cast 12
    let a_b %u8 cast 13
    let opaque %u8 cast 255
    let b_r %u8 cast 31
    let b_g %u8 cast 32
    let b_b %u8 cast 33
    let color_a %Rgba8888 rgba8888_new a_r a_g a_b opaque
    let color_b %Rgba8888 rgba8888_new b_r b_g b_b opaque
    match gui_rgba8888_software_surface_create 4 1:
        Result::Err _:
            11
        Result::Ok surface:
            write_four_pixels surface color_a color_b

fn main %impure fn void i32 \void:
    let actual %i32 run_case 0
    let report:
        test::test_report_new "gui_render2d_row_tile_rle_storage"
        |> test::test_report_push test::assert_eq_i32 "RLE storage byte count" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
