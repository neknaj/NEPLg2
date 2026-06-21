# GUI render2d compositor tile RLE count step

このファイルは、F5mg の RGBA8888 compositor tile RLE count step bridge が F5mf count owner から lower count step を 1 slice だけ進め、completed count / encode / present / fallback へ進まないことを固定する。

source policy coverage labels:

- render2d_compositor_tile_rle_count_step_facade_ok
- render2d_compositor_tile_rle_count_step_zero_budget_pending_ok
- render2d_compositor_tile_rle_count_step_completion_status_ok
- render2d_compositor_tile_rle_count_step_metadata_ok
- render2d_compositor_tile_rle_count_step_negative_budget_payload_recovery_ok
- render2d_compositor_tile_rle_count_step_no_fake_owner_no_completed_no_encode_no_present_no_fallback

## compositor tile RLE count step advances lower count owner

[目的/もくてき]:
- F5mf count owner の metadata を保持したまま、lower `gui_rgba8888_row_tile_rle_count_step_budget` を 1 回ずつ通すことを確認します。
- zero budget では Pending、completion budget では Completed になり、accumulated run count と cursor next pixel index が進むことを確認します。
- negative budget error では fake continuation owner を作らず、payload owner へ回収して entry owner へ戻せることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_compositor_tile_rle_count_step_contract\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"zero budget then completed\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"negative budget payload recovery\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "alloc/gui/render2d/compositor_batch_range" as *
#import "alloc/gui/render2d/compositor_byte_storage" as *
#import "alloc/gui/render2d/compositor_frame_entry" as *
#import "alloc/gui/render2d/compositor_tile_payload" as *
#import "alloc/gui/render2d/compositor_tile_plan" as *
#import "alloc/gui/render2d/compositor_tile_rle_count" as *
#import "alloc/gui/render2d/compositor_tile_rle_count_step" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/row_tile_payload" as *
#import "alloc/gui/render2d/row_tile_plan" as *
#import "alloc/gui/render2d/row_tile_rle" as *
#import "alloc/gui/render2d/row_tile_rle_count" as *
#import "alloc/gui/render2d/row_tile_rle_drain" as *
#import "alloc/gui/render2d/software_surface" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/dirty_region" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as test

// render2d_compositor_tile_rle_count_step_facade_ok
// render2d_compositor_tile_rle_count_step_zero_budget_pending_ok
// render2d_compositor_tile_rle_count_step_completion_status_ok
// render2d_compositor_tile_rle_count_step_metadata_ok
// render2d_compositor_tile_rle_count_step_negative_budget_payload_recovery_ok
// render2d_compositor_tile_rle_count_step_no_fake_owner_no_completed_no_encode_no_present_no_fallback

fn free_entry_code %fn GuiRgba8888CompositorFrameEntryOwner fn i32 i32 \entry\code:
    match gui_rgba8888_compositor_frame_entry_owner_free entry:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_surface_write %fn GuiRgba8888SoftwareSurfaceWriteError fn i32 i32 \error\code:
    match gui_rgba8888_software_surface_write_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_dirty_push %fn GuiRgba8888SoftwareSurfaceDirtyPushError fn i32 i32 \error\code:
    match gui_rgba8888_software_surface_dirty_push_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_entry_prepare %fn GuiRgba8888CompositorFrameEntryPrepareError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_frame_entry_prepare_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_range_prepare %fn GuiRgba8888CompositorBatchRangeError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_batch_range_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_byte_prepare %fn GuiRgba8888CompositorByteStoragePrepareError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_byte_storage_prepare_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_tile_plan_prepare %fn GuiRgba8888CompositorTilePlanPrepareError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_plan_prepare_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_payload_prepare %fn GuiRgba8888CompositorTilePayloadPrepareError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_payload_prepare_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_payload_finish %fn GuiRgba8888CompositorTilePayloadFinishError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_payload_finish_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_count_start %fn GuiRgba8888CompositorTileRleCountStartError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_count_start_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_count_finish %fn GuiRgba8888CompositorTileRleCountFinishError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_count_finish_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_count_owner %fn GuiRgba8888CompositorTileRleCountOwner fn i32 i32 \owner\code:
    match gui_rgba8888_compositor_tile_rle_count_owner_free owner:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_count_step %fn GuiRgba8888CompositorTileRleCountStep fn i32 i32 \step\code:
    match gui_rgba8888_compositor_tile_rle_count_step_free step:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_count_step_error %fn GuiRgba8888CompositorTileRleCountStepError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_count_step_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn metadata_ok %fn &GuiRgba8888CompositorFrameEntryMetadata fn i32 bool \metadata\frame_id:
    and:
        eq gui_rgba8888_compositor_frame_entry_metadata_frame_id metadata frame_id
        and:
            eq gui_rgba8888_compositor_frame_entry_metadata_width metadata 2
            and:
                eq gui_rgba8888_compositor_frame_entry_metadata_height metadata 3
                and:
                    eq gui_rgba8888_compositor_frame_entry_metadata_row_start metadata 0
                    and:
                        eq gui_rgba8888_compositor_frame_entry_metadata_row_count metadata 3
                        and:
                            eq gui_rgba8888_compositor_frame_entry_metadata_batch_count metadata 1
                            eq gui_rgba8888_compositor_frame_entry_metadata_max_rows_per_batch metadata 3

fn status_pending %fn GuiRgba8888RowTileRleCountStepStatus bool \status:
    match status:
        GuiRgba8888RowTileRleCountStepStatus::Pending:
            true
        _:
            false

fn status_completed %fn GuiRgba8888RowTileRleCountStepStatus bool \status:
    match status:
        GuiRgba8888RowTileRleCountStepStatus::Completed:
            true
        _:
            false

fn cursor_complete_ok %fn &GuiRgba8888CompositorTileRleCountOwner bool \owner:
    match gui_rgba8888_compositor_tile_rle_count_owner_cursor_status owner:
        Result::Err _:
            false
        Result::Ok status:
            match status:
                GuiRgba8888RowTileRleCursorStatus::Complete:
                    true
                _:
                    false

fn count_owner_progress_ok %fn &GuiRgba8888CompositorTileRleCountOwner fn i32 fn i32 fn i32 bool \owner\frame_id\run_count\next_pixel_index:
    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_rle_count_owner_metadata owner
    and:
        metadata_ok &metadata frame_id
        and:
            eq gui_rgba8888_compositor_tile_rle_count_owner_accumulated_run_count owner run_count
            eq gui_rgba8888_compositor_tile_rle_count_owner_cursor_next_pixel_index owner next_pixel_index

fn category_invalid_command %fn Option GuiError bool \category:
    match category:
        Option::Some error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false
        Option::None:
            false

fn step_kind_drain_invalid_budget %fn GuiRgba8888CompositorTileRleCountStepErrorKind bool \kind:
    match kind:
        GuiRgba8888CompositorTileRleCountStepErrorKind::CountStepFailed lower_kind:
            match lower_kind:
                GuiRgba8888RowTileRleCountErrorKind::DrainFailed drain_kind:
                    match drain_kind:
                        GuiRgba8888RowTileRleDrainErrorKind::InvalidBudget:
                            true
                        _:
                            false
                _:
                    false

fn expect_complete_entry %fn GuiRgba8888CompositorFrameEntryOwner fn i32 i32 \entry\code:
    match gui_rgba8888_compositor_batch_range_prepare entry:
        Result::Ok extra:
            let recovered %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_range_owner_finish_entry extra
            free_entry_code recovered code
        Result::Err complete:
            match gui_rgba8888_compositor_batch_range_error_category_value &complete:
                Option::Some category:
                    match category:
                        GuiError::InvalidCommand:
                            let recovered %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_range_error_finish_entry complete
                            free_entry_code recovered 0
                        _:
                            fail_range_prepare complete code
                Option::None:
                    fail_range_prepare complete code

fn build_entry %fn i32 Result GuiRgba8888CompositorFrameEntryOwner i32 \frame_id:
    match gui_rgba8888_software_surface_create 2 3:
        Result::Err _:
            Result::Err 1
        Result::Ok surface0:
            let r0 %u8 cast 31
            let g0 %u8 cast 32
            let b0 %u8 cast 33
            let a0 %u8 cast 34
            let color0 %Rgba8888 rgba8888_new r0 g0 b0 a0
            match gui_rgba8888_software_surface_write_pixel surface0 0 2 color0:
                Result::Err error:
                    Result::Err fail_surface_write error 2
                Result::Ok surface1:
                    let r1 %u8 cast 41
                    let g1 %u8 cast 42
                    let b1 %u8 cast 43
                    let a1 %u8 cast 44
                    let color1 %Rgba8888 rgba8888_new r1 g1 b1 a1
                    match gui_rgba8888_software_surface_write_pixel surface1 1 2 color1:
                        Result::Err error:
                            Result::Err fail_surface_write error 3
                        Result::Ok surface2:
                            let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface2
                            match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
                                Result::Err error:
                                    Result::Err fail_dirty_push error 4
                                Result::Ok dirty_owner:
                                    let config %GuiRgba8888CompositorFrameEntryConfig gui_rgba8888_compositor_frame_entry_config frame_id 3
                                    match gui_rgba8888_compositor_frame_entry_prepare dirty_owner config:
                                        Result::Err error:
                                            Result::Err fail_entry_prepare error 5
                                        Result::Ok entry:
                                            Result::Ok entry

fn build_payload %fn GuiRgba8888CompositorFrameEntryOwner Result GuiRgba8888CompositorTilePayloadOwner i32 \entry:
    match gui_rgba8888_compositor_batch_range_prepare entry:
        Result::Err error:
            Result::Err fail_range_prepare error 10
        Result::Ok range_owner:
            match gui_rgba8888_compositor_byte_storage_prepare range_owner:
                Result::Err error:
                    Result::Err fail_byte_prepare error 11
                Result::Ok storage_owner:
                    let config %GuiRgba8888RowTilePlanConfig GuiRgba8888RowTilePlanConfig 2
                    match gui_rgba8888_compositor_tile_plan_prepare storage_owner config:
                        Result::Err error:
                            Result::Err fail_tile_plan_prepare error 12
                        Result::Ok tile_plan:
                            match gui_rgba8888_compositor_tile_payload_prepare tile_plan 1:
                                Result::Err error:
                                    Result::Err fail_payload_prepare error 13
                                Result::Ok payload:
                                    Result::Ok payload

fn finish_count_owner_entry %fn GuiRgba8888CompositorTileRleCountOwner fn bool fn i32 i32 \owner\ok\code:
    match gui_rgba8888_compositor_tile_rle_count_owner_finish_entry owner:
        Result::Err error:
            fail_count_finish error code
        Result::Ok entry:
            if ok:
                then expect_complete_entry entry code
                else free_entry_code entry add code 1

fn completed_step %fn GuiRgba8888CompositorTileRleCountOwner i32 \owner:
    match gui_rgba8888_compositor_tile_rle_count_step_budget owner 4:
        Result::Err error:
            fail_count_step_error error 33
        Result::Ok step:
            let status %GuiRgba8888RowTileRleCountStepStatus gui_rgba8888_compositor_tile_rle_count_step_status &step
            let owner1 %GuiRgba8888CompositorTileRleCountOwner gui_rgba8888_compositor_tile_rle_count_step_finish_owner step
            let ok %bool and:
                status_completed status
                and:
                    count_owner_progress_ok &owner1 198 2 2
                    cursor_complete_ok &owner1
            finish_count_owner_entry owner1 ok 34

fn zero_then_completed_step %fn GuiRgba8888CompositorTileRleCountOwner i32 \owner:
    match gui_rgba8888_compositor_tile_rle_count_step_budget owner 0:
        Result::Err error:
            fail_count_step_error error 31
        Result::Ok step:
            let status %GuiRgba8888RowTileRleCountStepStatus gui_rgba8888_compositor_tile_rle_count_step_status &step
            let owner1 %GuiRgba8888CompositorTileRleCountOwner gui_rgba8888_compositor_tile_rle_count_step_finish_owner step
            let ok %bool and:
                status_pending status
                count_owner_progress_ok &owner1 198 0 0
            if ok:
                then completed_step owner1
                else fail_count_owner owner1 32

fn run_success_payload %fn GuiRgba8888CompositorTilePayloadOwner i32 \payload:
    match gui_rgba8888_compositor_tile_rle_count_start payload:
        Result::Err error:
            fail_count_start error 30
        Result::Ok owner:
            zero_then_completed_step owner

fn run_error_payload %fn GuiRgba8888CompositorTilePayloadOwner i32 \payload:
    match gui_rgba8888_compositor_tile_rle_count_start payload:
        Result::Err error:
            fail_count_start error 40
        Result::Ok owner:
            match gui_rgba8888_compositor_tile_rle_count_step_budget owner sub 0 1:
                Result::Ok step:
                    fail_count_step step 41
                Result::Err error:
                    let kind_ok %bool step_kind_drain_invalid_budget gui_rgba8888_compositor_tile_rle_count_step_error_kind &error
                    let category_ok %bool category_invalid_command gui_rgba8888_compositor_tile_rle_count_step_error_category_value &error
                    let count_ok %bool eq gui_rgba8888_compositor_tile_rle_count_step_error_accumulated_run_count &error 0
                    let index_ok %bool eq gui_rgba8888_compositor_tile_rle_count_step_error_cursor_next_pixel_index &error 0
                    let ok %bool and:
                        kind_ok
                        and:
                            category_ok
                            and count_ok index_ok
                    match gui_rgba8888_compositor_tile_rle_count_step_error_finish_entry error:
                        Result::Err finish_error:
                            fail_payload_finish finish_error 42
                        Result::Ok entry:
                            if ok:
                                then expect_complete_entry entry 43
                                else free_entry_code entry 44

fn run_mode_payload %fn GuiRgba8888CompositorTilePayloadOwner fn i32 i32 \payload\mode:
    if eq mode 0:
        then run_success_payload payload
        else run_error_payload payload

fn run_case %fn i32 i32 \mode:
    let frame_id %i32 if eq mode 0 198 199
    match build_entry frame_id:
        Result::Err code:
            code
        Result::Ok entry:
            match build_payload entry:
                Result::Err code:
                    code
                Result::Ok payload:
                    run_mode_payload payload mode

fn main %impure fn void i32 \void:
    let success_actual %i32 run_case 0
    let error_actual %i32 run_case 1
    let report:
        test::test_report_new "gui_render2d_compositor_tile_rle_count_step_contract"
        |> test::test_report_push test::assert_eq_i32 "zero budget then completed" 0 success_actual
        |> test::test_report_push test::assert_eq_i32 "negative budget payload recovery" 0 error_actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## forged compositor tile RLE count steps are not public application surface

通常の application code は compositor tile RLE count step を直 constructor で作れない。count step が必要な場合は `gui_rgba8888_compositor_tile_rle_count_start` と `gui_rgba8888_compositor_tile_rle_count_step_budget` の owner boundary を通す。

neplg2:test[compile_fail]
diag_code: type.owner_aggregate.constructor_restricted
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "core/gui/dirty_region" as *
#import "core/result" as *

// render2d_compositor_tile_rle_count_step_no_fake_owner_no_completed_no_encode_no_present_no_fallback

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            0
        Result::Ok surface:
            let dirty_owner %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 709
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
                                                                                            let metadata %GuiRgba8888CompositorFrameEntryMetadata GuiRgba8888CompositorFrameEntryMetadata 709 1 1 0 1 1 1
                                                                                            let owner %GuiRgba8888CompositorTileRleCountOwner GuiRgba8888CompositorTileRleCountOwner count_owner metadata
                                                                                            let step %GuiRgba8888CompositorTileRleCountStep GuiRgba8888CompositorTileRleCountStep GuiRgba8888RowTileRleCountStepStatus::Completed owner
                                                                                            match gui_rgba8888_compositor_tile_rle_count_step_free step:
                                                                                                Result::Err _:
                                                                                                    0
                                                                                                Result::Ok _:
                                                                                                    0
```
