# GUI render2d row tile RLE count completed doctests

このファイルは、F5cf の RGBA8888 row tile RLE completed count owner が complete cursor と正の total run count だけを future encoded RLE transport の capacity evidence として受け入れることを固定する。

source policy coverage labels:

- render2d_row_tile_rle_count_completed_facade_ok
- render2d_row_tile_rle_count_completed_success_total_ok
- render2d_row_tile_rle_count_completed_pending_rejected_ok
- render2d_row_tile_rle_count_completed_error_recovers_owner_ok
- render2d_row_tile_rle_count_completed_no_encoded_buffer_no_platform_no_fallback

## completed count becomes capacity evidence

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_tile_rle_count_completed\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"completed total\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"pending rejected\" expected=\"0\" actual=\"0\" message=\"\"\n"
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

// render2d_row_tile_rle_count_completed_facade_ok
// render2d_row_tile_rle_count_completed_success_total_ok
// render2d_row_tile_rle_count_completed_pending_rejected_ok
// render2d_row_tile_rle_count_completed_error_recovers_owner_ok
// render2d_row_tile_rle_count_completed_no_encoded_buffer_no_platform_no_fallback

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

fn fail_count_owner %fn GuiRgba8888RowTileRleCountOwner fn i32 i32 \owner\code:
    match gui_rgba8888_row_tile_rle_count_owner_free owner:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_completed_owner %fn GuiRgba8888RowTileRleCountCompletedOwner fn i32 i32 \owner\code:
    match gui_rgba8888_row_tile_rle_count_completed_owner_free owner:
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

fn count_status_pending %fn GuiRgba8888RowTileRleCountStepStatus bool \status:
    match status:
        GuiRgba8888RowTileRleCountStepStatus::Pending:
            true
        _:
            false

fn count_status_completed %fn GuiRgba8888RowTileRleCountStepStatus bool \status:
    match status:
        GuiRgba8888RowTileRleCountStepStatus::Completed:
            true
        _:
            false

fn completed_kind_count_not_completed %fn GuiRgba8888RowTileRleCountCompletedErrorKind bool \kind:
    match kind:
        GuiRgba8888RowTileRleCountCompletedErrorKind::CountNotCompleted:
            true
        _:
            false

fn completed_pending_rejected %fn GuiRgba8888RowTileRleCountOwner i32 \owner:
    match gui_rgba8888_row_tile_rle_count_step_budget owner 0:
        Result::Err error:
            fail_count_start error 31
        Result::Ok step:
            let status %GuiRgba8888RowTileRleCountStepStatus gui_rgba8888_row_tile_rle_count_step_status &step
            let owner1 %GuiRgba8888RowTileRleCountOwner gui_rgba8888_row_tile_rle_count_step_finish_owner step
            let pending_ok %bool count_status_pending status
            match gui_rgba8888_row_tile_rle_count_completed_prepare owner1:
                Result::Ok completed:
                    fail_completed_owner completed 32
                Result::Err error:
                    let kind_ok %bool completed_kind_count_not_completed gui_rgba8888_row_tile_rle_count_completed_error_kind &error
                    let total_ok %bool eq gui_rgba8888_row_tile_rle_count_completed_error_total_run_count &error 0
                    let index_ok %bool eq gui_rgba8888_row_tile_rle_count_completed_error_cursor_next_pixel_index &error 0
                    let recovered_code %i32 if and kind_ok and total_ok index_ok 0 33
                    if pending_ok:
                        then fail_completed_error error recovered_code
                        else fail_completed_error error 34

fn completed_success %fn GuiRgba8888RowTileRleCountOwner i32 \owner:
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
                    let total_ok %bool eq gui_rgba8888_row_tile_rle_count_completed_owner_total_run_count &completed 3
                    let index_ok %bool eq gui_rgba8888_row_tile_rle_count_completed_owner_cursor_next_pixel_index &completed 4
                    let completed_code %i32 if and total_ok index_ok 0 23
                    if status_ok:
                        then fail_completed_owner completed completed_code
                        else fail_completed_owner completed 24

fn run_count_mode %fn GuiRgba8888RowTileRleCursorOwner fn i32 i32 \cursor\mode:
    match gui_rgba8888_row_tile_rle_count_start cursor:
        Result::Err error:
            fail_count_start error 20
        Result::Ok owner:
            if eq mode 0:
                then completed_success owner
                else completed_pending_rejected owner

fn run_payload %fn GuiRgba8888RowTilePlanOwner fn i32 i32 \tile_owner\mode:
    match gui_rgba8888_row_tile_payload_prepare tile_owner 0:
        Result::Err error:
            fail_payload error 14
        Result::Ok payload:
            match gui_rgba8888_row_tile_rle_cursor_start payload:
                Result::Err error:
                    fail_rle_start error 15
                Result::Ok cursor:
                    run_count_mode cursor mode

fn run_tile_plan %fn GuiRgba8888RowByteStorageOwner fn i32 i32 \storage_owner\mode:
    let config %GuiRgba8888RowTilePlanConfig GuiRgba8888RowTilePlanConfig 1
    match gui_rgba8888_row_tile_plan_prepare storage_owner config:
        Result::Err error:
            fail_tile_plan error 13
        Result::Ok tile_owner:
            run_payload tile_owner mode

fn run_storage %fn GuiRgba8888RowBatchRangeOwner fn i32 i32 \range_owner\mode:
    match gui_rgba8888_row_byte_storage_prepare range_owner:
        Result::Err error:
            fail_storage error 12
        Result::Ok storage_owner:
            run_tile_plan storage_owner mode

fn run_pipeline %fn GuiRgba8888SoftwareSurfaceOwner fn i32 i32 \surface\mode:
    let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
    match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
        Result::Err error:
            fail_dirty_push error 4
        Result::Ok dirty_owner:
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 709
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
                                                    run_storage range_owner mode

fn write_four_pixels %fn GuiRgba8888SoftwareSurfaceOwner fn Rgba8888 fn Rgba8888 fn i32 i32 \surface0\color_a\color_b\mode:
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
                                    run_pipeline surface4 mode

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
            write_four_pixels surface color_a color_b mode

fn main %impure fn void i32 \void:
    let completed_actual %i32 run_case 0
    let pending_actual %i32 run_case 1
    let report:
        test::test_report_new "gui_render2d_row_tile_rle_count_completed"
        |> test::test_report_push test::assert_eq_i32 "completed total" 0 completed_actual
        |> test::test_report_push test::assert_eq_i32 "pending rejected" 0 pending_actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## completed count owner constructor is not public application surface

通常の application code は completed count owner を直 constructor で作れない。後続 transport は `gui_rgba8888_row_tile_rle_count_completed_prepare` を通した exact capacity evidence だけを受け取る。

neplg2:test[compile_fail]
diag_code: type.owner_aggregate.constructor_restricted
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "core/gui/dirty_region" as *
#import "core/result" as *

// render2d_row_tile_rle_count_completed_no_encoded_buffer_no_platform_no_fallback

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            0
        Result::Ok surface:
            let dirty_owner %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 710
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
                                                                    match gui_rgba8888_row_tile_payload_prepare tile_owner 0:
                                                                        Result::Err _:
                                                                            0
                                                                        Result::Ok payload:
                                                                            match gui_rgba8888_row_tile_rle_cursor_start payload:
                                                                                Result::Err _:
                                                                                    0
                                                                                Result::Ok cursor_owner:
                                                                                    match gui_rgba8888_row_tile_rle_count_start cursor_owner:
                                                                                        Result::Err _:
                                                                                            0
                                                                                        Result::Ok count_owner:
                                                                                            let completed %GuiRgba8888RowTileRleCountCompletedOwner GuiRgba8888RowTileRleCountCompletedOwner count_owner 1
                                                                                            match gui_rgba8888_row_tile_rle_count_completed_owner_free completed:
                                                                                                Result::Err _:
                                                                                                    0
                                                                                                Result::Ok _:
                                                                                                    0
```
