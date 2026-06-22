# GUI render2d compositor tile RLE completed count

このファイルは、F5mh の RGBA8888 compositor tile RLE completed count bridge が F5mf/F5mg count owner から lower completed evidence へ進み、count step 再実行 / encode / present / fallback へ進まないことを固定する。

source policy coverage labels:

- render2d_compositor_tile_rle_count_completed_facade_ok
- render2d_compositor_tile_rle_count_completed_success_total_ok
- render2d_compositor_tile_rle_count_completed_metadata_ok
- render2d_compositor_tile_rle_count_completed_pending_rejected_ok
- render2d_compositor_tile_rle_count_completed_error_recovers_count_owner_ok
- render2d_compositor_tile_rle_count_completed_finish_free_delegation_ok
- render2d_compositor_tile_rle_count_completed_no_count_step_no_encode_no_present_no_fallback

## compositor tile RLE completed count becomes capacity evidence

[目的/もくてき]:
- F5mf count owner の metadata を保持したまま、lower `gui_rgba8888_row_tile_rle_count_completed_prepare` を通すことを確認します。
- completed success では total run count と cursor next pixel index が確定し、metadata が保持されることを確認します。
- pending cursor は owner-bearing error になり、metadata 付き count owner へ回収できることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_compositor_tile_rle_count_completed_contract\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"completed count evidence\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"pending recovery\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/dirty_region" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as test

// render2d_compositor_tile_rle_count_completed_facade_ok
// render2d_compositor_tile_rle_count_completed_success_total_ok
// render2d_compositor_tile_rle_count_completed_metadata_ok
// render2d_compositor_tile_rle_count_completed_pending_rejected_ok
// render2d_compositor_tile_rle_count_completed_error_recovers_count_owner_ok
// render2d_compositor_tile_rle_count_completed_finish_free_delegation_ok
// render2d_compositor_tile_rle_count_completed_no_count_step_no_encode_no_present_no_fallback

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

fn fail_count_step_error %fn GuiRgba8888CompositorTileRleCountStepError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_count_step_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_completed_owner %fn GuiRgba8888CompositorTileRleCountCompletedOwner fn i32 i32 \owner\code:
    match gui_rgba8888_compositor_tile_rle_count_completed_owner_free owner:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_completed_error %fn GuiRgba8888CompositorTileRleCountCompletedError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_count_completed_error_free error:
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

fn status_completed %fn GuiRgba8888RowTileRleCountStepStatus bool \status:
    match status:
        GuiRgba8888RowTileRleCountStepStatus::Completed:
            true
        _:
            false

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

fn completed_kind_count_not_completed %fn GuiRgba8888CompositorTileRleCountCompletedErrorKind bool \kind:
    match kind:
        GuiRgba8888CompositorTileRleCountCompletedErrorKind::CompletedPrepareFailed lower_kind:
            match lower_kind:
                GuiRgba8888RowTileRleCountCompletedErrorKind::CountNotCompleted:
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

fn completed_owner_ok %fn &GuiRgba8888CompositorTileRleCountCompletedOwner fn i32 bool \owner\frame_id:
    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_rle_count_completed_owner_metadata owner
    and:
        metadata_ok &metadata frame_id
        and:
            eq gui_rgba8888_compositor_tile_rle_count_completed_owner_total_run_count owner 2
            eq gui_rgba8888_compositor_tile_rle_count_completed_owner_cursor_next_pixel_index owner 2

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

fn completed_prepare_success %fn GuiRgba8888CompositorTileRleCountOwner i32 \owner:
    match gui_rgba8888_compositor_tile_rle_count_step_budget owner 4:
        Result::Err error:
            fail_count_step_error error 31
        Result::Ok step:
            let status %GuiRgba8888RowTileRleCountStepStatus gui_rgba8888_compositor_tile_rle_count_step_status &step
            let owner1 %GuiRgba8888CompositorTileRleCountOwner gui_rgba8888_compositor_tile_rle_count_step_finish_owner step
            match gui_rgba8888_compositor_tile_rle_count_completed_prepare owner1:
                Result::Err error:
                    fail_completed_error error 32
                Result::Ok completed:
                    let ok %bool and:
                        status_completed status
                        completed_owner_ok &completed 298
                    match gui_rgba8888_compositor_tile_rle_count_completed_owner_finish_entry completed:
                        Result::Err error:
                            fail_count_finish error 33
                        Result::Ok entry:
                            if ok:
                                then expect_complete_entry entry 34
                                else free_entry_code entry 35

fn completed_prepare_pending %fn GuiRgba8888CompositorTileRleCountOwner i32 \owner:
    match gui_rgba8888_compositor_tile_rle_count_completed_prepare owner:
        Result::Ok completed:
            fail_completed_owner completed 41
        Result::Err error:
            let kind_ok %bool completed_kind_count_not_completed gui_rgba8888_compositor_tile_rle_count_completed_error_kind &error
            let category_ok %bool category_invalid_command gui_rgba8888_compositor_tile_rle_count_completed_error_category_value &error
            let total_ok %bool eq gui_rgba8888_compositor_tile_rle_count_completed_error_total_run_count &error 0
            let index_ok %bool eq gui_rgba8888_compositor_tile_rle_count_completed_error_cursor_next_pixel_index &error 0
            let count %GuiRgba8888CompositorTileRleCountOwner gui_rgba8888_compositor_tile_rle_count_completed_error_finish_count_owner error
            let recover_ok %bool count_owner_progress_ok &count 299 0 0
            match gui_rgba8888_compositor_tile_rle_count_owner_finish_entry count:
                Result::Err finish_error:
                    fail_count_finish finish_error 42
                Result::Ok entry:
                    let ok %bool and:
                        kind_ok
                        and:
                            category_ok
                            and:
                                total_ok
                                and index_ok recover_ok
                    if ok:
                        then expect_complete_entry entry 43
                        else free_entry_code entry 44

fn run_payload %fn GuiRgba8888CompositorTilePayloadOwner fn i32 i32 \payload\mode:
    match gui_rgba8888_compositor_tile_rle_count_start payload:
        Result::Err error:
            fail_count_start error 20
        Result::Ok owner:
            if eq mode 0:
                then completed_prepare_success owner
                else completed_prepare_pending owner

fn run_case %fn i32 i32 \mode:
    let frame_id %i32 if eq mode 0 298 299
    match build_entry frame_id:
        Result::Err code:
            code
        Result::Ok entry:
            match build_payload entry:
                Result::Err code:
                    code
                Result::Ok payload:
                    run_payload payload mode

fn main %impure fn void i32 \void:
    let success_actual %i32 run_case 0
    let pending_actual %i32 run_case 1
    let report:
        test::test_report_new "gui_render2d_compositor_tile_rle_count_completed_contract"
        |> test::test_report_push test::assert_eq_i32 "completed count evidence" 0 success_actual
        |> test::test_report_push test::assert_eq_i32 "pending recovery" 0 pending_actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
