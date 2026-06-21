# GUI render2d compositor tile RLE count

このファイルは、F5mf の RGBA8888 compositor tile RLE count start bridge が F5me compositor tile payload owner から lower row tile RLE count owner へ進み、drain / encode / present / fallback へ進まないことを固定する。

source policy coverage labels:

- render2d_compositor_tile_rle_count_facade_ok
- render2d_compositor_tile_rle_count_start_ok
- render2d_compositor_tile_rle_count_metadata_ok
- render2d_compositor_tile_rle_count_status_ok
- render2d_compositor_tile_rle_count_finish_entry_ok
- render2d_compositor_tile_rle_count_no_drain_no_encode_no_present_no_fallback

## compositor tile RLE count bridge starts lower count owner

[目的/もくてき]:
- F5me tile payload owner の metadata を保持したまま、lower `gui_rgba8888_row_tile_rle_cursor_start` と `gui_rgba8888_row_tile_rle_count_start` を 1 回ずつ通すことを確認します。
- start 直後の count owner が accumulated run count 0、cursor next pixel index 0、Ready status であることを確認します。
- count owner を frame entry owner へ戻せることを確認し、drain / count step / encode / present へ進まないことを source policy label で固定します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_compositor_tile_rle_count_contract\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
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
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/row_tile_payload" as *
#import "alloc/gui/render2d/row_tile_plan" as *
#import "alloc/gui/render2d/row_tile_rle" as *
#import "alloc/gui/render2d/software_surface" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/dirty_region" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as test

// render2d_compositor_tile_rle_count_facade_ok
// render2d_compositor_tile_rle_count_start_ok
// render2d_compositor_tile_rle_count_metadata_ok
// render2d_compositor_tile_rle_count_status_ok
// render2d_compositor_tile_rle_count_finish_entry_ok
// render2d_compositor_tile_rle_count_no_drain_no_encode_no_present_no_fallback

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

fn cursor_ready_ok %fn &GuiRgba8888CompositorTileRleCountOwner bool \owner:
    match gui_rgba8888_compositor_tile_rle_count_owner_cursor_status owner:
        Result::Err _:
            false
        Result::Ok status:
            match status:
                GuiRgba8888RowTileRleCursorStatus::Ready:
                    true
                _:
                    false

fn count_owner_ok %fn &GuiRgba8888CompositorTileRleCountOwner bool \owner:
    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_rle_count_owner_metadata owner
    and:
        metadata_ok &metadata 97
        and:
            eq gui_rgba8888_compositor_tile_rle_count_owner_accumulated_run_count owner 0
            and:
                eq gui_rgba8888_compositor_tile_rle_count_owner_cursor_next_pixel_index owner 0
                cursor_ready_ok owner

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

fn run_count_from_payload %fn GuiRgba8888CompositorTilePayloadOwner i32 \payload:
    match gui_rgba8888_compositor_tile_rle_count_start payload:
        Result::Err error:
            fail_count_start error 14
        Result::Ok count_owner:
            let ok %bool count_owner_ok &count_owner
            match gui_rgba8888_compositor_tile_rle_count_owner_finish_entry count_owner:
                Result::Err error:
                    fail_count_finish error 15
                Result::Ok entry:
                    if ok:
                        then expect_complete_entry entry 16
                        else free_entry_code entry 17

fn run_case %fn void i32 \void:
    match build_entry 97:
        Result::Err code:
            code
        Result::Ok entry:
            match build_payload entry:
                Result::Err code:
                    code
                Result::Ok payload:
                    run_count_from_payload payload

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_compositor_tile_rle_count_contract"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
