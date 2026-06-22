# GUI render2d compositor tile RLE write step

このファイルは、F5mn の RGBA8888 compositor tile RLE write step bridge が F5mm write cursor owner から lower write step へ進み、encoded seal / packet / present / fallback へ進まないことを固定する。

source policy coverage labels:

- render2d_compositor_tile_rle_write_step_facade_ok
- render2d_compositor_tile_rle_write_step_status_error_kind_runtime_ok
- render2d_compositor_tile_rle_write_step_one_run_ok
- render2d_compositor_tile_rle_write_step_completion_status_ok
- render2d_compositor_tile_rle_write_step_progress_ok
- render2d_compositor_tile_rle_write_step_metadata_ok
- render2d_compositor_tile_rle_write_step_finish_payload_recovery_ok
- render2d_compositor_tile_rle_write_step_error_recovery_source_policy_ok
- render2d_compositor_tile_rle_write_step_free_delegates_write_cursor_ok
- render2d_compositor_tile_rle_write_step_no_encoded_packet_present_no_fallback

## write step facade and value-only status wrappers compile in the wasm runner

[目的/もくてき]:
- F5mn facade が lower write step status と wrapped lower error kind を公開型として扱えることを確認します。
- owner-backed write cursor はこの軽量 smoke では forge せず、step の owner 遷移は下の source policy と constructor 制限テストで固定します。

neplg2:test
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/compositor_tile_rle_write_step" as *
#import "alloc/gui/render2d/row_tile_rle_storage" as *
#import "core/math" as *

// render2d_compositor_tile_rle_write_step_facade_ok
// render2d_compositor_tile_rle_write_step_status_error_kind_runtime_ok

fn write_status_code %fn GuiRgba8888RowTileRleWriteStepStatus i32 \status:
    match status:
        GuiRgba8888RowTileRleWriteStepStatus::WroteRun:
            1
        GuiRgba8888RowTileRleWriteStepStatus::Completed:
            2

fn write_error_kind_code %fn GuiRgba8888CompositorTileRleWriteStepErrorKind i32 \kind:
    match kind:
        GuiRgba8888CompositorTileRleWriteStepErrorKind::WriteStepFailed lower:
            match lower:
                GuiRgba8888RowTileRleWriteStepErrorKind::WrittenByteCountOverflow:
                    3
                _:
                    4

fn main %impure fn void i32 \void:
    let status %GuiRgba8888RowTileRleWriteStepStatus GuiRgba8888RowTileRleWriteStepStatus::WroteRun
    let lower %GuiRgba8888RowTileRleWriteStepErrorKind GuiRgba8888RowTileRleWriteStepErrorKind::WrittenByteCountOverflow
    let kind %GuiRgba8888CompositorTileRleWriteStepErrorKind GuiRgba8888CompositorTileRleWriteStepErrorKind::WriteStepFailed lower
    if and eq write_status_code status 1 eq write_error_kind_code kind 3:
        then 0
        else 1
```

## compositor write cursor source policy advances one lower write step

[目的/もくてき]:
- F5mm write cursor owner の metadata を保持したまま、lower `gui_rgba8888_row_tile_rle_write_cursor_step_one` を 1 回だけ通すことを source policy で確認します。
- lower step の `WroteRun` / `Completed` status、count/progress/metadata access、payload recovery / free delegation は `nodesrc/test_web_gui_font_rendering_contract.js` で実装順序を固定します。
- default WASM runner では public owner chain 全体の Resource check が timeout するため、この owner-backed E2E fixture は source policy 用に skip します。

neplg2:test[skip]
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/software_surface" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/compositor_frame_entry" as *
#import "alloc/gui/render2d/compositor_batch_range" as *
#import "alloc/gui/render2d/compositor_byte_storage" as *
#import "alloc/gui/render2d/compositor_tile_plan" as *
#import "alloc/gui/render2d/compositor_tile_payload" as *
#import "alloc/gui/render2d/compositor_tile_rle_count" as *
#import "alloc/gui/render2d/compositor_tile_rle_count_step" as *
#import "alloc/gui/render2d/compositor_tile_rle_count_completed" as *
#import "alloc/gui/render2d/compositor_tile_rle_encode_seed" as *
#import "alloc/gui/render2d/compositor_tile_rle_encode_cursor" as *
#import "alloc/gui/render2d/compositor_tile_rle_writer_plan" as *
#import "alloc/gui/render2d/compositor_tile_rle_storage" as *
#import "alloc/gui/render2d/compositor_tile_rle_write_cursor" as *
#import "alloc/gui/render2d/compositor_tile_rle_write_step" as *
#import "alloc/gui/render2d/row_tile_plan" as *
#import "alloc/gui/render2d/row_tile_rle_count" as *
#import "alloc/gui/render2d/row_tile_rle_storage" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/dirty_region" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

// render2d_compositor_tile_rle_write_step_facade_ok
// render2d_compositor_tile_rle_write_step_one_run_ok
// render2d_compositor_tile_rle_write_step_completion_status_ok
// render2d_compositor_tile_rle_write_step_progress_ok
// render2d_compositor_tile_rle_write_step_metadata_ok
// render2d_compositor_tile_rle_write_step_finish_payload_recovery_ok
// render2d_compositor_tile_rle_write_step_error_recovery_source_policy_ok
// render2d_compositor_tile_rle_write_step_free_delegates_write_cursor_ok
// render2d_compositor_tile_rle_write_step_no_encoded_packet_present_no_fallback

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

fn fail_writer_plan_error %fn GuiRgba8888CompositorTileRleWriterPlanError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_writer_plan_error_free error:
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

fn fail_write_cursor_start %fn GuiRgba8888CompositorTileRleWriteCursorStartError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_write_cursor_start_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_write_cursor_finish %fn GuiRgba8888CompositorTileRleWriteCursorFinishError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_write_cursor_finish_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_write_step_error %fn GuiRgba8888CompositorTileRleWriteStepError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_rle_write_step_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn metadata_ok %fn &GuiRgba8888CompositorFrameEntryMetadata fn i32 bool \metadata\frame_id:
    and:
        eq gui_rgba8888_compositor_frame_entry_metadata_frame_id metadata frame_id
        and:
            eq gui_rgba8888_compositor_frame_entry_metadata_width metadata 1
            and:
                eq gui_rgba8888_compositor_frame_entry_metadata_height metadata 1
                and:
                    eq gui_rgba8888_compositor_frame_entry_metadata_row_start metadata 0
                    and:
                        eq gui_rgba8888_compositor_frame_entry_metadata_row_count metadata 1
                        and:
                            eq gui_rgba8888_compositor_frame_entry_metadata_batch_count metadata 1
                            eq gui_rgba8888_compositor_frame_entry_metadata_max_rows_per_batch metadata 1

fn count_status_completed %fn GuiRgba8888RowTileRleCountStepStatus bool \status:
    match status:
        GuiRgba8888RowTileRleCountStepStatus::Completed:
            true
        _:
            false

fn write_status_wrote_run %fn GuiRgba8888RowTileRleWriteStepStatus bool \status:
    match status:
        GuiRgba8888RowTileRleWriteStepStatus::WroteRun:
            true
        _:
            false

fn write_status_completed %fn GuiRgba8888RowTileRleWriteStepStatus bool \status:
    match status:
        GuiRgba8888RowTileRleWriteStepStatus::Completed:
            true
        _:
            false

fn write_step_owner_ok %fn &GuiRgba8888CompositorTileRleWriteStep fn i32 fn i32 fn i32 fn i32 fn i32 bool \step\frame_id\written_runs\written_bytes\next_pixel\pixel_count:
    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_rle_write_step_metadata step
    and:
        metadata_ok &metadata frame_id
        and:
            eq gui_rgba8888_compositor_tile_rle_write_step_total_run_count step 1
            and:
                eq gui_rgba8888_compositor_tile_rle_write_step_encoded_byte_count step 12
                and:
                    eq gui_rgba8888_compositor_tile_rle_write_step_written_run_count step written_runs
                    and:
                        eq gui_rgba8888_compositor_tile_rle_write_step_written_byte_count step written_bytes
                        and:
                            eq gui_rgba8888_compositor_tile_rle_write_step_cursor_next_pixel_index step next_pixel
                            eq gui_rgba8888_compositor_tile_rle_write_step_cursor_pixel_count step pixel_count

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
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            Result::Err 1
        Result::Ok surface0:
            let r0 %u8 cast 31
            let g0 %u8 cast 32
            let b0 %u8 cast 33
            let a0 %u8 cast 34
            let color0 %Rgba8888 rgba8888_new r0 g0 b0 a0
            match gui_rgba8888_software_surface_write_pixel surface0 0 0 color0:
                Result::Err error:
                    Result::Err fail_surface_write error 2
                Result::Ok surface1:
                    let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface1
                    match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
                        Result::Err error:
                            Result::Err fail_dirty_push error 4
                        Result::Ok dirty_owner:
                            let config %GuiRgba8888CompositorFrameEntryConfig gui_rgba8888_compositor_frame_entry_config frame_id 1
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
                    let config %GuiRgba8888RowTilePlanConfig GuiRgba8888RowTilePlanConfig 1
                    match gui_rgba8888_compositor_tile_plan_prepare storage_owner config:
                        Result::Err error:
                            Result::Err fail_tile_plan_prepare error 12
                        Result::Ok tile_plan:
                            match gui_rgba8888_compositor_tile_payload_prepare tile_plan 1:
                                Result::Err error:
                                    Result::Err fail_payload_prepare error 13
                                Result::Ok payload:
                                    Result::Ok payload

fn write_step_finish_payload_success %fn GuiRgba8888CompositorTileRleWriteStep fn bool i32 \step\step_ok:
    match gui_rgba8888_compositor_tile_rle_write_step_finish_payload step:
        Result::Err error:
            fail_write_cursor_finish error 75
        Result::Ok payload:
            match gui_rgba8888_compositor_tile_payload_owner_finish_entry payload:
                Result::Err error:
                    fail_payload_finish error 76
                Result::Ok entry:
                    if step_ok:
                        then expect_complete_entry entry 77
                        else free_entry_code entry 78

fn write_step_completed_success %fn GuiRgba8888CompositorTileRleWriteCursorOwner fn bool i32 \owner\owner_ok:
    match gui_rgba8888_compositor_tile_rle_write_step_one owner:
        Result::Err error:
            fail_write_step_error error 73
        Result::Ok step3:
            let status3 %GuiRgba8888RowTileRleWriteStepStatus gui_rgba8888_compositor_tile_rle_write_step_status &step3
            let step3_ok %bool and:
                owner_ok
                and:
                    write_status_completed status3
                    write_step_owner_ok &step3 509 1 12 1 1
            write_step_finish_payload_success step3 step3_ok

fn write_step_complete_success %fn GuiRgba8888CompositorTileRleWriteCursorOwner fn bool i32 \owner\owner_ok:
    match gui_rgba8888_compositor_tile_rle_write_step_one owner:
        Result::Err error:
            fail_write_step_error error 70
        Result::Ok step1:
            let status1 %GuiRgba8888RowTileRleWriteStepStatus gui_rgba8888_compositor_tile_rle_write_step_status &step1
            let step1_status_ok %bool write_status_wrote_run status1
            let step1_ok %bool and:
                owner_ok
                and:
                    step1_status_ok
                    write_step_owner_ok &step1 509 1 12 1 1
            let owner1 %GuiRgba8888CompositorTileRleWriteCursorOwner gui_rgba8888_compositor_tile_rle_write_step_finish_owner step1
            write_step_completed_success owner1 step1_ok

fn write_cursor_start_success %fn GuiRgba8888CompositorTileRleStorageOwner i32 \storage:
    match gui_rgba8888_compositor_tile_rle_write_cursor_start storage:
        Result::Err error:
            fail_write_cursor_start error 64
        Result::Ok owner:
            write_step_complete_success owner true

fn storage_prepare_success %fn GuiRgba8888CompositorTileRleWriterPlanOwner i32 \plan:
    match gui_rgba8888_compositor_tile_rle_storage_prepare plan:
        Result::Err error:
            fail_storage_prepare error 54
        Result::Ok storage:
            write_cursor_start_success storage

fn writer_plan_success %fn GuiRgba8888CompositorTileRleEncodeCursorOwner i32 \ready:
    match gui_rgba8888_compositor_tile_rle_writer_plan_prepare ready:
        Result::Err error:
            fail_writer_plan_error error 44
        Result::Ok owner:
            storage_prepare_success owner

fn cursor_start_success %fn GuiRgba8888CompositorTileRleEncodeSeedOwner fn bool i32 \seed\status_ok:
    match gui_rgba8888_compositor_tile_rle_encode_cursor_start seed:
        Result::Err error:
            fail_cursor_error error 40
        Result::Ok ready:
            if status_ok:
                then writer_plan_success ready
                else:
                    match gui_rgba8888_compositor_tile_rle_encode_cursor_owner_free ready:
                        Result::Ok _:
                            41
                        Result::Err _:
                            42

fn seed_prepare_success %fn GuiRgba8888CompositorTileRleCountCompletedOwner fn bool i32 \completed\status_ok:
    match gui_rgba8888_compositor_tile_rle_encode_seed_prepare completed:
        Result::Err error:
            fail_seed_error error 33
        Result::Ok seed:
            cursor_start_success seed status_ok

fn completed_prepare_success %fn GuiRgba8888CompositorTileRleCountOwner i32 \owner:
    match gui_rgba8888_compositor_tile_rle_count_step_budget owner 1:
        Result::Err error:
            fail_count_step_error error 31
        Result::Ok step:
            let status %GuiRgba8888RowTileRleCountStepStatus gui_rgba8888_compositor_tile_rle_count_step_status &step
            let status_ok %bool count_status_completed status
            let owner1 %GuiRgba8888CompositorTileRleCountOwner gui_rgba8888_compositor_tile_rle_count_step_finish_owner step
            match gui_rgba8888_compositor_tile_rle_count_completed_prepare owner1:
                Result::Err error:
                    fail_completed_error error 32
                Result::Ok completed:
                    seed_prepare_success completed status_ok

fn run_payload %fn GuiRgba8888CompositorTilePayloadOwner i32 \payload:
    match gui_rgba8888_compositor_tile_rle_count_start payload:
        Result::Err error:
            fail_count_start error 20
        Result::Ok owner:
            completed_prepare_success owner

fn run_case %fn i32 i32 \mode:
    match build_entry 509:
        Result::Err code:
            code
        Result::Ok entry:
            match build_payload entry:
                Result::Err code:
                    code
                Result::Ok payload:
                    run_payload payload

fn main %impure fn void i32 \void:
    run_case 0
```

## malformed write cursor owner is not public application surface

通常の application code は malformed compositor write cursor owner を直 constructor で作れない。F5mn step error recovery の read-before-consume 順序と write cursor owner recovery は `nodesrc/test_web_gui_font_rendering_contract.js` の source policy で固定する。

neplg2:test[compile_fail]
diag_code: type.owner_aggregate.constructor_restricted
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *

// render2d_compositor_tile_rle_write_step_error_recovery_source_policy_ok

fn forge_compositor_write_cursor %fn GuiRgba8888RowTileRleWriteCursorOwner fn GuiRgba8888CompositorFrameEntryMetadata GuiRgba8888CompositorTileRleWriteCursorOwner \lower\metadata:
    GuiRgba8888CompositorTileRleWriteCursorOwner lower metadata

fn main %impure fn void i32 \void:
    0
```
