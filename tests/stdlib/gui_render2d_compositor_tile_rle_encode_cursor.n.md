# GUI render2d compositor tile RLE encode cursor

このファイルは、F5mj の RGBA8888 compositor tile RLE encode cursor bridge が F5mi seed owner から lower ready cursor へ進み、writer / storage / packet / present / fallback へ進まないことを固定する。

source policy coverage labels:

- render2d_compositor_tile_rle_encode_cursor_facade_ok
- render2d_compositor_tile_rle_encode_cursor_seed_to_ready_cursor_ok
- render2d_compositor_tile_rle_encode_cursor_total_and_cursor_progress_ok
- render2d_compositor_tile_rle_encode_cursor_metadata_ok
- render2d_compositor_tile_rle_encode_cursor_finish_payload_delegation_ok
- render2d_compositor_tile_rle_encode_cursor_error_recovers_payload_owner_ok
- render2d_compositor_tile_rle_encode_cursor_no_writer_storage_packet_present_no_fallback

## compositor seed becomes ready encode cursor

[目的/もくてき]:
- F5mi seed owner の metadata を保持したまま、lower `gui_rgba8888_row_tile_rle_encode_cursor_start` を通すことを確認します。
- ready cursor では exact total run count、cursor progress、metadata が保持されることを確認します。
- ready cursor owner を frame entry owner へ戻せることを確認し、writer / storage / packet / present へ進まないことを source policy label で固定します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_compositor_tile_rle_encode_cursor_contract\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"encode cursor\" expected=\"0\" actual=\"0\" message=\"\"\n"
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
#import "core/option" as *
#import "core/result" as *
#import "std/test" as test

// render2d_compositor_tile_rle_encode_cursor_facade_ok
// render2d_compositor_tile_rle_encode_cursor_seed_to_ready_cursor_ok
// render2d_compositor_tile_rle_encode_cursor_total_and_cursor_progress_ok
// render2d_compositor_tile_rle_encode_cursor_metadata_ok
// render2d_compositor_tile_rle_encode_cursor_finish_payload_delegation_ok
// render2d_compositor_tile_rle_encode_cursor_error_recovers_payload_owner_ok
// render2d_compositor_tile_rle_encode_cursor_no_writer_storage_packet_present_no_fallback

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

fn fail_count_step_error %fn GuiRgba8888CompositorTileRleCountStepError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_count_step_error_free error:
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

fn fail_seed_error %fn GuiRgba8888CompositorTileRleEncodeSeedError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_encode_seed_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_cursor_owner %fn GuiRgba8888CompositorTileRleEncodeCursorOwner fn i32 i32 \owner\code:
    match gui_rgba8888_compositor_tile_rle_encode_cursor_owner_free owner:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_cursor_error %fn GuiRgba8888CompositorTileRleEncodeCursorError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_encode_cursor_error_free error:
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

fn cursor_owner_ok %fn &GuiRgba8888CompositorTileRleEncodeCursorOwner fn i32 bool \owner\frame_id:
    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_rle_encode_cursor_owner_metadata owner
    and:
        metadata_ok &metadata frame_id
        and:
            eq gui_rgba8888_compositor_tile_rle_encode_cursor_owner_total_run_count owner 2
            and:
                eq gui_rgba8888_compositor_tile_rle_encode_cursor_owner_cursor_next_pixel_index owner 0
                eq gui_rgba8888_compositor_tile_rle_encode_cursor_owner_cursor_pixel_count owner 2

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

fn cursor_start_success %fn GuiRgba8888CompositorTileRleEncodeSeedOwner fn bool i32 \seed\status_ok:
    match gui_rgba8888_compositor_tile_rle_encode_cursor_start seed:
        Result::Err error:
            fail_cursor_error error 40
        Result::Ok owner:
            let ok %bool and:
                status_ok
                cursor_owner_ok &owner 409
            match gui_rgba8888_compositor_tile_rle_encode_cursor_owner_finish_entry owner:
                Result::Err error:
                    fail_payload_finish error 41
                Result::Ok entry:
                    if ok:
                        then expect_complete_entry entry 42
                        else free_entry_code entry 43

fn seed_prepare_success %fn GuiRgba8888CompositorTileRleCountCompletedOwner fn bool i32 \completed\status_ok:
    match gui_rgba8888_compositor_tile_rle_encode_seed_prepare completed:
        Result::Err error:
            fail_seed_error error 33
        Result::Ok seed:
            cursor_start_success seed status_ok

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
                    seed_prepare_success completed status_completed status

fn run_payload %fn GuiRgba8888CompositorTilePayloadOwner i32 \payload:
    match gui_rgba8888_compositor_tile_rle_count_start payload:
        Result::Err error:
            fail_count_start error 20
        Result::Ok owner:
            completed_prepare_success owner

fn run_case %fn i32 i32 \mode:
    match build_entry 409:
        Result::Err code:
            code
        Result::Ok entry:
            match build_payload entry:
                Result::Err code:
                    code
                Result::Ok payload:
                    run_payload payload

fn main %impure fn void i32 \void:
    let actual %i32 run_case 0
    let report:
        test::test_report_new "gui_render2d_compositor_tile_rle_encode_cursor_contract"
        |> test::test_report_push test::assert_eq_i32 "encode cursor" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
