# GUI render2d compositor tile RLE storage

このファイルは、F5ml の RGBA8888 compositor tile RLE storage bridge が F5mk writer plan owner から lower storage allocation へ進み、write step / encoded seal / packet / present / fallback へ進まないことを固定する。

source policy coverage labels:

- render2d_compositor_tile_rle_storage_facade_ok
- render2d_compositor_tile_rle_storage_writer_plan_to_storage_ok
- render2d_compositor_tile_rle_storage_exact_byte_count_ok
- render2d_compositor_tile_rle_storage_metadata_ok
- render2d_compositor_tile_rle_storage_finish_payload_recovery_ok
- render2d_compositor_tile_rle_storage_prepare_error_recovery_source_policy_ok
- render2d_compositor_tile_rle_storage_free_delegates_lower_storage_ok
- render2d_compositor_tile_rle_storage_no_write_encoded_packet_present_no_fallback

## compositor writer plan becomes storage owner

[目的/もくてき]:
- F5mk writer plan owner の metadata を保持したまま、lower `gui_rgba8888_row_tile_rle_storage_prepare` を通すことを確認します。
- storage owner では exact total run count、encoded byte count、cursor progress、metadata が保持されることを確認します。
- storage owner は payload recovery / free で閉じ、write step、encoded seal、packet、present へ進まないことを source policy label で固定します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_compositor_tile_rle_storage_contract\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"storage\" expected=\"0\" actual=\"0\" message=\"\"\n"
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

// render2d_compositor_tile_rle_storage_facade_ok
// render2d_compositor_tile_rle_storage_writer_plan_to_storage_ok
// render2d_compositor_tile_rle_storage_exact_byte_count_ok
// render2d_compositor_tile_rle_storage_metadata_ok
// render2d_compositor_tile_rle_storage_finish_payload_recovery_ok
// render2d_compositor_tile_rle_storage_prepare_error_recovery_source_policy_ok
// render2d_compositor_tile_rle_storage_free_delegates_lower_storage_ok
// render2d_compositor_tile_rle_storage_no_write_encoded_packet_present_no_fallback

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

fn fail_cursor_error %fn GuiRgba8888CompositorTileRleEncodeCursorError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_encode_cursor_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_writer_plan_owner %fn GuiRgba8888CompositorTileRleWriterPlanOwner fn i32 i32 \owner\code:
    match gui_rgba8888_compositor_tile_rle_writer_plan_owner_free owner:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_writer_plan_error %fn GuiRgba8888CompositorTileRleWriterPlanError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_writer_plan_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_storage_owner %fn GuiRgba8888CompositorTileRleStorageOwner fn i32 i32 \owner\code:
    match gui_rgba8888_compositor_tile_rle_storage_owner_free owner:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_storage_prepare %fn GuiRgba8888CompositorTileRleStoragePrepareError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_storage_prepare_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_storage_finish %fn GuiRgba8888CompositorTileRleStorageFinishError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_storage_finish_error_free error:
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

fn writer_plan_owner_ok %fn &GuiRgba8888CompositorTileRleWriterPlanOwner fn i32 bool \owner\frame_id:
    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_rle_writer_plan_owner_metadata owner
    and:
        metadata_ok &metadata frame_id
        and:
            eq gui_rgba8888_compositor_tile_rle_writer_plan_owner_total_run_count owner 2
            and:
                eq gui_rgba8888_compositor_tile_rle_writer_plan_owner_encoded_byte_count owner 24
                and:
                    eq gui_rgba8888_compositor_tile_rle_writer_plan_owner_cursor_next_pixel_index owner 0
                    eq gui_rgba8888_compositor_tile_rle_writer_plan_owner_cursor_pixel_count owner 2

fn storage_owner_ok %fn &GuiRgba8888CompositorTileRleStorageOwner fn i32 bool \owner\frame_id:
    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_rle_storage_owner_metadata owner
    and:
        metadata_ok &metadata frame_id
        and:
            eq gui_rgba8888_compositor_tile_rle_storage_owner_total_run_count owner 2
            and:
                eq gui_rgba8888_compositor_tile_rle_storage_owner_encoded_byte_count owner 24
                and:
                    eq gui_rgba8888_compositor_tile_rle_storage_owner_cursor_next_pixel_index owner 0
                    eq gui_rgba8888_compositor_tile_rle_storage_owner_cursor_pixel_count owner 2

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

fn storage_finish_payload_success %fn GuiRgba8888CompositorTileRleStorageOwner fn bool i32 \owner\owner_ok:
    match gui_rgba8888_compositor_tile_rle_storage_owner_finish_payload owner:
        Result::Err error:
            fail_storage_finish error 55
        Result::Ok payload:
            match gui_rgba8888_compositor_tile_payload_owner_finish_entry payload:
                Result::Err error:
                    fail_payload_finish error 56
                Result::Ok entry:
                    if owner_ok:
                        then expect_complete_entry entry 57
                        else free_entry_code entry 58

fn storage_free_success %fn GuiRgba8888CompositorTileRleStorageOwner fn bool i32 \owner\owner_ok:
    match gui_rgba8888_compositor_tile_rle_storage_owner_free owner:
        Result::Err _:
            61
        Result::Ok _:
            if owner_ok:
                then 0
                else 62

fn storage_prepare_success %fn GuiRgba8888CompositorTileRleWriterPlanOwner fn bool fn i32 i32 \plan\plan_ok\mode:
    match gui_rgba8888_compositor_tile_rle_storage_prepare plan:
        Result::Err error:
            fail_storage_prepare error 54
        Result::Ok owner:
            let owner_ok %bool and plan_ok storage_owner_ok &owner 509
            if eq mode 0:
                then storage_finish_payload_success owner owner_ok
                else storage_free_success owner owner_ok

fn writer_plan_success %fn GuiRgba8888CompositorTileRleEncodeCursorOwner fn bool fn i32 i32 \ready\ready_ok\mode:
    match gui_rgba8888_compositor_tile_rle_writer_plan_prepare ready:
        Result::Err error:
            fail_writer_plan_error error 44
        Result::Ok owner:
            let plan_ok %bool and ready_ok writer_plan_owner_ok &owner 509
            storage_prepare_success owner plan_ok mode

fn cursor_start_success %fn GuiRgba8888CompositorTileRleEncodeSeedOwner fn bool fn i32 i32 \seed\status_ok\mode:
    match gui_rgba8888_compositor_tile_rle_encode_cursor_start seed:
        Result::Err error:
            fail_cursor_error error 40
        Result::Ok ready:
            writer_plan_success ready status_ok mode

fn seed_prepare_success %fn GuiRgba8888CompositorTileRleCountCompletedOwner fn bool fn i32 i32 \completed\status_ok\mode:
    match gui_rgba8888_compositor_tile_rle_encode_seed_prepare completed:
        Result::Err error:
            fail_seed_error error 33
        Result::Ok seed:
            cursor_start_success seed status_ok mode

fn completed_prepare_success %fn GuiRgba8888CompositorTileRleCountOwner fn i32 i32 \owner\mode:
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
                    seed_prepare_success completed status_completed status mode

fn run_payload %fn GuiRgba8888CompositorTilePayloadOwner fn i32 i32 \payload\mode:
    match gui_rgba8888_compositor_tile_rle_count_start payload:
        Result::Err error:
            fail_count_start error 20
        Result::Ok owner:
            completed_prepare_success owner mode

fn run_case %fn i32 i32 \mode:
    match build_entry 509:
        Result::Err code:
            code
        Result::Ok entry:
            match build_payload entry:
                Result::Err code:
                    code
                Result::Ok payload:
                    run_payload payload mode

fn main %impure fn void i32 \void:
    let finish_payload_result %i32 run_case 0
    let free_result %i32 if eq finish_payload_result 0:
        then run_case 1
        else finish_payload_result
    let report:
        test::test_report_new "gui_render2d_compositor_tile_rle_storage_contract"
        |> test::test_report_push test::assert_eq_i32 "storage" 0 free_result
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## malformed writer plan owner is not public application surface

通常の application code は malformed compositor writer plan owner を直 constructor で作れない。F5ml prepare error recovery の read-before-consume 順序と writer plan owner recovery は `nodesrc/test_web_gui_font_rendering_contract.js` の source policy で固定する。

neplg2:test[compile_fail]
diag_code: type.owner_aggregate.constructor_restricted
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *

// render2d_compositor_tile_rle_storage_prepare_error_recovery_source_policy_ok

fn forge_compositor_writer_plan %fn GuiRgba8888RowTileRleWriterPlanOwner fn GuiRgba8888CompositorFrameEntryMetadata GuiRgba8888CompositorTileRleWriterPlanOwner \lower\metadata:
    GuiRgba8888CompositorTileRleWriterPlanOwner lower metadata

fn main %impure fn void i32 \void:
    0
```
