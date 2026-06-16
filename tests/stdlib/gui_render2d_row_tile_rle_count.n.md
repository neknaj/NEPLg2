# GUI render2d row tile RLE count

このファイルは、F5ce の RGBA8888 row tile RLE count owner が drain slice の emitted run count を安全に累積し、encoded buffer / host present / fallback に進まないことを固定する。

source policy coverage labels:

- render2d_row_tile_rle_count_facade_ok
- render2d_row_tile_rle_count_zero_budget_pending_ok
- render2d_row_tile_rle_count_partial_budget_accumulates_ok
- render2d_row_tile_rle_count_completion_total_ok
- render2d_row_tile_rle_count_negative_budget_wraps_lower_error_ok
- render2d_row_tile_rle_count_initial_complete_rejected_ok
- render2d_row_tile_rle_count_overflow_is_fatal_no_fake_owner_ok
- render2d_row_tile_rle_count_no_encoded_buffer_no_platform_no_fallback

## row tile RLE count accumulates drain run counts

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_tile_rle_count\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"partial then complete\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"negative budget lower error\" expected=\"0\" actual=\"0\" message=\"\"\n"
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

// render2d_row_tile_rle_count_facade_ok
// render2d_row_tile_rle_count_zero_budget_pending_ok
// render2d_row_tile_rle_count_partial_budget_accumulates_ok
// render2d_row_tile_rle_count_completion_total_ok
// render2d_row_tile_rle_count_negative_budget_wraps_lower_error_ok
// render2d_row_tile_rle_count_initial_complete_rejected_ok
// render2d_row_tile_rle_count_overflow_is_fatal_no_fake_owner_ok
// render2d_row_tile_rle_count_no_encoded_buffer_no_platform_no_fallback

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

fn fail_count_owner %fn GuiRgba8888RowTileRleCountOwner fn i32 i32 \owner\code:
    match gui_rgba8888_row_tile_rle_count_owner_free owner:
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

fn fail_count_error %fn GuiRgba8888RowTileRleCountError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_rle_count_error_free error:
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

fn count_kind_initial_complete %fn GuiRgba8888RowTileRleCountErrorKind bool \kind:
    match kind:
        GuiRgba8888RowTileRleCountErrorKind::InitialCursorComplete:
            true
        _:
            false

fn count_kind_drain_invalid_budget %fn GuiRgba8888RowTileRleCountErrorKind bool \kind:
    match kind:
        GuiRgba8888RowTileRleCountErrorKind::DrainFailed drain_kind:
            match drain_kind:
                GuiRgba8888RowTileRleDrainErrorKind::InvalidBudget:
                    true
                _:
                    false
        _:
            false

fn free_cursor_code %fn GuiRgba8888RowTileRleCursorOwner fn i32 i32 \cursor\code:
    match gui_rgba8888_row_tile_rle_cursor_free cursor:
        Result::Err _:
            code
        Result::Ok _:
            code

fn count_complete_initial_rejected %fn GuiRgba8888RowTileRleCursorOwner i32 \cursor:
    match gui_rgba8888_row_tile_rle_count_start cursor:
        Result::Ok owner:
            fail_count_owner owner 41
        Result::Err error:
            let kind_ok %bool count_kind_initial_complete gui_rgba8888_row_tile_rle_count_error_kind &error
            let count_ok %bool eq gui_rgba8888_row_tile_rle_count_error_accumulated_run_count &error 0
            let index_ok %bool eq gui_rgba8888_row_tile_rle_count_error_cursor_next_pixel_index &error 4
            let recovered %GuiRgba8888RowTileRleCursorOwner gui_rgba8888_row_tile_rle_count_error_finish_cursor error
            free_cursor_code recovered if and kind_ok and count_ok index_ok 0 42

fn count_tail_to_completion %fn GuiRgba8888RowTileRleCountOwner i32 \owner:
    match gui_rgba8888_row_tile_rle_count_step_budget owner 4:
        Result::Err error:
            fail_count_error error 35
        Result::Ok step:
            let status %GuiRgba8888RowTileRleCountStepStatus gui_rgba8888_row_tile_rle_count_step_status &step
            let owner1 %GuiRgba8888RowTileRleCountOwner gui_rgba8888_row_tile_rle_count_step_finish_owner step
            let status_ok %bool count_status_completed status
            let count_ok %bool eq gui_rgba8888_row_tile_rle_count_owner_accumulated_run_count &owner1 3
            let index_ok %bool eq gui_rgba8888_row_tile_rle_count_owner_cursor_next_pixel_index &owner1 4
            let cursor %GuiRgba8888RowTileRleCursorOwner gui_rgba8888_row_tile_rle_count_owner_finish_cursor owner1
            if and status_ok and count_ok index_ok:
                then count_complete_initial_rejected cursor
                else free_cursor_code cursor 36

fn count_partial_step %fn GuiRgba8888RowTileRleCountOwner i32 \owner:
    match gui_rgba8888_row_tile_rle_count_step_budget owner 2:
        Result::Err error:
            fail_count_error error 33
        Result::Ok step:
            let status %GuiRgba8888RowTileRleCountStepStatus gui_rgba8888_row_tile_rle_count_step_status &step
            let owner1 %GuiRgba8888RowTileRleCountOwner gui_rgba8888_row_tile_rle_count_step_finish_owner step
            let status_ok %bool count_status_pending status
            let count_ok %bool eq gui_rgba8888_row_tile_rle_count_owner_accumulated_run_count &owner1 2
            let index_ok %bool eq gui_rgba8888_row_tile_rle_count_owner_cursor_next_pixel_index &owner1 3
            if and status_ok and count_ok index_ok:
                then count_tail_to_completion owner1
                else fail_count_owner owner1 34

fn count_zero_budget_then_partial %fn GuiRgba8888RowTileRleCountOwner i32 \owner:
    match gui_rgba8888_row_tile_rle_count_step_budget owner 0:
        Result::Err error:
            fail_count_error error 31
        Result::Ok step:
            let status %GuiRgba8888RowTileRleCountStepStatus gui_rgba8888_row_tile_rle_count_step_status &step
            let owner1 %GuiRgba8888RowTileRleCountOwner gui_rgba8888_row_tile_rle_count_step_finish_owner step
            let status_ok %bool count_status_pending status
            let count_ok %bool eq gui_rgba8888_row_tile_rle_count_owner_accumulated_run_count &owner1 0
            let index_ok %bool eq gui_rgba8888_row_tile_rle_count_owner_cursor_next_pixel_index &owner1 0
            if and status_ok and count_ok index_ok:
                then count_partial_step owner1
                else fail_count_owner owner1 32

fn count_partial_then_complete %fn GuiRgba8888RowTileRleCursorOwner i32 \cursor:
    match gui_rgba8888_row_tile_rle_count_start cursor:
        Result::Err error:
            fail_count_start error 30
        Result::Ok owner:
            count_zero_budget_then_partial owner

fn count_negative_budget_error %fn GuiRgba8888RowTileRleCursorOwner i32 \cursor:
    match gui_rgba8888_row_tile_rle_count_start cursor:
        Result::Err error:
            fail_count_start error 43
        Result::Ok owner:
            match gui_rgba8888_row_tile_rle_count_step_budget owner sub 0 1:
                Result::Ok step:
                    fail_count_step step 44
                Result::Err error:
                    let kind_ok %bool count_kind_drain_invalid_budget gui_rgba8888_row_tile_rle_count_error_kind &error
                    let count_ok %bool eq gui_rgba8888_row_tile_rle_count_error_accumulated_run_count &error 0
                    let index_ok %bool eq gui_rgba8888_row_tile_rle_count_error_cursor_next_pixel_index &error 0
                    let recovered %GuiRgba8888RowTileRleCursorOwner gui_rgba8888_row_tile_rle_count_error_finish_cursor error
                    free_cursor_code recovered if and kind_ok and count_ok index_ok 0 45

fn run_count_mode %fn GuiRgba8888RowTileRleCursorOwner fn i32 i32 \cursor\mode:
    if eq mode 0:
        then count_partial_then_complete cursor
        else count_negative_budget_error cursor

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
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 707
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
    let partial_actual %i32 run_case 0
    let invalid_actual %i32 run_case 1
    let report:
        test::test_report_new "gui_render2d_row_tile_rle_count"
        |> test::test_report_push test::assert_eq_i32 "partial then complete" 0 partial_actual
        |> test::test_report_push test::assert_eq_i32 "negative budget lower error" 0 invalid_actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## forged row tile RLE count owners are not public application surface

通常の application code は row tile RLE count step を直 constructor で作れない。count が必要な場合は `gui_rgba8888_row_tile_rle_count_start` と `gui_rgba8888_row_tile_rle_count_step_budget` の owner boundary を通す。

neplg2:test[compile_fail]
diag_code: type.owner_aggregate.constructor_restricted
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "core/gui/dirty_region" as *
#import "core/result" as *

// render2d_row_tile_rle_count_no_encoded_buffer_no_platform_no_fallback

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            0
        Result::Ok surface:
            let dirty_owner %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 708
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
                                                                                            let step %GuiRgba8888RowTileRleCountStep GuiRgba8888RowTileRleCountStep GuiRgba8888RowTileRleCountStepStatus::Completed count_owner
                                                                                            match gui_rgba8888_row_tile_rle_count_step_free step:
                                                                                                Result::Err _:
                                                                                                    0
                                                                                                Result::Ok _:
                                                                                                    0
```
