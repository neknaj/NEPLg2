# GUI render2d compositor tile payload

このファイルは、F5me の RGBA8888 compositor tile payload bridge が F5md compositor tile plan owner から F5cb row tile payload owner へ進み、RLE / host present / fallback へ進まないことを固定する。

source policy coverage labels:

- render2d_compositor_tile_payload_facade_ok
- render2d_compositor_tile_payload_prepare_ok
- render2d_compositor_tile_payload_checked_metadata_ok
- render2d_compositor_tile_payload_tile_relative_read_ok
- render2d_compositor_tile_payload_invalid_index_recovery_ok
- render2d_compositor_tile_payload_finish_entry_ok
- render2d_compositor_tile_payload_no_rle_no_platform_no_fallback

## compositor tile payload bridge keeps tile-scoped bytes

[目的/もくてき]:
- F5md tile plan owner の metadata を保持したまま、lower `gui_rgba8888_row_tile_payload_prepare` を 1 回だけ通すことを確認します。
- success owner の checked descriptor / plan metadata と tile-relative byte read が lower payload view を通ることを確認します。
- invalid tile index でも owner-bearing prepare error から tile plan owner を回収し、metadata と category を失わないことを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_compositor_tile_payload_contract\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
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
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/row_tile_payload" as *
#import "alloc/gui/render2d/row_tile_plan" as *
#import "alloc/gui/render2d/software_surface" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/dirty_region" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as test

// render2d_compositor_tile_payload_facade_ok
// render2d_compositor_tile_payload_prepare_ok
// render2d_compositor_tile_payload_checked_metadata_ok
// render2d_compositor_tile_payload_tile_relative_read_ok
// render2d_compositor_tile_payload_invalid_index_recovery_ok
// render2d_compositor_tile_payload_finish_entry_ok
// render2d_compositor_tile_payload_no_rle_no_platform_no_fallback

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

fn fail_tile_plan_finish %fn GuiRgba8888CompositorTilePlanFinishError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_plan_finish_error_free error:
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

fn finish_payload_code %fn GuiRgba8888CompositorTilePayloadOwner fn i32 i32 \owner\code:
    match gui_rgba8888_compositor_tile_payload_owner_finish_entry owner:
        Result::Err error:
            fail_payload_finish error code
        Result::Ok entry:
            free_entry_code entry code

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

fn payload_byte_or_neg %fn &GuiRgba8888CompositorTilePayloadOwner fn i32 i32 \owner\index:
    match gui_rgba8888_compositor_tile_payload_owner_byte_at owner index:
        Result::Err _:
            -1
        Result::Ok value:
            value

fn payload_read_bounds_kind_ok %fn &GuiRgba8888CompositorTilePayloadOwner bool \owner:
    match gui_rgba8888_compositor_tile_payload_owner_byte_at owner 8:
        Result::Ok _:
            false
        Result::Err kind:
            match kind:
                GuiRgba8888RowTilePayloadReadErrorKind::PayloadIndexOutOfBounds:
                    true
                _:
                    false

fn descriptor_ok %fn &GuiRgba8888CompositorTilePayloadOwner bool \owner:
    match gui_rgba8888_compositor_tile_payload_owner_descriptor_checked owner:
        Result::Err _:
            false
        Result::Ok descriptor:
            and:
                eq gui_rgba8888_row_tile_descriptor_tile_index &descriptor 1
                and:
                    eq gui_rgba8888_row_tile_descriptor_row_start &descriptor 2
                    and:
                        eq gui_rgba8888_row_tile_descriptor_row_count &descriptor 1
                        and:
                            eq gui_rgba8888_row_tile_descriptor_byte_offset &descriptor 16
                            eq gui_rgba8888_row_tile_descriptor_byte_count &descriptor 8

fn plan_metadata_ok %fn &GuiRgba8888CompositorTilePayloadOwner bool \owner:
    match gui_rgba8888_compositor_tile_payload_owner_plan_metadata_checked owner:
        Result::Err _:
            false
        Result::Ok plan:
            and:
                eq gui_rgba8888_row_tile_plan_tile_rows &plan 2
                and:
                    eq gui_rgba8888_row_tile_plan_tile_count &plan 2
                    eq gui_rgba8888_row_tile_plan_byte_count &plan 24

fn byte_count_ok %fn &GuiRgba8888CompositorTilePayloadOwner bool \owner:
    match gui_rgba8888_compositor_tile_payload_owner_byte_count_checked owner:
        Result::Err _:
            false
        Result::Ok byte_count:
            eq byte_count 8

fn payload_ok %fn &GuiRgba8888CompositorTilePayloadOwner bool \owner:
    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_payload_owner_metadata owner
    and:
        metadata_ok &metadata 95
        and:
            descriptor_ok owner
            and:
                plan_metadata_ok owner
                and:
                    byte_count_ok owner
                    and:
                        eq payload_byte_or_neg owner 0 31
                        and:
                            eq payload_byte_or_neg owner 1 32
                            and:
                                eq payload_byte_or_neg owner 2 33
                                and:
                                    eq payload_byte_or_neg owner 3 34
                                    and:
                                        eq payload_byte_or_neg owner 4 41
                                        and:
                                            eq payload_byte_or_neg owner 7 44
                                            payload_read_bounds_kind_ok owner

fn invalid_prepare_kind_ok %fn &GuiRgba8888CompositorTilePayloadPrepareError bool \error:
    match gui_rgba8888_compositor_tile_payload_prepare_error_kind error:
        GuiRgba8888CompositorTilePayloadPrepareErrorKind::RowTilePayloadPrepareFailed lower_kind:
            match lower_kind:
                GuiRgba8888RowTilePayloadPrepareErrorKind::DescriptorInvalid descriptor_kind:
                    match descriptor_kind:
                        GuiRgba8888RowTilePlanDescriptorErrorKind::TileIndexOutOfBounds:
                            true
                        _:
                            false

fn invalid_prepare_category_ok %fn &GuiRgba8888CompositorTilePayloadPrepareError bool \error:
    match gui_rgba8888_compositor_tile_payload_prepare_error_category_value error:
        Option::Some category:
            match category:
                GuiError::InvalidCommand:
                    true
                _:
                    false
        Option::None:
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

fn build_tile_plan %fn GuiRgba8888CompositorFrameEntryOwner Result GuiRgba8888CompositorTilePlanOwner i32 \entry:
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
                            Result::Ok tile_plan

fn run_success_from_plan %fn GuiRgba8888CompositorTilePlanOwner i32 \tile_plan:
    match gui_rgba8888_compositor_tile_payload_prepare tile_plan 1:
        Result::Err error:
            fail_payload_prepare error 13
        Result::Ok payload:
            let ok %bool payload_ok &payload
            match gui_rgba8888_compositor_tile_payload_owner_finish_entry payload:
                Result::Err error:
                    fail_payload_finish error 14
                Result::Ok entry:
                    if ok:
                        then expect_complete_entry entry 15
                        else free_entry_code entry 16

fn run_success_case %fn void i32 \void:
    match build_entry 95:
        Result::Err code:
            code
        Result::Ok entry:
            match build_tile_plan entry:
                Result::Err code:
                    code
                Result::Ok tile_plan:
                    run_success_from_plan tile_plan

fn recover_invalid_payload_prepare %fn GuiRgba8888CompositorTilePayloadPrepareError i32 \error:
    let kind_ok %bool invalid_prepare_kind_ok &error
    let category_ok %bool invalid_prepare_category_ok &error
    let plan %GuiRgba8888CompositorTilePlanOwner gui_rgba8888_compositor_tile_payload_prepare_error_plan error
    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_plan_owner_metadata &plan
    let metadata_matches %bool metadata_ok &metadata 96
    let ok %bool and kind_ok and category_ok metadata_matches
    match gui_rgba8888_compositor_tile_plan_owner_finish_entry plan:
        Result::Err error:
            fail_tile_plan_finish error 23
        Result::Ok entry:
            if ok:
                then expect_complete_entry entry 24
                else free_entry_code entry 25

fn run_invalid_from_plan %fn GuiRgba8888CompositorTilePlanOwner i32 \tile_plan:
    match gui_rgba8888_compositor_tile_payload_prepare tile_plan 2:
        Result::Err error:
            recover_invalid_payload_prepare error
        Result::Ok payload:
            finish_payload_code payload 21

fn run_invalid_tile_index_case %fn void i32 \void:
    match build_entry 96:
        Result::Err code:
            code
        Result::Ok entry:
            match build_tile_plan entry:
                Result::Err code:
                    code
                Result::Ok tile_plan:
                    run_invalid_from_plan tile_plan

fn run_case %fn void i32 \void:
    let success %i32 run_success_case
    if ne success 0:
        then success
        else run_invalid_tile_index_case

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_compositor_tile_payload_contract"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
